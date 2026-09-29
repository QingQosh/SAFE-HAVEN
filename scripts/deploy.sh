#!/usr/bin/env bash
# Deploy and initialize SAFE-HAVEN on Stellar testnet or mainnet.
#
# Usage:
#   SOROBAN_SECRET_KEY=... bash scripts/deploy.sh testnet
#   SOROBAN_SECRET_KEY=... FEE_RECIPIENT=G... bash scripts/deploy.sh mainnet
#   SOROBAN_SECRET_KEY=... bash scripts/deploy.sh rollback testnet \
#     --artifact-dir deployments/testnet/20260826T120000Z

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

ACTION="deploy"
NETWORK=""
ARTIFACT_DIR=""
RPC_URL=""
NETWORK_PASSPHRASE=""
DEPLOYMENTS_DIR="${DEPLOYMENTS_DIR:-$ROOT_DIR/deployments}"
RAW_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/safe_haven.wasm"

usage() {
    cat <<'EOF'
Usage:
  scripts/deploy.sh [deploy] <testnet|mainnet> [options]
  scripts/deploy.sh rollback <testnet|mainnet> --artifact-dir <directory>

Environment:
  SOROBAN_SECRET_KEY  Required source/deployer identity.
  DEPLOYER_ADDRESS    Optional public address; defaults to the source identity address.
  ADMIN_ADDRESS       Optional contract admin; defaults to DEPLOYER_ADDRESS.
  FEE_RECIPIENT       Optional fee recipient; defaults to ADMIN_ADDRESS.
  MAX_DEPOSIT         Optional initialize() max deposit override.
  MAX_LOCK_SECS       Optional initialize() max lock duration override.
  DEPLOYMENTS_DIR     Optional artifact root; defaults to ./deployments.

Testnet is funded through Friendbot. Mainnet deployers must already be funded.
EOF
}

die() { echo "ERROR: $*" >&2; exit 1; }

require_command() { command -v "$1" > /dev/null || die "'$1' is required"; }

parse_args() {
    [[ $# -gt 0 ]] || { usage; exit 1; }
    [[ "$1" == "-h" || "$1" == "--help" ]] && { usage; exit 0; }
    if [[ "$1" == "rollback" ]]; then
        ACTION="rollback"
        shift
    elif [[ "$1" == "deploy" ]]; then
        shift
    fi

    [[ $# -gt 0 ]] || { usage; exit 1; }
    NETWORK="$1"
    shift

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --artifact-dir)
                [[ $# -ge 2 ]] || die "--artifact-dir requires a value"
                ARTIFACT_DIR="$2"
                shift 2
                ;;
            --rpc-url)
                [[ $# -ge 2 ]] || die "--rpc-url requires a value"
                RPC_URL="$2"
                shift 2
                ;;
            --network-passphrase)
                [[ $# -ge 2 ]] || die "--network-passphrase requires a value"
                NETWORK_PASSPHRASE="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *) die "unknown option: $1" ;;
        esac
    done

    [[ "$NETWORK" == "testnet" || "$NETWORK" == "mainnet" ]] || die "network must be testnet or mainnet"
    if [[ "$ACTION" == "rollback" && -z "$ARTIFACT_DIR" ]]; then
        die "rollback requires --artifact-dir"
    fi
}

configure_network() {
    if [[ "$NETWORK" == "testnet" ]]; then
        RPC_URL="${RPC_URL:-https://soroban-testnet.stellar.org}"
        NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
    else
        RPC_URL="${RPC_URL:-https://soroban.stellar.org}"
        NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Public Global Stellar Network ; September 2015}"
    fi
}

fund_deployer() {
    if [[ "$NETWORK" == "testnet" ]]; then
        echo ">>> Funding deployer through Friendbot..."
        curl --fail --silent --show-error "https://friendbot.stellar.org?addr=${DEPLOYER_ADDRESS}" > /dev/null
        echo "    Funded: $DEPLOYER_ADDRESS"
    else
        echo ">>> Mainnet funding check..."
        echo "    Mainnet has no Friendbot; verify $DEPLOYER_ADDRESS is funded before continuing."
    fi
}

build_wasm() {
    local optimized_wasm="$1"
    echo ">>> Building WASM..."
    cargo build --target wasm32-unknown-unknown --release
    echo ">>> Optimizing WASM..."
    stellar contract optimize --wasm "$RAW_WASM" --wasm-out "$optimized_wasm"
    echo "    Optimized size: $(wc -c < "$optimized_wasm") bytes"
}

invoke() {
    stellar contract invoke \
        --id "$CONTRACT_ID" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        -- "$@"
}

initialize_contract() {
    local args=(initialize --admin "$ADMIN_ADDRESS" --fee_recipient "$FEE_RECIPIENT")
    [[ -n "${MAX_DEPOSIT:-}" ]] && args+=(--max_deposit "$MAX_DEPOSIT")
    [[ -n "${MAX_LOCK_SECS:-}" ]] && args+=(--max_lock_secs "$MAX_LOCK_SECS")
    echo ">>> Initializing contract..."
    invoke "${args[@]}" > /dev/null
}

verify_contract() {
    local initialized stored_admin stored_fee
    initialized=$(invoke is_initialized | tr -d '"\r\n')
    stored_admin=$(invoke get_admin | tr -d '"\r\n')
    stored_fee=$(invoke get_fee_recipient | tr -d '"\r\n')
    [[ "$initialized" == "true" ]] || die "is_initialized returned '$initialized'"
    [[ "$stored_admin" == "$ADMIN_ADDRESS" ]] || die "get_admin returned '$stored_admin'"
    [[ "$stored_fee" == "$FEE_RECIPIENT" ]] || die "get_fee_recipient returned '$stored_fee'"
    echo "    Verified initialized=true admin=$stored_admin fee_recipient=$stored_fee"
}

write_artifacts() {
    local artifact_dir="$1" wasm_path="$2" ledger_time constants
    mkdir -p "$artifact_dir"
    cp "$wasm_path" "$artifact_dir/safe_haven.optimized.wasm"
    if [[ -f "$RAW_WASM" ]]; then
        cp "$RAW_WASM" "$artifact_dir/safe_haven.wasm"
    else
        cp "$wasm_path" "$artifact_dir/safe_haven.wasm"
    fi
    ledger_time=$(invoke get_time | tr -d '"\r\n')
    constants=$(invoke get_constants | tr -d '\r\n')
    sha256sum "$artifact_dir/safe_haven.optimized.wasm" | cut -d' ' -f1 > "$artifact_dir/wasm.sha256"
    cat > "$artifact_dir/manifest.env" <<EOF
ACTION=$ACTION
NETWORK=$NETWORK
CONTRACT_ID=$CONTRACT_ID
DEPLOYER_ADDRESS=$DEPLOYER_ADDRESS
ADMIN_ADDRESS=$ADMIN_ADDRESS
FEE_RECIPIENT=$FEE_RECIPIENT
RPC_URL=$RPC_URL
NETWORK_PASSPHRASE=$NETWORK_PASSPHRASE
LEDGER_TIME=$ledger_time
CONSTANTS=$constants
WASM_SHA256=$(cat "$artifact_dir/wasm.sha256")
CREATED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF
    printf '%s\n' "$CONTRACT_ID" > "$artifact_dir/contract-id.txt"
    printf 'Contract ID: %s\nNetwork: %s\nAdmin: %s\nLedger Time: %s\nConstants: %s\n' \
        "$CONTRACT_ID" "$NETWORK" "$ADMIN_ADDRESS" "$ledger_time" "$constants" \
        > "$ROOT_DIR/deploy_${NETWORK}.log"
    echo "    Artifacts: $artifact_dir"
}

deploy() {
    require_command cargo
    require_command stellar
    require_command curl
    require_command sha256sum
    [[ -n "${SOROBAN_SECRET_KEY:-}" ]] || die "SOROBAN_SECRET_KEY is not set"

    DEPLOYER_ADDRESS="${DEPLOYER_ADDRESS:-$(stellar keys address "$SOROBAN_SECRET_KEY")}" 
    ADMIN_ADDRESS="${ADMIN_ADDRESS:-$DEPLOYER_ADDRESS}"
    FEE_RECIPIENT="${FEE_RECIPIENT:-$ADMIN_ADDRESS}"

    local stamp artifact_dir optimized_wasm
    stamp=$(date -u +"%Y%m%dT%H%M%SZ")
    artifact_dir="$DEPLOYMENTS_DIR/$NETWORK/$stamp"
    optimized_wasm="$artifact_dir/safe_haven.optimized.wasm"
    mkdir -p "$artifact_dir"

    fund_deployer
    build_wasm "$optimized_wasm"
    echo ">>> Deploying contract..."
    CONTRACT_ID=$(stellar contract deploy \
        --wasm "$optimized_wasm" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" | tr -d '\r\n')
    [[ -n "$CONTRACT_ID" ]] || die "deployment returned an empty contract ID"
    echo "    Contract ID: $CONTRACT_ID"
    initialize_contract
    verify_contract
    write_artifacts "$artifact_dir" "$optimized_wasm"
    echo "Deployment successful. Use $artifact_dir for audit and rollback."
}

rollback() {
    require_command stellar
    require_command curl
    require_command sha256sum
    [[ -n "${SOROBAN_SECRET_KEY:-}" ]] || die "SOROBAN_SECRET_KEY is not set"
    [[ -d "$ARTIFACT_DIR" ]] || die "artifact directory not found: $ARTIFACT_DIR"
    local wasm_path="$ARTIFACT_DIR/safe_haven.optimized.wasm"
    [[ -f "$wasm_path" ]] || die "previous optimized WASM not found: $wasm_path"

    DEPLOYER_ADDRESS="${DEPLOYER_ADDRESS:-$(stellar keys address "$SOROBAN_SECRET_KEY")}" 
    ADMIN_ADDRESS="${ADMIN_ADDRESS:-$DEPLOYER_ADDRESS}"
    FEE_RECIPIENT="${FEE_RECIPIENT:-$ADMIN_ADDRESS}"
    fund_deployer
    echo ">>> Redeploying previous WASM from $ARTIFACT_DIR..."
    CONTRACT_ID=$(stellar contract deploy \
        --wasm "$wasm_path" \
        --source "$SOROBAN_SECRET_KEY" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" | tr -d '\r\n')
    initialize_contract
    verify_contract
    local stamp rollback_dir
    stamp=$(date -u +"%Y%m%dT%H%M%SZ")
    rollback_dir="$DEPLOYMENTS_DIR/$NETWORK/rollback-$stamp"
    mkdir -p "$rollback_dir"
    cp "$wasm_path" "$rollback_dir/safe_haven.optimized.wasm"
    write_artifacts "$rollback_dir" "$wasm_path"
    echo "Rollback deployment successful. New contract ID: $CONTRACT_ID"
}

parse_args "$@"
configure_network
if [[ "$ACTION" == "rollback" ]]; then
    rollback
else
    deploy
fi