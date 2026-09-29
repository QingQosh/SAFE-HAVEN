#!/usr/bin/env bash
# Usage: bash scripts/smoke_test_local.sh
#
# Smoke-tests the SAFE-HAVEN contract against a local Soroban
# standalone node (stellar network start local).
#
# Prerequisites:
#   - stellar CLI installed (https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli)
#   - Contract WASM built: make build
#   - jq installed (apt-get install jq / brew install jq)
#
# The script deploys a fresh contract, exercises core and admin functions,
# checks state after every scenario, and stops the local node on exit.

set -euo pipefail

WASM="target/wasm32-unknown-unknown/release/safe_haven.wasm"
NETWORK="local"
IDENTITY="smoke-test-user"

# ── helpers ──────────────────────────────────────────────────────────────────

pass() { echo "  ✓ $*"; }
fail() { echo "  ✗ $*" >&2; exit 1; }

assert_contains() {
    local label="$1" expected="$2" actual="$3"
    if echo "$actual" | grep -qF "$expected"; then
        pass "$label"
    else
        fail "$label — expected to contain '$expected', got: $actual"
    fi
}

# Assert that a numeric value matches expected
assert_eq() {
    local label="$1" expected="$2" actual="$3"
    if [ "$actual" = "$expected" ]; then
        pass "$label ($expected)"
    else
        fail "$label — expected '$expected', got '$actual'"
    fi
}

# Assert that a numeric value is greater than a threshold
assert_gt() {
    local label="$1" threshold="$2" actual="$3"
    if [ "$actual" -gt "$threshold" ] 2>/dev/null; then
        pass "$label ($actual)"
    else
        fail "$label — expected > $threshold, got: $actual"
    fi
}

# The CLI may print scalar values as JSON or plain text.
scalar() {
    tr -d '"' | tr -d '\r\n'
}

invoke() {
    local source="$1"
    shift
    stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$source" \
        --network "$NETWORK" \
        -- "$@"
}

read_only() {
    stellar contract invoke \
        --id "$CONTRACT_ID" \
        --network "$NETWORK" \
        -- "$@"
}

assert_state() {
    local label="$1" expected_admin="$2" expected_paused="$3" expected_count="$4"
    assert_eq "$label: initialized" "true" "$(read_only is_initialized | scalar)"
    assert_eq "$label: admin" "$expected_admin" "$(read_only get_admin | scalar)"
    assert_eq "$label: paused" "$expected_paused" "$(read_only is_paused | scalar)"
    assert_eq "$label: depositor count" "$expected_count" "$(read_only get_depositor_count | scalar)"
}

assert_error() {
    local label="$1" expected_code="$2"
    shift 2
    local output
    if output=$("$@" 2>&1); then
        fail "$label — command unexpectedly succeeded: $output"
    fi
    assert_contains "$label" "#$expected_code" "$output"
}

advance_ledgers() {
    local count="$1"
    for ((ledger = 0; ledger < count; ledger++)); do
        stellar ledger close --network "$NETWORK" > /dev/null
    done
}

# ── 1. Build check ────────────────────────────────────────────────────────────

echo "==> Checking WASM..."
[ -f "$WASM" ] || { echo "WASM not found. Run 'make build' first."; exit 1; }
pass "WASM found: $WASM"

# ── 2. Start local node ───────────────────────────────────────────────────────

echo "==> Starting local Soroban node..."
stellar network start "$NETWORK" --background 2>/dev/null || true
sleep 2
pass "Local node started"

cleanup() {
    echo "==> Stopping local node..."
    stellar network stop "$NETWORK" 2>/dev/null || true
}
trap cleanup EXIT

# ── 3. Identity & funding ─────────────────────────────────────────────────────

echo "==> Setting up identity..."
stellar keys generate "$IDENTITY" --network "$NETWORK" --fund 2>/dev/null || true
ADMIN_ADDR=$(stellar keys address "$IDENTITY")
pass "Identity: $ADMIN_ADDR"

# ── 4. Deploy ─────────────────────────────────────────────────────────────────

echo "==> Deploying contract..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK")
pass "Contract deployed: $CONTRACT_ID"

# ── 5. Initialize ─────────────────────────────────────────────────────────────

echo "==> Calling initialize..."
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$ADMIN_ADDR" \
    --fee_recipient "$ADMIN_ADDR" > /dev/null
pass "initialize OK"

echo "==> Verifying initialized contract state..."
assert_state "after initialize" "$ADMIN_ADDR" "false" "0"

# ── 5b. Verify depositor count starts at 0 ────────────────────────────────────

echo "==> Verifying initial depositor count..."
DEPOSITOR_COUNT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- get_depositor_count)
assert_eq "depositor_count == 0" "0" "$DEPOSITOR_COUNT"

# ── 6. Wrap native XLM as a token ────────────────────────────────────────────

echo "==> Wrapping native XLM..."
TOKEN_ID=$(stellar contract asset deploy \
    --asset native \
    --source "$IDENTITY" \
    --network "$NETWORK")
pass "Token: $TOKEN_ID"

# ── 7. Deposit ────────────────────────────────────────────────────────────────

echo "==> Calling deposit..."
# unlock_time = now + 120 seconds
UNLOCK_TIME=$(( $(date +%s) + 120 ))
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- deposit \
    --depositor "$ADMIN_ADDR" \
    --token "$TOKEN_ID" \
    --amount 1000 \
    --unlock_time "$UNLOCK_TIME" > /dev/null
pass "deposit OK"

# ── 7b. Verify depositor count incremented ────────────────────────────────────

echo "==> Verifying depositor count after deposit..."
DEPOSITOR_COUNT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- get_depositor_count)
assert_eq "depositor_count == 1" "1" "$DEPOSITOR_COUNT"

# ── 8. get_vault ──────────────────────────────────────────────────────────────

echo "==> Calling get_vault..."
VAULT_OUT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- get_vault \
    --depositor "$ADMIN_ADDR" \
    --deposit_id 0)

# Parse the JSON output to assert individual fields
VAULT_AMOUNT=$(echo "$VAULT_OUT" | jq -r '.amount // empty')
VAULT_UNLOCK=$(echo "$VAULT_OUT" | jq -r '.unlock_time // empty')
VAULT_PENALTY=$(echo "$VAULT_OUT" | jq -r '.penalty_bps // empty')

assert_eq "vault.amount == 1000" "1000" "$VAULT_AMOUNT"
assert_eq "vault.unlock_time == $UNLOCK_TIME" "$UNLOCK_TIME" "$VAULT_UNLOCK"
assert_eq "vault.penalty_bps == 0" "0" "$VAULT_PENALTY"
pass "get_vault returns expected values"

# ── 9. time_remaining ────────────────────────────────────────────────────────

echo "==> Calling time_remaining..."
TIME_OUT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- time_remaining \
    --depositor "$ADMIN_ADDR")
# Should be > 0 since we just deposited with a 120s lock
assert_gt "time_remaining > 0" "0" "$TIME_OUT"

# ── 9b. Verify time_remaining is approximately <= 120 ─────────────────────────

echo "==> Verifying time_remaining ≤ 120..."
if [ "$TIME_OUT" -le 120 ] 2>/dev/null; then
    pass "time_remaining <= 120 ($TIME_OUT)"
else
    fail "time_remaining should be <= 120, got: $TIME_OUT"
fi

# ── 10. withdraw (should fail — still locked) ─────────────────────────────────

echo "==> Calling withdraw (expect FundsStillLocked)..."
assert_error "withdraw fails while locked" "#4" stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- withdraw \
    --depositor "$ADMIN_ADDR" --deposit_id 0

# ── 10b. Verify vault still exists (was NOT removed by failed withdraw) ───────

echo "==> Verifying vault still exists after failed withdraw..."
VAULT_CHECK=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- get_vault \
    --depositor "$ADMIN_ADDR" \
    --deposit_id 0)
VAULT_CHECK_AMOUNT=$(echo "$VAULT_CHECK" | jq -r '.amount // empty')
assert_eq "vault still has amount 1000" "1000" "$VAULT_CHECK_AMOUNT"

# ── 10c. Verify depositor count unchanged after failed withdraw ───────────────

echo "==> Verifying depositor count unchanged after failed withdraw..."
DEPOSITOR_COUNT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- get_depositor_count)
assert_eq "depositor_count still == 1" "1" "$DEPOSITOR_COUNT"

assert_state "after locked withdrawal" "$ADMIN_ADDR" "false" "1"

# ── 11. Error cases: invalid amount, expired timestamp, unauthorized ─────────

echo "==> Testing invalid amount..."
INVALID_UNLOCK=$(( $(date +%s) + 120 ))
assert_error "zero amount rejected" "#1" stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- deposit \
    --depositor "$ADMIN_ADDR" --token "$TOKEN_ID" --amount 0 \
    --unlock_time "$INVALID_UNLOCK" --penalty_bps 0
assert_state "after invalid amount" "$ADMIN_ADDR" "false" "1"

echo "==> Testing expired timestamp..."
EXPIRED_TIME=$(( $(date +%s) - 1 ))
assert_error "expired timestamp rejected" "#2" stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- deposit \
    --depositor "$ADMIN_ADDR" --token "$TOKEN_ID" --amount 1000 \
    --unlock_time "$EXPIRED_TIME" --penalty_bps 0
assert_state "after expired timestamp" "$ADMIN_ADDR" "false" "1"

echo "==> Testing unauthorized admin action..."
UNAUTHORIZED_IDENTITY="smoke-test-unauthorized"
stellar keys generate "$UNAUTHORIZED_IDENTITY" --network "$NETWORK" --fund 2>/dev/null || true
UNAUTHORIZED_ADDR=$(stellar keys address "$UNAUTHORIZED_IDENTITY")
assert_error "non-admin pause rejected" "#7" stellar contract invoke \
    --id "$CONTRACT_ID" --source "$UNAUTHORIZED_IDENTITY" --network "$NETWORK" -- pause \
    --admin "$UNAUTHORIZED_ADDR"
assert_state "after unauthorized pause" "$ADMIN_ADDR" "false" "1"

# ── 12. cancel_deposit ───────────────────────────────────────────────────────

echo "==> Testing cancel_deposit..."
CANCEL_UNLOCK=$(( $(date +%s) + 120 ))
CANCEL_ID=$(stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- deposit \
    --depositor "$ADMIN_ADDR" --token "$TOKEN_ID" --amount 1000 \
    --unlock_time "$CANCEL_UNLOCK" --penalty_bps 500 | scalar)
assert_eq "cancel deposit id" "1" "$CANCEL_ID"
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- cancel_deposit \
    --depositor "$ADMIN_ADDR" --deposit_id "$CANCEL_ID" > /dev/null
CANCELLED=$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_vault \
    --depositor "$ADMIN_ADDR" --deposit_id "$CANCEL_ID")
assert_eq "cancelled deposit removed" "null" "$(echo "$CANCELLED" | jq -c '.')"
assert_state "after cancel" "$ADMIN_ADDR" "false" "1"

# ── 13. emergency_withdraw ──────────────────────────────────────────────────

echo "==> Testing emergency_withdraw..."
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- emergency_withdraw \
    --admin "$ADMIN_ADDR" --depositor "$ADMIN_ADDR" --deposit_id 0 > /dev/null
EMERGENCY_VAULT=$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_vault \
    --depositor "$ADMIN_ADDR" --deposit_id 0)
assert_eq "emergency deposit removed" "null" "$(echo "$EMERGENCY_VAULT" | jq -c '.')"
assert_state "after emergency withdrawal" "$ADMIN_ADDR" "false" "0"

# ── 14. pause/unpause ────────────────────────────────────────────────────────

echo "==> Testing pause and unpause..."
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- pause \
    --admin "$ADMIN_ADDR" > /dev/null
assert_state "after pause" "$ADMIN_ADDR" "true" "0"
PAUSED_UNLOCK=$(( $(date +%s) + 120 ))
assert_error "deposit while paused rejected" "#12" stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- deposit \
    --depositor "$ADMIN_ADDR" --token "$TOKEN_ID" --amount 1000 \
    --unlock_time "$PAUSED_UNLOCK" --penalty_bps 0
assert_state "after paused deposit" "$ADMIN_ADDR" "true" "0"
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- unpause \
    --admin "$ADMIN_ADDR" > /dev/null
assert_state "after unpause" "$ADMIN_ADDR" "false" "0"

# ── 15. Successful withdrawal after unlock ───────────────────────────────────

echo "==> Testing successful withdrawal after unlock..."
WITHDRAW_UNLOCK=$(( $(date +%s) + 60 ))
WITHDRAW_ID=$(stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- deposit \
    --depositor "$ADMIN_ADDR" --token "$TOKEN_ID" --amount 500 \
    --unlock_time "$WITHDRAW_UNLOCK" --penalty_bps 0 | scalar)
assert_eq "withdraw deposit id" "2" "$WITHDRAW_ID"
assert_state "before ledger advance" "$ADMIN_ADDR" "false" "1"
for ((ledger = 0; ledger < 15; ledger++)); do
    stellar ledger close --network "$NETWORK" > /dev/null
done
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- withdraw \
    --depositor "$ADMIN_ADDR" --deposit_id "$WITHDRAW_ID" > /dev/null
WITHDRAWN_VAULT=$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_vault \
    --depositor "$ADMIN_ADDR" --deposit_id "$WITHDRAW_ID")
assert_eq "withdrawn deposit removed" "null" "$(echo "$WITHDRAWN_VAULT" | jq -c '.')"
assert_state "after withdrawal" "$ADMIN_ADDR" "false" "0"

# ── 16. Two-step admin transfer ──────────────────────────────────────────────

echo "==> Testing two-step admin transfer..."
SUCCESSOR_IDENTITY="smoke-test-successor"
stellar keys generate "$SUCCESSOR_IDENTITY" --network "$NETWORK" --fund 2>/dev/null || true
SUCCESSOR_ADDR=$(stellar keys address "$SUCCESSOR_IDENTITY")
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- transfer_admin \
    --admin "$ADMIN_ADDR" --new_admin "$SUCCESSOR_ADDR" > /dev/null
assert_eq "pending admin" "$SUCCESSOR_ADDR" "$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_pending_admin | scalar)"
assert_state "after transfer proposal" "$ADMIN_ADDR" "false" "0"
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$SUCCESSOR_IDENTITY" --network "$NETWORK" -- accept_admin \
    --new_admin "$SUCCESSOR_ADDR" > /dev/null
assert_state "after transfer acceptance" "$SUCCESSOR_ADDR" "false" "0"

# ── 17. renounce_admin ───────────────────────────────────────────────────────

echo "==> Testing admin renunciation..."
stellar contract invoke \
    --id "$CONTRACT_ID" --source "$SUCCESSOR_IDENTITY" --network "$NETWORK" -- renounce_admin \
    --admin "$SUCCESSOR_ADDR" > /dev/null
assert_eq "admin removed after renounce" "null" "$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_admin | jq -c '.')"
assert_eq "pending admin removed after renounce" "null" "$(stellar contract invoke \
    --id "$CONTRACT_ID" --network "$NETWORK" -- get_pending_admin | jq -c '.')"
assert_state_after_renounce() {
    assert_eq "renounced contract initialized" "true" "$(read_only is_initialized | scalar)"
    assert_eq "renounced contract unpaused" "false" "$(read_only is_paused | scalar)"
    assert_eq "renounced contract empty" "0" "$(read_only get_depositor_count | scalar)"
}
assert_state_after_renounce
assert_error "emergency withdrawal after renounce rejected" "#7" stellar contract invoke \
    --id "$CONTRACT_ID" --source "$SUCCESSOR_IDENTITY" --network "$NETWORK" -- emergency_withdraw \
    --admin "$SUCCESSOR_ADDR" --depositor "$ADMIN_ADDR" --deposit_id 99
assert_state_after_renounce

echo ""
echo "All smoke tests passed."
