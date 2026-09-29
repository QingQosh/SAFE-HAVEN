# ADR-002 — Storage Layout: Monolithic DepositorList and cancel_deposit Error Semantics

| Field       | Value                        |
|-------------|------------------------------|
| Status      | Accepted                     |
| Date        | 2026-07-27                   |
| Deciders    | SAFE-HAVEN core contributors |
| Issues      | #113                         |

---

## Context

This ADR records two related storage-layer decisions that have generated contributor confusion:

1. Why `DepositorList` is a single, contract-wide `Vec<Address>` rather than being sharded or replaced with a different index.
2. Why `cancel_deposit` returns `VaultAlreadyUnlocked` (error 13) when the lock has expired, instead of returning `FundsStillLocked` (error 4).

---

## Decision 1 — Monolithic DepositorList with O(1) Logical Remove

### Structure

All depositor addresses are appended to a single `Vec<Address>` stored under `VaultKey::DepositorList`. Three per-depositor keys work in concert with it:

| Key                          | Type   | Lifecycle                                                   |
|------------------------------|--------|-------------------------------------------------------------|
| `VaultKey::DepositorFlag(Address)`   | `bool` | Set when depositor becomes active; **deleted** on last withdrawal |
| `VaultKey::DepositorInList(Address)` | `bool` | Set the first time an address is appended; **never deleted**     |
| `VaultKey::DepositorList`            | `Vec<Address>` | Append-only; never shrinks                           |

**Adding a depositor** (`add_depositor`):

1. If `DepositorFlag` is already `true` → return immediately (depositor already active, no-op).
2. Set `DepositorFlag = true`.
3. If `DepositorInList` is not set → push address onto `DepositorList` and set `DepositorInList = true`.

Step 3 only runs the first time an address is seen. Re-deposits after a full withdrawal skip the append, so the list stays free of duplicates without scanning it.

**Removing a depositor** (`remove_depositor`):

Delete `DepositorFlag`. The address remains in `DepositorList` as a stale entry. It is skipped at read time by checking whether `DepositorFlag` is present.

**Reading** (`get_depositor_count`, `get_depositors_page`):

Both functions iterate `DepositorList` and filter out any address whose `DepositorFlag` key is absent. Stale entries are invisible to callers.

### Why not shard the list?

A natural alternative is to split the list into fixed-size buckets (e.g., `DepositorShard(u32)`) and store a shard count alongside them. This pattern is common in Soroban for very large unbounded lists.

It was evaluated and rejected for three reasons:

1. **Pagination complexity.** `get_depositors_page(offset, limit)` requires a contiguous, ordered sequence of active depositors. With sharding, computing an offset that spans shard boundaries requires loading the active count of every preceding shard — or maintaining a separate per-shard active-count index. Either approach involves more storage reads per page and more code surface.

2. **Cross-shard coordination on write.** Every `add_depositor` call must determine which shard to append to. If a shard is full, the code must atomically move to the next shard and increment the shard count. Soroban's instruction-budget constraints make cross-key atomic operations expensive; a bug here could lose depositors from the index entirely.

3. **The actual list size is bounded in practice.** Each depositor is appended at most once (guarded by `DepositorInList`). For the workloads SAFE-HAVEN targets, the list grows slowly enough that deserialising it does not approach Soroban's budget limits. If the list ever becomes a bottleneck, sharding can be introduced in a future migration alongside `migrate()` without breaking the existing interface.

The monolithic-plus-flag approach keeps the write path to a constant number of small key operations (set flag, set in-list sentinel, push to list) and keeps pagination to a single sequential scan of one key.

### Why not a doubly-linked list or a bitmap?

A doubly-linked list in Soroban storage would require two extra pointer keys per node (`prev`, `next`) plus update logic on both ends of a remove, tripling the number of storage writes per withdrawal. Soroban charges per storage operation, making this approach materially more expensive for users.

A bitmap (one bit per address slot) cannot be used because Soroban's `contracttype` does not natively support arbitrary-length bitfields, and the address→slot mapping would itself require an additional index.

---

## Decision 2 — cancel_deposit Returns VaultAlreadyUnlocked, Not FundsStillLocked

### Error semantics

`cancel_deposit` is an **early exit** operation. Its entire premise is that the lock is still active and the depositor is willing to pay the penalty to exit before maturity.

This creates a natural three-way state space when `cancel_deposit` is called:

| State                              | Expected action          | Error returned         |
|------------------------------------|--------------------------|------------------------|
| Lock active, deposit exists        | Cancel, apply penalty    | _(success)_            |
| Lock **expired**, deposit exists   | Caller should use `withdraw` instead | `VaultAlreadyUnlocked` (13) |
| Deposit does not exist             | Nothing to cancel        | `NoDepositFound` (3)   |

`FundsStillLocked` (error 4) is used only by `withdraw` and `withdraw_to`. It answers the question "can I withdraw yet?" with a clear "no, the lock hasn't expired." That error makes no semantic sense for `cancel_deposit` because `cancel_deposit` succeeds precisely *because* the lock hasn't expired — a locked deposit is the normal, expected input.

Returning `FundsStillLocked` from `cancel_deposit` would reverse the meaning: it would imply that cancelling is only possible once the lock expires, which is the opposite of the feature's purpose.

### Why VaultAlreadyUnlocked specifically?

When the lock has expired and the depositor calls `cancel_deposit`, they are misusing the operation. The correct action is `withdraw`, which returns the full amount with no penalty. Returning `VaultAlreadyUnlocked` signals this precisely: "the vault has already reached maturity; there is nothing to cancel." This guides the caller toward the correct path (`withdraw`) rather than leaving them wondering whether the lock timing is the issue.

### Code evidence

```rust
// cancel_deposit — contract.rs
if now >= entry.unlock_time {
    return Err(VaultError::VaultAlreadyUnlocked);   // expired → wrong function
}
// ... apply penalty and transfer ...

// withdraw — contract.rs
if now < entry.unlock_time {
    return Err(VaultError::FundsStillLocked);        // locked → come back later
}
```

The symmetry is intentional:
- `withdraw` fails with `FundsStillLocked` when called **too early**.
- `cancel_deposit` fails with `VaultAlreadyUnlocked` when called **too late**.

Neither function uses the other's error code.

---

## Consequences

### DepositorList

**Positive**
- Write path: constant number of small key operations regardless of list size.
- No need to deserialise `DepositorList` on most writes (re-deposits skip the append entirely after the first time).
- Pagination is a single sequential scan of one storage key.
- No shard-boundary edge cases or coordination logic.

**Negative**
- `DepositorList` grows monotonically and is never compacted on-chain. For very high depositor churn (many unique addresses each depositing once), the raw list can become large. `get_depositor_count` and `get_depositors_page` pay a linear scan cost proportional to the raw list length, not the active count.
- A future sharding migration via `migrate()` is the intended remediation if list size becomes a performance concern.

### cancel_deposit Error Semantics

**Positive**
- The error returned is always semantically correct: `VaultAlreadyUnlocked` diagnoses the *caller's mistake* (using the wrong function), not a *timing constraint* (the lock expiry).
- `FundsStillLocked` retains a single, unambiguous meaning across the entire contract: "a lock-time check blocked a withdrawal."
- Client code can distinguish "too early to withdraw" (`FundsStillLocked`) from "too late to cancel" (`VaultAlreadyUnlocked`) without additional context.

**Negative**
- Callers who expect `cancel_deposit` to be a superset of `withdraw` (i.e., "always get my tokens back") will be surprised by `VaultAlreadyUnlocked`. This must be documented clearly in SDK wrappers.

---

## References

- `contracts/safe-haven/src/types.rs` — `VaultKey` variants
- `contracts/safe-haven/src/storage.rs` — `add_depositor`, `remove_depositor`, `get_depositor_count`, `get_depositors_page`
- `contracts/safe-haven/src/contract.rs` — `cancel_deposit`, `withdraw`, `withdraw_to`
- `contracts/safe-haven/src/errors.rs` — `VaultError` enum
