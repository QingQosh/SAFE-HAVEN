# cancel_deposit() Technical Analysis: Time-Lock Verification

**File**: `contracts/safe-haven/src/contract.rs:1454-1580`  
**Purpose**: Verify the function correctly checks that funds are still locked before allowing early exit with penalty

---

## Function Architecture

### Entry Point
```rust
pub fn cancel_deposit(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError>
```

### High-Level Flow

```
Input: depositor, deposit_id
  │
  ├─→ [1] Verify authentication
  │     depositor.require_auth() ← Auth-first pattern
  │
  ├─→ [2] TRY TIMESTAMP-BASED DEPOSIT
  │     ├─ Retrieve entry from storage
  │     ├─ Get current timestamp
  │     ├─ [CRITICAL] Check: now >= unlock_time?
  │     │   ├─ YES → return VaultAlreadyUnlocked (early exit, NO STATE CHANGE)
  │     │   └─ NO  → proceed to cancellation
  │     ├─ Accrue interest if applicable
  │     ├─ Remove storage entries
  │     ├─ Calculate penalty
  │     ├─ Split penalty: fee (30%) + stakers pool (70%)
  │     ├─ Transfer tokens
  │     └─ Emit events → return Ok(())
  │
  ├─→ [3] TRY LEDGER-BASED DEPOSIT
  │     ├─ Retrieve entry from storage
  │     ├─ Get current ledger sequence
  │     ├─ [CRITICAL] Check: current_ledger >= unlock_ledger?
  │     │   ├─ YES → return VaultAlreadyUnlocked (early exit, NO STATE CHANGE)
  │     │   └─ NO  → proceed to cancellation
  │     ├─ Remove storage entries
  │     ├─ Calculate penalty
  │     ├─ Split penalty
  │     ├─ Transfer tokens
  │     └─ Emit events → return Ok(())
  │
  ├─→ [4] TRY MULTI-TOKEN DEPOSIT
  │     ├─ Retrieve entry from storage
  │     ├─ Get current timestamp
  │     ├─ [CRITICAL] Check: now >= unlock_time?
  │     │   ├─ YES → return VaultAlreadyUnlocked (early exit, NO STATE CHANGE)
  │     │   └─ NO  → proceed
  │     ├─ [ADDITIONAL] Check withdrawal delay
  │     │   ├─ ACTIVE → return WithdrawalDelayActive (early exit, NO STATE CHANGE)
  │     │   └─ EXPIRED → proceed to cancellation
  │     ├─ Remove storage entries
  │     ├─ For each token in multi-token deposit:
  │     │   ├─ Calculate penalty
  │     │   ├─ Transfer fee to recipient
  │     │   ├─ Transfer refund to depositor
  │     │   └─ Emit events
  │     └─ return Ok(())
  │
  └─→ [5] NO DEPOSIT FOUND
        return NoDepositFound
```

---

## Critical Time-Check Implementation

### Timestamp-Based Deposits (Lines 1458-1463)

**Code**:
```rust
if let Some(mut entry) = storage::get_deposit(&env, &depositor, deposit_id) {
    let now = env.ledger().timestamp();              // Get current time
    if now >= entry.unlock_time {                    // Check if expired
        return Err(VaultError::VaultAlreadyUnlocked); // EARLY RETURN
    }
    // State mutations ONLY happen after this check
    // ...
}
```

**Detailed Flow**:
1. `storage::get_deposit()` - Attempts to retrieve the deposit entry (read-only)
   - If not found, falls through to next deposit type check
   - If found, enters the block and gets `mut entry`

2. `env.ledger().timestamp()` - Gets current ledger timestamp in seconds
   - Type: `u64`
   - Represents seconds since epoch

3. **TIME COMPARISON** ← THE CRITICAL CHECK
   ```
   if now >= entry.unlock_time {
       return Err(VaultError::VaultAlreadyUnlocked);
   }
   ```
   - Condition: `now >= entry.unlock_time`
   - If **TRUE**: Deposit is unlocked → Cannot cancel with penalty
   - If **FALSE**: Deposit is still locked → Can proceed with cancellation

4. **Early Return on Expiry**
   - Returns immediately with error code 13
   - **NO state changes** have occurred yet
   - Storage is untouched
   - No tokens transferred
   - No events emitted

**Important**: The `>=` operator means the deposit is considered unlocked at the **exact moment** `now == unlock_time`. This is correct because at that moment, the user could call `withdraw()` instead and receive the full amount.

---

### Ledger-Based Deposits (Lines 1515-1520)

**Code**:
```rust
if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
    let current_ledger = env.ledger().sequence();   // Get current ledger
    if current_ledger >= entry.unlock_ledger {      // Check if expired
        return Err(VaultError::VaultAlreadyUnlocked); // EARLY RETURN
    }
    // State mutations ONLY happen after this check
    // ...
}
```

**Detailed Flow**:
1. `storage::get_deposit_by_ledger_readonly()` - Retrieves ledger-based deposit (read-only)
   - Uses readonly variant to avoid unnecessary TTL bumps
   - If not found, falls through to next check

2. `env.ledger().sequence()` - Gets current ledger sequence number
   - Type: `u32`
   - Represents the ledger number in the blockchain

3. **LEDGER COMPARISON** ← THE CRITICAL CHECK
   ```
   if current_ledger >= entry.unlock_ledger {
       return Err(VaultError::VaultAlreadyUnlocked);
   }
   ```
   - Condition: `current_ledger >= entry.unlock_ledger`
   - If **TRUE**: Unlock ledger reached → Cannot cancel with penalty
   - If **FALSE**: Unlock ledger not reached → Can proceed with cancellation

4. **Early Return on Expiry**
   - Same pattern as timestamp-based deposits
   - Early return prevents any state changes

---

### Multi-Token Deposits (Lines 1559-1569)

**Code**:
```rust
if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
    let now = env.ledger().timestamp();              // Get current time
    if now >= entry.unlock_time {                    // Check if expired
        return Err(VaultError::VaultAlreadyUnlocked); // EARLY RETURN
    }
    let withdrawal_time = entry.unlock_time.saturating_add(entry.withdrawal_delay_secs);
    if now < withdrawal_time {                       // Check if delay active
        return Err(VaultError::WithdrawalDelayActive); // EARLY RETURN
    }
    // State mutations ONLY happen after BOTH checks
    // ...
}
```

**Detailed Flow**:
1. First time check (same as timestamp-based)
2. **Additional check**: Withdrawal delay
   - Multi-token deposits have a configurable withdrawal delay
   - Even after unlock_time, user must wait additional delay period
   - Formula: `withdrawal_time = unlock_time + withdrawal_delay_secs`
   - If `now < withdrawal_time`, delay is still active → cannot cancel

3. **Double Gate Protection**
   - Gate 1: Is unlock_time reached? (deposit locked?)
   - Gate 2: Is withdrawal_delay active? (post-unlock delay?)
   - Both must pass before cancellation allowed

---

## Order of Operations: Checks vs. Effects

### What Happens BEFORE the Time Check?

```rust
pub fn cancel_deposit(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
    depositor.require_auth();  // ← ONLY this
    
    // Everything else happens AFTER the time check
    if let Some(mut entry) = storage::get_deposit(&env, &depositor, deposit_id) {
        let now = env.ledger().timestamp();
        if now >= entry.unlock_time {  // ← TIME CHECK HERE
            return Err(VaultError::VaultAlreadyUnlocked);
        }
        // ... rest of code ...
    }
}
```

**Pre-Check Operations**:
1. ✅ `require_auth()` - Verify caller authorization
2. ✅ `storage::get_deposit()` - Read entry (no state change)
3. ✅ `env.ledger().timestamp()` - Read current time (no state change)

**All non-reversible operations happen AFTER the check.**

### What Happens AFTER the Time Check Passes?

```rust
// State mutations (in order):
storage::remove_deposit(&env, &depositor, deposit_id);              // Remove from storage
storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id); // Remove whitelist
if storage::get_deposit_ids(&env, &depositor).len() == 0 {
    storage::remove_depositor(&env, &depositor);                    // Remove if no deposits
}

// Penalty calculation (no state change yet)
let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
let refund = entry.amount.saturating_sub(penalty);

// Penalty splitting
let fee_recipient_share: i128 = (penalty * FEE_RECIPIENT_PENALTY_BPS as i128) / 10_000;
let stakers_share: i128 = penalty - fee_recipient_share;

// Token transfers (irreversible)
token_client.transfer(&contract, &fee_recipient, &fee_recipient_share);
token_client.transfer(...);  // stakers pool
token_client.transfer(&contract, &depositor, &refund);

// Events (immutable record)
events::penalty_split(...);
events::deposit_cancelled(...);

return Ok(());
```

### State Rollback on Time-Check Failure?

**Answer**: No rollback needed.

If the time check fails (`now >= entry.unlock_time`):
- ❌ Storage was not removed (deposit still exists)
- ❌ No tokens were transferred (balances unchanged)
- ❌ No events were emitted (no record of attempt)
- ✅ Function returns with error code 13

The transaction reverts naturally if called by a user; no manual rollback needed because **no state was changed**.

---

## Boundary Conditions

### Exact Unlock Time Scenario

**Setup**:
```
unlock_time = 1000 seconds
current_time = 1000 seconds
```

**Check**: `if now >= entry.unlock_time` → `if 1000 >= 1000` → TRUE

**Result**: Returns `VaultAlreadyUnlocked` ✅ CORRECT

**Reasoning**: At the exact unlock moment, user should call `withdraw()` instead of `cancel_deposit()`. The penalty mechanism should not apply.

---

### Just Before Unlock

**Setup**:
```
unlock_time = 1000 seconds
current_time = 999 seconds
```

**Check**: `if now >= entry.unlock_time` → `if 999 >= 1000` → FALSE

**Result**: Proceeds with cancellation ✅ CORRECT

**Reasoning**: While still locked, early exit with penalty is allowed.

---

### Long After Unlock

**Setup**:
```
unlock_time = 1000 seconds
current_time = 1_000_000 seconds
```

**Check**: `if now >= entry.unlock_time` → `if 1_000_000 >= 1000` → TRUE

**Result**: Returns `VaultAlreadyUnlocked` ✅ CORRECT

**Reasoning**: No time limit on calling `withdraw()`. User can withdraw anytime after unlock.

---

## Error Handling

### Error Code Definition

**File**: `contracts/safe-haven/src/errors.rs:19`

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    InvalidAmount = 1,
    UnlockTimeNotInFuture = 2,
    NoDepositFound = 3,
    FundsStillLocked = 4,
    DepositAlreadyExists = 5,
    LockDurationTooLong = 6,
    Unauthorized = 7,
    AmountTooLarge = 8,
    InvalidPenaltyBps = 9,
    InvalidAdmin = 10,
    LockDurationTooShort = 11,
    ContractPaused = 12,
    VaultAlreadyUnlocked = 13,  ← Used by cancel_deposit()
    MissingFeeRecipient = 14,
    // ...
}
```

### Error Propagation

When `cancel_deposit()` encounters an expired lock:

```rust
if now >= entry.unlock_time {
    return Err(VaultError::VaultAlreadyUnlocked);  // Convert to Result::Err
}
```

The `Result<(), VaultError>::Err(...)` bubbles up to caller with error code 13.

### User-Facing Behavior

**Frontend/SDK Receives**: Error code 13  
**Display**: "Funds are no longer locked. Use withdraw() to claim full amount without penalty."

---

## Penalty Calculation (Only If Lock Still Active)

This code only executes **after** the time check passes:

```rust
// Only reached if: now < entry.unlock_time
let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
let refund = entry.amount.saturating_sub(penalty);

// Penalty split: 30% fee recipient, 70% stakers pool
let fee_recipient_share: i128 = (penalty * FEE_RECIPIENT_PENALTY_BPS as i128) / 10_000;
let stakers_share: i128 = penalty - fee_recipient_share;

// Transfer fee to recipient (early exit penalty)
token_client.transfer(&contract, &fee_recipient, &fee_recipient_share);

// Add to stakers rewards pool (incentivize staking)
let current_pool = storage::get_rewards_pool(&env);
storage::set_rewards_pool(&env, current_pool + stakers_share);

// Return remaining funds to user
token_client.transfer(&contract, &depositor, &refund);
```

**Key Point**: This penalty distribution only happens if the time check passes.

---

## State Cleaning (Only If Lock Still Active)

This code only executes **after** the time check passes:

```rust
// Remove NFT evolution record (if deposit was NFT-backed)
storage::remove_nft_evolution(&env, &depositor, deposit_id);

// Remove sustainability metrics (carbon tracking)
storage::remove_sustainability_metrics(&env, &depositor, deposit_id);

// Remove withdrawal whitelist
storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);

// Remove deposit entry
storage::remove_deposit(&env, &depositor, deposit_id);

// Remove depositor if no remaining deposits
if storage::get_deposit_ids(&env, &depositor).len() == 0 {
    storage::remove_depositor(&env, &depositor);
}
```

All cleanup happens **after** time verification.

---

## Comparison with withdraw()

### cancel_deposit() - Early Exit

```rust
if now >= entry.unlock_time {
    return Err(VaultAlreadyUnlocked);  // Cannot call when expired
}
// Penalty applied
// User gets: amount - penalty
```

### withdraw() - Normal Withdrawal

```rust
if now < entry.unlock_time {
    return Err(FundsStillLocked);  // Cannot call while locked
}
// No penalty
// User gets: full amount
```

**Inverse Conditions**: Each function precisely rejects the other's valid scenario.

---

## Conclusion

The `cancel_deposit()` function implements a robust time-lock verification system:

✅ **Time check occurs first** - Before any state changes  
✅ **Early return on expiry** - Prevents penalty claiming after unlock  
✅ **Clear error message** - Error code 13 (`VaultAlreadyUnlocked`)  
✅ **Works for all deposit types** - Timestamp, ledger, multi-token  
✅ **No state rollback needed** - Check happens before mutations  
✅ **Boundary conditions correct** - Uses `>=` operator appropriately  

The implementation follows Soroban security best practices and prevents the intended attack vector: claiming penalties after becoming eligible for penalty-free withdrawal.

