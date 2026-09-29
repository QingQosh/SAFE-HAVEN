# Testing Guide: RPC Batch Optimization

This guide provides step-by-step instructions to verify that the RPC rate-limiting fix works correctly.

## Prerequisites

- Deployed SAFE-HAVEN contract with the new `get_deposit_batch` function
- Multiple test deposits (10+ recommended)
- Browser with Freighter wallet installed
- Network developer tools to monitor RPC calls

## Test Scenarios

### Scenario 1: Monitor RPC Call Reduction

**Objective:** Verify that loading deposits now uses fewer RPC calls.

**Steps:**
1. Open browser Developer Tools → Network tab
2. Filter by fetch requests
3. Connect wallet to the frontend
4. Navigate to Dashboard page
5. Observe the network requests

**Expected Results:**
- With 20 deposits: **3-4 total RPC calls** (was 40+ before)
  - 1× `get_deposit_ids` 
  - 1× `get_time`
  - 1× `get_deposit_batch`
- Requests complete quickly without rate-limit errors
- All deposits display in the vault list

**Verification:**
- Count the number of POST requests to the RPC endpoint
- Each request should have a method name in the body (check Network tab → Request payload)
- No failed/429 responses

### Scenario 2: Vault List Completeness

**Objective:** Ensure all deposits load without silent failures.

**Steps:**
1. Create 5-20 deposits with different unlock times
2. Refresh the page multiple times
3. Check Dashboard to confirm all deposits appear
4. Note if any deposits are missing

**Expected Results:**
- All deposits appear consistently
- No missing/null deposits
- Vault list matches contract state

**How to verify consistency:**
- Query contract directly: `soroban contract invoke ... --operation get_deposit_ids` 
- Count should match UI display

### Scenario 3: Time Remaining Accuracy

**Objective:** Verify client-side time calculation matches contract values.

**Steps:**
1. Load the Dashboard with deposits having known unlock times
2. For a deposit with ID `123` and known unlock time:
   ```bash
   soroban contract invoke ... --operation get_vault \
     --arg-sc-val "address:G..." \
     --arg-sc-val "u32:123"
   ```
   Note the `unlock_time` value (Unix timestamp)
3. Compare with frontend display:
   - Get current time from `getTimeRemaining` if needed for verification
   - Calculate: `timeRemaining = max(0, unlockTime - now)`
4. Verify countdown ticks correctly (decrements by 1 each second)

**Expected Results:**
- Time remaining matches calculated value ± 1 second (due to network latency)
- Countdown updates smoothly
- When time reaches 0, withdrawal button becomes enabled

**Precision check before withdrawal:**
- In withdraw flow, you can still call `getTimeRemaining` directly for final validation
- This ensures funds are truly unlocked before the transaction is signed

### Scenario 4: Batch Size Boundary Testing

**Objective:** Test batching logic when user has exactly 25, 26, 50, and 51 deposits.

**Steps:**
1. Create deposits to hit these exact counts
2. Monitor RPC calls for each scenario

**Expected Results:**
- 25 deposits: 3 RPC calls total (getDepositIds, getTime, getDepositBatch×1)
- 26 deposits: 4 RPC calls total (getDepositIds, getTime, getDepositBatch×2)
- 50 deposits: 4 RPC calls total (getDepositIds, getTime, getDepositBatch×2)
- 51 deposits: 5 RPC calls total (getDepositIds, getTime, getDepositBatch×3)

### Scenario 5: Error Handling

**Objective:** Verify the UI gracefully handles network errors.

**Steps:**
1. Open DevTools
2. Go to Network tab → Settings
3. Enable "Offline" mode
4. Refresh the page
5. Try to load deposits

**Expected Results:**
- Loading spinner appears briefly
- Error message displays: "Failed to load deposits"
- Error is dismissable
- Retry functionality works when connection is restored

### Scenario 6: Large Deposit Count Performance

**Objective:** Test performance with 100+ deposits.

**Steps:**
1. Create 100+ deposits
2. Load Dashboard
3. Measure time to display complete vault list
4. Monitor CPU/memory usage during load

**Expected Results:**
- Dashboard loads in < 3 seconds (even with 100+ deposits)
- No browser slowdown or UI freezing
- Network tab shows only ~5 RPC calls (for 100 deposits)

**Benchmark targets:**
- 50 deposits: < 1.5s
- 100 deposits: < 2.5s
- 200 deposits: < 4s

## Regression Testing

### Test 1: Old Features Still Work

- **Deposit workflow:** Create new deposit, verify it appears in list
- **Withdraw workflow:** Unlock deposit, withdraw successfully
- **Cancel deposit:** Early exit with penalty works
- **Admin functions:** Pause, emergency withdraw, admin transfer

### Test 2: Multiple Clients

Load the frontend in two browser windows:
- Connect with same wallet address to both
- Create a deposit in one window
- Refresh the other window
- Verify new deposit appears in the second window

## Manual Contract Testing

If Rust dev environment is available:

```bash
cd contracts/safe-haven

# Run unit tests
cargo test --features testutils

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Test with soroban CLI (if local node available)
make smoke-test-local
```

Expected: All tests pass, no new compilation errors.

## Network Tab Analysis

### Before (Old Implementation)
Look for pattern:
- Multiple `get_vault` calls (one per deposit)
- Multiple `time_remaining` calls (one per deposit)
- All fired in rapid succession (~100ms window)

**Example with 5 deposits:**
```
POST /rpc - 200 OK (get_vault id:1)
POST /rpc - 200 OK (time_remaining id:1)
POST /rpc - 200 OK (get_vault id:2)
POST /rpc - 200 OK (time_remaining id:2)
POST /rpc - 200 OK (get_vault id:3)
POST /rpc - 200 OK (time_remaining id:3)
...
Total: 10+ requests
```

### After (New Implementation)
Look for pattern:
- One `get_deposit_ids` call
- One `get_time` call
- One or more `get_deposit_batch` calls
- Spaced out appropriately

**Example with 5 deposits:**
```
POST /rpc - 200 OK (get_deposit_ids)
POST /rpc - 200 OK (get_time)
POST /rpc - 200 OK (get_deposit_batch ids:[1,2,3,4,5])
Total: 3 requests
```

## Success Criteria

All of the following must be true:

- [ ] RPC call count reduced by 85%+ for typical use case (20 deposits)
- [ ] No missing/incomplete deposits in vault list
- [ ] Time remaining displays accurately and counts down
- [ ] All existing features (deposit, withdraw, admin) still work
- [ ] No new TypeScript errors introduced
- [ ] Performance improved (faster load times)
- [ ] Error handling is graceful

## Reporting Issues

If you find problems:

1. **Incomplete vault list:** Note the number of deposits vs. number displayed, check browser console for errors
2. **Slow loading:** Record load time and deposit count, compare to expected benchmarks above
3. **Inaccurate time remaining:** Check if it matches contract value, note the difference
4. **Rate-limit errors:** Look for 429 responses in network tab, note RPC endpoint used

Include:
- Number of deposits
- Browser and OS
- RPC endpoint being used
- Screenshots/recordings if applicable
- Browser console errors (if any)
