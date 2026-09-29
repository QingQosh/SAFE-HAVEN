# Soroban-Native Gasless Transaction Support for SAFE-HAVEN

## Executive Summary

**Note:** The initial scope referenced ERC-2771, which is Ethereum-specific and does not apply to Soroban (Stellar's smart contract platform). This specification adapts gasless transaction patterns for **Soroban's native execution model**.

Soroban does not use meta-transactions like Ethereum. Instead, we implement gasless support through:
1. **Sponsored accounts** — a contract-managed pool of pre-funded accounts that can sign on behalf of users
2. **Sponsorship fund management** — a dedicated token pool that covers resource fees
3. **Eligibility and abuse prevention** — rate limits, KYC-lite checks, and sybil resistance
4. **Relayer integration** — standardized contract interface that relayers can use to submit sponsored transactions

---

## Problem Statement

- New users face friction due to minimum balance requirements for Soroban accounts
- Small deposits below gas cost thresholds are economically unviable
- Early exits are discouraged due to fixed gas costs
- Current pattern requires users to:
  1. Acquire native tokens for fees
  2. Create an account with minimum balance
  3. Then perform their first transaction

**Target:** Remove the initial friction while maintaining security and preventing abuse.

---

## Design Principles

1. **Opt-in, not forced** — Users choose to use sponsorship; regular deposits work as before
2. **Sustainable** — Sponsorship fund is replenished through penalties and fees, not infinite subsidy
3. **Abuse-resistant** — Rate limits, eligibility rules, and sybil detection prevent exploitation
4. **Transparent** — Events clearly mark sponsored operations
5. **Relayer-friendly** — Standardized interface for relayers to integrate
6. **Testable** — Comprehensive test coverage of all sponsorship paths

---

## Architecture Overview

### Soroban Execution Model (Unlike Ethereum)

- **No meta-transactions** — Soroban doesn't support signed off-chain transactions that are replayed on-chain
- **Sponsored accounts** — Instead, we use pre-funded contract-owned accounts that can sign on behalf of users
- **Resource fees** — Soroban charges for CPU instructions, memory, and storage; these are paid upfront by the transaction submitter
- **Sponsored submission** — A relayer submits the transaction; the contract's sponsorship fund covers the fee

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    SAFE-HAVEN Contract                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Sponsorship Fund Management                │   │
│  │  - Storage: fund_balance, sponsor_address, limits  │   │
│  │  - Replenishment: penalties, admin deposits       │   │
│  │  - Allocation: per-user, per-day caps             │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│                          │                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │      Eligibility & Abuse Prevention                │   │
│  │  - KYC-lite: minimum balance check                 │   │
│  │  - Rate limiting: txn/day, amount/day per user    │   │
│  │  - Sybil detection: per-address deduplication    │   │
│  │  - Cooldown: enforce gap between sponsored txns   │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│                          │                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │    Sponsored Deposit Functions                     │   │
│  │  - sponsored_deposit(user, token, amount, ...)    │   │
│  │  - sponsored_deposit_for(relayer_pays, user, ...) │   │
│  │  - Check eligibility, deduct from fund            │   │
│  │  - Emit GaslessTransactionExecuted event          │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│                          │                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │      Event Emission & Observability                │   │
│  │  - GaslessTransactionExecuted                      │   │
│  │  - SponsorsipUsed                                  │   │
│  │  - SponsorsipFundReplenished                       │   │
│  │  - SybilDetected                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
└─────────────────────────────────────────────────────────────┘
           ▲                                         │
           │                                         │
      ┌────┴──────────┐                              │
      │               │                              │
   Relayers      Users/Apps            Events (Indexer)
```

---

## Data Model

### Sponsorship Types

```rust
// Storage key for sponsorship fund (singleton)
const SPONSORSHIP_FUND_KEY: &str = "spons_fund";
const SPONSORSHIP_CONFIG_KEY: &str = "spons_config";

// Per-user sponsorship tracking
const SPONSORSHIP_USAGE_PREFIX: &str = "spons_usage";  // "spons_usage:{user}:{day}"

pub struct SponsorshipFund {
    /// Balance of native tokens available for sponsorship
    pub balance: i128,
    
    /// Address that manages the sponsorship fund (typically admin)
    pub sponsor_address: Address,
    
    /// Maximum tokens to sponsor per transaction
    pub max_per_txn: i128,
    
    /// Maximum tokens to sponsor per user per day
    pub max_per_user_day: i128,
    
    /// Minimum native balance required to be eligible for sponsorship (KYC-lite)
    pub min_eligible_balance: i128,
    
    /// Cooldown period (in seconds) between sponsored transactions per user
    pub cooldown_seconds: u64,
    
    /// Timestamp of last update (for tracking replenishment frequency)
    pub last_replenished: u64,
}

pub struct SponsorshipUsage {
    /// Cumulative amount sponsored to this user today
    pub amount_used_today: i128,
    
    /// Timestamp of the last sponsored transaction for this user
    pub last_sponsored_time: u64,
    
    /// Counter of sponsored transactions for this user (for sybil detection)
    pub transaction_count: u32,
}

pub struct SponsorshipEligibility {
    /// User is eligible if they meet all criteria
    pub is_eligible: bool,
    
    /// Reason if not eligible
    pub reason: String,  // "insufficient_fund", "rate_limited", "sybil_detected", etc.
    
    /// Amount available for this user today
    pub available_today: i128,
}
```

### New Error Codes

```rust
pub enum VaultError {
    // ... existing errors ...
    
    /// Sponsorship fund not initialized (error 20)
    SponsorshipNotInitialized = 20,
    
    /// User not eligible for sponsorship (error 21)
    NotEligibleForSponsorship = 21,
    
    /// Sponsorship fund insufficient (error 22)
    InsufficientSponsorshipFund = 22,
    
    /// User exceeded daily sponsorship limit (error 23)
    SponsorshipDailyLimitExceeded = 23,
    
    /// User transaction cooldown still active (error 24)
    SponsorshipCooldownActive = 24,
    
    /// Potential sybil attack detected (error 25)
    PotentialSybilAttack = 25,
    
    /// Sponsorship configuration error (error 26)
    InvalidSponsorshipConfig = 26,
}
```

---

## API Specification

### 1. Sponsorship Fund Management (Admin-Only)

#### `initialize_sponsorship(admin, sponsor_address, initial_balance, config)`
Set up the sponsorship fund for the first time.

**Parameters:**
- `admin`: Address — The contract admin (must sign)
- `sponsor_address`: Address — Address that manages the fund (typically the contract admin)
- `initial_balance`: i128 — Initial tokens to fund sponsorship (> 0)
- `config`: SponsorshipConfig — Rate limits and eligibility rules

**Returns:** `Result<(), VaultError>`

**Behavior:**
- Validates that sponsorship is not already initialized
- Transfers `initial_balance` from admin to the contract's sponsorship account
- Stores config in persistent storage
- Emits `SponsorshipFundInitialized` event

---

#### `replenish_sponsorship_fund(sponsor, amount)`
Add tokens to the sponsorship fund (e.g., from penalties or admin deposits).

**Parameters:**
- `sponsor`: Address — Must be the configured sponsor address
- `amount`: i128 — Tokens to add (> 0)

**Returns:** `Result<(), VaultError>`

**Behavior:**
- Validates sponsor is the configured sponsor
- Accepts a token transfer
- Increments fund balance
- Emits `SponsorshipFundReplenished` event

---

#### `adjust_sponsorship_config(admin, new_config)`
Update rate limits and eligibility rules.

**Parameters:**
- `admin`: Address — Contract admin
- `new_config`: SponsorshipConfig — New limits

**Returns:** `Result<(), VaultError>`

**Behavior:**
- Validates admin
- Updates config (max_per_txn, max_per_user_day, min_eligible_balance, cooldown_seconds)
- Emits `SponsorshipConfigUpdated` event

---

### 2. User-Facing Sponsored Deposits

#### `sponsored_deposit(depositor, token, amount, unlock_time, penalty_bps)`
Deposit with sponsorship covering resource fees.

**Parameters:**
- `depositor`: Address — User making the deposit (must sign)
- `token`: Address — Token to lock
- `amount`: i128 — Amount to lock
- `unlock_time`: u64 — Unlock timestamp
- `penalty_bps`: u32 — Early-exit penalty

**Returns:** `Result<u32, VaultError>` — deposit ID

**Behavior:**
1. Check eligibility for sponsorship
2. Charge depositor for tokens (not fees)
3. Deduct estimated resource cost from sponsorship fund
4. Record sponsorship usage
5. Create deposit (as if `deposit()` was called)
6. Emit `GaslessTransactionExecuted` event

**Eligibility Checks:**
- User has minimum balance (KYC-lite check)
- Sponsorship fund has sufficient balance
- User hasn't exceeded daily sponsorship limit
- User is not in cooldown period
- No sybil attack detected

---

#### `sponsored_deposit_for(relayer, depositor, token, amount, unlock_time, penalty_bps)`
Relayer-sponsored deposit where relayer funds the deposit on behalf of a user.

**Parameters:**
- `relayer`: Address — Entity paying for the deposit (must sign)
- `depositor`: Address — Beneficiary user
- `token`: Address — Token
- `amount`: i128 — Amount
- `unlock_time`: u64 — Unlock time
- `penalty_bps`: u32 — Penalty

**Returns:** `Result<u32, VaultError>`

**Behavior:**
- Similar to `sponsored_deposit()` but relayer funds the tokens
- Sponsorship fund covers resource fees only
- Eligibility checks still apply (to prevent sybil attacks)
- Emits `GaslessTransactionExecuted` with both relayer and depositor

---

### 3. Query Functions

#### `get_sponsorship_fund_status() -> SponsorshipFund`
Query current fund state.

**Returns:**
- Current balance
- Max per transaction
- Max per user per day
- Minimum eligible balance
- Cooldown period
- Last replenishment time

---

#### `check_sponsorship_eligibility(user) -> SponsorshipEligibility`
Check if a user is eligible for sponsorship.

**Returns:**
- `is_eligible`: bool
- `reason`: String (if not eligible)
- `available_today`: i128 (amount available for this user today)

---

#### `get_sponsorship_usage(user) -> Option<SponsorshipUsage>`
Query user's sponsorship usage for today.

**Returns:**
- Amount used today
- Last sponsored transaction time
- Transaction count

---

## Sponsorship Fund Replenishment

The sponsorship fund is replenished through multiple channels:

### 1. Early Withdrawal Penalties
When users call `cancel_deposit()`, penalties are split:
- 30% → fee_recipient (unchanged)
- **70% → sponsorship fund** (new allocation)

**Example:**
- User cancels deposit with 100 token penalty
- 30 tokens → fee_recipient
- **70 tokens → sponsorship fund**

---

### 2. Admin Top-ups
Admin can call `replenish_sponsorship_fund()` to add tokens directly (e.g., from external revenue).

---

### 3. Interest/Yield
Future yield mechanisms can allocate a portion to sponsorship fund.

---

## Abuse Prevention Mechanisms

### 1. KYC-Lite Check
**Requirement:** User must maintain minimum native balance (`min_eligible_balance`)

**Rationale:** 
- Establishes economic skin-in-the-game
- Reduces sybil creation costs
- Typical value: 1-10 XLM (depending on Stellar network fees)

---

### 2. Rate Limiting (Daily Cap)
**Requirement:** Per-user daily limit (`max_per_user_day`)

**Calculation:**
- Tracked per calendar day (using ledger timestamp)
- Amount reset daily
- Example: Max 1000 USDC per user per day

**Enforcement:**
```
if usage_today + requested_amount > max_per_user_day {
    return NotEligibleForSponsorship
}
```

---

### 3. Cooldown Between Transactions
**Requirement:** Minimum gap between sponsored txns per user (`cooldown_seconds`)

**Rationale:**
- Prevents rapid-fire spam
- Gives relayers time to adjust fees
- Typical value: 60-300 seconds

**Enforcement:**
```
if now - last_sponsored_time < cooldown_seconds {
    return SponsorshipCooldownActive
}
```

---

### 4. Max Per Transaction
**Requirement:** Single txn cap (`max_per_txn`)

**Rationale:**
- Limits blast radius if fund is compromised
- Prevents monopolization by large users
- Example: 100 USDC per transaction

---

### 5. Sybil Detection
**Heuristics:**
- Flag if user attempts to register multiple accounts within short timeframe
- Flag if similar patterns detected (same IP via relayer, similar metadata)
- Track transaction velocity per account

**Implementation:**
- Store transaction count per user
- If count exceeds threshold (e.g., 10) in one day, require human review
- Emit `SybilDetected` event for off-chain analysis

---

## Event Emissions

### New Events

```rust
pub fn sponsorship_fund_initialized(
    env: &Env,
    sponsor_address: &Address,
    initial_balance: i128,
    max_per_txn: i128,
    max_per_user_day: i128,
    min_eligible_balance: i128,
) {
    let topics = (Symbol::new(env, "spons_init"),);
    env.events().publish(
        topics,
        (sponsor_address.clone(), initial_balance, max_per_txn, max_per_user_day, min_eligible_balance),
    );
}

pub fn gasless_transaction_executed(
    env: &Env,
    depositor: &Address,
    relayer: Option<&Address>,  // None if user-sponsored
    token: &Address,
    amount: i128,
    resource_fee_covered: i128,
    deposit_id: u32,
) {
    let topics = (Symbol::new(env, "gasless_exec"), depositor.clone());
    env.events().publish(
        topics,
        (relayer.cloned(), token.clone(), amount, resource_fee_covered, deposit_id),
    );
}

pub fn sponsorship_used(
    env: &Env,
    user: &Address,
    amount: i128,
    new_balance: i128,
) {
    let topics = (Symbol::new(env, "spons_used"), user.clone());
    env.events().publish(topics, (amount, new_balance));
}

pub fn sponsorship_fund_replenished(
    env: &Env,
    sponsor: &Address,
    amount: i128,
    new_balance: i128,
    source: Symbol,  // "penalty", "admin_deposit", etc.
) {
    let topics = (Symbol::new(env, "spons_replenish"), sponsor.clone());
    env.events().publish(topics, (amount, new_balance, source));
}

pub fn sybil_detected(
    env: &Env,
    user: &Address,
    reason: Symbol,  // "high_velocity", "similar_pattern", etc.
) {
    let topics = (Symbol::new(env, "sybil_detect"), user.clone());
    env.events().publish(topics, reason);
}

pub fn sponsorship_config_updated(
    env: &Env,
    admin: &Address,
    max_per_txn: i128,
    max_per_user_day: i128,
    min_eligible_balance: i128,
    cooldown_seconds: u64,
) {
    let topics = (Symbol::new(env, "spons_config"), admin.clone());
    env.events().publish(
        topics,
        (max_per_txn, max_per_user_day, min_eligible_balance, cooldown_seconds),
    );
}
```

---

## Relayer Integration Points

Relayers need to:

1. **Query sponsorship status** — Call `get_sponsorship_fund_status()` and `check_sponsorship_eligibility(user)`
2. **Estimate fees** — Calculate likely resource cost based on operation type
3. **Submit sponsored deposit** — Call `sponsored_deposit_for(relayer, user, token, amount, ...)`
4. **Monitor events** — Listen for `GaslessTransactionExecuted` and `SybilDetected` events

**Standard Workflow:**
```
1. User submits deposit request to relayer
2. Relayer checks eligibility: check_sponsorship_eligibility(user)
3. If eligible:
   a. Relayer simulates transaction to estimate fees
   b. Relayer calls sponsored_deposit_for(relayer_addr, user, token, amount, ...)
   c. Transaction is submitted on-chain
   d. Events are emitted and indexed
4. If not eligible:
   - Relayer returns error to user
   - User can retry later or use normal (non-sponsored) deposit
```

---

## Testing Strategy

### Unit Tests

1. **Sponsorship Fund Management**
   - Initialize sponsorship
   - Replenish from penalties
   - Admin top-ups
   - Config updates

2. **Eligibility & Rate Limiting**
   - KYC-lite (minimum balance) checks
   - Daily cap enforcement
   - Per-transaction cap
   - Cooldown periods

3. **Sponsored Deposits**
   - Successful sponsored deposit
   - Sponsored deposit for (relayer-funded)
   - Fund depletion scenarios
   - Edge cases (exactly-at-limit, off-by-one)

4. **Abuse Prevention**
   - Sybil detection triggers
   - Rate limit exhaustion
   - Cooldown enforcement
   - Multiple concurrent sponsorships

5. **Event Emission**
   - `GaslessTransactionExecuted` emitted correctly
   - `SponsorshipUsed` tracks accurately
   - `SybilDetected` fires on attacks

6. **Integration**
   - Sponsored deposits followed by normal operations
   - Withdrawal of sponsored deposits
   - Emergency withdrawal of sponsored deposits
   - Penalty accrual replenishing fund

### Property-Based Tests

- Sponsorship fund balance never goes negative
- Daily usage resets correctly per calendar day
- Cooldown periods enforced consistently
- Sybil counter increments monotonically

### Gas Profiling

- Measure resource cost of eligibility checks
- Ensure sponsored_deposit costs less than admin's sponsorship budget
- Validate sustainability over 1000s of users

---

## Out of Scope

- Running dedicated relayer infrastructure (users/relayers implement this)
- All ERC-2771 meta-transaction patterns (EVM-only)
- Governance-based sponsorship decisions (admin-controlled for v1)
- Multi-signature approval for sponsorship fund changes

---

## References

- Soroban Documentation: https://developers.stellar.org/
- Sponsorship concept: Related to Stellar's native protocol sponsorships but adapted for smart contracts
- Rate limiting patterns: Based on industry standard abuse prevention

