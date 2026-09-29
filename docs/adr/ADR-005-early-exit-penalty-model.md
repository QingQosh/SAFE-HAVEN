# ADR-005 — Early-Exit Penalty Model and Fee Recipient

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-07-27 |
| **Deciders** | SAFE-HAVEN core contributors |
| **Issue / PR** | — |
| **Supersedes** | — |
| **Superseded by** | — |

---

## Context

SAFE-HAVEN allows depositors to exit a vault before its unlock time by
calling `cancel_deposit`. This "early exit" capability is a deliberate
design choice: forcing users to wait until the lock expires is not always
desirable (emergencies, changed circumstances), and a mechanism with zero
cost would remove the commitment value of the lock entirely.

The design must answer three questions:

1. **Who bears the penalty?** The depositor calling `cancel_deposit` pays a
   penalty on the principal amount.
2. **Who receives the penalty?** The penalty must go somewhere that does not
   benefit the admin — sending it to the admin would create a perverse
   incentive for the admin to front-run withdrawals.
3. **How large can the penalty be, and who sets it?** A depositor should be
   able to make their own lock-breaking cost meaningful or negligible
   depending on the use case.

---

## Decision

### Penalty structure

The penalty is expressed in **basis points (BPS)** set by the depositor at
deposit time:

- Range: 0–10,000 BPS (0%–100% of the deposited amount)
- The value is stored in `VaultEntry.penalty_bps` and
  `LedgerVaultEntry.penalty_bps`
- It is immutable after the deposit is created — it cannot be changed by the
  depositor, the admin, or anyone else

The depositor-controlled penalty is the mechanism that makes a vault a
credible commitment. A depositor who wants a strong commitment sets a high
penalty; one who just wants a time-based reminder can set 0 BPS.

**Penalty calculation (integer arithmetic, no floating point):**

```
penalty_amount = (deposit_amount × penalty_bps) / 10_000
return_amount  = deposit_amount - penalty_amount
```

The division uses integer arithmetic. Soroban contracts do not support
floating-point, and `i128` is used throughout to preserve precision at scale.
The rounding is truncating (floor). Any dust lost to truncation stays in the
penalty amount rather than the return amount, slightly favoring the fee
recipient over the depositor. This is intentional — it avoids a scenario
where rounding returns slightly more than the depositor is entitled to.

### Fee recipient

The penalty amount is transferred to a `fee_recipient` address set during
`initialize`. Key properties:

- The fee recipient is not the admin. An admin address that is also the fee
  recipient would create a conflict of interest. The contract does not
  enforce this separation (both are arbitrary `Address` values), but it is
  strongly recommended in the deployment guide and governance policy.
- The fee recipient can be a protocol treasury, a burn address, a multisig,
  or any Stellar address — the contract is agnostic.
- If `penalty_bps > 0` but no fee recipient is configured, `deposit` fails
  with `MissingFeeRecipient` (error 14). This prevents a state where a
  depositor expects a penalty-backed commitment but the penalty has nowhere
  to go.

### 100% penalty (full burn / forfeit)

`penalty_bps = 10_000` (100%) is explicitly valid. A depositor who sets
this effectively commits to forfeiting the entire deposit if they exit early.
The full amount is sent to the fee recipient; the depositor receives zero.
This is the maximum-commitment mode.

### Zero penalty

`penalty_bps = 0` is valid and produces no fee transfer. `cancel_deposit`
with zero penalty is equivalent to an unconditional early withdraw — no
partial transfer, no fee. This is useful for time-reminder vaults where the
commitment is social rather than financial.

---

## Consequences

### Positive

- Depositor-controlled BPS gives users full authority over their own
  commitment strength without admin involvement.
- Integer BPS arithmetic is deterministic and auditable with no precision
  loss beyond one-stroop truncation.
- The `MissingFeeRecipient` guard prevents silent misconfiguration where a
  penalty is specified but would be dropped.
- Sending penalties to a configurable `fee_recipient` (not the contract or
  admin) keeps the incentive structure clean.
- 100% penalty is supported without special-casing — the same code path
  handles every BPS value.

### Negative / Risks

- The fee recipient is set at initialization and can only be updated by the
  admin. If the fee recipient address becomes inaccessible (key loss, smart
  wallet upgrade), penalties sent to it are effectively locked. Mitigation:
  use a multisig or smart wallet for the fee recipient, not a single EOA.
- There is no partial-penalty schedule (e.g., "10% if you exit in the first
  half of the lock period, 5% in the second half"). The penalty is fixed at
  deposit time and does not change. This simplicity is intentional but may
  be a limitation for advanced use cases.
- A depositor who sets `penalty_bps = 0` can cancel any time with no cost,
  making the lock a non-binding hint. This is by design but could be
  confusing to users who expect all vaults to have a meaningful commitment.
  The UI should clearly surface the penalty value.

---

## Alternatives Considered

### Alternative A — Admin-set global penalty rate

A single global `penalty_bps` set by the admin during `initialize` or via a
setter, applied to all cancellations.

Rejected because it removes depositor agency. A global rate set by an admin
is equivalent to a fee, not a self-imposed commitment. The value proposition
of the contract — "you set your own exit cost" — is lost. It also creates
a trust dependency on the admin to set a fair rate.

### Alternative B — Penalty sent to the contract (protocol fee)

Penalty amounts accumulate in the contract and are withdrawn by the admin.

Rejected because it gives the admin a direct financial incentive to trigger
or encourage early exits (e.g., by pausing and unpausing strategically,
or via social pressure). Penalties must go to a third-party address that is
distinct from the admin to keep the admin's incentives aligned with
depositors.

### Alternative C — Time-weighted penalty decay

The penalty decreases linearly from `penalty_bps` to 0 as the unlock time
approaches. For example, with `penalty_bps = 5000` and a 30-day lock, the
penalty on day 15 would be 2500 BPS.

Rejected at this stage because it adds significant complexity:
- Penalty calculation requires knowing both the original deposit timestamp
  and the current timestamp, which means more storage or derived computation.
- The decay formula is not obvious to users without documentation.
- A simpler fixed-penalty model is easier to audit and reason about.

Time-weighted decay is a valid future extension if demand exists.

### Alternative D — Penalty burned (sent to zero address)

The penalty amount is burned rather than sent to a recipient.

Rejected because Stellar's token standard does not have a standardized burn
mechanism; sending to a zero address is not a guaranteed burn. Using a
configurable `fee_recipient` achieves the same effective result (penalty
leaves the depositor) while allowing protocol-specific handling (treasury,
buyback, etc.).

---

## References

- `contracts/safe-haven/src/contract.rs` — `deposit`, `cancel_deposit`,
  penalty calculation
- `contracts/safe-haven/src/types.rs` — `VaultEntry.penalty_bps`,
  `LedgerVaultEntry.penalty_bps`
- `contracts/safe-haven/src/errors.rs` — `VaultError::InvalidPenaltyBps`,
  `VaultError::MissingFeeRecipient`
- `contracts/safe-haven/src/storage.rs` — `VaultKey::FeeRecipient`
- README — "Early exit" and "Contract API" sections
