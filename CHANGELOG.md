# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## How to Use This File

### For contributors

Every non-trivial PR **must** add an entry under `[Unreleased]` before it can be merged.
Add your entry to the appropriate sub-section. Use the imperative mood and link the issue
number in parentheses at the end (e.g. `(#42)`).

When in doubt, add an entry. It is easier to remove a line than to reconstruct history.

### For maintainers releasing a new version

1. Decide the next version number following the [Versioning Guide](#versioning-guide) below.
2. Replace `[Unreleased]` with `[X.Y.Z] - YYYY-MM-DD` (today's date in UTC).
3. Add a fresh empty `[Unreleased]` section at the top.
4. Update the comparison links at the bottom of this file.
5. Tag the commit: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`.

### What belongs in each section

| Section | When to use |
|---|---|
| `### Breaking Changes` | Any change that requires callers to update their code, deployment scripts, or integration |
| `### Added` | New public functions, new events, new storage keys, new CLI targets, new CI jobs |
| `### Changed` | Behaviour changes to existing functions that are backward-compatible |
| `### Deprecated` | Things that still work but will be removed in the next major version |
| `### Removed` | Things that no longer exist |
| `### Fixed` | Bug fixes — one line per issue, link the GitHub issue |
| `### Security` | Vulnerabilities fixed — these get a dedicated entry regardless of scope |

**Breaking Changes** is the most important section. If you add anything there, the major version must be bumped (see Versioning Guide).

---

## Versioning Guide

SAFE-HAVEN uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html): **MAJOR.MINOR.PATCH**

### When to bump each component

| Component | Bump when… | Examples |
|---|---|---|
| **MAJOR** | Any breaking change to the on-chain contract ABI, storage layout, or behaviour that requires existing integrations to change | Renaming a function, changing a parameter type, removing a function, changing error codes, altering the `VaultEntry` struct layout |
| **MINOR** | New backward-compatible functionality | New contract function, new event, new query, new Makefile target, new CI job |
| **PATCH** | Bug fixes and security patches that do not change the public API | Fix incorrect error code, fix TTL bump regression, fix frontend display bug |

### Pre-release and build metadata

- Pre-release versions: `1.0.0-alpha.1`, `1.0.0-rc.1`
- Testnet-only releases are tagged as pre-release until mainnet deployment is confirmed.
- The version in `contracts/safe-haven/Cargo.toml` must match the git tag.

### Smart contract versioning note

Because deployed Soroban contracts are **immutable**, a MAJOR version bump implies a new
contract deployment. Existing depositors on the old contract are unaffected and must migrate
voluntarily. Migration paths must be documented in the release notes.

---

## Release Notes Template

When cutting a new release, copy this template and fill it in above the previous release entry.
Delete sections that have no content for this release.

```markdown
## [X.Y.Z] - YYYY-MM-DD

> One-sentence summary of what this release is about.

### ⚠️ Breaking Changes

<!--
  List every change that requires callers / operators to update their code or deployment.
  Be specific: what changed, what the old behaviour was, and what the new behaviour is.
  Link a migration guide if the change is complex.
-->

- **`function_name` parameter order changed** — previously `(a, b, c)`, now `(a, c, b)`. Update all call sites. (#NNN)

### Added

- Short description of new feature — link to docs if the feature is non-obvious. (#NNN)

### Changed

- Short description of what changed and why. (#NNN)

### Deprecated

- `old_function_name` is deprecated in favour of `new_function_name`. Will be removed in vX+1.0.0.

### Removed

- `removed_function_name` — was deprecated since vX.Y.0. (#NNN)

### Fixed

- Short description of the bug and what was incorrect. (#NNN)

### Security

- Short description of the vulnerability class and impact, without details that could be
  weaponised against unpatched deployments. Full advisory: GHSA-xxxx-xxxx-xxxx. (#NNN)

---

### Migration Notes (if applicable)

Step-by-step instructions for users upgrading from the previous version.
Include contract redeployment steps, storage migration commands, and frontend update steps.

### Upgrade Checklist

- [ ] Run `make check` against the new version
- [ ] Deploy new contract to testnet and verify with `is_initialized`
- [ ] Update `VITE_CONTRACT_ID` in frontend `.env`
- [ ] Run smoke tests: `make smoke-test-local`
- [ ] Tag the release: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
```

---

## [Unreleased]

### Added

- `deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps)` — locks tokens until a specific Stellar ledger sequence number is reached instead of a wall-clock timestamp; shares the per-depositor deposit-ID counter with timestamp-based deposits (#88)
- `withdraw_to(depositor, deposit_id, recipient)` — withdraws unlocked tokens to an arbitrary recipient address rather than the depositor; supports both timestamp-based and ledger-based deposits
- `pause(admin)` / `unpause(admin)` — admin can halt new deposits without affecting existing ones; deposits rejected with `ContractPaused` while paused
- `is_paused()` — read-only query returning the current pause state
- `get_ledger_vault(depositor, deposit_id)` — returns the `LedgerVaultEntry` for a ledger-sequence-based deposit, or `None` if not found (#44)
- `get_vault_batch(depositors, deposit_id)` — fetches the same deposit ID across up to 25 depositors in a single RPC call
- `get_deposit_batch(depositor, deposit_ids)` — fetches up to 25 deposits for one depositor in a single RPC call, returning `(deposit_id, Option<VaultEntry>)` pairs
- `get_deposits_page(offset, limit)` — paginated flat list of all active timestamp-based deposits across every depositor, returned as `(depositor, deposit_id, VaultEntry)` triples; ledger-based deposits are excluded
- `get_storage_version()` — returns the on-chain schema version written by the last `migrate()` call, or `None` for pre-versioning deployments
- `migrate(admin)` — admin-only storage-migration hook; idempotent (returns `false` when already at current version, `true` when migration was applied); establishes the versioned schema pattern for future field additions
- `cancel_transfer_admin(admin)` now emits an `adm_xfr_cancel` event so off-chain indexers observe the cancellation immediately
- `contract_initialized` event emitted by `initialize`, carrying `(admin, fee_recipient, max_deposit, max_lock_secs)`
- `dep_by_ledger` event emitted by `deposit_by_ledger`, carrying `(amount, unlock_ledger, deposit_id)`
- `withdraw_to` event emitted by the new `withdraw_to` function, carrying `(recipient, amount)`
- `paused` / `unpaused` events emitted by `pause` / `unpause`
- `LedgerVaultEntry` contract type — mirrors `VaultEntry` but stores `unlock_ledger: u32` instead of `unlock_time: u64`
- `VaultKey::DepositByLedger(Address, u32)` storage key for ledger-based deposits
- `VaultKey::ActiveDepositIds(Address)` — explicit list of active deposit IDs per depositor, enabling O(1) lookups without scanning all counters
- `VaultKey::DepositorFlag(Address)` — O(1) active-depositor flag enabling `remove_depositor` without list compaction
- `VaultKey::DepositorInList(Address)` — append-once flag preventing duplicate entries in `DepositorList` across re-deposits
- `VaultKey::Paused` and `VaultKey::StorageVersion` storage keys
- `STORAGE_VERSION = 1` constant; serves as the baseline for future `migrate()` steps
- `MAX_BATCH_SIZE = 25` constant capping `get_vault_batch` and `get_deposit_batch` result sizes
- `time_remaining` now handles ledger-based deposits, returning estimated seconds as `remaining_ledgers × LEDGER_SECONDS` (#21)
- `withdraw` and `cancel_deposit` both handle ledger-based deposits transparently, checking `env.ledger().sequence() >= unlock_ledger`
- `emergency_withdraw` handles ledger-based deposits in addition to timestamp-based ones
- Minimum lock enforcement for `deposit_by_ledger`: `unlock_ledger` must exceed `current_ledger + MIN_LOCK_LEDGERS` (12 ledgers ≈ 60 s) (#139)
- `STORAGE_VERSION` constant in `types.rs` and `storage::get_storage_version` / `storage::set_storage_version` helpers
- Smoke-test script (`scripts/smoke_test_local.sh`) enhanced with `jq`-based value assertions
- Playwright end-to-end smoke test covering wallet connect and dashboard rendering
- Vitest unit tests for formatting utilities, Stellar SDK helpers, `DepositPage`, and `useDeposits` hook
- CI: frontend build and type-check job
- CI: `cargo-geiger` pinned to `0.11.7` with install caching
- CI: npm dependency caching in the frontend CI job
- CI: WASM artifact retention reduced to 7 days; artifacts uploaded as GitHub release assets on tag pushes
- CI: required status checks documented for branch protection

### Changed

- `withdraw` and `cancel_deposit` now attempt timestamp-based lookup first and fall back to ledger-based lookup, removing the need for callers to know which storage path holds their deposit
- `get_deposit_ids` returns IDs for both timestamp-based and ledger-based active deposits via the new `ActiveDepositIds` list
- `remove_depositor` is now O(1) using `DepositorFlag` instead of scanning and compacting `DepositorList`
- `depositor` field restored to `VaultEntry` (previously removed in #20) to make batch queries self-describing
- Storage TTL BUMP_THRESHOLD derived from `MAX_LOCK_DURATION_SECS` rather than a hardcoded constant, keeping the two values in sync
- `deposit_id` is now included in `dep_cancel` event data (#42)
- `emrg_wdraw` event now carries `deposit_id` in its data payload (#43)
- WASM size threshold unified to 65 536 bytes across all CI checks
- `actions/checkout` pinned to `v4` in `geiger`, `deny`, and `deploy-testnet` CI jobs (previously referenced non-existent `v6`)
- Shell variables in the `check-wasm-size` Makefile target are now properly escaped to prevent unintended Make expansion
- Duplicate `cargo fmt` check removed from the `pr-title-lint` CI job
- Duplicate WASM size check removed from CI; single authoritative check remains

### Fixed

- `accept_admin` correctly rejects calls after `cancel_transfer_admin` clears the pending admin (#28, #63)
- `time_remaining` returns `0` for nonexistent deposits instead of panicking
- Depositor pagination overflow when `offset + limit` exceeds the depositor list length (#140)
- `get_deposits_page` skips depositors whose `DepositorFlag` has been cleared, preventing stale entries in paginated results

### Security

- `cancel_transfer_admin` emits `adm_xfr_cancel` event, ensuring off-chain monitors cannot observe a stale pending admin after cancellation

---

## [0.1.0] - 2026-05-31

> Initial production release — timestamp-based vault with admin controls and full CI.

### Added

#### Contract functions

- `initialize(admin, fee_recipient, max_deposit?, max_lock_secs?)` — one-time setup; sets admin and fee recipient; optionally overrides compile-time limits for `MAX_DEPOSIT_AMOUNT` and `MAX_LOCK_DURATION_SECS`
- `deposit(depositor, token, amount, unlock_time, penalty_bps)` — locks tokens until `unlock_time`; returns a per-depositor `deposit_id`
- `deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps)` — third-party `payer` funds a vault on behalf of `depositor`
- `withdraw(depositor, deposit_id)` — returns the full locked amount to the depositor once `unlock_time` has passed
- `cancel_deposit(depositor, deposit_id)` — early exit before unlock; applies `penalty_bps` penalty sent to `fee_recipient`, remainder returned to depositor
- `emergency_withdraw(admin, depositor, deposit_id)` — admin-only; returns funds to the depositor regardless of lock time; funds always go to the depositor, never to the admin
- `transfer_admin(admin, new_admin)` — step 1 of two-step admin transfer; nominates a pending admin
- `accept_admin(new_admin)` — step 2; pending admin accepts and becomes the active admin
- `cancel_transfer_admin(admin)` — cancels a pending admin transfer
- `renounce_admin(admin)` — permanently removes admin privileges; makes the vault fully trustless
- `get_vault(depositor, deposit_id)` — returns the vault entry without bumping TTL
- `get_deposit_ids(depositor)` — returns all active deposit IDs for a depositor
- `time_remaining(depositor, deposit_id)` — seconds until unlock; `0` if already unlocked or not found
- `get_time()` — current ledger timestamp
- `get_admin()` — current admin, or `None` if renounced
- `get_pending_admin()` — pending admin during a transfer, or `None`
- `get_fee_recipient()` — fee recipient set at initialization
- `get_constants()` — effective `(MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS)` for this deployment
- `get_depositor_count()` — total number of addresses with an active deposit
- `get_depositors(offset, limit)` — paginated list of active depositor addresses
- `is_initialized()` — whether `initialize` has been called

#### Protocol constants

- `MAX_DEPOSIT_AMOUNT = 1_000_000_000_000_000` (10^15 token base units)
- `MAX_LOCK_DURATION_SECS = 157_788_000` (~5 years)
- `MIN_LOCK_DURATION_SECS = 60` (60 seconds)

#### Error codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `InvalidAmount` | Amount ≤ 0 |
| 2 | `UnlockTimeNotInFuture` | `unlock_time` ≤ current ledger time |
| 3 | `NoDepositFound` | No active deposit for this depositor/id |
| 4 | `FundsStillLocked` | Lock period not yet expired |
| 5 | `DepositAlreadyExists` | Reserved; never emitted |
| 6 | `LockDurationTooLong` | Lock period exceeds `MAX_LOCK_DURATION_SECS` |
| 7 | `Unauthorized` | Caller is not the admin or pending admin |
| 8 | `AmountTooLarge` | Amount exceeds `MAX_DEPOSIT_AMOUNT` |
| 9 | `InvalidPenaltyBps` | `penalty_bps` > 10 000 |
| 10 | `InvalidAdmin` | Nominated admin is the same as the current admin |
| 11 | `LockDurationTooShort` | Lock period shorter than `MIN_LOCK_DURATION_SECS` |
| 12 | `ContractPaused` | Deposits are paused |
| 13 | `VaultAlreadyUnlocked` | `cancel_deposit` called after the lock has already expired |
| 14 | `MissingFeeRecipient` | `penalty_bps` > 0 but no fee recipient is configured |

#### Events

| Event | Topics | Data |
|-------|--------|------|
| `deposit` | `("deposit", depositor, token)` | `(amount, unlock_time, deposit_id)` |
| `withdraw` | `("withdraw", depositor, token)` | `(amount, deposit_id)` |
| `emrg_wdraw` | `("emrg_wdraw", depositor)` | `(admin, token, amount, deposit_id)` |
| `dep_cancel` | `("dep_cancel", depositor, token)` | `(amount, penalty, deposit_id)` |
| `adm_xfr_init` | `("adm_xfr_init", current_admin)` | `pending_admin` |
| `adm_xfr_done` | `("adm_xfr_done", new_admin)` | `()` |
| `adm_renounce` | `("adm_renounce", former_admin)` | `()` |

#### Storage

- All entries use Persistent Storage with a TTL bump threshold of ~30 days (`518_400` ledgers) and a bump target of ~5.2 years (`33_000_000` ledgers)
- Storage keys: `Admin`, `PendingAdmin`, `Initialized`, `FeeRecipient`, `MaxDeposit`, `MaxLockSecs`, `DepositCounter(depositor)`, `Deposit(depositor, id)`, `DepositorList`

#### Security properties

- Checks-Effects-Interactions pattern enforced on every withdrawal path; storage cleared before any token transfer
- `require_auth()` is the first statement in every mutating function
- No re-entrancy surface; state removed before external token calls
- Bounded inputs: amount capped at 10^15, lock duration capped at 5 years with a 60-second minimum
- Emergency withdraw always sends funds to the depositor, never to the admin
- Two-step admin transfer prevents accidental key loss
- Admin can permanently renounce privileges for fully trustless operation
- `features = ["testutils"]` only in `[dev-dependencies]`; testutils never compiled into production WASM

#### Infrastructure

- Rust workspace with `soroban-sdk v22`
- Makefile targets: `build`, `test`, `check`, `optimize`, `check-wasm-size`, `audit`, `deny`, `deploy-testnet`, `smoke-test-local`
- CI (GitHub Actions): lint → test → build WASM → check WASM size
- Testnet deploy script with atomic deploy + initialize to prevent front-running
- `rust-toolchain.toml` pinning stable Rust with the `wasm32-unknown-unknown` target
- `deny.toml` for license allowlist and dependency ban policy
- 48+ unit tests covering all functions, error codes, and boundary conditions

---

[Unreleased]: https://github.com/kenedybok3/SAFE-HAVEN/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kenedybok3/SAFE-HAVEN/releases/tag/v0.1.0
