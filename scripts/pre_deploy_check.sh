#!/usr/bin/env bash
# =============================================================================
#  SAFE-HAVEN — Pre-Deployment Check Script
#  Automates the Phase 1 technical checks from ROLLOUT_CHECKLIST.md.
#
#  Usage:
#    bash scripts/pre_deploy_check.sh
#
#  Exit codes:
#    0 — all checks passed
#    1 — one or more checks failed
#
#  Checks performed:
#    1. cargo fmt --check          (formatting)
#    2. cargo clippy               (lint, warnings-as-errors)
#    3. cargo test                 (unit tests)
#    4. cargo audit                (security vulnerabilities)
#    5. make check-wasm-size       (WASM size limit, skipped if not built)
#    6. CHANGELOG.md [Unreleased]  (release notes present)
# =============================================================================

set -uo pipefail

# ---------------------------------------------------------------------------
# Colour helpers (disabled automatically when not a TTY)
# ---------------------------------------------------------------------------
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BOLD='' RESET=''
fi

# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------
FAILED_CHECKS=()
PASSED_CHECKS=()
SKIPPED_CHECKS=()

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
pass() {
    local name="$1"
    echo -e "${GREEN}  ✔ PASS${RESET}  ${name}"
    PASSED_CHECKS+=("$name")
}

fail() {
    local name="$1"
    local reason="${2:-}"
    if [[ -n "$reason" ]]; then
        echo -e "${RED}  ✘ FAIL${RESET}  ${name} — ${reason}"
    else
        echo -e "${RED}  ✘ FAIL${RESET}  ${name}"
    fi
    FAILED_CHECKS+=("$name")
}

skip() {
    local name="$1"
    local reason="${2:-}"
    if [[ -n "$reason" ]]; then
        echo -e "${YELLOW}  ⚠ SKIP${RESET}  ${name} — ${reason}"
    else
        echo -e "${YELLOW}  ⚠ SKIP${RESET}  ${name}"
    fi
    SKIPPED_CHECKS+=("$name")
}

# Run a command, capture combined output, return exit code.
# Usage: run_check <name> <cmd> [args...]
run_check() {
    local name="$1"
    shift
    local output
    if output=$("$@" 2>&1); then
        pass "$name"
        return 0
    else
        echo "$output" | sed 's/^/    /'
        fail "$name"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# Navigate to repo root
# ---------------------------------------------------------------------------
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

echo ""
echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
echo -e "${BOLD}  SAFE-HAVEN Pre-Deployment Check${RESET}"
echo -e "${BOLD}  $(date -u '+%Y-%m-%dT%H:%M:%SZ')${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
echo ""

# ---------------------------------------------------------------------------
# Check 1 — cargo fmt
# ---------------------------------------------------------------------------
echo -e "${BOLD}[1/6] Formatting (cargo fmt --check)${RESET}"
if ! command -v cargo > /dev/null 2>&1; then
    fail "cargo fmt" "cargo not found in PATH"
else
    output=$(cargo fmt --all -- --check 2>&1)
    fmt_exit=$?
    if [ $fmt_exit -eq 0 ]; then
        pass "cargo fmt"
    else
        echo "$output" | sed 's/^/    /'
        echo ""
        echo -e "    ${YELLOW}Hint: run \`cargo fmt --all\` to fix formatting, then re-run this script.${RESET}"
        fail "cargo fmt" "unformatted files detected"
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Check 2 — cargo clippy
# ---------------------------------------------------------------------------
echo -e "${BOLD}[2/6] Lint (cargo clippy --all-targets --features testutils -- -D warnings)${RESET}"
if ! command -v cargo > /dev/null 2>&1; then
    fail "cargo clippy" "cargo not found in PATH"
else
    output=$(cargo clippy --all-targets --features testutils -- -D warnings 2>&1)
    clippy_exit=$?
    if [ $clippy_exit -eq 0 ]; then
        pass "cargo clippy"
    else
        echo "$output" | sed 's/^/    /'
        echo ""
        echo -e "    ${YELLOW}Hint: fix the warnings above, then re-run this script.${RESET}"
        fail "cargo clippy" "clippy warnings or errors found"
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Check 3 — cargo test
# ---------------------------------------------------------------------------
echo -e "${BOLD}[3/6] Unit tests (cargo test --features testutils)${RESET}"
if ! command -v cargo > /dev/null 2>&1; then
    fail "cargo test" "cargo not found in PATH"
else
    output=$(cargo test --features testutils 2>&1)
    test_exit=$?
    if [ $test_exit -eq 0 ]; then
        # Extract and display test summary line(s) only
        test_summary=$(echo "$output" | grep -E '^test result' || true)
        if [[ -n "$test_summary" ]]; then
            echo "$test_summary" | sed 's/^/    /'
        fi
        pass "cargo test"
    else
        echo "$output" | sed 's/^/    /'
        echo ""
        echo -e "    ${YELLOW}Hint: fix the failing tests above, then re-run this script.${RESET}"
        fail "cargo test" "one or more tests failed"
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Check 4 — cargo audit
# ---------------------------------------------------------------------------
echo -e "${BOLD}[4/6] Security audit (cargo audit)${RESET}"
if ! command -v cargo-audit > /dev/null 2>&1 && ! cargo audit --version > /dev/null 2>&1; then
    skip "cargo audit" "cargo-audit not installed — run \`cargo install cargo-audit\` to enable"
else
    audit_output=$(cargo audit 2>&1)
    audit_exit=$?

    # cargo audit exits 0 when clean, non-zero when vulnerabilities found.
    # We specifically want to fail on HIGH or CRITICAL; warn on lower severity.
    if [ $audit_exit -eq 0 ]; then
        pass "cargo audit"
    else
        # Check whether the output contains HIGH or CRITICAL advisories.
        # cargo audit labels these with "Severity: high" / "Severity: critical" (case-insensitive).
        if echo "$audit_output" | grep -qiE "Severity:\s+(high|critical)"; then
            echo "$audit_output" | grep -iE "(error\[|warning\[|Severity:|Advisory:|ID:|Crate:)" | sed 's/^/    /'
            echo ""
            echo -e "    ${RED}HIGH or CRITICAL vulnerability found. Deployment must not proceed.${RESET}"
            echo -e "    ${YELLOW}Hint: run \`cargo audit\` for the full report and advisory details.${RESET}"
            fail "cargo audit" "HIGH/CRITICAL vulnerabilities detected"
        else
            # Lower-severity advisories: warn but don't block.
            echo "$audit_output" | grep -iE "(warning\[|Severity:|Advisory:|ID:|Crate:)" | head -20 | sed 's/^/    /'
            echo ""
            echo -e "    ${YELLOW}Low/medium advisories detected. No HIGH/CRITICAL — not blocking.${RESET}"
            echo -e "    ${YELLOW}Review the output of \`cargo audit\` and assess manually.${RESET}"
            pass "cargo audit"
        fi
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Check 5 — WASM size
# ---------------------------------------------------------------------------
echo -e "${BOLD}[5/6] WASM size (make check-wasm-size)${RESET}"
OPTIMIZED_WASM="$ROOT_DIR/target/safe_haven.optimized.wasm"
RAW_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/safe_haven.wasm"

if [ ! -f "$OPTIMIZED_WASM" ] && [ ! -f "$RAW_WASM" ]; then
    skip "check-wasm-size" "no WASM artifact found — run \`make build\` (and optionally \`make optimize\`) first"
elif [ ! -f "$OPTIMIZED_WASM" ]; then
    skip "check-wasm-size" "optimized WASM not found (only raw WASM present) — run \`make optimize\` first; skipping size check"
else
    if ! command -v make > /dev/null 2>&1; then
        # Fallback: check size directly without make
        MAX_BYTES=65536
        ACTUAL=$(wc -c < "$OPTIMIZED_WASM")
        echo "    Optimized WASM size: ${ACTUAL} bytes (limit: ${MAX_BYTES})"
        if [ "$ACTUAL" -gt "$MAX_BYTES" ]; then
            fail "check-wasm-size" "WASM too large: ${ACTUAL} bytes exceeds limit of ${MAX_BYTES} bytes"
        else
            pass "check-wasm-size"
        fi
    else
        output=$(make check-wasm-size 2>&1)
        wasm_exit=$?
        echo "$output" | grep -E "(bytes|ERROR|size)" | head -5 | sed 's/^/    /'
        if [ $wasm_exit -eq 0 ]; then
            pass "check-wasm-size"
        else
            echo ""
            fail "check-wasm-size" "WASM exceeds 64 KB size limit"
        fi
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Check 6 — CHANGELOG.md [Unreleased] section has content
# ---------------------------------------------------------------------------
echo -e "${BOLD}[6/6] CHANGELOG.md [Unreleased] section has content${RESET}"
CHANGELOG="$ROOT_DIR/CHANGELOG.md"

if [ ! -f "$CHANGELOG" ]; then
    fail "CHANGELOG [Unreleased]" "CHANGELOG.md not found at $CHANGELOG"
else
    # Extract content between ## [Unreleased] and the next ## [...] heading.
    # We expect at least one non-blank line between those two markers.
    unreleased_block=$(awk '
        /^## \[Unreleased\]/ { in_block=1; next }
        in_block && /^## \[/ { exit }
        in_block { print }
    ' "$CHANGELOG")

    # Strip blank lines and check if anything remains
    non_empty=$(echo "$unreleased_block" | grep -v '^\s*$' || true)

    if [[ -z "$non_empty" ]]; then
        echo ""
        echo -e "    ${RED}The [Unreleased] section in CHANGELOG.md is empty or missing.${RESET}"
        echo -e "    ${YELLOW}Hint: document all changes for this release under \`## [Unreleased]\`${RESET}"
        echo -e "    ${YELLOW}before creating the version tag.${RESET}"
        fail "CHANGELOG [Unreleased]" "[Unreleased] section is empty"
    else
        # Show first few lines of the section for confidence
        echo "$non_empty" | head -5 | sed 's/^/    /'
        if [ "$(echo "$non_empty" | wc -l)" -gt 5 ]; then
            echo "    ... (truncated)"
        fi
        pass "CHANGELOG [Unreleased]"
    fi
fi
echo ""

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
echo -e "${BOLD}  Summary${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
echo ""

if [ ${#PASSED_CHECKS[@]} -gt 0 ]; then
    echo -e "${GREEN}Passed (${#PASSED_CHECKS[@]}):${RESET}"
    for c in "${PASSED_CHECKS[@]}"; do
        echo -e "  ${GREEN}✔${RESET}  $c"
    done
    echo ""
fi

if [ ${#SKIPPED_CHECKS[@]} -gt 0 ]; then
    echo -e "${YELLOW}Skipped (${#SKIPPED_CHECKS[@]}):${RESET}"
    for c in "${SKIPPED_CHECKS[@]}"; do
        echo -e "  ${YELLOW}⚠${RESET}  $c"
    done
    echo ""
fi

if [ ${#FAILED_CHECKS[@]} -gt 0 ]; then
    echo -e "${RED}Failed (${#FAILED_CHECKS[@]}):${RESET}"
    for c in "${FAILED_CHECKS[@]}"; do
        echo -e "  ${RED}✘${RESET}  $c"
    done
    echo ""
    echo -e "${RED}${BOLD}  ✘ PRE-DEPLOY CHECK: FAIL${RESET}"
    echo -e "  Fix the issues above before deploying to mainnet."
    echo ""
    exit 1
else
    echo -e "${GREEN}${BOLD}  ✔ PRE-DEPLOY CHECK: PASS${RESET}"
    if [ ${#SKIPPED_CHECKS[@]} -gt 0 ]; then
        echo -e "  ${YELLOW}(${#SKIPPED_CHECKS[@]} check(s) were skipped — review manually)${RESET}"
    fi
    echo ""
    exit 0
fi
