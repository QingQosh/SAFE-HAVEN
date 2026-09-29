# ADR-001 — Dual Deposit Types: Timestamp vs Ledger Sequence

| Field       | Value                            |
|-------------|----------------------------------|
| Status      | Accepted                         |
| Date        | 2026-07-27                       |
| Deciders    | SAFE-HAVEN core contributors     |
| Issue       | #88                              |

---

## Context

SAFE-HAVEN locks tokens until a future point in time. The original design expressed that point as a wall-clock timestamp (`unlock_time: u64`, Unix seconds) checked against `env.ledger().timestamp()`.

A class of use-cases — most prominently "release after exactly N more blocks" or "release at a known network upgrade ledger" — cannot be expressed cleanly with wall-clock time because:

1. Stellar's ledger timestamps are set by validators and can drift or be skewed by up to a few minutes.
2. Some on-chain protocols coordinate actions by ledger sequence number, not by calendar date. Expressing a wall-clock constraint for an event defined in ledger terms requires an externally supplied approximation (5 s/ledger) that bakes in a modelling error from the start.
3. Off-chain tooling that monitors ledger sequence cannot directly compare a raw sequence number with a Unix timestamp without an extra conversion step that erodes auditability.

---

## Decision

Introduce a second concrete deposit type, `LedgerVaultEntry`, alongside the existing `VaultEntry`. The two types differ in exactly one field:

| Field         | `VaultEntry` (timestamp)  | `LedgerVaultEntry` (ledger) |
|---------------|---------------------------|------------------------------|
| `token`       | `Address`                 | `Address`                    |
| `amount`      | `i128`                    | `i128`                       |
| `depositor`   | `Address`                 | `Address`                    |
| `penalty_bps` | `u32`                     | `u32`                        |
| `unlock_time` | `u64` (Unix seconds)      | _(absent)_                   |
| `unlock_ledger` | _(absent)_              | `u32` (ledger sequence)      |

Each type has its own storage key variant:

```rust
VaultKey::Deposit(Address, u32)          // timestamp-based
VaultKey::DepositByLedger(Address, u32)  // ledger-based
```

Both variants share the **same per-depositor deposit counter** (`VaultKey::DepositCounter`), ensuring deposit IDs are globally unique for a given depositor regardless of type, and appear together in `get_deposit_ids`.

The entry-points affected are:

| Operation              | Timestamp | Ledger |
|------------------------|-----------|--------|
| `deposit`              | ✓         |        |
| `deposit_for`          | ✓         |        |
| `deposit_by_ledger`    |           | ✓      |
| `withdraw`             | ✓ then ✓  | ✓ then ✓ (tries timestamp first, then ledger) |
| `withdraw_to`          | ✓ / ✓    |        |
| `cancel_deposit`       | ✓ / ✓    |        |
| `emergency_withdraw`   | ✓ / ✓    |        |
| `get_vault`            | ✓         |        |
| `get_ledger_vault`     |           | ✓      |

`withdraw`, `withdraw_to`, `cancel_deposit`, and `emergency_withdraw` perform a **two-stage lookup**: they check the timestamp key first; if not found they check the ledger key. This keeps the public API surface stable — callers pass only `(depositor, deposit_id)` and do not need to know which type they are operating on.

### Unlock condition

```
// Timestamp-based
env.ledger().timestamp() >= entry.unlock_time

// Ledger-based
env.ledger().sequence() >= entry.unlock_ledger
```

### Minimum lock enforcement

Timestamp deposits require `lock_duration >= MIN_LOCK_DURATION_SECS` (60 s).  
Ledger deposits require `ledger_gap >= MIN_LOCK_LEDGERS` (12 ledgers ≈ 60 s at 5 s/ledger).  
Both minimums are equivalent in wall-clock terms but enforced by independent code paths because the units differ.

---

## Alternatives Considered

### A. Single type with a discriminant field

Add an enum or boolean `by_ledger: bool` plus both `unlock_time` and `unlock_ledger` to a single `VaultEntry`, leaving one field as zero/unused.

Rejected because:
- Soroban's persistent storage format is determined at the `#[contracttype]` level; adding a field to an already-deployed `VaultEntry` requires a migration, which would affect all existing deposits.
- A struct with two unlock fields and a discriminant is harder to audit than two purpose-built structs.
- Zero-filled unused fields are a common source of subtle bugs (e.g., accidentally reading `unlock_time` on a ledger deposit).

### B. Convert ledger to timestamp at deposit time

At `deposit_by_ledger` call time, multiply `unlock_ledger` by 5 s and store the result as a regular `VaultEntry.unlock_time`.

Rejected because:
- The conversion is approximate. If the real network closes ledgers faster or slower than 5 s, the stored value diverges from the user's intent and cannot be corrected after the fact.
- It discards the original ledger number, making it impossible to verify on-chain exactly which ledger the user specified.
- It conflates two semantically different things: "at calendar time T" vs "after ledger L".

### C. Fully separate counter per type

Maintain `DepositCounterByLedger(Address)` separate from `DepositCounter(Address)` so ledger deposits have their own ID namespace.

Rejected because:
- Callers would need to track which namespace a deposit ID belongs to before calling `withdraw` or `cancel_deposit`.
- The two-stage lookup strategy only works cleanly when IDs are globally unique per depositor, which a shared counter guarantees.

---

## Consequences

**Positive**
- Ledger-based locking is exact — no approximation error in the stored unlock condition.
- Existing timestamp deposits are completely unaffected; no migration is needed.
- `withdraw`, `cancel_deposit`, and `emergency_withdraw` work transparently on both types via the two-stage lookup.
- `get_deposit_ids` returns IDs for both types from a single call.

**Negative / Known Limitations**
- `deposit_by_ledger` does not enforce a maximum lock duration. `deposit` and `deposit_for` reject durations longer than `max_lock_secs` (default 5 years). Ledger deposits only enforce a minimum gap (`MIN_LOCK_LEDGERS = 12`). An arbitrarily large `unlock_ledger` is accepted. This is a known gap tracked as an open issue.
- The React frontend only exposes `deposit` and `deposit_for`. `deposit_by_ledger` must be called via the Stellar CLI or a direct SDK integration.
- `get_vault(depositor, id)` returns `None` for ledger-based deposits. Callers must use `get_ledger_vault(depositor, id)` instead.
- `get_deposits_page` (flat paginated view) iterates only timestamp-based entries.
- `time_remaining` for a ledger deposit returns `remaining_ledgers × 5` — an estimate, not a guaranteed wall-clock value.

---

## References

- `contracts/safe-haven/src/types.rs` — `VaultEntry`, `LedgerVaultEntry`, `VaultKey`
- `contracts/safe-haven/src/storage.rs` — `set_deposit_by_ledger`, `get_deposit_by_ledger_readonly`, `remove_deposit_by_ledger`
- `contracts/safe-haven/src/contract.rs` — `deposit_by_ledger`, two-stage lookup in `withdraw` / `cancel_deposit`
- README Known Limitations section
