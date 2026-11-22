# cancel_deposit() - Visual Verification Guide

**Date**: 2026-10-19  
**Purpose**: Visual representation of time-lock check enforcement

---

## Function Call Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  cancel_deposit(&depositor, deposit_id)                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ require_auth()   │ ◄─── Auth-First Pattern
                    │ (line 1455)      │
                    └──────────────────┘
                              │
                              ▼
                  ┌───────────────────────┐
                  │ Try Timestamp-Based   │
                  │ Deposit Lookup        │
                  └───────────────────────┘
                         │            │
            Found ◄───────┘            └─────► Not Found
              │                               (Fall Through)
              ▼
        ┌──────────────────────┐
        │ Get: now = Ledger    │
        │ Timestamp            │
        │ (line 1459)          │
        └──────────────────────┘
              │
              ▼
    ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
    ┃  ⭐ TIME CHECK (CRITICAL) ┃
    ┃  if now >= unlock_time    ┃
    ┃  (line 1460)              ┃
    ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
         │              │
      TRUE             FALSE
         │              │
         ▼              ▼
    ┌─────────────┐  ┌────────────────────┐
    │   EARLY     │  │ [STATE MUTATIONS]  │
    │   RETURN    │  │ - Remove storage   │
    │ Error Code: │  │ - Calculate penalty│
    │     13      │  │ - Transfer tokens  │
    │(Vault       │  │ - Emit events      │
    │Already      │  │ - return Ok(())    │
    │Unlocked)    │  └────────────────────┘
    │ ✓ NO STATE  │
    │   CHANGED   │
    └─────────────┘
```

---

## Success vs Failure Comparison

### ✅ Scenario 1: WHILE LOCKED (Success)

```
Timeline: ──────[DEPOSIT]────→─ [CANCEL] ─────────→[UNLOCK]─────
                 Created at T0   at T0+1min         at T0+3600

Time Check: if now >= unlock_time
            if T0 >= T0+3600
            = FALSE ✅

Flow:
  1. Get timestamp: now = T0
  2. Check: T0 >= T0+3600? NO
  3. Proceed with cancellation
  4. Remove storage ✅
  5. Calculate penalty ✅
  6. Transfer tokens ✅
  7. Emit events ✅
  8. return Ok(())

State After:
  ✅ Deposit removed
  ✅ Tokens returned (with penalty)
  ✅ Events recorded
```

### ❌ Scenario 2: AFTER UNLOCK (Failure)

```
Timeline: ──────[DEPOSIT]─────────────────→[UNLOCK]─ [CANCEL] ─→
                 Created at T0              at T0+3600  at T0+3601

Time Check: if now >= unlock_time
            if T0+3601 >= T0+3600
            = TRUE ✅

Flow:
  1. Get timestamp: now = T0+3601
  2. Check: T0+3601 >= T0+3600? YES
  3. ⭐ EARLY RETURN with Error 13
  
  ❌ STOPPED HERE - NO FURTHER EXECUTION

State After:
  ✅ Deposit still exists (not removed)
  ✅ Token balances unchanged
  ✅ No events emitted
  ✅ Error returned to user
```

---

## State Transition Diagram

### Success Path (While Locked)

```
VAULT STATE TRANSITIONS

[Before Cancel]
┌──────────────────────────────────┐
│ Deposit(id=0)                    │
│  ├─ amount: 1000                 │
│  ├─ unlock_time: T0+3600         │
│  ├─ penalty_bps: 1000 (10%)      │
│  └─ status: ACTIVE               │
│                                  │
│ Alice Balance: 9000              │
│ Fee Recipient Balance: 0         │
│ Staker Rewards Pool: 0           │
└──────────────────────────────────┘
           │
           │ cancel_deposit(&alice, 0)  [called at T0]
           │ Time Check: T0 >= T0+3600? NO ✅
           ▼
[After Cancel Success]
┌──────────────────────────────────┐
│ Deposit(id=0): REMOVED           │
│                                  │
│ Alice Balance: 9900              │
│  (9000 + 900 refund)             │
│                                  │
│ Fee Recipient Balance: 30        │
│  (30% of 100 penalty)            │
│                                  │
│ Staker Rewards Pool: 70          │
│  (70% of 100 penalty)            │
└──────────────────────────────────┘
```

### Failure Path (After Unlock)

```
VAULT STATE TRANSITIONS

[Before Cancel Attempt]
┌──────────────────────────────────┐
│ Deposit(id=0)                    │
│  ├─ amount: 1000                 │
│  ├─ unlock_time: T0+3600         │
│  ├─ penalty_bps: 1000 (10%)      │
│  └─ status: ACTIVE               │
│                                  │
│ Alice Balance: 9000              │
│ Fee Recipient Balance: 0         │
│ Staker Rewards Pool: 0           │
└──────────────────────────────────┘
           │
           │ cancel_deposit(&alice, 0)  [called at T0+3601]
           │ Time Check: T0+3601 >= T0+3600? YES ❌
           │ EARLY RETURN: Error 13 (VaultAlreadyUnlocked)
           ▼
[After Cancel Failure]
┌──────────────────────────────────┐
│ Deposit(id=0)                    │
│  ├─ amount: 1000                 │
│  ├─ unlock_time: T0+3600         │
│  ├─ penalty_bps: 1000 (10%)      │
│  └─ status: ACTIVE               │
│  ↑ UNCHANGED ↑                   │
│                                  │
│ Alice Balance: 9000              │
│  ↑ UNCHANGED ↑                   │
│                                  │
│ Fee Recipient Balance: 0         │
│  ↑ UNCHANGED ↑                   │
│                                  │
│ Staker Rewards Pool: 0           │
│  ↑ UNCHANGED ↑                   │
│                                  │
│ ERROR: VaultAlreadyUnlocked      │
│  (Error Code: 13)                │
└──────────────────────────────────┘
```

---

## Penalty Calculation Flowchart

### Only Executed AFTER Time Check Passes

```
                           ┌──────────────────┐
                           │  Time Check OK   │
                           │  (now < unlock)  │
                           └──────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────┐
                    │ penalty = (amount *      │
                    │ penalty_bps) / 10000     │
                    │                          │
                    │ Example: (1000 * 1000) / │
                    │ 10000 = 100              │
                    └──────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────┐
                    │ refund = amount -        │
                    │ penalty                  │
                    │                          │
                    │ Example: 1000 - 100 =    │
                    │ 900                      │
                    └──────────────────────────┘
                                   │
                                   ▼
          ┌────────────────────────┴────────────────────────┐
          │                                                  │
          ▼                                                  ▼
    ┌─────────────────────┐                    ┌──────────────────────┐
    │ fee_share = penalty │                    │ staker_share =       │
    │ * 3000 / 10000      │                    │ penalty - fee_share  │
    │ = 100 * 3000 /      │                    │ = 100 - 30 = 70     │
    │   10000 = 30        │                    └──────────────────────┘
    └─────────────────────┘                              │
          │                                              │
          ▼                                              ▼
    ┌─────────────────────┐                    ┌──────────────────────┐
    │ Transfer 30 to Fee  │                    │ Add 70 to Rewards    │
    │ Recipient           │                    │ Pool                 │
    └─────────────────────┘                    └──────────────────────┘
          │                                              │
          ▼                                              ▼
    ┌─────────────────────────────────────────────────────────────┐
    │ Transfer 900 refund to User                                 │
    │ Emit Events                                                 │
    │ return Ok(())                                               │
    └─────────────────────────────────────────────────────────────┘
```

---

## Time Check Decision Tree

```
                    ┌─────────────────────────┐
                    │ Call cancel_deposit()   │
                    └──────────────┬──────────┘
                                   │
                          ┌────────▼────────┐
                          │ Require Auth    │
                          └────────┬────────┘
                                   │
                   ┌───────────────▼───────────────┐
                   │ Get Deposit Entry from Storage│
                   └───────────────┬───────────────┘
                                   │
                       ┌───────────▼────────────┐
                       │ Get Current Timestamp  │
                       │ (or Ledger Sequence)   │
                       └───────────┬────────────┘
                                   │
              ┌────────────────────▼────────────────────┐
              │   ⭐ CRITICAL TIME CHECK ⭐              │
              │   if now >= entry.unlock_time           │
              │   (or: current_ledger >= unlock_ledger) │
              └────────────────────┬────────────────────┘
                                   │
                  ┌────────────────┴───────────────┐
                  │                                │
              TRUE │                          FALSE│
                  ▼                                ▼
         ┌──────────────────┐        ┌────────────────────┐
         │ EARLY RETURN     │        │ PROCEED WITH       │
         │                  │        │ STATE MUTATIONS    │
         │ Error Code: 13   │        │                    │
         │ "VaultAlready    │        │ ✅ Remove storage  │
         │  Unlocked"       │        │ ✅ Transfer tokens │
         │                  │        │ ✅ Emit events     │
         │ ❌ NO CHANGES    │        │ ✅ return Ok(())   │
         └──────────────────┘        └────────────────────┘
```

---

## Test Matrix

### Decision Grid: When Can cancel_deposit() Succeed?

```
                    │  Deposit Time (T0)
                    │
      Current Time  │  Unlocked After  │  During Lock  │  Way Future
      Relative to   │  T0+1h (100%)    │   (Within Hour)│ (>1 year later)
      Lock Time     │  Penalty: 10%    │   Penalty: 10% │ Penalty: 10%
    ────────────────┼──────────────────┼────────────────┼───────────────
    Just Created    │  ✅ OK           │  ✅ OK         │  ✅ OK
    (T0 + 1 sec)    │  Penalty: 100bps │  Penalty: 100  │  Penalty: 100
                    │                  │  Refund: 900   │  Refund: 900
    ────────────────┼──────────────────┼────────────────┼───────────────
    Near Unlock     │  ✅ OK           │  ✅ OK         │  ✅ OK
    (T0 + 3599s)    │  Penalty: 100    │  Penalty: 100  │  Penalty: 100
                    │  Refund: 900     │  Refund: 900   │  Refund: 900
    ────────────────┼──────────────────┼────────────────┼───────────────
    At Unlock       │  ❌ ERROR 13     │  ❌ ERROR 13   │  ❌ ERROR 13
    (T0 + 3600s)    │  VaultAlready    │  VaultAlready  │  VaultAlready
                    │  Unlocked        │  Unlocked      │  Unlocked
    ────────────────┼──────────────────┼────────────────┼───────────────
    Past Unlock     │  ❌ ERROR 13     │  ❌ ERROR 13   │  ❌ ERROR 13
    (T0 + 3601s)    │  VaultAlready    │  VaultAlready  │  VaultAlready
                    │  Unlocked        │  Unlocked      │  Unlocked
    ────────────────┼──────────────────┼────────────────┼───────────────
    Long After      │  ❌ ERROR 13     │  ❌ ERROR 13   │  ❌ ERROR 13
    (T0 + 1year)    │  VaultAlready    │  VaultAlready  │  VaultAlready
                    │  Unlocked        │  Unlocked      │  Unlocked

Key: ✅ OK = Cancellation succeeds with penalty
     ❌ ERROR 13 = Cancellation fails, user should call withdraw()
```

---

## Code Location Quick Reference

```
Function: cancel_deposit()
File: contracts/safe-haven/src/contract.rs
Line: 1454-1580

Key Sections:
  • Auth check:           Line 1455
  • Timestamp check:      Lines 1458-1461
  • Ledger check:         Lines 1515-1520
  • Multi-token check:    Lines 1559-1564
  • State removal:        Lines 1476-1479
  • Penalty calculation:  Line 1484
  • Penalty split:        Lines 1487-1488
  • Token transfers:      Lines 1498-1502
  • Event emission:       Lines 1507-1508

Error Definition:
  File: contracts/safe-haven/src/errors.rs
  Line: 19
  Code: VaultAlreadyUnlocked = 13

Tests:
  File: contracts/safe-haven/src/test.rs
  Tests: Multiple covering all scenarios
  - test_cancel_deposit_zero_penalty_returns_full_amount()
  - test_cancel_deposit_after_unlock_fails()
  - test_cancel_deposit_partial_penalty_splits_correctly()
  - test_cancel_ledger_deposit_after_unlock_fails()
```

---

## Performance Impact

### No State Change on Error

```
On Success:                      │ On Error:
✅ Remove deposit from storage   │ ❌ Nothing removed
✅ Transfer penalty to fee       │ ❌ No transfers
✅ Add to staker pool            │ ❌ No pool updates
✅ Transfer refund to user       │ ❌ No refunds
✅ Emit events                   │ ❌ No events
                                 │
Ledger writes: ~5-10             │ Ledger writes: 0
Gas cost: Higher                 │ Gas cost: Lower
                                 │ (error path is cheaper)
```

---

## Conclusion

The `cancel_deposit()` function implements a robust time-lock check that:

1. ✅ **Occurs first** - Before any state changes
2. ✅ **Returns immediately** - On expiry with error code 13
3. ✅ **Works consistently** - Across all deposit types
4. ✅ **Is clearly tested** - All paths verified
5. ✅ **Prevents exploitation** - No late-time penalty claiming

The visual guides above demonstrate that the implementation correctly enforces the time-lock requirement and prevents the identified attack vector.

