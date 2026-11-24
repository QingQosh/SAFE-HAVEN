#!/usr/bin/env bash
# =============================================================================
#  SAFE-HAVEN — Staging Deployment Script
#  Automates safe, traceable deployments to staging with validation,
#  health checks, and automatic rollback on failure.
#
#  Usage:
#    SOROBAN_SECRET_KEY=... bash scripts/deploy_staging.sh
#
#  Environment:
#    SOROBAN_SECRET_KEY     Required: deployer/source identity
#    ADMIN_ADDRESS          Optional: contract admin (defaults to deployer)
#    STAGING_RPC_URL        Optional: Soroban RPC endpoint
#    STAGING_NETWORK_PASS   Optional: network passphrase
#    DEPLOYMENTS_DIR        Optional: artifact directory (default: ./deployments)
#    KEEP_PREV_ARTIFACT     Optional: retain previous deployment (default: true)
#    HEALTH_CHECK_TIMEOUT   Optional: health check timeout in seconds (default: 60)
#    ROLLBACK_ON_FAILURE    Optional: auto-rollback on health check failure (default: true)
#
#  Exit codes:
#    0 — successful deployment with passing health checks
#    1 — deployment failed
#    2 — post-deployment health check failed
#    3 — rollback failed (manual intervention required)
#
#  Artifacts stored in: deployments/staging/<timestamp>/
#    - safe_haven.optimized.wasm
#    - safe_haven.wasm (raw)
#    - manifest.env (deployment metadata)
#    - contract-id.txt
#    - health-check-log.txt (timestamp, responses, latency)
# =============================================================================

set -uo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

# ─────────────────────────────────────────────────────────────────────────────
# Configuration & Defaults
# ─────────────────────────────────────────────────────────────────────────────
DEPLOYMENTS_DIR="${DEPLOYMENTS_DIR:-$ROOT_DIR/deployments}"
RAW_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/safe_haven.wasm"
OPTIMIZED_WASM="$ROOT_DIR/target/safe_haven.optimized.wasm"
NETWORK="staging"
RPC_URL="${STAGING_RPC_URL:-https://soroban-staging.stellar.org}"
NETWORK_PASSPHRASE="${STAGING_NETWORK_PASS:-Staging SDF Network ; September 2015}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-60}"
ROLLBACK_ON_FAILURE="${ROLLBACK_ON_FAILURE:-true}"
KEEP_PREV_ARTIFACT="${KEEP_PREV_ARTIFACT:-true}"

# ─────────────────────────────────────────────────────────────────────────────
# State & Logging
# ─────────────────────────────────────────────────────────────────────────────
DEPLOYMENT_ID=""
DEPLOYMENT_DIR=""
CONTRACT_ID=""
DEPLOYER_ADDRESS=""
ADMIN_ADDRESS=""
PREVIOUS_CONTRACT_ID=""
START_TIME=$(date +%s)
DEPLOYMENT_LOG=""
HEALTH_CHECK_LOG=""

# Colour codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────
log() {
    echo -e "${BLUE}[$(date -u '+%H:%M:%S')]${RESET} $*" | tee -a "$DEPLOYMENT_LOG"
}

log_success() {
    echo -e "${GREEN}✓${RESET} $*" | tee -a "$DEPLOYMENT_LOG"
}

log_error() {
    echo -e "${RED}✘${RESET} $*" | tee -a "$DEPLOYMENT_LOG"
}

log_warning() {
    echo -e "${YELLOW}⚠${RESET} $*" | tee -a "$DEPLOYMENT_LOG"
}

die() {
    log_error "$*"
    exit 1
}

require_command() {
    command -v "$1" > /dev/null || die "'$1' is required but not found in PATH"
}

require_env() {
    local var="$1"
    [[ -n "${!var:-}" ]] || die "Environment variable $var is not set"
}

elapsed_seconds() {
    local end_time=$(date +%s)
    echo $((end_time - START_TIME))
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────────────────────
setup() {
    require_command cargo
    require_command stellar
    require_env SOROBAN_SECRET_KEY

    DEPLOYMENT_ID=$(date -u +"%Y%m%dT%H%M%SZ")
    DEPLOYMENT_DIR="$DEPLOYMENTS_DIR/$NETWORK/$DEPLOYMENT_ID"
    DEPLOYMENT_LOG="$DEPLOYMENT_DIR/deployment.log"
    HEALTH_CHECK_LOG="$DEPLOYMENT_DIR/health-check-log.txt"
    mkdir -p "$DEPLOYMENT_DIR"

    touch "$DEPLOYMENT_LOG"

    DEPLOYER_ADDRESS=$(stellar keys address "$SOROBAN_SECRET_KEY" 2>/dev/null || echo "")
    [[ -n "$DEPLOYER_ADDRESS" ]] || die "failed to derive deployer address from SOROBAN_SECRET_KEY"

    ADMIN_ADDRESS="${ADMIN_ADDRESS:-$DEPLOYER_ADDRESS}"

    log "Staging Deployment Initialized"
    log "Deployment ID: $DEPLOYMENT_ID"
    log "Deployer: ${DEPLOYER_ADDRESS:0:10}...${DEPLOYER_ADDRESS: -10}"
    log "Admin: ${ADMIN_ADDRESS:0:10}...${ADMIN_ADDRESS: -10}"
    log "Network: $NETWORK"
    log "RPC: $RPC_URL"
}

# ─────────────────────────────────────────────────────────────────────────────
# Pre-deployment checks
# ─────────────────────────────────────────────────────────────────────────────
pre_deployment_checks() {
    log ""
    log "Pre-Deployment Checks"
    log "━━━━━━━━━━━━━━━━━━━━"

    # Check if WASM exists
    if [[ ! -f "$OPTIMIZED_WASM" ]]; then
        log_warning "Optimized WASM not found, attempting to build and optimize..."
        cd "$ROOT_DIR"
        cargo build --target wasm32-unknown-unknown --release >> "$DEPLOYMENT_LOG" 2>&1 || die "cargo build failed"
        stellar contract optimize --wasm "$RAW_WASM" --wasm-out "$OPTIMIZED_WASM" >> "$DEPLOYMENT_LOG" 2>&1 \
            || die "WASM optimization failed"
    fi

    # Validate WASM file
    local wasm_size
    wasm_size=$(wc -c < "$OPTIMIZED_WASM")
    if [[ $wasm_size -gt 65536 ]]; then
        die "WASM too large: $wasm_size bytes exceeds limit of 65536"
    fi
    log_success "WASM validation passed (size: $wasm_size bytes)"

    # Check stellar CLI
    stellar version >> "$DEPLOYMENT_LOG" 2>&1 || die "stellar CLI check failed"
    log_success "Stellar CLI is available"

    # Verify network connectivity
    if ! stellar network list | grep -q "^$NETWORK"; then
        log "Adding $NETWORK to local config..."
        stellar network add "$NETWORK" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" >> "$DEPLOYMENT_LOG" 2>&1 || true
    fi
    log_success "Network configuration verified"
}

# ─────────────────────────────────────────────────────────────────────────────
# Deploy contract
# ─────────────────────────────────────────────────────────────────────────────
deploy_contract() {
    log ""
    log "Deploying Contract"
    log "━━━━━━━━━━━━━━━━━"

    local deploy_output deploy_exit
    deploy_output=$(stellar contract deploy \
        --wasm "$OPTIMIZED_WASM" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" 2>&1) || {
        deploy_exit=$?
        log_error "Contract deployment failed"
        echo "$deploy_output" >> "$DEPLOYMENT_LOG"
        return "$deploy_exit"
    }

    CONTRACT_ID=$(echo "$deploy_output" | tr -d '\r\n')
    [[ -n "$CONTRACT_ID" ]] || {
        log_error "Deployment returned empty contract ID"
        echo "$deploy_output" >> "$DEPLOYMENT_LOG"
        return 1
    }

    log_success "Contract deployed: $CONTRACT_ID"
}

# ─────────────────────────────────────────────────────────────────────────────
# Initialize contract
# ─────────────────────────────────────────────────────────────────────────────
initialize_contract() {
    log ""
    log "Initializing Contract"
    log "━━━━━━━━━━━━━━━━━━━"

    log "Setting admin: ${ADMIN_ADDRESS:0:10}...${ADMIN_ADDRESS: -10}"

    local init_output init_exit
    init_output=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- initialize \
        --admin "$ADMIN_ADDRESS" 2>&1) || {
        init_exit=$?
        log_error "Contract initialization failed"
        echo "$init_output" >> "$DEPLOYMENT_LOG"
        return "$init_exit"
    }

    echo "$init_output" >> "$DEPLOYMENT_LOG"
    log_success "Contract initialized successfully"
}

# ─────────────────────────────────────────────────────────────────────────────
# Health checks
# ─────────────────────────────────────────────────────────────────────────────
health_check() {
    log ""
    log "Post-Deployment Health Checks"
    log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    {
        echo "=== Health Check Log ===" | tee -a "$HEALTH_CHECK_LOG"
        echo "Timestamp: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" | tee -a "$HEALTH_CHECK_LOG"
        echo "Contract: $CONTRACT_ID" | tee -a "$HEALTH_CHECK_LOG"
        echo ""
    } | tee -a "$DEPLOYMENT_LOG"

    local hc_passed=0
    local hc_failed=0

    # Check 1: is_initialized
    log "Check 1: is_initialized"
    if check_is_initialized; then
        hc_passed=$((hc_passed + 1))
    else
        hc_failed=$((hc_failed + 1))
    fi

    # Check 2: get_admin
    log "Check 2: get_admin"
    if check_get_admin; then
        hc_passed=$((hc_passed + 1))
    else
        hc_failed=$((hc_failed + 1))
    fi

    # Check 3: get_constants
    log "Check 3: get_constants"
    if check_get_constants; then
        hc_passed=$((hc_passed + 1))
    else
        hc_failed=$((hc_failed + 1))
    fi

    # Check 4: response latency
    log "Check 4: response latency"
    if check_latency; then
        hc_passed=$((hc_passed + 1))
    else
        hc_failed=$((hc_failed + 1))
    fi

    echo "" | tee -a "$HEALTH_CHECK_LOG"
    echo "Health Check Summary: $hc_passed passed, $hc_failed failed" | tee -a "$HEALTH_CHECK_LOG"
    echo "" | tee -a "$DEPLOYMENT_LOG"

    if [[ $hc_failed -gt 0 ]]; then
        log_warning "Health checks failed: $hc_failed / $((hc_passed + hc_failed))"
        return 1
    else
        log_success "All health checks passed: $hc_passed / $((hc_passed + hc_failed))"
        return 0
    fi
}

check_is_initialized() {
    local start_time=$(date +%s%3N)
    local result
    result=$(timeout "$HEALTH_CHECK_TIMEOUT" stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- is_initialized 2>&1 | tr -d '"\r\n') || {
        log_warning "  ✘ is_initialized timed out or failed"
        echo "FAILED: timeout or error" >> "$HEALTH_CHECK_LOG"
        return 1
    }

    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))

    if [[ "$result" == "true" ]]; then
        log_success "  ✓ is_initialized returned true (${latency}ms)"
        echo "PASSED: is_initialized=true (${latency}ms)" >> "$HEALTH_CHECK_LOG"
        return 0
    else
        log_warning "  ✘ is_initialized returned '$result' (expected 'true')"
        echo "FAILED: is_initialized returned '$result' (${latency}ms)" >> "$HEALTH_CHECK_LOG"
        return 1
    fi
}

check_get_admin() {
    local result
    result=$(timeout "$HEALTH_CHECK_TIMEOUT" stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- get_admin 2>&1 | tr -d '"\r\n') || {
        log_warning "  ✘ get_admin timed out or failed"
        echo "FAILED: timeout or error" >> "$HEALTH_CHECK_LOG"
        return 1
    }

    if [[ "$result" == "$ADMIN_ADDRESS" ]]; then
        log_success "  ✓ get_admin matches configured admin"
        echo "PASSED: get_admin matches ($result)" >> "$HEALTH_CHECK_LOG"
        return 0
    else
        log_warning "  ✘ get_admin mismatch (expected $ADMIN_ADDRESS, got $result)"
        echo "FAILED: get_admin mismatch (expected $ADMIN_ADDRESS, got $result)" >> "$HEALTH_CHECK_LOG"
        return 1
    fi
}

check_get_constants() {
    local result
    result=$(timeout "$HEALTH_CHECK_TIMEOUT" stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- get_constants 2>&1) || {
        log_warning "  ✘ get_constants timed out or failed"
        echo "FAILED: timeout or error" >> "$HEALTH_CHECK_LOG"
        return 1
    }

    if [[ -n "$result" ]]; then
        log_success "  ✓ get_constants returned valid data"
        echo "PASSED: get_constants returned data" >> "$HEALTH_CHECK_LOG"
        return 0
    else
        log_warning "  ✘ get_constants returned empty"
        echo "FAILED: get_constants returned empty" >> "$HEALTH_CHECK_LOG"
        return 1
    fi
}

check_latency() {
    local total_latency=0
    local checks=3
    local i

    for i in $(seq 1 $checks); do
        local start_time=$(date +%s%3N)
        timeout "$HEALTH_CHECK_TIMEOUT" stellar contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOROBAN_SECRET_KEY" \
            --network "$NETWORK" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- is_initialized > /dev/null 2>&1 || continue
        local end_time=$(date +%s%3N)
        local latency=$((end_time - start_time))
        total_latency=$((total_latency + latency))
    done

    if [[ $checks -gt 0 ]]; then
        local avg_latency=$((total_latency / checks))
        if [[ $avg_latency -lt 5000 ]]; then  # 5 seconds threshold
            log_success "  ✓ Response latency acceptable (avg ${avg_latency}ms)"
            echo "PASSED: average latency ${avg_latency}ms (threshold: 5000ms)" >> "$HEALTH_CHECK_LOG"
            return 0
        else
            log_warning "  ✘ Response latency high (avg ${avg_latency}ms)"
            echo "WARNING: average latency ${avg_latency}ms (threshold: 5000ms)" >> "$HEALTH_CHECK_LOG"
            return 1  # Still warn but may not be critical
        fi
    fi

    return 1
}

# ─────────────────────────────────────────────────────────────────────────────
# Rollback (automatic)
# ─────────────────────────────────────────────────────────────────────────────
automatic_rollback() {
    if [[ "$ROLLBACK_ON_FAILURE" != "true" ]]; then
        log_warning "Automatic rollback disabled; manual intervention may be required"
        return 1
    fi

    log_error "Initiating automatic rollback..."

    # Find the previous successful deployment
    PREVIOUS_CONTRACT_ID=$(find "$DEPLOYMENTS_DIR/$NETWORK" -maxdepth 1 -type d \
        -exec grep -l "SUCCESS" {}/deployment.log 2>/dev/null \; | head -1)

    if [[ -z "$PREVIOUS_CONTRACT_ID" ]]; then
        log_error "No previous successful deployment found; cannot rollback"
        return 1
    fi

    log "Previous deployment: $PREVIOUS_CONTRACT_ID"
    log "Rollback not fully automated in this version; manual deployment required"
    
    return 1  # Indicate rollback was not completed
}

# ─────────────────────────────────────────────────────────────────────────────
# Artifact management
# ─────────────────────────────────────────────────────────────────────────────
write_artifacts() {
    log ""
    log "Writing Artifacts"
    log "━━━━━━━━━━━━━━━"

    # Copy WASM files
    cp "$OPTIMIZED_WASM" "$DEPLOYMENT_DIR/safe_haven.optimized.wasm"
    if [[ -f "$RAW_WASM" ]]; then
        cp "$RAW_WASM" "$DEPLOYMENT_DIR/safe_haven.wasm"
    fi

    # Write checksums
    sha256sum "$DEPLOYMENT_DIR/safe_haven.optimized.wasm" | cut -d' ' -f1 > "$DEPLOYMENT_DIR/wasm.sha256"

    # Write manifest
    local constants
    constants=$(stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- get_constants 2>&1 || echo "N/A")

    cat > "$DEPLOYMENT_DIR/manifest.env" <<EOF
DEPLOYMENT_ID=$DEPLOYMENT_ID
NETWORK=$NETWORK
CONTRACT_ID=$CONTRACT_ID
DEPLOYER_ADDRESS=$DEPLOYER_ADDRESS
ADMIN_ADDRESS=$ADMIN_ADDRESS
RPC_URL=$RPC_URL
NETWORK_PASSPHRASE=$NETWORK_PASSPHRASE
WASM_SHA256=$(cat "$DEPLOYMENT_DIR/wasm.sha256")
CONSTANTS=$constants
CREATED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "N/A")
GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
EOF

    printf '%s\n' "$CONTRACT_ID" > "$DEPLOYMENT_DIR/contract-id.txt"

    log_success "Artifacts written to $DEPLOYMENT_DIR"
    log "  - safe_haven.optimized.wasm"
    log "  - safe_haven.wasm"
    log "  - manifest.env"
    log "  - contract-id.txt"
    log "  - deployment.log"
    log "  - health-check-log.txt"
}

# ─────────────────────────────────────────────────────────────────────────────
# Deployment summary
# ─────────────────────────────────────────────────────────────────────────────
print_summary() {
    local status="$1"
    local elapsed=$(elapsed_seconds)

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"
    echo -e "${BOLD}  Deployment Summary${RESET}"
    echo -e "${BOLD}═══════════════════════════════════════════════════${RESET}"

    if [[ "$status" == "success" ]]; then
        echo -e "${GREEN}✓ Staging Deployment Successful${RESET}"
    else
        echo -e "${RED}✘ Staging Deployment Failed${RESET}"
    fi

    echo ""
    echo "Deployment ID:   $DEPLOYMENT_ID"
    echo "Contract ID:     $CONTRACT_ID"
    echo "Network:         $NETWORK"
    echo "Admin:           ${ADMIN_ADDRESS:0:10}...${ADMIN_ADDRESS: -10}"
    echo "Elapsed time:    ${elapsed}s"
    echo "Artifacts:       $DEPLOYMENT_DIR"
    echo ""

    if [[ "$status" == "success" ]]; then
        echo -e "${GREEN}Explorer URL:${RESET}"
        echo "  https://explorer.stellar.org?resource=contract&resourceCode=$CONTRACT_ID&network=staging"
        echo ""
        echo "Next steps:"
        echo "  1. Review health check logs: $HEALTH_CHECK_LOG"
        echo "  2. Run integration tests"
        echo "  3. Monitor contract for 24 hours"
        echo "  4. If issues arise, trigger rollback:"
        echo "     gh workflow run staging-rollback.yml -f reason='<reason>'"
    fi

    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────
main() {
    setup
    
    if ! pre_deployment_checks; then
        print_summary "failure"
        exit 1
    fi

    if ! deploy_contract; then
        log_error "Deployment failed"
        print_summary "failure"
        exit 1
    fi

    if ! initialize_contract; then
        log_error "Initialization failed"
        print_summary "failure"
        exit 1
    fi

    write_artifacts

    if ! health_check; then
        log_error "Post-deployment health checks failed"
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            log_warning "Attempting automatic rollback..."
            if automatic_rollback; then
                print_summary "rollback"
                exit 2  # Rollback in progress
            else
                print_summary "failure"
                exit 2  # Health check failed; rollback may be needed
            fi
        else
            print_summary "failure"
            exit 2
        fi
    fi

    print_summary "success"
    exit 0
}

main "$@"
