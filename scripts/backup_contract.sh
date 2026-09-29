#!/usr/bin/env bash

set -euo pipefail

: "${CONTRACT_ID:?Set CONTRACT_ID to the deployed SAFE-HAVEN contract ID}"
: "${STELLAR_SOURCE:?Set STELLAR_SOURCE to a configured Stellar CLI identity}"

NETWORK="${NETWORK:-testnet}"
RPC_URL="${RPC_URL:-}"
BACKUP_DIR="${BACKUP_DIR:-backups}"
BACKUP_STORAGE="${BACKUP_STORAGE:-ipfs}"
IPFS_API_URL="${IPFS_API_URL:-http://127.0.0.1:5001}"
S3_BUCKET="${S3_BUCKET:-}"
S3_PREFIX="${S3_PREFIX:-safe-haven/}"
PAGE_SIZE="${PAGE_SIZE:-25}"

command -v jq >/dev/null || { echo "ERROR: jq is required." >&2; exit 1; }
command -v stellar >/dev/null || { echo "ERROR: stellar CLI is required." >&2; exit 1; }
[[ "$PAGE_SIZE" =~ ^[1-9][0-9]*$ ]] || { echo "ERROR: PAGE_SIZE must be a positive integer." >&2; exit 1; }

mkdir -p "$BACKUP_DIR"
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
safe_timestamp=${timestamp//:/-}
backup_file="$BACKUP_DIR/safe-haven-${CONTRACT_ID}-${safe_timestamp}.json"
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT

cli_args=(--id "$CONTRACT_ID" --source "$STELLAR_SOURCE" --network "$NETWORK")
if [[ -n "$RPC_URL" ]]; then
    cli_args+=(--rpc-url "$RPC_URL")
fi

invoke() {
    stellar contract invoke "${cli_args[@]}" -- "$@"
}

echo "==> Exporting SAFE-HAVEN state from $CONTRACT_ID"

admin=$(invoke get_admin)
pending_admin=$(invoke get_pending_admin)
fee_recipient=$(invoke get_fee_recipient)
constants=$(invoke get_constants)
initialized=$(invoke is_initialized)
paused=$(invoke is_paused)
storage_version=$(invoke get_storage_version)
contract_version=$(invoke version)
ledger_time=$(invoke get_time)

depositors='[]'
depositor_offset=0
depositor_total=0
while :; do
    page=$(invoke get_depositors --offset "$depositor_offset" --limit "$PAGE_SIZE")
    items=$(jq -c '.items // []' <<<"$page")
    page_count=$(jq 'length' <<<"$items")
    depositor_total=$(jq -r '.total_count // 0' <<<"$page")
    depositors=$(jq -c --argjson page "$items" '. + $page' <<<"$depositors")
    (( page_count == 0 || depositor_offset + page_count >= depositor_total )) && break
    depositor_offset=$((depositor_offset + page_count))
done

deposits='[]'
while IFS= read -r depositor; do
    ids=$(invoke get_deposit_ids --depositor "$depositor")
    while IFS= read -r deposit_id; do
        deposit_type=$(invoke get_deposit_type --depositor "$depositor" --deposit_id "$deposit_id")
        if [[ "$deposit_type" == *"TimeBased"* ]]; then
            entry=$(invoke get_vault --depositor "$depositor" --deposit_id "$deposit_id")
            type="time"
        else
            entry=$(invoke get_ledger_vault --depositor "$depositor" --deposit_id "$deposit_id")
            type="ledger"
        fi
        deposits=$(jq -c \
            --arg depositor "$depositor" \
            --arg deposit_id "$deposit_id" \
            --arg type "$type" \
            --argjson entry "$entry" \
            '. + [{depositor: $depositor, deposit_id: ($deposit_id | tonumber), type: $type, entry: $entry}]' \
            <<<"$deposits")
    done < <(jq -r '.[]' <<<"$ids")
done < <(jq -r '.[]' <<<"$depositors")

jq -n \
    --arg exported_at "$timestamp" \
    --arg contract_id "$CONTRACT_ID" \
    --arg network "$NETWORK" \
    --arg contract_version "$contract_version" \
    --argjson ledger_time "$ledger_time" \
    --argjson admin "$admin" \
    --argjson pending_admin "$pending_admin" \
    --argjson fee_recipient "$fee_recipient" \
    --argjson constants "$constants" \
    --argjson initialized "$initialized" \
    --argjson paused "$paused" \
    --argjson storage_version "$storage_version" \
    --argjson depositor_count "$depositor_total" \
    --argjson depositors "$depositors" \
    --argjson deposits "$deposits" \
    '{schema_version: 1, exported_at: $exported_at, contract_id: $contract_id, network: $network,
      export_scope: "current logical state via public read-only contract methods",
      contract: {version: $contract_version, ledger_time: $ledger_time, initialized: $initialized,
        paused: $paused, storage_version: $storage_version, admin: $admin,
        pending_admin: $pending_admin, fee_recipient: $fee_recipient, constants: $constants},
      depositor_count: $depositor_count, depositors: $depositors, deposits: $deposits}' \
    >"$backup_file"

case "$BACKUP_STORAGE" in
    ipfs)
        command -v curl >/dev/null || { echo "ERROR: curl is required for IPFS uploads." >&2; exit 1; }
        ipfs_response=$(curl -fsS -X POST -F "file=@$backup_file" "$IPFS_API_URL/api/v0/add?pin=true")
        ipfs_cid=$(jq -er '.Hash' <<<"$ipfs_response")
        echo "IPFS CID: $ipfs_cid"
        ;;
    s3)
        : "${S3_BUCKET:?Set S3_BUCKET when BACKUP_STORAGE=s3}"
        command -v aws >/dev/null || { echo "ERROR: aws CLI is required for S3 uploads." >&2; exit 1; }
        aws s3 cp "$backup_file" "s3://$S3_BUCKET/${S3_PREFIX}${backup_file##*/}"
        ;;
    local)
        echo "Backup retained locally; configure BACKUP_STORAGE=ipfs or s3 for remote storage."
        ;;
    *)
        echo "ERROR: BACKUP_STORAGE must be ipfs, s3, or local." >&2
        exit 1
        ;;
esac

echo "Backup written: $backup_file"