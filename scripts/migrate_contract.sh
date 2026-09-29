#!/usr/bin/env bash
# =============================================================
#  migrate_contract.sh
#  Checks whether a deployed SAFE-HAVEN instance is still on the
#  legacy schema and runs the versioned migration hook if needed.
#
#  Requirements:
#    - soroban-cli installed
#    - SOROBAN_SECRET_KEY set to the admin account used for the upgrade
#
#  Environment variables:
#    CONTRACT_ID        The deployed contract ID to migrate
#    ADMIN_ADDRESS      The admin address that is allowed to call migrate()
#    NETWORK            Soroban network name (default: testnet)
#    RPC_URL            RPC endpoint (default: testnet)
#    NETWORK_PASSPHRASE Network passphrase for the selected network
#
#  Usage:
#    export SOROBAN_SECRET_KEY=S...
#    export CONTRACT_ID=CC...
#    export ADMIN_ADDRESS=G...
#    bash scripts/migrate_contract.sh
# =============================================================

set -euo pipefail

NETWORK="${NETWORK:-testnet}"
RPC_URL="${RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
CONTRACT_ID="${CONTRACT_ID:-${1:-}}"
ADMIN_ADDRESS="${ADMIN_ADDRESS:-${2:-}}"
SOURCE_SECRET="${SOROBAN_SECRET_KEY:-}"

if [[ -z "$CONTRACT_ID" ]]; then
  echo "ERROR: CONTRACT_ID is required."
  echo "Usage: CONTRACT_ID=... ADMIN_ADDRESS=... SOROBAN_SECRET_KEY=... bash scripts/migrate_contract.sh"
  exit 1
fi

if [[ -z "$ADMIN_ADDRESS" ]]; then
  echo "ERROR: ADMIN_ADDRESS is required."
  echo "Usage: CONTRACT_ID=... ADMIN_ADDRESS=... SOROBAN_SECRET_KEY=... bash scripts/migrate_contract.sh"
  exit 1
fi

if [[ -z "$SOURCE_SECRET" ]]; then
  echo "ERROR: SOROBAN_SECRET_KEY is required."
  exit 1
fi

read_version() {
  soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "$SOURCE_SECRET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- get_storage_version 2>/dev/null || echo "null"
}

CURRENT_VERSION="$(read_version)"
CURRENT_VERSION="${CURRENT_VERSION//[$'\r\n']}"

if [[ "$CURRENT_VERSION" == "null" || "$CURRENT_VERSION" == "None" || -z "$CURRENT_VERSION" ]]; then
  CURRENT_VERSION=0
fi

if (( CURRENT_VERSION >= 1 )); then
  echo "Storage version already up to date: $CURRENT_VERSION"
  echo "No migration was required."
  exit 0
fi

echo "Legacy schema detected (version $CURRENT_VERSION). Running migration..."

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$SOURCE_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- migrate \
  --admin "$ADMIN_ADDRESS"

UPDATED_VERSION="$(read_version)"
UPDATED_VERSION="${UPDATED_VERSION//[$'\r\n']}"

if [[ "$UPDATED_VERSION" == "null" || "$UPDATED_VERSION" == "None" || -z "$UPDATED_VERSION" ]]; then
  UPDATED_VERSION=0
fi

echo "Migration complete. Current storage version: $UPDATED_VERSION"
