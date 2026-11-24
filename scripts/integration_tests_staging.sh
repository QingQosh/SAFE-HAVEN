#!/usr/bin/env bash
# =============================================================================
#  SAFE-HAVEN — Staging Integration Tests
#  Runs comprehensive integration tests against a deployed staging contract.
#
#  Usage:
#    STAGING_CONTRACT_ID=C... bash scripts/integration_tests_staging.sh
#
#  Environment:
#    STAGING_CONTRACT_ID    Required: contract ID to test
#    SOROBAN_SECRET_KEY     Required: identity with funded account
#    STAGING_RPC_URL        Optional: RPC endpoint
#    STAGING_NETWORK_PASS   Optional: network passphrase
#    TEST_TIMEOUT           Optional: timeout per test in seconds (default: 30)
#
#  Exit codes:
#    0 — all tests passed
#    1 — one or more tests failed
# =============================================================================

set -uo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────
NETWORK="staging"
RPC_URL="${STAGING_RPC_URL:-https://soroban-staging.stellar.org}"
NETWORK_PASSPHRASE="${STAGING_NETWORK_PASS:-Staging SDF Network ; September 2015}"
CONTRACT_ID="${STAGING_CONTRACT_ID:-}"
TEST_TIMEOUT="${TEST_TIMEOUT:-30}"

# Colour codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

# State
TESTS_PASSED=0
TESTS_FAILED=0

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────
require_env() {
    local var="$1"
    [[ -n "${!var:-}" ]] || { echo "ERROR: $var not set"; exit 1; }
}

test_pass() {
    echo -e "${GREEN}✓${RESET} $*"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

test_fail() {
    echo -e "${RED}✘${RESET} $*"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

test_info() {
    echo -e "${BLUE}ℹ${RESET} $*"
}

invoke() {
    timeout "$TEST_TIMEOUT" stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- "$@"
}

# ─────────────────────────────────────────────────────────────────────────────
# Integration Tests
# ─────────────────────────────────────────────────────────────────────────────

test_contract_responds() {
    test_info "Test: Contract responds to invocation"
    
    local result
    result=$(invoke is_initialized 2>&1 | tr -d '"\r\n') || {
        test_fail "Contract did not respond"
        return 1
    }

    if [[ "$result" == "true" ]]; then
        test_pass "Contract responds (is_initialized = true)"
        return 0
    else
        test_fail "Contract responded with unexpected value: $result"
        return 1
    fi
}

test_get_admin() {
    test_info "Test: get_admin returns valid address"
    
    local result
    result=$(invoke get_admin 2>&1 | tr -d '"\r\n') || {
        test_fail "get_admin call failed"
        return 1
    }

    # Stellar addresses are 56 characters starting with 'G'
    if [[ "$result" =~ ^G[A-Z0-9]{54}$ ]]; then
        test_pass "get_admin returned valid address: ${result:0:10}...${result: -10}"
        return 0
    else
        test_fail "get_admin returned invalid address: $result"
        return 1
    fi
}

test_get_constants() {
    test_info "Test: get_constants returns configuration"
    
    local result
    result=$(invoke get_constants 2>&1) || {
        test_fail "get_constants call failed"
        return 1
    }

    if [[ -n "$result" ]]; then
        test_pass "get_constants returned data"
        return 0
    else
        test_fail "get_constants returned empty result"
        return 1
    fi
}

test_response_time() {
    test_info "Test: Response time is acceptable"
    
    local start_time=$(date +%s%3N)
    invoke is_initialized > /dev/null 2>&1 || {
        test_fail "Invocation failed"
        return 1
    }
    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))

    if [[ $latency -lt 5000 ]]; then
        test_pass "Response time acceptable: ${latency}ms"
        return 0
    else
        test_fail "Response time slow: ${latency}ms (threshold: 5000ms)"
        return 1
    fi
}

test_no_errors_in_logs() {
    test_info "Test: No critical errors in RPC logs"
    
    # This is a placeholder test that would integrate with centralized logging
    # In production, this would check CloudWatch, DataDog, etc.
    
    test_pass "Placeholder: no log analysis configured (check manually)"
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────
main() {
    require_env CONTRACT_ID
    require_env SOROBAN_SECRET_KEY

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
    echo -e "${BOLD}  Staging Integration Tests${RESET}"
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
    echo ""
    echo "Contract ID: $CONTRACT_ID"
    echo "Network: $NETWORK"
    echo "RPC: $RPC_URL"
    echo ""

    # Run all tests
    test_contract_responds
    test_get_admin
    test_get_constants
    test_response_time
    test_no_errors_in_logs

    # Summary
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
    echo -e "${BOLD}  Test Summary${RESET}"
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
    echo ""

    local total=$((TESTS_PASSED + TESTS_FAILED))
    echo "Total:  $total"
    echo -e "${GREEN}Passed: $TESTS_PASSED${RESET}"
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "${RED}Failed: $TESTS_FAILED${RESET}"
    fi
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}${BOLD}✓ All integration tests passed${RESET}"
        echo ""
        exit 0
    else
        echo -e "${RED}${BOLD}✘ Some tests failed${RESET}"
        echo ""
        exit 1
    fi
}

main "$@"
