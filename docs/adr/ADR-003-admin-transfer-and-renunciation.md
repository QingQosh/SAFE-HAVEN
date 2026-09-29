# ADR-003 — Two-Step Admin Transfer and Permanent Renunciation

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-07-27 |
| **Deciders** | SAFE-HAVEN core contributors |
| **Issue / PR** | #28, #63 |
| **Supersedes** | — |
| **Superseded by** | — |

---

## Context

SAFE-HAVEN has an admin role with privileged capabilities:

- `emergency_withdraw` — forcibly returns a depositor's funds regardless of
  lock status (funds go to the depositor, never the admin)
- `pause` / `unpause` — halts new deposits without affecting existing ones
- `transfer_admin` / `accept_admin` — hands off the admin key
- `renounce_admin` — permanently removes admin, making the contract trustless

The simplest implementation of admin transfer is a single-step atomic swap:
`transfer_admin(new_admin)` immediately replaces the current admin. This is
dangerous in practice because:

1. A typo or copy-paste error in the new admin address makes the contract
   permanently unmanageable with no recovery path.
2. On Stellar, contract addresses and key pairs look similar and differ only
   in a few characters; a one-character mistake is easy to miss.
3. A compromised key used to sign `transfer_admin` could silently redirect
   control to an attacker address, with no on-chain signal before the new
   admin accepts.

There is also a separate design question: should admin power be irrevocable,
or should the contract support a fully trustless mode where no address can
call privileged functions?

---

## Decision

### Two-step transfer

Admin transfer uses a two-step commit-reveal pattern:

1. **`transfer_admin(admin, new_admin)`** — the current admin nominates a
   successor. The `new_admin` address is stored in `VaultKey::PendingAdmin`
   but the current admin remains active. An `adm_xfr_propose` event is
   emitted.

2. **`accept_admin(new_admin)`** — the nominated address must call this
   function, proving it controls the key. Only on acceptance does
   `VaultKey::Admin` update to `new_admin` and `VaultKey::PendingAdmin`
   clear. An `adm_xfr_accept` event is emitted.

A third function **`cancel_transfer_admin(admin)`** lets the current admin
abort a pending transfer at any time. It clears `VaultKey::PendingAdmin`
and emits `adm_xfr_cancel`. This prevents a scenario where a transfer is
initiated and then forgotten, leaving an unclaimed pending-admin state
indefinitely.

**Key invariants:**

- Only the current `admin` can call `transfer_admin` and
  `cancel_transfer_admin`.
- Only the `pending_admin` (the exact address nominated) can call
  `accept_admin`.
- `accept_admin` fails with `Unauthorized` after `cancel_transfer_admin`
  clears the pending admin (closes #28, #63).
- No admin operation is possible while `VaultKey::Admin` is unset, except
  `initialize`.

### Permanent renunciation

**`renounce_admin(admin)`** deletes `VaultKey::Admin` without setting a
replacement. After this call:

- `get_admin()` returns `None`.
- All admin-gated functions (`emergency_withdraw`, `pause`, `transfer_admin`,
  etc.) fail with `Unauthorized` because the `require_auth` check against a
  `None` admin has no valid address to authorize.
- The contract continues operating normally for depositors: `deposit`,
  `withdraw`, `cancel_deposit`, and all read queries are unaffected.
- The renunciation is **permanent and irreversible** — there is no function
  to restore an admin after renunciation.

This enables a trustless deployment where the contract operator publicly
calls `renounce_admin` after setup, and users can verify via `get_admin()`
that no privileged key exists.

---

## Consequences

### Positive

- A typo in the new-admin address is caught before control transfers: the
  new admin must prove key ownership by calling `accept_admin`.
- `cancel_transfer_admin` removes the stale pending-admin risk without
  requiring a time-lock or guardian.
- Trustless mode is verifiable on-chain via `get_admin()` returning `None`.
- All state transitions emit events, giving off-chain monitors a complete
  audit trail.

### Negative / Risks

- Two-step transfer requires two transactions instead of one. This is a
  minor UX cost that is outweighed by the safety benefit.
- If the admin key is lost before `accept_admin` is called, the pending
  admin must still call `accept_admin` to complete the transfer. There is no
  force-accept path. Mitigation: always test the new key in a non-critical
  context before initiating a transfer on a production contract.
- `renounce_admin` is irreversible. Callers must be certain before calling
  it. There is no on-chain confirmation step; the Stellar CLI will prompt for
  transaction signing, which serves as the confirmation step in practice.
- Emergency recovery is impossible after renunciation. A depositor whose
  private key is lost cannot use `emergency_withdraw` to recover funds,
  because there is no admin to call it.

---

## Alternatives Considered

### Alternative A — Single-step atomic transfer

`transfer_admin(new_admin)` immediately replaces the current admin.

Rejected because a typo in `new_admin` produces an irreversible loss of
control. There is no recovery path on Soroban — contracts cannot be
upgraded or their storage overwritten without a pre-existing migration
function. This risk is not acceptable for a value-custody contract.

### Alternative B — Time-locked transfer with guardian

Introduce a time delay (e.g., 48 hours) before a transfer takes effect, plus
a guardian address that can veto. This is the pattern used by some DeFi
protocols.

Rejected because it adds significant complexity (guardian role, veto logic,
time-lock storage) while the simpler two-step approach already addresses the
key risk (typos). A time-lock does not prevent a compromised key from
initiating a transfer; it only delays it. The commit-reveal approach prevents
the transfer from completing without the new key's explicit participation.

### Alternative C — Multisig admin

Require M-of-N signatures for admin operations.

Rejected at this stage because Stellar's native multisig (account
signatories and thresholds) applies at the account level, not the contract
level. Implementing a contract-level multisig is a significantly larger
scope. This can be layered on top of the two-step pattern in a future release
if demand exists.

---

## References

- `contracts/safe-haven/src/contract.rs` — `transfer_admin`, `accept_admin`,
  `cancel_transfer_admin`, `renounce_admin`
- `contracts/safe-haven/src/storage.rs` — `VaultKey::Admin`,
  `VaultKey::PendingAdmin`
- `contracts/safe-haven/src/events.rs` — `adm_xfr_propose`, `adm_xfr_accept`,
  `adm_xfr_cancel`
- CHANGELOG — "Fixed: `accept_admin` correctly rejects calls after
  `cancel_transfer_admin` clears the pending admin (#28, #63)"
