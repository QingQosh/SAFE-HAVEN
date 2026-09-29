# SAFE-HAVEN

[![Rust](https://img.shields.io/badge/Rust-1.81%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Soroban SDK](https://img.shields.io/badge/Soroban-SDK%20v22-blue?logo=stellar)](https://github.com/stellar/rs-soroban-sdk)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)
[![Tests](https://github.com/kenedybok3/SAFE-HAVEN/actions/workflows/ci.yml/badge.svg)](https://github.com/kenedybok3/SAFE-HAVEN/actions)

A production-ready decentralized vault on the Stellar blockchain (Soroban) — with a full React/TypeScript frontend.

Tokens are locked in the smart contract until a future timestamp. Early exits are possible with a configurable penalty. Admin rights can be permanently renounced for fully trustless operation.

---

## Project Structure

```
SAFE-HAVEN/
├── contracts/safe-haven/       Smart contract (Rust / Soroban)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              Crate root
│       ├── contract.rs         All public entry points
│       ├── types.rs            VaultKey, VaultEntry, constants
│       ├── errors.rs           VaultError enum (14 codes)
│       ├── events.rs           Event emission helpers
│       ├── storage.rs          Persistent storage + TTL helpers
│       └── test.rs             48+ unit tests
│
├── frontend/                   React + TypeScript + Vite (UI)
│   ├── src/
│   │   ├── App.tsx
│   │   ├── config.ts           Contract ID, RPC URLs
│   │   ├── context/            Freighter wallet
│   │   ├── hooks/              useDeposits, useContractInfo
│   │   ├── lib/                Stellar SDK helpers, formatting
│   │   ├── components/         Header, TabNav, DepositCard, etc.
│   │   └── pages/              Dashboard, Deposit, Withdraw, Admin
│   ├── .env.example
│   └── README.md
│
├── Cargo.toml                  Rust workspace
├── Makefile                    Build / test / lint / deploy
└── STRUCTURE.md                Detailed project layout
```

---

## Quick Start

### Smart Contract

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli

# Optional: jq for JSON parsing in smoke tests (apt-get install jq / brew install jq)

# One-shot: install all recommended dev tools
make install-tools

# Build
make build

# Test
make test

# Run all tests on file change (requires cargo-watch, installed by make install-tools)
make watch

# Full local dev environment (build + deploy + frontend)
make dev

# Deploy to testnet
export SOROBAN_SECRET_KEY=S...
make deploy-testnet

# Deploy to mainnet (the deployer must already be funded; no mainnet faucet exists)
export SOROBAN_SECRET_KEY=S...
make deploy-mainnet

# Redeploy a retained immutable WASM as a new contract ID
export SOROBAN_SECRET_KEY=S...
make rollback NETWORK=testnet ARTIFACT_DIR=deployments/testnet/<timestamp>
```

### Frontend

```bash
cd frontend
npm install
cp .env.example .env   # set VITE_CONTRACT_ID to your deployed contract
npm run dev            # -> http://localhost:5173
```

See [`frontend/README.md`](./frontend/README.md) for the full frontend guide.

See [`DISASTER_RECOVERY.md`](./DISASTER_RECOVERY.md) for disaster scenarios, recovery procedures, roles, and escalation rules.

See [`MONITORING.md`](./MONITORING.md) for contract health checks, alert thresholds, storage/TTL monitoring, and failed-transaction observability.

See [`LEGAL.md`](./LEGAL.md) for Terms of Service, Privacy Policy, disclaimers, GDPR compliance, data retention policy, and the on-chain audit trail reference.

See [`SUPPORT.md`](./SUPPORT.md) for the version lifecycle, maintenance and support matrix, bug-fix SLAs by severity, and EOL policy.

See [`BRANCHING.md`](./BRANCHING.md) for the branching strategy, naming conventions, branch protection rules, release checklist, hotfix procedure, and rollback procedures.

See [`POSTMORTEM.md`](./POSTMORTEM.md) for the incident post-mortem process, blameless culture guidelines, post-mortem template, action item tracking, and quarterly review process.

Deployment artifacts are written to `deployments/<network>/<timestamp>/`, including raw and optimized WASM, contract ID, manifest, and checksum. Each deployment also updates `deploy_<network>.log` for CI compatibility. Soroban contracts are immutable, so rollback means deploying the retained previous WASM as a new contract and updating the frontend contract ID after verification.

---

## Overview

| Property | Value |
|---|---|
| Network | Stellar (Soroban) |
| Language | Rust |
| SDK | soroban-sdk v22 |
| Storage | Persistent (per-depositor) |
| Max deposit | 10^15 units |
| Max lock duration | 5 years |
| Min lock duration | 60 seconds |
| Early-exit penalty | 0-100% (basis points, set at deposit time) |

---

## How It Works

1. **Deposit** - User calls `deposit(token, amount, unlock_time, penalty_bps)` — tokens transfer into the contract
2. **Storage** - Contract stores a `VaultEntry` in persistent storage keyed by `(depositor, deposit_id)`
3. **Verification** - On `withdraw()`, contract checks `ledger.timestamp() >= unlock_time`
4. **Unlock** - Tokens returned to depositor. Otherwise call fails with `FundsStillLocked`
5. **Early exit** - `cancel_deposit()` returns funds minus penalty; penalty goes to `fee_recipient`
6. **Admin recovery** - Admin can emergency-withdraw any deposit (funds always go to depositor, never admin)
7. **Trustless mode** - Admin can be permanently renounced via `renounce_admin()`

---

## MEV Protection

SAFE-HAVEN includes built-in **MEV (Maximal Extractable Value) protection** to prevent sandwich attacks and front-running. The system uses a **commit-reveal scheme** combined with **time-weighted average pricing (TWAP)** to detect and mitigate MEV extraction.

### Key Features

- **Commit-Reveal Scheme** — Depositors privately commit transaction details (via hash), then reveal after the commit is immutable. Prevents attackers from predicting and front-running orders.
- **Price Monitoring** — Continuous tracking of token prices; stores up to 24 hours of hourly price samples.
- **Sandwich Attack Detection** — Compares current price against TWAP. Deviations exceeding 200 basis points (2%) trigger attack detection.
- **MEV Recovery & Redistribution** — Recovered value is credited to affected users' MEV pools, which they can claim individually.
- **Transparent Event Logging** — All MEV detections, commits, and recoveries are emitted as on-chain events for audit and monitoring.

### MEV Protection API

| Function | Purpose |
|---|---|
| `mev_commit(depositor, deposit_id, commit_hash)` | Submit a private commit to enable MEV protection |
| `mev_reveal(depositor, deposit_id, token, amount, price, nonce)` | Reveal the commit and trigger MEV detection |
| `claim_mev_recovery(depositor)` | Claim recovered MEV from the pool |
| `get_mev_status_query(depositor, deposit_id)` | Query MEV protection status (`Unprotected`, `Committed`, `Revealed`, `AttackDetected`) |
| `get_mev_detections(depositor, deposit_id, offset, limit)` | Paginated view of detected attacks |
| `get_mev_pool_total()` | Query total MEV pool (pending distribution) |
| `finalize_mev_redistribution(admin)` | Admin function to finalize MEV redistribution |

### Example: MEV Protection Workflow

```
1. Alice deposits 1000 USDC with 1% penalty, locked for 1000 seconds
   → deposit_id = 0

2. Alice submits MEV commit (private order details via hash):
   commit_hash = Keccak256(usdc_addr || 1000 || 102 || nonce)
   mev_commit(alice, 0, commit_hash)
   → Status: Committed

3. Within 30 minutes, Alice reveals her actual transaction:
   mev_reveal(alice, 0, usdc_addr, 1000, 102, nonce)
   → Contract verifies hash matches
   → Status: Revealed
   → Price sample recorded

4. Contract detects sandwich attack via TWAP:
   If current_price (102) deviates from TWAP (100) by > 200 bps
   → MEV attack detected!
   → 2 USDC recovered and added to MEV pool
   → Event: mev_detected(alice, 0, 200, 2)

5. Alice claims her recovery:
   mev_recovered = claim_mev_recovery(alice)
   → Returns: 2 USDC
   → Tokens transferred to alice
```

### Configuration

| Constant | Value | Description |
|---|---|---|
| `MEV_REVEAL_WINDOW_SECS` | 1,800 (30 min) | Time allowed to reveal after commit |
| `MEV_PRICE_DEVIATION_THRESHOLD_BPS` | 200 (2%) | Deviation threshold to trigger detection |

For more details, see **[MEV_PROTECTION.md](./MEV_PROTECTION.md)**.

---

## Contract API

### Initialization

#### `initialize(admin, fee_recipient, max_deposit?, max_lock_secs?)`
One-time setup. Sets admin and fee recipient. Optionally overrides compile-time limits.

---

### Core Functions

#### `deposit(depositor, token, amount, unlock_time, penalty_bps) -> u32`
Locks tokens. Returns the deposit ID.

#### `deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps) -> u32`
Payer funds a vault for a different beneficiary.

#### Smart wallets and delegated deposits
Smart wallets use their contract address as `depositor`; the vault relies on Soroban `require_auth()` and does not require an EOA. Multisig and account-abstraction wallets keep their own approval policy and authorize the vault and token calls.

For time-bounded delegated funding, use `authorize_session_key`, `deposit_with_session_key`, and `revoke_session_key`. The session key is the payer, while the smart wallet remains the deposit owner. See [Smart Wallet Integration](docs/SMART_WALLETS.md) for the security model and integration checklist.

#### `deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps) -> u32`
Locks tokens until a specific Stellar ledger sequence number is reached, instead of a wall-clock timestamp.

Use this when you need to express a lock period in terms of on-chain ledger progression — for example, "release after the network has produced exactly N more ledgers" — rather than relying on the ledger's timestamp field.

**Parameters**

| Parameter | Type | Description |
|---|---|---|
| `depositor` | `Address` | Account locking the tokens. Must sign the transaction. |
| `token` | `Address` | SAC-compatible token contract address. |
| `amount` | `i128` | Amount to lock (> 0, ≤ `max_deposit`). |
| `unlock_ledger` | `u32` | Ledger sequence number at or after which withdrawal is permitted. Must be > `current_ledger + 12` (minimum gap of 12 ledgers ≈ 60 seconds at 5 s/ledger). |
| `penalty_bps` | `u32` | Early-exit penalty in basis points (0–10000). Requires a `fee_recipient` to be configured if > 0. |

**Returns** the deposit ID (`u32`), shared with the same per-depositor counter as timestamp-based deposits.

**Withdrawal** — `withdraw()` and `withdraw_to()` both accept ledger-based deposits. On withdrawal the contract checks `env.ledger().sequence() >= unlock_ledger`; if the ledger sequence has not yet reached the target the call fails with `FundsStillLocked`.

**Estimating wall-clock time from a ledger number** — Stellar produces a new ledger roughly every 5 seconds. To approximate the unlock time in seconds from the current ledger:

```
estimated_seconds = (unlock_ledger - current_ledger) × 5
```

> **⚠️ IMPORTANT: This is an ESTIMATE, not a guaranteed prediction.**
>
> - Actual ledger close times vary by ±1-2 seconds depending on network conditions
> - The 5-second value is a Stellar network consensus target, not a hard guarantee
> - Use this estimate for UI display and rough scheduling only
> - Do **not** rely on this for precise, critical timing — use timestamp-based deposits instead
> - When withdrawal is called, the actual check is `current_ledger >= unlock_ledger`, which is exact
> - The `time_remaining(depositor, id)` query returns `(remaining_ledgers × 5)` for ledger-based deposits, which is the same estimate
>
> See the **"Ledger-Based Deposit Time Estimation: Precision & Confidence"** section below for a detailed decision tree, use-case guidance, and accuracy details.

#### `withdraw(depositor, deposit_id)`
Withdraws if `now >= unlock_time`.

#### `withdraw_to(depositor, deposit_id, recipient)`
Withdraws to a different address.

#### `cancel_deposit(depositor, deposit_id)`
Early exit with penalty. Penalty is split: 30% goes to fee_recipient, 70% accumulates in the staker rewards pool; remainder returned to depositor.

---

### Staker Registry Functions

The staker registry allows users to register as stakers and earn a portion of penalties accrued from early deposit exits. This creates an incentive mechanism where stakers share in the penalties paid by users who exit early.

#### `register_staker(staker, amount) -> Result<(), VaultError>`
Register or update a staker's stake amount.

**Parameters**

| Parameter | Type | Description |
|---|---|---|
| `staker` | `Address` | The staker's address. Must sign the transaction. |
| `amount` | `i128` | Stake amount in contract tokens (> 0). |

**Returns** `Ok(())` on success, or an error code if validation fails.

**Behavior**
- Validates that `amount > 0` — zero or negative amounts are rejected with `InvalidStakeAmount`.
- Updates the staker's entry if already registered, or creates a new entry.
- Maintains `total_staked` (sum of all registered stakes) for proportional reward calculation.
- Adds the staker to the staker list on first registration.
- Emits a `StakerRegistered` event.
- Requires authentication from the staker.

**Example**
```rust
// Alice registers with 1000 tokens as stake
vault.register_staker(&alice, &1000)?;

// Later, Alice increases her stake to 2000
vault.register_staker(&alice, &2000)?;
```

#### `claim_staker_rewards(staker) -> Result<(), VaultError>`
Claim accumulated rewards from the staker rewards pool.

**Parameters**

| Parameter | Type | Description |
|---|---|---|
| `staker` | `Address` | The staker claiming rewards. Must sign the transaction. |

**Returns** `Ok(())` on success, or an error code if no rewards are available or staker is not registered.

**Behavior**
- Validates that the staker is registered — returns `StakerNotFound` if not.
- Calculates the staker's proportional share: `(stake_amount / total_staked) × rewards_pool`.
- Validates that the calculated reward is > 0 — returns `NoRewardsToClaim` if the pool is empty or the proportion rounds to zero.
- Deducts the reward from the rewards pool.
- Tracks cumulative rewards claimed per staker for auditing.
- Emits a `RewardsClaimed` event.
- Requires authentication from the staker.

**Example**
```rust
// Alice claims her proportional share of the rewards pool
vault.claim_staker_rewards(&alice)?;

// If Alice has stake 1000 and total_staked is 4000, and rewards_pool is 700:
// Alice's reward = (1000 / 4000) × 700 = 175 tokens
```

---

### Penalty Splitting & Rewards Pool

When a user calls `cancel_deposit()` to exit early, the penalty is split as follows:

| Recipient | Percentage | Basis Points |
|---|---|---|
| Fee Recipient | 30% | 3000 bps |
| Staker Rewards Pool | 70% | 7000 bps |

**Example**
If a user cancels a deposit with 100 tokens penalty (10% of 1000):
- Fee Recipient receives 30 tokens (100 × 0.30)
- Staker Rewards Pool receives 70 tokens (100 × 0.70)

Registered stakers can then claim their proportional share of the rewards pool based on their stake amount relative to total staked.

---

### Admin Functions

#### `emergency_withdraw(admin, depositor, deposit_id)`
Admin-only. Returns funds to depositor regardless of lock time.

#### `pause(admin)` / `unpause(admin)`
Halts / restores deposits.

#### `transfer_admin(admin, new_admin)` / `accept_admin(new_admin)`
Two-step admin transfer.

#### `cancel_transfer_admin(admin)`
Cancels a pending transfer.

#### `renounce_admin(admin)`
Permanently removes admin. Contract becomes fully trustless.

---

### Read-only Queries

| Function | Returns |
|---|---|
| `get_vault(depositor, id)` | `Option<VaultEntry>` |
| `get_deposit_ids(depositor)` | `Vec<u32>` |
| `time_remaining(depositor, id)` | `u64` seconds |
| `get_time()` | Current ledger timestamp |
| `version()` | `String` — contract version from Cargo.toml |
| `get_admin()` | `Option<Address>` |
| `get_pending_admin()` | `Option<Address>` |
| `get_fee_recipient()` | `Option<Address>` |
| `get_constants()` | `(max_deposit, max_lock_secs)` |
| `get_depositor_count()` | `u32` |
| `get_depositors(offset, limit)` | `Page<Address>` — items + total_count |
| `is_paused()` | `bool` |
| `is_initialized()` | `bool` |

---

## Sustainability Metrics

SAFE-HAVEN tracks environmental impact across all deposits to support ESG goals and climate commitments.

### Carbon Footprint Tracking

Each deposit automatically calculates a **carbon footprint** based on:
- **Amount locked** — larger deposits = higher impact
- **Lock duration** — longer locks = higher impact
- **Baseline multiplier** — configurable environmental cost per unit-second (default: 1 gram CO2e per unit per second)

**Formula**: `carbon_footprint = amount × duration_seconds × CARBON_BASELINE_PER_UNIT_SECOND`

### Renewable Energy Percentage

Every deposit tracks an assumed **renewable energy percentage** (default: 50%). This represents the energy mix used for hosting and network operations.

### Carbon Offset Integration

Users can configure a **penalty basis points** (0–10,000) that simultaneously:
1. **Penalizes early withdrawals** — funds are sent to the fee recipient
2. **Offsets carbon** — penalty % is converted to CO2 offset

**Formula**: `carbon_offset = carbon_footprint × (penalty_bps / 10,000)`

### Sustainability Metrics Queries

#### `get_sustainability_metrics(depositor, deposit_id) -> Option<SustainabilityMetrics>`
Retrieves calculated metrics for a single deposit:
```rust
pub struct SustainabilityMetrics {
    pub deposit_id: u32,
    pub depositor: Address,
    pub carbon_footprint: i128,           // grams CO2e
    pub renewable_energy_percent: u32,   // 0-100%
    pub carbon_offset_grams: i128,       // grams CO2 offset
    pub timestamp: u64,                  // when recorded
}
```

#### `generate_sustainability_report(depositor) -> SustainabilityReport`
Aggregates metrics across all active deposits:
```rust
pub struct SustainabilityReport {
    pub depositor: Address,
    pub total_carbon_footprint: i128,        // sum across all deposits
    pub average_renewable_energy_percent: u32, // weighted average
    pub total_carbon_offset: i128,           // sum of all offsets
    pub active_deposit_count: u32,           // number of open vaults
    pub report_timestamp: u64,
}
```

### Sustainability Milestones

The contract emits `SustainabilityMilestone` events when depositors achieve environmental goals:

| Milestone | Condition |
|---|---|
| **Carbon Neutral** | 100% of carbon footprint is offset |
| **High Renewable** | ≥ 75% renewable energy across deposits |
| **Carbon Negative** | Total offset exceeds total footprint |
| **Large Offset** | Total offset ≥ 1 billion grams CO2e (1,000 tonnes) |

**Behavior**: Each milestone is emitted at most once per depositor (using a bitmap flag). Once achieved, the same milestone will not emit again unless reset.

### Lifecycle of Metrics

| Event | Action |
|---|---|
| **Deposit** | Metrics calculated, stored, aggregates updated, milestones checked |
| **Withdraw** | Metrics removed from storage (no longer needed) |
| **Cancel** | Metrics removed, aggregates decremented |
| **Query** | `get_sustainability_metrics()` and `generate_sustainability_report()` read-only (no writes) |

### Example: Tracking a Sustainable Vault

```
1. Alice deposits 1,000 USDC for 100 days with 50% early-exit penalty
   → carbon_footprint = 1,000 × 8,640,000 seconds × 1 = 8.64 billion grams CO2e
   → carbon_offset = 8.64 billion × 0.5 = 4.32 billion grams (50% offset)

2. She makes a second deposit of 500 USDC for 50 days with 100% penalty
   → carbon_footprint = 500 × 4,320,000 × 1 = 2.16 billion grams
   → carbon_offset = 2.16 billion (full offset)

3. Her sustainability report shows:
   → total_carbon_footprint = 10.8 billion grams
   → total_carbon_offset = 6.48 billion grams
   → average_renewable_energy = 50%
   → SustainabilityMilestone::HighRenewable is NOT emitted (50% < 75% threshold)
   → SustainabilityMilestone::LargeOffset IS emitted (6.48B > 1B threshold)
```

---

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `InvalidAmount` | Amount <= 0 |
| 2 | `UnlockTimeNotInFuture` | unlock_time (or unlock_ledger) <= current value |
| 3 | `NoDepositFound` | No active deposit at the given (depositor, deposit_id) |
| 4 | `FundsStillLocked` | Lock not yet expired |
| 5 | `DepositAlreadyExists` | Reserved — defined in the enum to hold the slot and prevent future code-number collisions, but never emitted by any current code path |
| 6 | `LockDurationTooLong` | Exceeds 5 years |
| 7 | `Unauthorized` | Caller is not the admin (or contract is not initialized) |
| 8 | `AmountTooLarge` | Exceeds 10^15 |
| 9 | `InvalidPenaltyBps` | penalty_bps > 10000 |
| 10 | `InvalidAdmin` | Proposed new admin is same as current admin |
| 11 | `LockDurationTooShort` | Lock duration < 60 seconds (or < 12 ledgers for `deposit_by_ledger`) |
| 12 | `ContractPaused` | Deposits are paused |
| 13 | `VaultAlreadyUnlocked` | `cancel_deposit` called after the lock has already expired |
| 14 | `MissingFeeRecipient` | penalty_bps > 0 but no fee_recipient is configured |
| 15 | `AlreadyInitialized` | `initialize` was called on an already-initialized contract |
| 16 | `InvalidStakeAmount` | Staker registration with amount <= 0 |
| 17 | `StakerNotFound` | Staker not registered in the staker registry |
| 18 | `NoRewardsToClaim` | Rewards pool is empty or staker's share rounds to zero |
| 19 | `InsufficientStakeAmount` | Insufficient staked amount for operation |

---

## Security Properties

| Property | Implementation |
|---|---|
| Checks-Effects-Interactions | Storage cleared before token transfer on every withdrawal |
| Auth-first | `require_auth()` is the first call in every mutating function |
| No re-entrancy | State removed before any external token call |
| Bounded inputs | Amount capped at 10^15; lock 60s-5yr |
| No admin theft | Emergency withdraw always sends to depositor |
| Trustless mode | `renounce_admin()` permanently removes admin |
| Safe transfer | Two-step admin transfer prevents key loss |

---

## Soroban Developer Notes

### Ledger TTL and Storage Expiry

Soroban uses a **time-to-live (TTL)** system for all persistent storage entries. Every entry written to the ledger has a limited lifespan — measured in ledgers, not wall-clock time — after which it **expires and is pruned**. If a storage entry expires, reading it returns `None` as if it were never written.

SAFE-HAVEN mitigates this by bumping TTL on every storage write and read that matters:

- **On write**: every `set_*` helper calls `extend_ttl` with a `BUMP_TARGET` that covers the maximum lock duration (~5 years) plus a `BUMP_THRESHOLD` buffer.
- **On read**: the `get_deposit` (mutable) helper also extends TTL. The `*_readonly` variants do *not* extend TTL — they are used by queries that should not incur a write-cost.
- **Edge case**: a deposit that sits untouched for longer than `BUMP_TARGET` ledgers (~31.5M ledgers) could theoretically expire. In practice this exceeds the maximum lock duration, and any withdrawal attempt would need to re-create the entry via a migration or re-deposit.

**Takeaway**: always use the write-path `get_deposit` (not `get_deposit_readonly`) for operations that will later mutate the entry. Read-only queries (like `get_vault`) use the readonly variant to avoid unnecessary ledger writes.

### Instruction Budget Limits

Every Soroban transaction has a **CPU instruction budget** (default ~100M instructions for testnet, ~50M for mainnet). Functions that iterate over unbounded collections can exhaust this budget and fail mid-execution.

SAFE-HAVEN's paginated views respect this limit:

- `get_depositors(offset, limit)` and `get_deposits_page(offset, limit)` use a `limit` parameter and stop early. Keep `limit` ≤ 50 in production.
- `get_vault_batch(depositors, deposit_id)` and `get_deposit_batch(depositor, deposit_ids)` clamp input size to `MAX_BATCH_SIZE` (25).
- The `DepositorList` can grow large over time, but `remove_depositor` is O(1) — it clears only a flag. The list is append-only and stale entries are skipped during enumeration.

**Takeaway**: always paginate with reasonable limits. Don't attempt to fetch all deposits or all depositors in a single RPC call.

### Simulation vs. Submission

Soroban distinguishes between two phases of a transaction:

1. **Simulation** — the RPC dry-runs your transaction against the current ledger state to compute the exact footprint, resource fees, and result. Simulation is read-only and free.
2. **Submission** — the signed transaction is broadcast to the network. This consumes real resources and costs fees.

Key implications:

- A simulation that succeeds does **not** guarantee submission will succeed. Ledger state can change between simulation and submission (e.g., another transaction withdraws the funds you were about to claim).
- Always simulate before submitting to catch errors early and estimate fees.
- The frontend's `stellar.ts` helpers simulate first, then submit, and surface any discrepancy as an error.

### `require_auth()` Must Be First

In Soroban, **`require_auth()` must be the very first call in every mutating (non-readonly) contract function**. Calling it after storage reads, transfers, or other operations is an anti-pattern that can lead to:

- **Wasted compute**: if auth fails, all preceding work is discarded but still counted against the instruction budget.
- **Re-entrancy risk**: performing state changes before auth verification opens a window for re-entrant calls.

SAFE-HAVEN enforces this convention: every mutating function calls `caller.require_auth()` as its first meaningful statement (after the function signature). The `Security Properties` table above documents this as "Auth-first".

**Takeaway**: when adding new mutating functions, always put `require_auth()` first. The contract's security model depends on it.

---

## Ledger-Based Deposit Time Estimation: Precision & Confidence

When using `deposit_by_ledger()`, the contract stores a target ledger sequence number. To estimate when that ledger will close in wall-clock time, use the formula:

```
estimated_seconds = (unlock_ledger - current_ledger) × 5
```

### How accurate is this estimate?

**Excellent for rough UI display** (±5–10 seconds over typical durations):
- Stellar's consensus layer targets a 5-second ledger close time on average
- Over a 1-hour lock (720 ledgers), the estimate will typically be within ±2–3 minutes
- For long locks (days or weeks), the relative error shrinks further

**Not suitable for precise scheduling**:
- Actual ledger close times vary by ±1–2 seconds due to network conditions, validator clock skew, and consensus timeouts
- Over-the-counter exchanges, time-sensitive payment settlements, or critical business logic should NOT rely on this estimate
- For exact timing, use timestamp-based deposits (`deposit()`) instead

### When should I use `time_remaining()` for ledger-based deposits?

**Safe to use**:
- Display in a UI dashboard showing "roughly X seconds remaining"
- Showing a progress bar for visual feedback
- Non-critical notifications like "deposit will unlock soon"

**Not safe to use**:
- Calculating precise interest accrual or compounding
- Triggering critical financial workflows ("execute settlement exactly when deposit unlocks")
- Legal or compliance deadlines that require exact timestamps
- Any system that cannot tolerate ±2–5 seconds of error

### Decision tree: Which deposit type should I use?

```
Do you need EXACT wall-clock precision?
├─ YES  → Use deposit() or deposit_for() (timestamp-based)
│        The unlock check is exact: env.ledger().timestamp() >= unlock_time
│
└─ NO   → Do you need to express the lock in ledger terms?
         (e.g., "release after block 12,345,678")?
         ├─ YES  → Use deposit_by_ledger()
         │        Unlock check is exact: env.ledger().sequence() >= unlock_ledger
         │        time_remaining() will return an estimate ±1–2 seconds
         │
         └─ NO   → Use deposit() or deposit_for() (timestamp-based)
                   Simpler, widely supported in the UI, no approximation needed
```

### Implementation Details

The 5-second estimate is defined in `storage.rs::LEDGER_SECONDS`:

```rust
pub const LEDGER_SECONDS: u64 = 5;
```

This constant is used by:
- `contract.rs::time_remaining()` — returns `remaining_ledgers × LEDGER_SECONDS` for ledger-based deposits
- `constants.rs::MIN_LOCK_LEDGERS` — enforces minimum 12 ledgers (~60 seconds)
- Storage TTL calculations — ensures vault state persists longer than maximum lock duration

The estimate formula is deterministic and does not change; network delays may cause the actual ledger close to drift slightly, but the formula itself is reliable for UI purposes.

---

## Known Limitations

The following gaps apply specifically to `deposit_by_ledger` deposits. All other deposit types (`deposit`, `deposit_for`) are unaffected.

| Limitation | Detail |
|---|---|
| **No frontend support** | The React UI only exposes `deposit` and `deposit_for`. Ledger-based deposits must be made via the Stellar CLI or a custom SDK integration. |
| **No maximum lock duration** | `deposit` and `deposit_for` reject lock durations longer than `max_lock_secs` (default 5 years). `deposit_by_ledger` only enforces a *minimum* gap of 12 ledgers (`MIN_LOCK_LEDGERS`). There is no equivalent upper-bound check on `unlock_ledger`, so arbitrarily far-future ledger numbers are accepted. |
| **`get_vault` returns `None`** | The `get_vault(depositor, id)` query only searches timestamp-based entries. To retrieve a ledger-based deposit, use `get_ledger_vault(depositor, id)` which returns `Option<LedgerVaultEntry>`. |
| **`time_remaining` is an estimate** | For ledger-based deposits, `time_remaining` returns `remaining_ledgers × 5` seconds. This is an approximation because actual ledger close times vary by ±1-2 seconds and are not exactly 5 seconds. **Do not rely on this value for precise scheduling or critical timing.** Use this estimate for UI display and rough scheduling only. The actual withdrawal check (`current_ledger >= unlock_ledger`) is exact and will work correctly. |
| **`get_deposits_page` excludes ledger-based deposits** | The paginated flat deposits view only iterates over timestamp-based `VaultEntry` records. To enumerate ledger-based deposits, use `get_depositors` + `get_deposit_ids` + `get_ledger_vault`. |

These limitations are tracked as open issues and will be addressed in future releases.

---

```bash
make build            # Compile to WASM
make test             # Run all tests
make watch            # Auto-run tests on file change
make lint             # Clippy
make fmt              # Format
make check            # fmt + lint + test + audit + deny
make optimize         # Optimize WASM with soroban CLI
make check-wasm-size  # Fail if WASM > 64 KB
make dev              # Build + deploy locally + start frontend
make deploy-testnet   # Deploy to Stellar testnet
make smoke-test-local # End-to-end test against local node
make install-tools    # Install all recommended dev tools
make audit            # cargo audit (security)
make deny             # cargo deny (licenses)
```

---

## Prediction Markets

SAFE-HAVEN includes a decentralized prediction market system that allows users to create, bet on, and resolve outcome-based markets tied to deposit behavior. Markets provide price discovery, hedging opportunities, and incentive alignment while remaining resistant to manipulation.

### Market Types

Markets can predict various SAFE-HAVEN deposit outcomes:

| Market Type | Description | Example |
|---|---|---|
| `aggregate_deposits` | Total value of active deposits | "Sum of all deposits > 1M tokens" |
| `avg_lock_time` | Average deposit lock duration | "Average lock time > 90 days" |
| `unique_depositors` | Number of distinct depositors | "Active depositors > 500" |
| `deposit_distribution` | Distribution across lock periods | "50% locked > 180 days" |
| `penalty_accrual` | Total penalties collected | "Penalties > 50K tokens" |

### Market Architecture

**Two-phase resolution**:
1. **Betting Phase** — Users place bets until `close_time`
2. **Resolution Phase** — Oracle submits outcome between `close_time` and `resolution_deadline`

**Anti-manipulation mechanisms**:
- **Authorized oracles** — Only designated oracle can resolve each market
- **Resolution deadline** — Forces resolution within a fixed window
- **Duplicate submission prevention** — Oracle can submit once per market
- **Outcome validation** — Invalid outcomes rejected at submission time
- **Market closure required** — Resolution only possible on closed markets

### Market API

#### Initialization

##### `init_prediction_markets(admin, fee_recipient, default_fee_bps?)`

Initialize the prediction market subsystem.

| Parameter | Type | Description |
|---|---|---|
| `admin` | `Address` | Admin address (can create markets, cancel) |
| `fee_recipient` | `Address` | Receives fees from claimed winnings |
| `default_fee_bps` | `Option<u32>` | Default fee in basis points (0-1000, default: 100 = 1%) |

**Returns** `Ok(())` on success.

---

#### Market Creation

##### `create_prediction_market(creator, market_type, description, oracle, close_time, resolution_deadline, outcome_names, fee_bps?)`

Create a new prediction market.

| Parameter | Type | Description |
|---|---|---|
| `creator` | `Address` | Must sign the transaction |
| `market_type` | `String` | Market category (e.g., "aggregate_deposits") |
| `description` | `String` | Human-readable description |
| `oracle` | `Address` | Oracle address that will resolve the market |
| `close_time` | `u64` | Unix timestamp when betting closes |
| `resolution_deadline` | `u64` | Deadline for oracle to submit resolution |
| `outcome_names` | `Vec<String>` | Names of 2-4 possible outcomes |
| `fee_bps` | `Option<u32>` | Basis points fee (0-1000, optional) |

**Returns** `u32` — the new market_id.

**Constraints**:
- `close_time > now` and `close_time - now >= MIN_MARKET_DURATION_SECS` (1 hour)
- `resolution_deadline > close_time` and `resolution_deadline - close_time >= min_resolution_time_secs`
- `outcome_names.len()` must be 2-4
- `fee_bps` must be ≤ MAX_FEE_BPS (1000)

**Example**:
```rust
// Market: "Will aggregate deposits exceed 1M tokens in the next 30 days?"
let market_id = contract.create_prediction_market(
    creator: alice,
    market_type: "aggregate_deposits",
    description: "Total deposits > 1M USDC?",
    oracle: oracle_address,
    close_time: now + 30_days,
    resolution_deadline: now + 30_days + 1_hour,
    outcome_names: vec!["Yes (>1M)", "No (<=1M)"],
    fee_bps: Some(100),  // 1% fee
)?;
```

---

#### Betting

##### `place_prediction_bet(bettor, market_id, outcome_id, amount, token)`

Place a bet on a market outcome.

| Parameter | Type | Description |
|---|---|---|
| `bettor` | `Address` | Must sign the transaction |
| `market_id` | `u32` | Target market |
| `outcome_id` | `u32` | Outcome to bet on (0-3) |
| `amount` | `i128` | Wager amount (1 - 10^15 units) |
| `token` | `Address` | Token contract address for payment |

**Returns** `Ok(())` on success.

**Constraints**:
- Market must be `Open` and `now < close_time`
- `outcome_id < outcome_count`
- `amount` must be 1 - 10^15
- Bettor cannot place two bets on the same outcome in the same market
- Bettor must approve token transfer

**Example**:
```rust
// Bet 100 USDC on "Yes (>1M)" outcome
contract.place_prediction_bet(
    bettor: alice,
    market_id: 1,
    outcome_id: 0,  // "Yes (>1M)"
    amount: 100 * 10^6,  // 100 USDC
    token: usdc_address,
)?;
```

---

#### Market Management

##### `close_prediction_market(closer, market_id)`

Close market to new bets. Only admin or oracle can call.

**Returns** `Ok(())` on success.

---

##### `resolve_prediction_market(oracle, market_id, winning_outcome, resolution_data?)`

Submit resolution for a closed market.

| Parameter | Type | Description |
|---|---|---|
| `oracle` | `Address` | Must be the market's designated oracle |
| `market_id` | `u32` | Market to resolve |
| `winning_outcome` | `u32` | Winning outcome ID (0-3) |
| `resolution_data` | `Option<i128>` | Optional: aggregate value (e.g., total deposits) |

**Returns** `Ok(())` on success.

**Constraints**:
- `now <= resolution_deadline`
- Market must be `Closed`
- `winning_outcome < outcome_count`
- Oracle cannot submit twice

**Example**:
```rust
// Resolve market: outcome 0 ("Yes") won, final aggregate = 1.2M
contract.resolve_prediction_market(
    oracle: oracle_address,
    market_id: 1,
    winning_outcome: 0,
    resolution_data: Some(1_200_000 * 10^6),  // 1.2M USDC
)?;
```

---

#### Claiming Winnings

##### `claim_prediction_winnings(bettor, market_id, outcome_id, token)`

Claim winnings for a resolved market.

| Parameter | Type | Description |
|---|---|---|
| `bettor` | `Address` | Must sign the transaction |
| `market_id` | `u32` | Resolved market |
| `outcome_id` | `u32` | Outcome bettor wagered on |
| `token` | `Address` | Token to receive payout in |

**Returns** `i128` — payout amount (after fees).

**Constraints**:
- Market must be `Resolved`
- `outcome_id` must be the market's `winning_outcome`
- Bettor cannot claim twice

**Winnings calculation**:
```
winnings = (bet_amount / total_winning_bets) × total_pool
fee = winnings × (fee_bps / 10_000)
payout = winnings - fee
```

**Example**:
```rust
// Alice bet 100 on outcome 0. Outcome 0 won.
// Total winning bets: 500. Total pool: 2000.
// Alice's winnings = (100 / 500) × 2000 = 400
// With 1% fee: payout = 400 - 4 = 396
let payout = contract.claim_prediction_winnings(
    bettor: alice,
    market_id: 1,
    outcome_id: 0,
    token: usdc_address,
)?;
// payout = 396 USDC (minus fees sent to fee_recipient)
```

---

#### Market Operations

##### `cancel_prediction_market(admin, market_id, token)`

Cancel a market and refund all bets. Admin only.

**Returns** `i128` — total refunded.

---

##### `set_prediction_market_paused(admin, paused)`

Pause/unpause market creation.

---

### Read-Only Queries

| Function | Returns | Description |
|---|---|---|
| `get_prediction_market(market_id)` | `Option<PredictionMarket>` | Market details and status |
| `get_market_outcome(market_id, outcome_id)` | `Option<MarketOutcome>` | Total bets and bettor count on outcome |
| `get_user_bet(market_id, outcome_id, bettor)` | `Option<Bet>` | User's bet details |

---

### Data Structures

#### `PredictionMarket`

```rust
pub struct PredictionMarket {
    pub market_id: u32,
    pub market_type: String,
    pub description: String,
    pub oracle: Address,
    pub close_time: u64,
    pub resolution_deadline: u64,
    pub status: MarketStatus,  // Open, Closed, Resolved, Cancelled
    pub winning_outcome: Option<u32>,
    pub total_pool: i128,
    pub bettor_count: u32,
    pub outcome_count: u32,
    pub created_at: u64,
    pub resolution_data: Option<i128>,
    pub fee_bps: u32,
}
```

#### `MarketOutcome`

```rust
pub struct MarketOutcome {
    pub outcome_id: u32,
    pub name: String,
    pub total_bet_amount: i128,
    pub bettor_count: u32,
}
```

#### `Bet`

```rust
pub struct Bet {
    pub bettor: Address,
    pub market_id: u32,
    pub outcome_id: u32,
    pub amount: i128,
    pub timestamp: u64,
    pub claimed: bool,
}
```

---

### Security Properties

| Property | Implementation |
|---|---|
| **Auth enforcement** | All state-mutating functions require caller authentication |
| **Oracle integrity** | Only designated oracle can resolve each market |
| **Manipulation resistance** | Resolution deadline, duplicate submission checks, outcome validation |
| **Fair distribution** | Proportional payout: `(bet / winning_pool) × total_pool` |
| **Fee accountability** | Fees tracked per claim, sent to designated fee_recipient |
| **No re-entrancy** | Storage state updated before token transfers |
| **Overflow safety** | All arithmetic uses saturating operations |

---

### Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1000 | `MarketNotFound` | Market ID does not exist |
| 1001 | `MarketNotOpen` | Market status is not Open |
| 1002 | `InvalidMarketStatus` | Invalid status for operation |
| 1003 | `InvalidOutcomeCount` | Outcome count not in 2-4 range |
| 1004 | `InvalidResolutionTime` | Resolution timing invalid |
| 1005 | `ResolutionTimeInPast` | Deadline in the past |
| 1006 | `InvalidFeeBps` | Fee basis points > 1000 |
| 1007 | `BettingClosed` | Market closed to new bets |
| 1008 | `InvalidBetAmount` | Bet amount out of range |
| 1009 | `InvalidOutcomeId` | Outcome ID invalid for market |
| 1010 | `BetNotFound` | User has no bet on outcome |
| 1011 | `AlreadyBetOnOutcome` | User already bet on this outcome |
| 1012 | `InsufficientFunds` | Not enough tokens for bet |
| 1013 | `MarketNotClosedYet` | Market still accepting bets |
| 1014 | `MarketAlreadyResolved` | Market already resolved |
| 1015 | `InvalidWinningOutcome` | Outcome ID invalid for winning |
| 1016 | `ResolutionDeadlineExceeded` | Past deadline for resolution |
| 1017 | `UnauthorizedOracle` | Not the designated oracle |
| 1018 | `OracleSubmissionExists` | Oracle already submitted |
| 1019 | `NoWinningBets` | Bettor did not win |
| 1020 | `BetsAlreadyClaimed` | Already claimed winnings |
| 1021 | `InvalidClaimAmount` | Calculated payout invalid |
| 1022 | `ClaimingBeforeResolution` | Market not yet resolved |
| 1023 | `Unauthorized` | Caller not authorized |
| 1024 | `InvalidAdmin` | Invalid admin address |
| 1025 | `MarketPaused` | Market creation paused |

---

### Example: Full Market Lifecycle

```rust
// 1. INITIALIZE (contract deployment)
contract.init_prediction_markets(
    admin: admin_address,
    fee_recipient: treasury_address,
    default_fee_bps: Some(100),  // 1%
)?;

// 2. CREATE MARKET
let market_id = contract.create_prediction_market(
    creator: alice,
    market_type: "aggregate_deposits",
    description: "Will deposits exceed 1M USDC?",
    oracle: oracle_address,
    close_time: now + 30.days(),
    resolution_deadline: now + 31.days(),
    outcome_names: vec!["Yes (>1M)", "No (<=1M)"],
    fee_bps: None,  // Use default 1%
)?;

// 3. PLACE BETS (until close_time)
contract.place_prediction_bet(
    bettor: alice,
    market_id: 1,
    outcome_id: 0,  // "Yes"
    amount: 100 * 10^6,
    token: usdc_address,
)?;

contract.place_prediction_bet(
    bettor: bob,
    market_id: 1,
    outcome_id: 1,  // "No"
    amount: 50 * 10^6,
    token: usdc_address,
)?;

// 4. CLOSE MARKET (by oracle or admin)
contract.close_prediction_market(closer: oracle_address, market_id: 1)?;

// 5. RESOLVE MARKET (by oracle before deadline)
contract.resolve_prediction_market(
    oracle: oracle_address,
    market_id: 1,
    winning_outcome: 0,  // "Yes" won
    resolution_data: Some(1_200_000 * 10^6),
)?;

// 6. CLAIM WINNINGS (by winners)
let payout = contract.claim_prediction_winnings(
    bettor: alice,
    market_id: 1,
    outcome_id: 0,
    token: usdc_address,
)?;
// Alice receives: (100 / 100) × 150 × 0.99 = 148.5 USDC
// Treasury receives: 1.5 USDC (1% fee)
```

---

## Use Cases

- **Savings** - Lock funds for a fixed period to enforce discipline
- **Token vesting** - Team/investor tokens released on a schedule
- **HODL commitments** - Commit to not selling until a future date
- **Escrow** - Time-gated release of payment
- **Price discovery** — Prediction markets reveal community expectations about deposit behavior
- **Hedging** — Users offset risk by betting against favorable outcomes
- **Engagement** — Markets incentivize participation and align stakeholder interests

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [CHANGELOG.md](./CHANGELOG.md).

## License

MIT
