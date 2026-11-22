# Test Additions Summary: Read-Only Query Functions

**Date:** 2026-10-21  
**File Modified:** `contracts/safe-haven/src/test.rs`  
**Total Tests Added:** 18

---

## Overview

Added comprehensive test coverage for read-only query functions (`get_admin()`, `get_pending_admin()`, `get_fee_recipient()`, `get_constants()`) to verify:

1. Correct return values
2. Consistent behavior across multiple sequential reads
3. Proper state reflection across admin lifecycle
4. No interference with mutating operations
5. Proper handling of initial/empty states

---

## Test Categories

### 1. Basic Value Retrieval (4 tests)

These tests verify that query functions return the correct configured values.

#### `test_get_admin_returns_current_admin()`
- **Purpose:** Verify `get_admin()` returns the admin address set during init
- **Assertions:** `vault.get_admin() == Some(admin)`

#### `test_get_admin_returns_none_after_renounce()`
- **Purpose:** Verify `get_admin()` returns `None` after admin is renounced
- **Setup:** Call `vault.renounce_admin(&admin)`
- **Assertions:** `vault.get_admin() == None`

#### `test_get_pending_admin_returns_none_initially()`
- **Purpose:** Verify `get_pending_admin()` returns `None` before any transfer
- **Assertions:** `vault.get_pending_admin() == None`

#### `test_get_fee_recipient_returns_configured_recipient()`
- **Purpose:** Verify `get_fee_recipient()` returns the configured recipient
- **Assertions:** `vault.get_fee_recipient() == Some(fee)`

---

### 2. Pending Admin State Transitions (3 tests)

These tests verify `get_pending_admin()` correctly reflects state changes through the admin transfer lifecycle.

#### `test_get_pending_admin_returns_pending_after_transfer()`
- **Purpose:** Verify `get_pending_admin()` returns the pending admin after transfer is initiated
- **Setup:** `vault.transfer_admin(&admin, &alice)`
- **Assertions:** `vault.get_pending_admin() == Some(alice)`

#### `test_get_pending_admin_returns_none_after_accept()`
- **Purpose:** Verify `get_pending_admin()` returns `None` after pending admin accepts
- **Setup:** 
  - `vault.transfer_admin(&admin, &alice)`
  - `vault.accept_admin(&alice)`
- **Assertions:** `vault.get_pending_admin() == None`

#### `test_get_pending_admin_returns_none_after_cancel()`
- **Purpose:** Verify `get_pending_admin()` returns `None` after transfer is cancelled
- **Setup:**
  - `vault.transfer_admin(&admin, &alice)`
  - `vault.cancel_transfer_admin(&admin)`
- **Assertions:** `vault.get_pending_admin() == None`

---

### 3. Constants Queries (3 tests)

These tests verify `get_constants()` returns both default and custom limits correctly.

#### `test_get_constants_returns_default_limits()`
- **Purpose:** Verify `get_constants()` returns compiled-in default limits when no custom values set
- **Assertions:** 
  - `max_amount == MAX_DEPOSIT_AMOUNT`
  - `max_duration == MAX_LOCK_DURATION_SECS`

#### `test_get_constants_returns_custom_limits()`
- **Purpose:** Verify `get_constants()` returns custom limits set during init
- **Setup:** `setup_with_limits(Some(50_000), Some(7200))`
- **Assertions:**
  - `max_amount == 50_000`
  - `max_duration == 7200`

#### `test_get_constants_partial_custom_limits()`
- **Purpose:** Verify `get_constants()` returns default for unspecified limit and custom for specified limit
- **Setup:** `setup_with_limits(None, Some(1800))`
- **Assertions:**
  - `max_amount == MAX_DEPOSIT_AMOUNT` (default)
  - `max_duration == 1800` (custom)

---

### 4. Consistency Across Multiple Reads (4 tests)

These tests verify that repeated calls to the same query function return identical results, confirming readonly behavior and no side effects.

#### `test_multiple_get_admin_calls_are_consistent()`
- **Purpose:** Verify three sequential `get_admin()` calls return the same value
- **Assertions:** `call1 == call2 == call3 == Some(admin)`

#### `test_multiple_get_pending_admin_calls_are_consistent()`
- **Purpose:** Verify three sequential `get_pending_admin()` calls return the same value
- **Setup:** `vault.transfer_admin(&admin, &alice)`
- **Assertions:** `call1 == call2 == call3 == Some(alice)`

#### `test_multiple_get_fee_recipient_calls_are_consistent()`
- **Purpose:** Verify three sequential `get_fee_recipient()` calls return the same value
- **Assertions:** `call1 == call2 == call3 == Some(fee)`

#### `test_multiple_get_constants_calls_are_consistent()`
- **Purpose:** Verify three sequential `get_constants()` calls return the same value
- **Setup:** `setup_with_limits(Some(75_000), Some(10800))`
- **Assertions:** All three calls return `(75_000, 10800)`

---

### 5. Integration: Readonly Queries with Mutations (2 tests)

These tests verify that readonly queries don't interfere with subsequent mutating operations.

#### `test_readonly_queries_dont_block_admin_transfer()`
- **Purpose:** Verify multiple readonly queries don't prevent subsequent mutations
- **Flow:**
  1. Perform 4 readonly query calls (admin, pending_admin, constants, admin)
  2. Call `vault.transfer_admin(&admin, &alice)` (mutation)
  3. Call `vault.accept_admin(&alice)` (mutation)
- **Assertions:**
  - `vault.get_pending_admin() == Some(alice)` after transfer
  - `vault.get_admin() == Some(alice)` after accept

#### `test_admin_queries_across_transfer_lifecycle()`
- **Purpose:** Verify queries correctly reflect state changes at each lifecycle stage
- **Flow:**
  1. Query initial admin/pending_admin state (before transfer)
  2. Initiate transfer, query state (after transfer proposal)
  3. Accept transfer, query state (after acceptance)
- **Assertions:**
  - Before: `admin != None`, `pending_admin == None`
  - After proposal: `admin unchanged`, `pending_admin == alice`
  - After accept: `admin == alice`, `pending_admin == None`

---

### 6. Edge Cases & Persistence (3 tests)

These tests verify query functions handle edge cases and maintain correctness across various operations.

#### `test_fee_recipient_unchanged_across_operations()`
- **Purpose:** Verify `get_fee_recipient()` remains consistent even after deposits and cancellations (operations that use the fee recipient)
- **Flow:**
  1. Query fee recipient
  2. Make deposit with penalty
  3. Query fee recipient again
  4. Cancel deposit (which uses fee recipient)
  5. Query fee recipient again
- **Assertions:** Fee recipient unchanged across all operations

#### `test_get_constants_persists_across_queries()`
- **Purpose:** Verify `get_constants()` returns same value across many sequential calls (10 iterations)
- **Setup:** `setup_with_limits(Some(100_000), Some(2592000))`
- **Flow:** 10-iteration loop calling `get_constants()`
- **Assertions:** All 10 calls return identical values `(100_000, 2592000)`

---

## Test Patterns & Best Practices

All tests follow existing patterns in the test suite:

1. **Naming:** `test_<function>_<behavior>` format
2. **Setup:** Use `setup()` or `setup_with_limits()` helpers
3. **Assertions:** Use `assert_eq!()` for value comparison
4. **Comments:** Minimal (test name is self-documenting)
5. **Organization:** Grouped by category with section headers

---

## Code Location

**File:** `contracts/safe-haven/src/test.rs`  
**Lines Added:** ~250 lines of test code + comments

**Start Location:** After line 4763 (end of existing tests)  
**Test Section Header:** `// ================================================================ // READONLY QUERY TESTS (TTL / Storage Path Verification) // ================================================================`

---

## Running the Tests

To run all 18 new tests:

```bash
cd contracts/safe-haven
cargo test --features testutils readonly
```

To run a specific test:

```bash
cargo test --features testutils test_get_admin_returns_current_admin
```

To run all query-related tests:

```bash
cargo test --features testutils get_ multiple_ admin_ fee_recipient constants
```

---

## Test Dependencies

All tests depend on:
- `setup()` helper — initializes contract and returns common test addresses
- `setup_with_limits()` helper — initializes contract with custom limits
- Standard assertions and test framework

No new test infrastructure or helpers were added; all tests use existing framework.

---

## Coverage Summary

| Function Tested | # Tests | Coverage |
|---|---|---|
| `get_admin()` | 3 | Basic retrieval, after renounce, multiple calls |
| `get_pending_admin()` | 5 | Initial state, after transfer, after accept, after cancel, multiple calls |
| `get_fee_recipient()` | 2 | Basic retrieval, persistence across operations, multiple calls |
| `get_constants()` | 4 | Default limits, custom limits, partial custom, multiple calls |
| Integration | 2 | Readonly-mutation interaction, lifecycle queries |
| Edge cases | 2 | Persistence, repeated queries |
| **Total** | **18** | **Comprehensive coverage** |

---

## Verification Completeness

✅ All acceptance criteria covered:
1. ✅ `get_admin()` returns correct values (3 tests)
2. ✅ `get_pending_admin()` returns correct values (5 tests)
3. ✅ `get_fee_recipient()` returns correct values (2 tests)
4. ✅ Multiple reads are consistent (4 tests)
5. ✅ Readonly queries don't block mutations (2 tests)
6. ✅ Edge cases and persistence (2 tests)
