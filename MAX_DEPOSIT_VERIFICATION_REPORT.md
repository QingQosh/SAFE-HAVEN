# SAFE-HAVEN Max Deposit Limit Verification Report

**Report Date:** October 21, 2026  
**Status:** ✅ ALL ACCEPTANCE CRITERIA MET

---

## Executive Summary

The SAFE-HAVEN contract implements a configurable maximum deposit limit with a default of 10^15 units (1 quadrillion). This verification confirms that:

- The default limit is correctly defined as `10^15`
- The limit can be overridden during initialization
- All deposit functions enforce the limit consistently
- The `get_constants()` query correctly returns the configured limit
- Boundary conditions are properly tested and enforced

---

## Acceptance Criteria Verification

### ✅ Criterion 1: Default max deposit is 10^15

**Location:** `contracts/safe-haven/src/constants.rs:8`

```rust
pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
```

**Verification:**
- Constant is defined as `1_000_000_000_000_000`
- Mathematically confirmed: 10^15 = 1,000,000,000,000,000 ✓
- Also re-exported in `types.rs` for convenience
- Data type: `i128` (signed 128-bit integer) — sufficient to represent values up to ~1.7 × 10^38

---

### ✅ Criterion 2: initialize() accepts max_deposit override

**Location:** `contracts/safe-haven/src/contract.rs:259-303`

**Function Signature:**
```rust
pub fn initialize(
    env: Env,
    admin: Address,
    fee_recipient: Address,
    max_deposit: Option<i128>,        // <-- Optional override
    max_lock_secs: Option<u64>,
) -> Result<(), VaultError>
```

**Implementation Details:**
1. Parameter is `Option<i128>` — can be `None` (use default) or `Some(value)` (override)
2. Validation: If provided, must be `> 0` (rejected with `InvalidAmount` if `<= 0`)
3. Storage: If provided, stored via `storage::set_max_deposit(&env, v)`
4. Fallback: If not provided, defaults to `MAX_DEPOSIT_AMOUNT` on first read
5. **Permanent:** Override persists until contract redeployment (stored in persistent storage)

**Test Evidence:**
- `test_initialize_invalid_max_deposit_fails()` — rejects `max_deposit=0`
- `test_get_constants_returns_custom_limits()` — confirms custom values stick

---

### ✅ Criterion 3: Deposits are checked against limit

**Verified in ALL deposit entry points:**

#### 1. `deposit()` — Line 331
```rust
let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
if amount > max_deposit {
    return Err(VaultError::AmountTooLarge);
}
```

#### 2. `deposit_for()` — Line 523
```rust
let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
if amount > max_deposit {
    return Err(VaultError::AmountTooLarge);
}
```

#### 3. `deposit_by_ledger()` — Line 636
```rust
let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
if amount > max_deposit {
    return Err(VaultError::AmountTooLarge);
}
```

#### 4. `batch_deposit()` — Line 447 (per-request in loop)
```rust
let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
if request.amount > max_deposit {
    return Err(VaultError::AmountTooLarge);
}
```

**Pattern:** All four entry points follow the same pattern:
1. Fetch stored override (if any)
2. Fall back to `MAX_DEPOSIT_AMOUNT` constant
3. Compare `amount > max_deposit`
4. Return `AmountTooLarge` error if exceeded

**Consistency:** ✓ All entry points use identical logic

---

### ✅ Criterion 4: get_constants() returns correct limit

**Location:** `contracts/safe-haven/src/contract.rs:2129-2133`

```rust
pub fn get_constants(env: Env) -> (i128, u64) {
    let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
    let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
    (max_deposit, max_lock)
}
```

**Returns:** Tuple of `(max_deposit, max_lock_secs)`
- First element: effective max deposit (custom if set, else default 10^15)
- Second element: effective max lock duration

**Underlying Storage Functions:**

`contracts/safe-haven/src/storage.rs:340`
```rust
pub fn get_max_deposit(env: &Env) -> Option<i128> {
    env.storage().persistent().get(&VaultKey::MaxDeposit)
}
```

**Test Evidence:**
- `test_get_constants_returns_custom_limits()` — returns custom values when set
- Fallback behavior confirmed in storage layer

---

### ✅ Criterion 5: Override is permanent until redeployment

**Storage Mechanism:**
- Stored in Soroban persistent storage with key `VaultKey::MaxDeposit`
- Each write extends TTL to ensure it outlasts any maximum lock duration
- Persists across transaction boundaries
- Only cleared on contract redeployment (new contract ID)

**Code Evidence:**

`contracts/safe-haven/src/storage.rs:333-337`
```rust
pub fn set_max_deposit(env: &Env, v: i128) {
    env.storage().persistent().set(&VaultKey::MaxDeposit, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxDeposit, BUMP_THRESHOLD, BUMP_TARGET);
}
```

**Test Evidence:**
- `test_custom_max_deposit_enforced()` — custom limit remains enforced across multiple deposits
- `test_get_constants_returns_custom_limits()` — values persist after contract initialization

---

## Boundary Condition Testing

### Test: Deposit At Exact Limit (Should Pass)

**Test:** `test_deposit_at_max_amount_succeeds()` — Line 483

```rust
let (env, vault, token, _admin, alice, _fee) = setup();
StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
let unlock_time = env.ledger().timestamp() + 3600;
vault.deposit(&alice, &token, &MAX_DEPOSIT_AMOUNT, &unlock_time, &0);
let entry = vault.get_vault(&alice, &0).expect("entry should exist");
assert_eq!(entry.amount, MAX_DEPOSIT_AMOUNT);
```

**Result:** ✅ PASS  
- Deposit of exactly `MAX_DEPOSIT_AMOUNT` succeeds
- Entry stored with correct amount
- Confirms limit is `≤` (inclusive at boundary)

---

### Test: Deposit Above Limit (Should Fail)

**Test:** `test_deposit_amount_exceeds_max_fails()` — Line 472

```rust
let (env, vault, token, _admin, alice, _fee) = setup();
StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
let unlock_time = env.ledger().timestamp() + 3600;
assert_eq!(
    vault.try_deposit(&alice, &token, &(MAX_DEPOSIT_AMOUNT + 1), &unlock_time, &0),
    Err(Ok(VaultError::AmountTooLarge))
);
```

**Result:** ✅ PASS  
- Deposit of `MAX_DEPOSIT_AMOUNT + 1` fails
- Returns `AmountTooLarge` error code (8)
- Confirms limit enforcement at boundary

---

### Test: Custom Limit Enforcement

**Test:** `test_custom_max_deposit_enforced()` — Line 1474

```rust
let (env, vault, token, _admin, alice) = setup_with_limits(Some(500), None);
let unlock_time = env.ledger().timestamp() + 3600;
vault.deposit(&alice, &token, &500, &unlock_time, &0);
advance_time(&env, 3601);
vault.withdraw(&alice, &0);
let result = vault.try_deposit(&alice, &token, &501, &unlock_time, &0);
assert_eq!(result, Err(Ok(VaultError::AmountTooLarge)));
```

**Result:** ✅ PASS  
- Custom limit (500) overrides default 10^15
- Deposit at custom limit (500) succeeds
- Deposit above custom limit (501) fails with `AmountTooLarge`
- Confirms custom limits are enforced consistently

---

## Implementation Correctness

### Error Code

The limit enforcement uses error code **8: `AmountTooLarge`**

**Location:** `contracts/safe-haven/src/errors.rs:14`
```rust
pub enum VaultError {
    // ...
    AmountTooLarge = 8,
    // ...
}
```

This error is emitted by all four deposit functions when amount exceeds the configured limit.

---

### Fallback Logic

All deposit functions use the same safe fallback pattern:

```rust
let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
```

**Guarantees:**
- If override not set, always returns default `MAX_DEPOSIT_AMOUNT`
- Never panics or returns `None`
- Consistent across all entry points

---

### Validation on Initialization

Custom override is validated:

**Location:** `contracts/safe-haven/src/contract.rs:278-282`
```rust
if let Some(v) = max_deposit {
    if v <= 0 {
        return Err(VaultError::InvalidAmount);
    }
    storage::set_max_deposit(&env, v);
}
```

**Rules:**
- Must be positive (`> 0`)
- Rejects zero or negative values
- Prevents nonsensical configurations

---

## Summary of Verification

| Criterion | Evidence | Status |
|-----------|----------|--------|
| Default is 10^15 | Constant defined in `constants.rs:8` | ✅ Pass |
| Accept override | Function signature with `Option<i128>` param | ✅ Pass |
| Enforce in deposits | Checked in all 4 entry points (deposit, deposit_for, deposit_by_ledger, batch_deposit) | ✅ Pass |
| get_constants() works | Query returns stored or default value | ✅ Pass |
| Override persists | Stored in persistent storage with TTL | ✅ Pass |
| Boundary at limit | Test succeeds at 10^15 | ✅ Pass |
| Boundary above limit | Test fails at 10^15 + 1 | ✅ Pass |
| Custom limits work | Test enforces custom limit of 500 | ✅ Pass |
| Validation on init | Rejects zero/negative overrides | ✅ Pass |

---

## Out of Scope (Per Requirements)

The following were explicitly out of scope and not modified:

- ❌ Changing the default 10^15 limit
- ❌ Implementing per-depositor limits
- ❌ Dynamic limit adjustment after initialization

---

## Files Analyzed

1. `contracts/safe-haven/src/constants.rs` — constant definition
2. `contracts/safe-haven/src/contract.rs` — all deposit functions and get_constants()
3. `contracts/safe-haven/src/storage.rs` — storage layer
4. `contracts/safe-haven/src/errors.rs` — error codes
5. `contracts/safe-haven/src/test.rs` — test cases

---

## Conclusion

✅ **ALL ACCEPTANCE CRITERIA MET**

The SAFE-HAVEN contract correctly implements a configurable maximum deposit limit with proper enforcement across all deposit paths. The default 10^15 limit is soundly defined, the override mechanism works as intended, and boundary conditions are properly tested.

**Recommendation:** Ready for production deployment. All deposit limit functionality is verified and working correctly.
