# Contract Backups

`scripts/backup_contract.sh` creates a daily JSON snapshot of the deployed SAFE-HAVEN contract and stores it in IPFS or an S3-compatible cloud bucket.

The export contains the current logical state available through the contract's public read-only methods: configuration, active depositors, and both timestamp-based and ledger-based deposits. Soroban does not provide contract code with arbitrary storage iteration, so expired or removed records that are no longer returned by those methods are not included. Keep the contract ID, network, export timestamp, schema version, and storage version with every snapshot to support recovery decisions.

## Configuration

Required:

```bash
export CONTRACT_ID="C..."
export STELLAR_SOURCE="backup-reader"
```

Optional settings include `NETWORK` (default `testnet`), `RPC_URL`, `PAGE_SIZE` (default `25`), and `BACKUP_DIR` (default `backups`). The Stellar CLI identity must be configured locally and only needs read access for these queries.

For IPFS, run an IPFS API endpoint and optionally set `IPFS_API_URL` (default `http://127.0.0.1:5001`):

```bash
BACKUP_STORAGE=ipfs bash scripts/backup_contract.sh
```

For S3 or an S3-compatible service, configure the AWS CLI credentials and set `S3_BUCKET`; `S3_PREFIX` is optional:

```bash
BACKUP_STORAGE=s3 S3_BUCKET=my-safe-haven-backups bash scripts/backup_contract.sh
```

## Daily Schedule

Run the command from the repository root with cron. Use an absolute path and redirect output to an operational log:

```cron
15 2 * * * cd /opt/safe-haven && CONTRACT_ID="C..." STELLAR_SOURCE="backup-reader" NETWORK="testnet" BACKUP_STORAGE="ipfs" /opt/safe-haven/scripts/backup_contract.sh >> /var/log/safe-haven-backup.log 2>&1
```

Review the resulting IPFS CID or cloud object, monitor failed runs, and periodically test that a snapshot can be downloaded and parsed with `jq`.