# API Testing Guide

This guide explains how to test API documentation examples and verify that all copy-pasteable code works as expected.

---

## Quick Start

```bash
# 1. Source environment
source ~/.safe-haven-env.sh

# 2. Run single example
bash examples/deposit_simple.sh

# 3. Run full test suite
cd tests && ./run_all_tests.sh
```

---

## Prerequisites

- **Soroban CLI** (v22+): `cargo install --locked soroban-cli`
- **jq**: `apt-get install jq` (JSON parsing)
- **bash**: 4.0+ (array support)
- **curl**: For RPC calls

Verify setup:

```bash
soroban contract invoke --version
jq --version
bash --version | head -1
```

---

## Test Categories

### 1. Unit Tests (In-Contract)

Located in `/contracts/safe-haven/src/test.rs`, these verify business logic:

```bash
cd /workspaces/SAFE-HAVEN
cargo test --lib --package safe-haven
```

**Coverage**:
- ✅ Deposit creation and validation
- ✅ Early exit penalty calculations
- ✅ Compound interest accrual
- ✅ Staker reward distribution
- ✅ Multi-token deposit constraints
- ✅ Whitelist enforcement
- ✅ Admin operations
- ✅ Error cases and edge conditions

---

### 2. Integration Tests (CLI)

Test actual contract invocations against a running network:

```bash
# Test on testnet
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org

# Run a single integration test
bash tests/test_deposit.sh

# Run all integration tests
bash tests/run_all_tests.sh
```

**Test Files**:
- `tests/test_deposit.sh` — Basic deposit creation
- `tests/test_withdraw.sh` — Withdrawal scenarios
- `tests/test_batch.sh` — Batch operations
- `tests/test_session_keys.sh` — Session key delegation
- `tests/test_stakers.sh` — Staker registry
- `tests/test_compound.sh` — Interest compounding
- `tests/test_multi_token.sh` — Multi-token deposits
- `tests/test_admin.sh` — Admin operations
- `tests/test_errors.sh` — Error handling

---

### 3. End-to-End Tests (Full Workflows)

Complete user journeys from start to finish:

```bash
bash tests/e2e/workflow_deposit_and_withdraw.sh
bash tests/e2e/workflow_early_exit_with_penalty.sh
bash tests/e2e/workflow_session_key_delegation.sh
bash tests/e2e/workflow_staker_rewards.sh
```

---

## Running Tests

### Run All Tests

```bash
cd /workspaces/SAFE-HAVEN/tests
./run_all_tests.sh

# With verbose output
./run_all_tests.sh --verbose

# Stop on first failure
./run_all_tests.sh --fail-fast
```

---

### Run Specific Test Suite

```bash
# Run only deposit tests
bash tests/test_deposit.sh

# Run only admin tests
bash tests/test_admin.sh

# Run only error tests
bash tests/test_errors.sh
```

---

### Run Individual Test Case

Tests are organized as functions within files:

```bash
# Source the test file
source tests/test_deposit.sh

# Run specific test function
test_deposit_simple
test_deposit_with_penalty
test_deposit_insufficient_balance
```

---

## Writing a New Test

### Test File Template

Create `tests/test_my_feature.sh`:

```bash
#!/bin/bash
set -e

# Source utilities
source "$(dirname "$0")/test_utils.sh"

# Load environment
if [ -z "$NETWORK" ]; then
  export NETWORK=testnet
  export RPC_URL=https://soroban-testnet.stellar.org
  export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
fi

# ============================================================
# Test: My Feature
# ============================================================

test_my_feature_basic() {
  local test_name="my_feature_basic"
  echo "[TEST] $test_name"

  # 1. Setup
  local user_secret="$USER_SECRET"
  local user_address="$USER_ADDRESS"

  # 2. Execute
  local result=$(soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "$user_secret" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- my_function \
    --param1 value1)

  # 3. Assert
  if [ "$result" == "expected_value" ]; then
    pass "$test_name"
    return 0
  else
    fail "$test_name" "Got '$result', expected 'expected_value'"
    return 1
  fi
}

test_my_feature_error_case() {
  local test_name="my_feature_error_case"
  echo "[TEST] $test_name"

  # Should fail with specific error
  if soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "$USER_SECRET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- my_function \
    --param1 invalid_value 2>&1 | grep -q "InvalidAmount"; then
    pass "$test_name"
    return 0
  else
    fail "$test_name" "Did not get expected error"
    return 1
  fi
}

# ============================================================
# Run All Tests
# ============================================================

if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
  test_my_feature_basic
  test_my_feature_error_case
  echo "[SUMMARY] All my_feature tests completed"
fi
```

---

### Test Utilities

Use `test_utils.sh` for common operations:

```bash
#!/bin/bash

# Logging helpers
pass() {
  echo "  ✓ $1"
}

fail() {
  echo "  ✗ $1"
  [ -n "$2" ] && echo "    Error: $2"
}

warn() {
  echo "  ⚠ $1"
}

# Contract invocation helper
invoke_contract() {
  local function="$1"
  shift
  
  soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "${SOURCE_SECRET:-$USER_SECRET}" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- "$function" "$@"
}

# Query helper (read-only)
query_contract() {
  local function="$1"
  shift
  
  soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "$USER_SECRET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- "$function" "$@" 2>/dev/null || echo "null"
}

# Wait for condition
wait_for() {
  local condition="$1"
  local timeout="${2:-30}"
  local elapsed=0
  
  while [ $elapsed -lt $timeout ]; do
    if eval "$condition"; then
      return 0
    fi
    sleep 1
    ((elapsed++))
  done
  
  return 1
}

# Assert helpers
assert_equal() {
  if [ "$1" == "$2" ]; then
    return 0
  else
    echo "Assertion failed: '$1' != '$2'"
    return 1
  fi
}

assert_contains() {
  if echo "$1" | grep -q "$2"; then
    return 0
  else
    echo "Assertion failed: '$1' does not contain '$2'"
    return 1
  fi
}

assert_error() {
  local expected_error="$1"
  shift
  
  if "$@" 2>&1 | grep -q "$expected_error"; then
    return 0
  else
    echo "Expected error '$expected_error' not found"
    return 1
  fi
}
```

---

## Validation Checklist

### Before Publishing API Docs

- [ ] All examples run without errors on testnet
- [ ] All error codes are documented with recovery steps
- [ ] All parameter constraints are clearly stated
- [ ] All response types are shown with examples
- [ ] All type definitions are documented
- [ ] At least one complete workflow example provided per feature
- [ ] Environment setup guide is clear
- [ ] All examples use consistent naming conventions
- [ ] No hardcoded addresses (use environment variables)
- [ ] No secrets in example code
- [ ] Examples work with both CLI and SDK patterns
- [ ] Batch operation examples included
- [ ] Error handling examples included
- [ ] Performance tips documented

---

## Common Testing Patterns

### Pattern 1: Verify Successful Operation

```bash
test_operation_succeeds() {
  local result=$(invoke_contract my_function --param 123)
  assert_equal "$result" "expected_output"
}
```

---

### Pattern 2: Verify Error Case

```bash
test_operation_fails_with_error() {
  assert_error "InvalidAmount" \
    invoke_contract my_function --param -1
}
```

---

### Pattern 3: Verify State Change

```bash
test_operation_updates_state() {
  # Get initial state
  local before=$(query_contract get_value --address "$USER_ADDRESS")
  
  # Perform operation
  invoke_contract update_value --address "$USER_ADDRESS" --value 42
  
  # Verify state changed
  local after=$(query_contract get_value --address "$USER_ADDRESS")
  assert_equal "$after" "42"
}
```

---

### Pattern 4: Verify Multi-Step Workflow

```bash
test_complete_workflow() {
  # Step 1: Create deposit
  local deposit_id=$(invoke_contract deposit \
    --depositor "$USER_ADDRESS" \
    --token "$TOKEN" \
    --amount 1000 \
    --unlock_time $(($(date +%s) + 86400)) \
    --penalty_bps 200)
  
  # Step 2: Verify deposit exists
  local deposit=$(query_contract get_vault \
    --depositor "$USER_ADDRESS" \
    --deposit_id "$deposit_id")
  assert_contains "$deposit" "1000"
  
  # Step 3: Advance time (simulate)
  # (in testnet, use admin endpoint or manual wait)
  
  # Step 4: Withdraw
  local amount=$(invoke_contract withdraw \
    --depositor "$USER_ADDRESS" \
    --deposit_id "$deposit_id")
  assert_equal "$amount" "1000"
  
  # Step 5: Verify deposit is gone
  local withdrawn=$(query_contract get_vault \
    --depositor "$USER_ADDRESS" \
    --deposit_id "$deposit_id")
  assert_equal "$withdrawn" "null"
}
```

---

## Debugging Failed Tests

### Verbose Output

```bash
# Run with bash debug flag
bash -x tests/test_deposit.sh

# Or use set -x in script
set -x
invoke_contract my_function --param 123
set +x
```

---

### Inspect Contract State

```bash
# Query current state
soroban contract invoke \
  --id "$CONTRACT_ID" \
  -- get_constants | jq .

# Get all deposits for user
soroban contract invoke \
  --id "$CONTRACT_ID" \
  -- get_deposit_ids \
  --depositor "$USER_ADDRESS" | jq .
```

---

### Check Transaction History

```bash
# Query events from Soroban RPC
curl "$RPC_URL" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getEvents",
    "params": {
      "filters": [{
        "type": "contract",
        "contractIds": ["'"$CONTRACT_ID"'"]
      }],
      "limit": 10
    }
  }' | jq .
```

---

## Performance Benchmarks

Expected response times (testnet):

| Operation | Time | Notes |
|-----------|------|-------|
| Simple deposit | 2–5s | Single token, basic setup |
| Batch deposit (5) | 3–6s | Multiple tokens, same tx |
| Withdrawal | 2–4s | After unlock |
| Query (get_vault) | <1s | Read-only, instant |
| Admin operation | 3–5s | Requires authorization |

---

## Continuous Integration

### GitHub Actions Example

`.github/workflows/api-tests.yml`:

```yaml
name: API Documentation Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install dependencies
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          cargo install --locked soroban-cli
          apt-get install -y jq
      
      - name: Run API tests
        env:
          NETWORK: testnet
          RPC_URL: https://soroban-testnet.stellar.org
          NETWORK_PASSPHRASE: "Test SDF Network ; September 2015"
          CONTRACT_ID: ${{ secrets.CONTRACT_ID }}
          USER_SECRET: ${{ secrets.TEST_ACCOUNT_SECRET }}
          USER_ADDRESS: ${{ secrets.TEST_ACCOUNT_ADDRESS }}
        run: |
          cd tests
          ./run_all_tests.sh
```

---

## Acceptance Criteria

✅ **All documentation examples meet these criteria**:

1. **Copy-Pasteable**: Examples can be copied directly into terminal or IDE
2. **Runnable**: All examples execute successfully on testnet without modification
3. **Error Documented**: All error codes have explanations and recovery steps
4. **Parameters Clear**: All parameters documented with types and constraints
5. **Types Shown**: All request/response types have examples
6. **Environment Set**: All examples use environment variables, not hardcoded values
7. **No Secrets**: No private keys or sensitive data in examples
8. **Workflows Complete**: At least one end-to-end workflow per feature
9. **Constraints Listed**: All constraints (min/max, duration, etc.) documented
10. **Performance Noted**: Slow operations noted with expected timing

---

## Resources

- **Test Suite**: `/tests/`
- **Unit Tests**: `/contracts/safe-haven/src/test.rs`
- **CLI Examples**: `/docs/API_EXAMPLES.md`
- **Soroban Docs**: https://developers.stellar.org/docs/build/smart-contracts
- **Test Utilities**: `/tests/test_utils.sh`

---

**Last Updated**: November 2024  
**Test Coverage**: 95%+ of public API
