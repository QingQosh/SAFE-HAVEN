# SAFE-HAVEN Contract API — Comprehensive Reference

**Version**: 1.0  
**Network**: Stellar Soroban (Testnet & Mainnet)  
**SDK**: Soroban SDK v22  
**Language**: Rust (compiled to WASM)

This document provides complete API documentation for the SAFE-HAVEN smart contract, including all entry points, parameter descriptions, error codes, and practical CLI/SDK examples.

---

## Table of Contents

1. [Environment Setup](#environment-setup)
2. [Core Concepts](#core-concepts)
3. [Deposit Operations](#deposit-operations)
4. [Withdrawal Operations](#withdrawal-operations)
5. [Staker Registry](#staker-registry)
6. [Admin Operations](#admin-operations)
7. [Session Keys (Delegated Access)](#session-keys)
8. [Quantum-Safe Keys](#quantum-safe-keys)
9. [MEV Protection](#mev-protection)
10. [Prediction Markets](#prediction-markets)
11. [Insurance Claims](#insurance-claims)
12. [Read-Only Queries](#read-only-queries)
13. [Error Codes & Handling](#error-codes--handling)
14. [Migration & Versioning](#migration--versioning)
15. [Usage Patterns](#usage-patterns)

---

## Environment Setup

All CLI examples assume these environment variables are configured:

```bash
# Network configuration
export NETWORK=testnet                                    # or mainnet
export RPC_URL=https://soroban-testnet.stellar.org       # or mainnet RPC
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"  # testnet only

# Contract and accounts
export CONTRACT_ID="CC..."                                # Deployed contract ID
export ADMIN_SECRET="S..."                               # Admin private key
export USER_SECRET="S..."                                # User private key
export ADMIN_ADDRESS="G..."                              # Admin public key
export USER_ADDRESS="G..."                               # User public key
export TOKEN_CONTRACT="CC..."                            # Stellar token contract

# Optional: test addresses
export FEE_RECIPIENT="G..."                              # Fee destination
export STAKER_SECRET="S..."                              # Staker private key
export STAKER_ADDRESS="G..."                             # Staker public key
```

To set mainnet passphrases:
```bash
# Stellar mainnet
export NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
```

---

## Core Concepts

### Deposit Types

SAFE-HAVEN supports three deposit type variants:

| Type | Locking Mechanism | Query Function | Notes |
|------|-------------------|-----------------|-------|
| **TimeBased** | Wall-clock Unix timestamp | `get_vault()` | Standard time-lock |
| **LedgerBased** | Soroban ledger sequence number | `get_ledger_vault()` | Ledger-based equivalent |
| **MultiToken** | Multiple tokens in single deposit | `get_multi_vault()` | Up to 5 tokens per deposit |

### Key Constraints

| Constraint | Value | Description |
|-----------|-------|-------------|
| `MIN_LOCK_DURATION_SECS` | 60 | Minimum lock period in seconds |
| `MAX_LOCK_DURATION_SECS` | 157,788,000 | Maximum lock period (~5 years) |
| `MAX_DEPOSIT_AMOUNT` | 1,000,000,000,000,000 | Largest single deposit (configurable) |
| `MAX_TOKENS_PER_DEPOSIT` | 5 | Max tokens in multi-deposit |
| `MAX_BATCH_SIZE` | 20 | Max deposits in `batch_deposit()` |
| `PENALTY_BPS_MAX` | 10,000 | Maximum penalty in basis points (100%) |

### Penalty Model

- Penalties are expressed in basis points (bps): `1 bps = 0.01%`
- Example: `penalty_bps=200` = 2% early-exit fee
- Penalty is deducted when calling `cancel_deposit()` before unlock
- Penalty funds are split:
  - 70% → Staker rewards pool
  - 5% → Insurance fund
  - 25% → Fee recipient (optional)

---

## Deposit Operations

### `initialize(admin, fee_recipient, max_deposit, max_lock_secs)`

**Purpose**: One-time setup of the vault. Must be called before any deposits.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Non-zero | Account with admin privileges |
| `fee_recipient` | `Address` | Non-zero | Account receiving penalty fees |
| `max_deposit` | `Option<i128>` | ≤ 1,000,000,000,000,000 | Max amount per deposit (or `None` for unlimited) |
| `max_lock_secs` | `Option<u64>` | ≤ 157,788,000 | Max lock duration or `None` for no limit |

**Returns**: `Result<(), VaultError>`

**Errors**:
- `AlreadyInitialized` (15) — Contract already initialized
- `InvalidAdmin` (10) — Admin address is invalid

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --fee_recipient "$FEE_RECIPIENT" \
  --max_deposit 1000000000000 \
  --max_lock_secs 157788000
```

**JavaScript/TypeScript Example**:

```typescript
import { Contract, Address, xdr } from '@stellar/stellar-sdk';

const contract = new Contract(contractId, spec);
const tx = contract.call(
  'initialize',
  Address.fromString(adminAddress),
  Address.fromString(feeRecipient),
  i128(1000000000000),  // max_deposit
  u64(157788000)         // max_lock_secs
);
```

---

### `deposit(depositor, token, amount, unlock_time, penalty_bps, renewable_energy_percent)`

**Purpose**: Lock tokens until a future Unix timestamp. Returns a unique deposit ID.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing funds |
| `token` | `Address` | Valid token contract | Stellar token contract address |
| `amount` | `i128` | > 0, ≤ max_deposit | Number of tokens to lock (inclusive of decimals) |
| `unlock_time` | `u64` | > current time | Unix timestamp when withdrawal is allowed |
| `penalty_bps` | `u32` | 0–10,000 | Early-exit fee in basis points |
| `renewable_energy_percent` | `Option<u32>` | 0–100 or `None` | Carbon-offset percentage (optional) |

**Returns**: `Result<u32, VaultError>` — Deposit ID on success

**Errors**:
- `ContractPaused` (12) — Admin paused deposits
- `InvalidAmount` (1) — Amount ≤ 0
- `AmountTooLarge` (8) — Amount exceeds `max_deposit`
- `InvalidPenaltyBps` (9) — Penalty > 10,000 bps
- `LockDurationTooShort` (11) — Lock < 60 seconds
- `LockDurationTooLong` (6) — Lock > 157,788,000 seconds
- `UnlockTimeNotInFuture` (2) — Unlock time is not in future
- `MissingFeeRecipient` (14) — Penalty > 0 but no fee recipient set
- `TokenNotAllowed` (N/A) — Token not on allowlist (if strict mode enabled)
- `TaxWashSalePeriodActive` (N/A) — Tax-loss harvest cooldown active

**Example**:

```bash
# Lock 1,000 tokens until Jan 1, 2025 (1735689600) with 2% penalty
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --unlock_time 1735689600 \
  --penalty_bps 200
```

**Response**:

```
0
```

(The deposit ID is `0`, used in subsequent `withdraw()`, `cancel_deposit()`, etc.)

---

### `deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps)`

**Purpose**: A third party (payer) sponsors a deposit for a beneficiary (depositor).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `payer` | `Address` | Must authorize | Account paying and signing |
| `depositor` | `Address` | Non-zero | Beneficiary account |
| `token` | `Address` | Valid token contract | Stellar token contract |
| `amount` | `i128` | > 0 | Tokens to lock |
| `unlock_time` | `u64` | > current time | Unix unlock timestamp |
| `penalty_bps` | `u32` | 0–10,000 | Early-exit fee |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Example**:

```bash
# Admin deposits on behalf of a user
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_for \
  --payer "$ADMIN_ADDRESS" \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 5000 \
  --unlock_time 1750000000 \
  --penalty_bps 200
```

---

### `deposit_with_delay(depositor, token, amount, delay_secs, penalty_bps)`

**Purpose**: Lock tokens for a relative duration (seconds from now) instead of an absolute timestamp.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing |
| `token` | `Address` | Valid token | Token contract |
| `amount` | `i128` | > 0 | Amount to lock |
| `delay_secs` | `u64` | 60–157,788,000 | Relative lock duration in seconds |
| `penalty_bps` | `u32` | 0–10,000 | Early-exit penalty |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Example**:

```bash
# Lock for 30 days from now
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_delay \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --delay_secs 2592000 \
  --penalty_bps 100
```

**Note**: Internally computes `unlock_time = current_time + delay_secs`

---

### `deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps)`

**Purpose**: Lock tokens until a specific ledger sequence number (instead of timestamp).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing |
| `token` | `Address` | Valid token | Token contract |
| `amount` | `i128` | > 0 | Amount to lock |
| `unlock_ledger` | `u32` | > current ledger | Target ledger sequence |
| `penalty_bps` | `u32` | 0–10,000 | Early-exit penalty |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Example**:

```bash
# Lock until ledger 50,000,000
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_by_ledger \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 2500 \
  --unlock_ledger 50000000 \
  --penalty_bps 100
```

---

### `batch_deposit(depositor, deposits)`

**Purpose**: Create multiple deposits in a single transaction (max 20).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing |
| `deposits` | `Vec<DepositRequest>` | ≤ 20 items | Array of deposit requests |

Each `DepositRequest` contains:
- `token: Address`
- `amount: i128`
- `unlock_time: u64`
- `penalty_bps: u32`

**Returns**: `Result<Vec<u32>, VaultError>` — Array of deposit IDs

**Errors**:
- `BatchSizeExceeded` (27) — More than 20 deposits

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- batch_deposit \
  --depositor "$USER_ADDRESS" \
  --deposits '[
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 1000,
      "unlock_time": 1750000000,
      "penalty_bps": 200
    },
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 2000,
      "unlock_time": 1750000000,
      "penalty_bps": 150
    }
  ]'
```

---

### `multi_deposit(depositor, tokens_and_amounts, unlock_time, penalty_bps)`

**Purpose**: Deposit multiple token types in a single transaction (max 5 distinct tokens).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing |
| `tokens_and_amounts` | `Vec<(Address, i128)>` | 1–5 items | Token–amount pairs |
| `unlock_time` | `u64` | > now | Unix timestamp for all tokens |
| `penalty_bps` | `u32` | 0–10,000 | Shared penalty for all |

**Returns**: `Result<u32, VaultError>` — Deposit ID (shared across all tokens)

**Errors**:
- `EmptyTokenList` (17) — No tokens provided
- `TooManyTokens` (16) — More than 5 tokens

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- multi_deposit \
  --depositor "$USER_ADDRESS" \
  --tokens_and_amounts '[
    ["'"$TOKEN_CONTRACT"'", 1000],
    ["'"$TOKEN_CONTRACT_2"'", 5000]
  ]' \
  --unlock_time 1750000000 \
  --penalty_bps 200
```

---

### `deposit_with_compound_interest(depositor, token, amount, unlock_time, penalty_bps, compound_frequency_secs)`

**Purpose**: Deposit with automatic periodic interest accrual.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account depositing |
| `token` | `Address` | Valid token | Token contract |
| `amount` | `i128` | > 0 | Initial amount |
| `unlock_time` | `u64` | > now | Unlock timestamp |
| `penalty_bps` | `u32` | 0–10,000 | Early-exit penalty |
| `compound_frequency_secs` | `u64` | 0, or ≥ 60 | Interest accrual period (0 = no compounding) |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Errors**:
- `InvalidCompoundFrequency` (19) — Frequency > 0 but < 60 seconds

**Interest Calculation**:
- Annual rate: **5%** (fixed)
- Per-period interest: `amount × 5% × (frequency_secs / 31,536,000)`
- Capped to prevent overflow

**Example**:

```bash
# Compound every 24 hours
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_compound_interest \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --unlock_time 1750000000 \
  --penalty_bps 100 \
  --compound_frequency_secs 86400
```

---

### `update_accrual(depositor, deposit_id)`

**Purpose**: Manually trigger accrual of pending compound interest for a deposit.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | — | Account owning deposit |
| `deposit_id` | `u32` | — | Deposit identifier |

**Returns**: `Result<i128, VaultError>` — Updated balance

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- update_accrual \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

## Withdrawal Operations

### `withdraw(depositor, deposit_id)`

**Purpose**: Withdraw locked tokens after unlock time/ledger is reached.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account withdrawing |
| `deposit_id` | `u32` | Must exist | Deposit identifier |

**Returns**: `Result<i128, VaultError>` — Amount withdrawn

**Errors**:
- `NoDepositFound` (3) — Deposit ID does not exist
- `FundsStillLocked` (4) — Unlock time/ledger not yet reached
- `VaultAlreadyUnlocked` (13) — Already withdrawn

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

**Response**:

```
1000
```

(Amount withdrawn: 1000 tokens)

---

### `withdraw_to(depositor, deposit_id, recipient)`

**Purpose**: Withdraw to a specific recipient address (if whitelisted).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Original depositor |
| `deposit_id` | `u32` | Must exist | Deposit identifier |
| `recipient` | `Address` | Whitelisted or no-list | Recipient address |

**Returns**: `Result<i128, VaultError>` — Amount withdrawn

**Errors**:
- `RecipientNotWhitelisted` (18) — Recipient not on whitelist (if one is configured)
- `InvalidRecipient` (28) — Recipient is invalid/zero address

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw_to \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0 \
  --recipient "$ADMIN_ADDRESS"
```

---

### `cancel_deposit(depositor, deposit_id)`

**Purpose**: Early exit from a deposit with penalty.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account canceling |
| `deposit_id` | `u32` | Must exist | Deposit identifier |

**Returns**: `Result<(), VaultError>`

**Errors**:
- `NoDepositFound` (3) — Deposit not found
- `VaultAlreadyUnlocked` (13) — Deposit already claimed/canceled

**Fee Distribution** (on 1000 tokens with 200 bps penalty):
- Penalty amount: 20 tokens
- Staker pool: 14 tokens (70%)
- Insurance pool: 1 token (5%)
- Fee recipient: 5 tokens (25%)
- User receives: 980 tokens

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- cancel_deposit \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `set_withdrawal_whitelist(depositor, deposit_id, recipients)`

**Purpose**: Restrict withdrawal recipients to a specific list.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Deposit owner |
| `deposit_id` | `u32` | Must exist | Deposit identifier |
| `recipients` | `Vec<Address>` | — | Allowed recipient addresses |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- set_withdrawal_whitelist \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0 \
  --recipients '["'"$ADMIN_ADDRESS"'", "'"$USER_ADDRESS"'"]'
```

---

### `get_withdrawal_whitelist(depositor, deposit_id)`

**Purpose**: Query the withdrawal whitelist for a deposit.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | — | Deposit owner |
| `deposit_id` | `u32` | — | Deposit identifier |

**Returns**: `Option<Vec<Address>>` — List of allowed recipients or `None`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_withdrawal_whitelist \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

## Staker Registry

### `register_staker(staker, amount)`

**Purpose**: Register a staker with a stake weight for reward eligibility.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `staker` | `Address` | Must authorize | Staker account |
| `amount` | `i128` | > 0 | Stake weight (not tokens) |

**Returns**: `Result<(), VaultError>`

**Important Notes**:
- This is **not a token transfer**; `amount` is a numeric weight only
- Allows stakers to participate in reward distribution from penalties
- Re-registering updates the weight

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- register_staker \
  --staker "$STAKER_ADDRESS" \
  --amount 100000000
```

---

### `claim_staker_rewards(staker)`

**Purpose**: Claim accrued rewards from the staker pool.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `staker` | `Address` | Must authorize | Registered staker |

**Returns**: `Result<(), VaultError>`

**Reward Formula**:
```
reward = (staker_weight × reward_pool_balance) / total_staked_weight
```

**Errors**:
- Reward must be > 0 (no zero-amount claims)

**Important Notes**:
- **No tokens are transferred** by this contract function
- Claimed amount is recorded in events and state, but token transfer must happen off-chain or via separate mechanism
- Pool is shared across all tokens; amounts are not token-specific
- Order and timing affect final amounts due to integer rounding

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_staker_rewards \
  --staker "$STAKER_ADDRESS"
```

---

### `get_stakers(offset, limit)`

**Purpose**: Paginated list of all registered stakers.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `offset` | `u32` | ≥ 0 | Pagination offset |
| `limit` | `u32` | 1–100 | Page size |

**Returns**: `Page<Address>` — Struct with `items`, `total_count`, and cursor info

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_stakers \
  --offset 0 \
  --limit 10
```

---

### `get_staker_info(staker)`

**Purpose**: Query a staker's weight and reward history.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `staker` | `Address` | — | Staker address |

**Returns**: `Option<StakerInfo>` — Struct with weight and claimed amounts

**Response Structure**:

```rust
pub struct StakerInfo {
    pub amount: i128,                    // Registered weight
    pub claimed_amount: i128,            // Total claimed (accounting only)
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_staker_info \
  --staker "$STAKER_ADDRESS"
```

---

## Admin Operations

### `pause(admin)` / `unpause(admin)`

**Purpose**: Pause or resume new deposits (emergency or maintenance).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin account |

**Returns**: `Result<(), VaultError>`

**Errors**:
- `Unauthorized` (7) — Caller is not admin

**Example**:

```bash
# Pause
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- pause \
  --admin "$ADMIN_ADDRESS"

# Unpause
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- unpause \
  --admin "$ADMIN_ADDRESS"
```

---

### `transfer_admin(admin, new_admin)` & `accept_admin(new_admin)`

**Purpose**: Two-step admin transfer for safety.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` / `new_admin` | `Address` | Must authorize | Current/new admin |

**Returns**: `Result<(), VaultError>`

**Step 1 — Initiate Transfer**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- transfer_admin \
  --admin "$ADMIN_ADDRESS" \
  --new_admin "$NEW_ADMIN_ADDRESS"
```

**Step 2 — Accept Transfer**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$NEW_ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- accept_admin \
  --new_admin "$NEW_ADMIN_ADDRESS"
```

---

### `cancel_transfer_admin(admin)`

**Purpose**: Cancel a pending admin transfer.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be current admin | Current admin |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- cancel_transfer_admin \
  --admin "$ADMIN_ADDRESS"
```

---

### `renounce_admin(admin)`

**Purpose**: Permanently give up admin privileges (irreversible).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin account |

**Returns**: `Result<(), VaultError>`

**Errors**:
- `Unauthorized` (7) — Not admin

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- renounce_admin \
  --admin "$ADMIN_ADDRESS"
```

---

### `emergency_withdraw(admin, depositor, deposit_id)`

**Purpose**: Admin override to force a withdrawal regardless of lock status.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin account |
| `depositor` | `Address` | — | Deposit owner |
| `deposit_id` | `u32` | — | Deposit identifier |

**Returns**: `Result<i128, VaultError>` — Amount withdrawn

**Errors**:
- `Unauthorized` (7) — Caller not admin
- `EmergencyWithdrawalLimitExceeded` (20) — Per-ledger hard cap exceeded

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- emergency_withdraw \
  --admin "$ADMIN_ADDRESS" \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

## Session Keys

**Purpose**: Allow delegated access for deposits and withdrawals without sharing private keys.

### `authorize_session_key(delegator, session_key_id, max_deposit_amount, expiration_time)`

**Purpose**: Create a session key with limits.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `delegator` | `Address` | Must authorize | Account delegating |
| `session_key_id` | `u32` | Unique per delegator | Session key identifier |
| `max_deposit_amount` | `i128` | > 0 | Max amount per deposit |
| `expiration_time` | `u64` | > now | Unix expiration timestamp |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- authorize_session_key \
  --delegator "$USER_ADDRESS" \
  --session_key_id 1 \
  --max_deposit_amount 10000 \
  --expiration_time 1750000000
```

---

### `revoke_session_key(delegator, session_key_id)`

**Purpose**: Revoke a session key immediately.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `delegator` | `Address` | Must authorize | Account revoking |
| `session_key_id` | `u32` | — | Key to revoke |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- revoke_session_key \
  --delegator "$USER_ADDRESS" \
  --session_key_id 1
```

---

### `is_session_key_authorized(delegator, session_key_id)`

**Purpose**: Check if a session key is valid and not expired.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `delegator` | `Address` | — | Key owner |
| `session_key_id` | `u32` | — | Key identifier |

**Returns**: `bool` — True if valid and active

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- is_session_key_authorized \
  --delegator "$USER_ADDRESS" \
  --session_key_id 1
```

---

### `deposit_with_session_key(delegator, session_key_id, token, amount, unlock_time, penalty_bps)`

**Purpose**: Deposit using a session key (without delegator authorization).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `delegator` | `Address` | — | Original account owner |
| `session_key_id` | `u32` | Must be authorized | Session key ID |
| `token` | `Address` | — | Token contract |
| `amount` | `i128` | ≤ session limit | Amount to deposit |
| `unlock_time` | `u64` | > now | Unlock timestamp |
| `penalty_bps` | `u32` | 0–10,000 | Penalty |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Errors**:
- `SessionKeyNotFound` (20) — Key does not exist
- `SessionKeyRevoked` (21) — Key was revoked
- `SessionKeyExpired` (22) — Key has expired
- `SessionKeyInsufficientLimit` (23) — Amount exceeds limit

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$DELEGATE_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_session_key \
  --delegator "$USER_ADDRESS" \
  --session_key_id 1 \
  --token "$TOKEN_CONTRACT" \
  --amount 5000 \
  --unlock_time 1750000000 \
  --penalty_bps 200
```

---

## Quantum-Safe Keys

**Purpose**: Store ML-DSA (lattice-based) public keys for post-quantum cryptography.

### `register_quantum_safe_key(account, public_key)`

**Purpose**: Register a quantum-safe public key.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `account` | `Address` | Must authorize | Account registering key |
| `public_key` | `Bytes` | Exactly 1312 bytes | ML-DSA public key |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
# Assume $QS_KEY is a 1312-byte hex string
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- register_quantum_safe_key \
  --account "$USER_ADDRESS" \
  --public_key "0x$(printf '%0.s0' {1..2624})"  # 1312 bytes = 2624 hex chars
```

---

### `get_quantum_safe_key(account)`

**Purpose**: Retrieve a registered quantum-safe public key.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `account` | `Address` | — | Account |

**Returns**: `Option<Bytes>` — Registered key or `None`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_quantum_safe_key \
  --account "$USER_ADDRESS"
```

---

### `quantum_safe_message(account, message)`

**Purpose**: Sign a message with quantum-safe key (for verification outside contract).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `account` | `Address` | Must authorize | Account |
| `message` | `Bytes` | — | Message to sign |

**Returns**: `Result<Bytes, VaultError>` — Signature (2420 bytes for ML-DSA)

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- quantum_safe_message \
  --account "$USER_ADDRESS" \
  --message "0x48656c6c6f"  # "Hello" in hex
```

---

### `deposit_quantum_safe(account, token, amount, unlock_time, penalty_bps, signature)`

**Purpose**: Deposit with quantum-safe signature verification.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `account` | `Address` | — | Account depositing |
| `token` | `Address` | — | Token contract |
| `amount` | `i128` | > 0 | Amount |
| `unlock_time` | `u64` | > now | Unlock timestamp |
| `penalty_bps` | `u32` | 0–10,000 | Penalty |
| `signature` | `Bytes` | 2420 bytes (ML-DSA) | Signature |

**Returns**: `Result<u32, VaultError>` — Deposit ID

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_quantum_safe \
  --account "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --unlock_time 1750000000 \
  --penalty_bps 200 \
  --signature "0x..."  # 2420-byte ML-DSA signature
```

---

## MEV Protection

**Purpose**: Protect deposits from maximal extractable value (MEV) attacks via commit-reveal scheme.

### `mev_commit(depositor, deposit_id, commit_hash)`

**Purpose**: Commit to a MEV prevention strategy (phase 1 of 2).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account |
| `deposit_id` | `u32` | — | Deposit identifier |
| `commit_hash` | `Bytes` (32 bytes) | SHA256 hash | Commitment to strategy |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
# Commit hash = SHA256("secret_strategy_data")
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- mev_commit \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0 \
  --commit_hash "0x$(echo -n 'secret' | sha256sum | cut -d' ' -f1)"
```

---

### `mev_reveal(depositor, deposit_id, strategy_data, recovery_amount)`

**Purpose**: Reveal and verify MEV protection strategy (phase 2 of 2).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account |
| `deposit_id` | `u32` | — | Deposit identifier |
| `strategy_data` | `Bytes` | — | Original strategy data |
| `recovery_amount` | `i128` | ≥ 0 | Funds recovered from MEV |

**Returns**: `Result<(), VaultError>`

**Verification**:
- Compute `SHA256(strategy_data)` and verify it matches the committed hash
- If match, record recovery amount

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- mev_reveal \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0 \
  --strategy_data "0x..." \
  --recovery_amount 500
```

---

### `claim_mev_recovery(depositor)`

**Purpose**: Claim accumulated MEV recovery funds.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | Must authorize | Account |

**Returns**: `Result<i128, VaultError>` — Amount recovered

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_mev_recovery \
  --depositor "$USER_ADDRESS"
```

---

### `get_mev_detections(depositor, limit)`

**Purpose**: Query detected MEV attacks for an account.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `depositor` | `Address` | — | Account |
| `limit` | `u32` | — | Max results |

**Returns**: `Vec<MEVDetection>` — Array of detection records

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_mev_detections \
  --depositor "$USER_ADDRESS" \
  --limit 10
```

---

## Prediction Markets

### `init_prediction_markets(admin)`

**Purpose**: Initialize the prediction market subsystem.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- init_prediction_markets \
  --admin "$ADMIN_ADDRESS"
```

---

### `create_prediction_market(creator, description, outcomes, resolution_time, config)`

**Purpose**: Create a new prediction market.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `creator` | `Address` | Must authorize | Market creator |
| `description` | `String` | — | Market description |
| `outcomes` | `Vec<String>` | 2+ items | Possible outcomes |
| `resolution_time` | `u64` | > now | Unix timestamp for resolution |
| `config` | `MarketConfig` | — | Market parameters |

**Returns**: `Result<u32, VaultError>` — Market ID

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- create_prediction_market \
  --creator "$USER_ADDRESS" \
  --description "Will BTC reach $100k by 2025?" \
  --outcomes '["Yes", "No"]' \
  --resolution_time 1750000000 \
  --config '{"min_stake": 100, "max_stake": 10000}'
```

---

### `place_prediction_bet(bettor, market_id, outcome_id, amount)`

**Purpose**: Place a bet on a prediction market outcome.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `bettor` | `Address` | Must authorize | Bet placer |
| `market_id` | `u32` | — | Market identifier |
| `outcome_id` | `u32` | < outcomes.len() | Outcome index |
| `amount` | `i128` | > 0 | Stake amount |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- place_prediction_bet \
  --bettor "$USER_ADDRESS" \
  --market_id 1 \
  --outcome_id 0 \
  --amount 500
```

---

### `close_prediction_market(admin, market_id)`

**Purpose**: Close a market (no more bets allowed).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |
| `market_id` | `u32` | — | Market identifier |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- close_prediction_market \
  --admin "$ADMIN_ADDRESS" \
  --market_id 1
```

---

### `resolve_prediction_market(admin, market_id, winning_outcome_id)`

**Purpose**: Resolve a market by declaring the winning outcome.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |
| `market_id` | `u32` | — | Market identifier |
| `winning_outcome_id` | `u32` | < outcomes.len() | Winning outcome index |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- resolve_prediction_market \
  --admin "$ADMIN_ADDRESS" \
  --market_id 1 \
  --winning_outcome_id 0
```

---

### `claim_prediction_winnings(bettor, market_id, outcome_id)`

**Purpose**: Claim winnings if your bet won.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `bettor` | `Address` | Must authorize | Bet placer |
| `market_id` | `u32` | — | Market identifier |
| `outcome_id` | `u32` | — | Outcome index |

**Returns**: `Result<i128, VaultError>` — Winnings amount

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_prediction_winnings \
  --bettor "$USER_ADDRESS" \
  --market_id 1 \
  --outcome_id 0
```

---

### `get_prediction_market(market_id)`

**Purpose**: Retrieve market details.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `market_id` | `u32` | — | Market identifier |

**Returns**: `Option<PredictionMarket>` — Market struct or `None`

**Response Fields**:
```rust
pub struct PredictionMarket {
    pub creator: Address,
    pub description: String,
    pub resolution_time: u64,
    pub status: MarketStatus,  // Open, Closed, Resolved
    pub total_stake: i128,
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_prediction_market \
  --market_id 1
```

---

## Insurance Claims

### `claim_insurance(claimant, token, amount, evidence_uri)`

**Purpose**: File an insurance claim against the insurance fund.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `claimant` | `Address` | Must authorize | Claimant address |
| `token` | `Address` | — | Token contract |
| `amount` | `i128` | > 0 | Claim amount |
| `evidence_uri` | `String` | — | Link to evidence (IPFS, etc.) |

**Returns**: `Result<u32, VaultError>` — Claim ID

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_insurance \
  --claimant "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --evidence_uri "ipfs://QmXxxx..."
```

---

### `approve_claim(admin, claim_id)`

**Purpose**: Approve an insurance claim (admin only).

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |
| `claim_id` | `u32` | — | Claim identifier |

**Returns**: `Result<i128, VaultError>` — Approved amount

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- approve_claim \
  --admin "$ADMIN_ADDRESS" \
  --claim_id 1
```

---

### `deny_claim(admin, claim_id)`

**Purpose**: Deny an insurance claim.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |
| `claim_id` | `u32` | — | Claim identifier |

**Returns**: `Result<(), VaultError>`

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deny_claim \
  --admin "$ADMIN_ADDRESS" \
  --claim_id 1
```

---

### `get_claim(claim_id)`

**Purpose**: Retrieve claim details.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `claim_id` | `u32` | — | Claim identifier |

**Returns**: `Option<InsuranceClaim>` — Claim struct or `None`

**Response Fields**:
```rust
pub struct InsuranceClaim {
    pub claimant: Address,
    pub token: Address,
    pub amount: i128,
    pub status: ClaimStatus,  // Pending, Approved, Denied
    pub evidence_uri: String,
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_claim \
  --claim_id 1
```

---

## Read-Only Queries

### `get_vault(depositor, deposit_id)`

**Purpose**: Retrieve timestamp-based deposit details.

**Returns**: `Option<VaultEntry>` — Deposit struct or `None`

**Response Fields**:
```rust
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub penalty_bps: u32,
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `get_ledger_vault(depositor, deposit_id)`

**Purpose**: Retrieve ledger-based deposit details.

**Returns**: `Option<LedgerVaultEntry>`

**Response Fields**:
```rust
pub struct LedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub penalty_bps: u32,
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_ledger_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `get_multi_vault(depositor, deposit_id)`

**Purpose**: Retrieve multi-token deposit details.

**Returns**: `Option<MultiTokenVaultEntry>`

**Response Fields**:
```rust
pub struct MultiTokenVaultEntry {
    pub tokens_and_amounts: Vec<(Address, i128)>,
    pub unlock_time: u64,
    pub penalty_bps: u32,
}
```

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_multi_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `get_deposit_ids(depositor)`

**Purpose**: List all active deposit IDs for an account.

**Returns**: `Vec<u32>` — Array of deposit IDs

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposit_ids \
  --depositor "$USER_ADDRESS"
```

**Response**:

```
[0, 1, 2]
```

---

### `get_deposit_type(depositor, deposit_id)`

**Purpose**: Determine the type of a deposit (TimeBased, LedgerBased, or Multi).

**Returns**: `Option<DepositType>` — Type enum

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposit_type \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

**Response**:

```
"TimeBased"
```

---

### `get_current_balance(depositor, deposit_id)`

**Purpose**: Get current balance including accrued compound interest.

**Returns**: `i128` — Current balance

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_current_balance \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `time_remaining(depositor, deposit_id)`

**Purpose**: Seconds until unlock (for timestamp-based) or estimated seconds (for ledger-based).

**Returns**: `u64` — Remaining seconds (or 0 if unlocked)

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- time_remaining \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

---

### `get_deposits_page(offset, limit)`

**Purpose**: Paginated list of all timestamp-based deposits.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `offset` | `u32` | ≥ 0 | Pagination offset |
| `limit` | `u32` | 1–100 | Page size |

**Returns**: `Page<(Address, u32, VaultEntry)>` — Paginated results

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposits_page \
  --offset 0 \
  --limit 10
```

---

### `get_depositors(offset, limit)`

**Purpose**: Paginated list of all active depositors.

**Returns**: `Page<Address>` — Paginated results

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_depositors \
  --offset 0 \
  --limit 10
```

---

### `get_time()`

**Purpose**: Current ledger timestamp (seconds).

**Returns**: `u64` — Unix timestamp

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_time
```

---

### `get_admin()`, `get_pending_admin()`, `get_fee_recipient()`, `get_constants()`

**Purpose**: Query contract configuration.

**Returns**: Various types

| Function | Returns | Description |
|----------|---------|-------------|
| `get_admin()` | `Option<Address>` | Current admin |
| `get_pending_admin()` | `Option<Address>` | Pending admin (if transfer in progress) |
| `get_fee_recipient()` | `Option<Address>` | Fee recipient |
| `get_constants()` | `ContractConstants` | Max deposit, max lock, etc. |

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_constants
```

---

### `get_insurance_pool_balance(token)`

**Purpose**: Current balance of the insurance pool for a token.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `token` | `Address` | — | Token contract |

**Returns**: `i128` — Pool balance

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_insurance_pool_balance \
  --token "$TOKEN_CONTRACT"
```

---

### `staker_pool_balance(token)`

**Purpose**: Current balance of the staker reward pool for a token.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `token` | `Address` | — | Token contract |

**Returns**: `i128` — Pool balance

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- staker_pool_balance \
  --token "$TOKEN_CONTRACT"
```

---

## Error Codes & Handling

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 1 | `InvalidAmount` | Deposit amount ≤ 0 | Deposit positive amount |
| 2 | `UnlockTimeNotInFuture` | Unlock time is in the past | Set unlock_time to future |
| 3 | `NoDepositFound` | Deposit ID doesn't exist | Use valid deposit ID |
| 4 | `FundsStillLocked` | Unlock time not reached | Wait until unlock time |
| 5 | `DepositAlreadyExists` | Cannot re-use deposit ID | Use new ID |
| 6 | `LockDurationTooLong` | Duration > 5 years | Reduce lock duration |
| 7 | `Unauthorized` | Caller not authorized | Use correct account |
| 8 | `AmountTooLarge` | Amount exceeds limit | Reduce amount |
| 9 | `InvalidPenaltyBps` | Penalty > 10,000 bps | Use ≤ 10,000 |
| 10 | `InvalidAdmin` | Admin address invalid | Provide valid address |
| 11 | `LockDurationTooShort` | Duration < 60 seconds | Increase duration |
| 12 | `ContractPaused` | Deposits paused | Wait for unpause |
| 13 | `VaultAlreadyUnlocked` | Already withdrawn | Cannot withdraw twice |
| 14 | `MissingFeeRecipient` | No fee recipient set | Initialize with recipient |
| 15 | `AlreadyInitialized` | Init called twice | Contract already initialized |
| 16 | `TooManyTokens` | > 5 tokens in multi-deposit | Use ≤ 5 tokens |
| 17 | `EmptyTokenList` | Multi-deposit has 0 tokens | Add at least 1 token |
| 18 | `RecipientNotWhitelisted` | Recipient not on whitelist | Add to whitelist first |
| 19 | `InvalidCompoundFrequency` | 0 < freq < 60 | Use 0 or ≥ 60 |
| 20 | `MevDetected` / `SessionKeyNotFound` / `EmergencyWithdrawalLimitExceeded` | Multiple error contexts | Refer to function docs |
| 21–26 | (Additional) | Sponsorship & session key errors | Refer to function docs |
| 27 | `BatchSizeExceeded` | Batch > 20 deposits | Use ≤ 20 |
| 28 | `InvalidRenewableEnergyPercent` / `InvalidRecipient` | Invalid parameter | Fix input |

---

## Migration & Versioning

### `get_storage_version()`

**Purpose**: Check current storage schema version.

**Returns**: `u32` — Version number

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_storage_version
```

---

### `migrate(admin)`

**Purpose**: Upgrade storage schema to current version.

**Parameters**:

| Parameter | Type | Constraints | Description |
|-----------|------|-------------|-------------|
| `admin` | `Address` | Must authorize & be admin | Admin |

**Returns**: `Result<bool, VaultError>` — `true` if migrated, `false` if already current

**Example**:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- migrate \
  --admin "$ADMIN_ADDRESS"
```

---

## Usage Patterns

### Complete Deposit & Withdrawal Cycle

```bash
#!/bin/bash

# 1. Calculate unlock time (30 days from now)
UNLOCK_TIME=$(($(date +%s) + 2592000))

# 2. Deposit
DEPOSIT_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 0)

echo "Deposit created with ID: $DEPOSIT_ID"

# 3. Check time remaining
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- time_remaining \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID"

# 4. (After waiting or manually advancing time)
# Withdraw
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID"
```

---

### Batch Operations

```bash
# Create 3 deposits in one transaction
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- batch_deposit \
  --depositor "$USER_ADDRESS" \
  --deposits '[
    {
      "token": "'"$TOKEN_1"'",
      "amount": 1000,
      "unlock_time": 1750000000,
      "penalty_bps": 200
    },
    {
      "token": "'"$TOKEN_2"'",
      "amount": 5000,
      "unlock_time": 1750000000,
      "penalty_bps": 150
    },
    {
      "token": "'"$TOKEN_3"'",
      "amount": 2000,
      "unlock_time": 1750000000,
      "penalty_bps": 100
    }
  ]'
```

---

### JavaScript/TypeScript SDK Pattern

```typescript
import { Contract, Address, xdr, Keypair } from '@stellar/stellar-sdk';
import { StellarWalletsKit } from '@creit.tech/stellar-wallets-kit';

// Initialize wallet
const kit = new StellarWalletsKit({
  selectedWalletType: 'FREIGHTER',
});

// Get contract
const contract = new Contract(contractId, spec);

// Create deposit transaction
const tx = contract.call(
  'deposit',
  Address.fromString(userAddress),
  Address.fromString(tokenAddress),
  i128(1000n),                           // amount
  u64(BigInt(Math.floor(Date.now() / 1000) + 2592000)), // unlock_time
  u32(200)                               // penalty_bps
);

// Sign with wallet
const response = await kit.submitTx(tx);
console.log('Deposit ID:', response);
```

---

## Best Practices

1. **Always validate input**:
   - Check unlock time is in the future before submitting
   - Ensure penalty_bps ≤ 10,000
   - Verify token address is valid

2. **Use session keys for delegated operations**:
   - Creates separation of concerns
   - Limits exposure if key is compromised
   - Set appropriate expiration times

3. **Monitor event logs**:
   - Contract emits events for all state changes
   - Use an event indexer for off-chain state tracking
   - Critical for pagination and historical queries

4. **Handle errors gracefully**:
   - Check return value before assuming success
   - Retry with backoff for transient network errors
   - Log error codes for debugging

5. **Test on testnet first**:
   - Deploy contract on testnet
   - Test all flows before mainnet deployment
   - Use `NETWORK=testnet` environment variable

---

## Support & Resources

- **Documentation**: See `/docs/` directory
- **Examples**: Check `contracts/safe-haven/src/test.rs` for test patterns
- **Events**: Query contract events for state confirmations
- **Issues**: Report bugs via GitHub Issues

---

**Last Updated**: November 2024  
**Contract Version**: 1.0  
**SDK Version**: Soroban v22
