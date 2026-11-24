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

## Retention Policy

Snapshots are retained in tiers to satisfy compliance requirements while keeping storage bounded. Apply the policy to the objects produced by the daily job (IPFS pins or S3 objects under `S3_PREFIX`):

| Tier | Kept | Purpose |
| --- | --- | --- |
| Daily | 30 most recent daily snapshots | Short-term recovery from recent corruption or bad deploys |
| Weekly | 12 weekly snapshots (Sunday) | Medium-term rollback across a quarter |
| Monthly | 12 monthly snapshots (1st of month) | Compliance retention (1 year) |

Snapshots older than the monthly tier (12 months) may be pruned. Never prune a snapshot that is the only copy of a tier boundary until its replacement has been verified. For S3, use a lifecycle rule or a small pruning script that keeps the newest object per tier and deletes the rest; for IPFS, unpin objects that fall outside the policy.

## Monitoring and Alerts

A backup that silently fails is worse than no backup. The daily job must be monitored so that a failed or missing run triggers an alert:

- The cron entry redirects stdout/stderr to `/var/log/safe-haven-backup.log`; the script exits non-zero on failure.
- Wrap the cron command so a non-zero exit sends an alert (email, Slack webhook, or PagerDuty), for example:

```cron
15 2 * * * cd /opt/safe-haven && CONTRACT_ID="C..." STELLAR_SOURCE="backup-reader" NETWORK="testnet" BACKUP_STORAGE="ipfs" /opt/safe-haven/scripts/backup_contract.sh >> /var/log/safe-haven-backup.log 2>&1 || /opt/safe-haven/scripts/alert_backup_failure.sh
```

- Add a dead-man's-switch check (e.g. a healthchecks.io ping or a separate cron that alerts if no snapshot was produced in the last 26 hours) so a missed run is also detected.
- Alert on: non-zero exit, empty/invalid JSON output, and no new snapshot within the expected window.

## Disaster Recovery Procedure

Restoration must complete in under 1 hour. Follow these steps when the contract state is lost or corrupted:

1. **Identify the target snapshot.** Pick the most recent verified snapshot that predates the incident. Record its CID or S3 key, export timestamp, schema version, and storage version.
2. **Retrieve the snapshot.** Download from IPFS (`ipfs get <CID>`) or S3 (`aws s3 cp s3://<bucket>/<key> ./snapshot.json`).
3. **Validate.** Confirm the JSON parses (`jq . snapshot.json`) and that the contract ID and network match the deployment being restored.
4. **Reconstruct state.** Re-apply the snapshot's configuration, depositors, and deposits to the target contract using the admin/restore entry points, in the order: configuration, depositors, timestamp deposits, ledger deposits. Verify counts against the snapshot after each phase.
5. **Verify.** Run the contract's read-only queries and compare totals and depositor lists against the snapshot. Confirm the restored contract responds correctly to a sample deposit/withdraw.
6. **Record.** Log the snapshot used, restore start/end times, and verification results in the incident record.

## Monthly Restoration Test

On the first business day of each month, perform a restoration drill against a non-production (testnet) deployment using the most recent monthly snapshot:

1. Retrieve and validate the snapshot as in steps 1–3 above.
2. Reconstruct state on the testnet contract and run the verification in step 5.
3. Record the drill duration; it must be under 1 hour. If it exceeds the target, investigate and update this procedure.
4. File the drill result (snapshot used, duration, pass/fail) with the operations log so compliance can confirm monthly testing.
## Encryption at Rest

Sensitive fields (for example social security numbers and credit card data) are encrypted at the field level before they are persisted, so a raw database dump or backup snapshot never contains them in plaintext. The application decrypts them on read and re-encrypts on write; encrypted values cannot be read directly from the database.

### Key Management

- The encryption key is supplied to the application through the `ENCRYPTION_KEY` environment variable and is never committed to the repository or written into backups.
- Store the key in a secrets manager (or the deployment platform's secret store) and inject it at runtime; restrict access to the application service only.
- Backups contain only ciphertext. To restore and use encrypted fields, the same key must be available to the restoring application; without it the encrypted columns remain unreadable.
- Key rotation is out of scope for this change; when rotating, re-encrypt existing values with the new key before retiring the old one.

### Search on Encrypted Fields

Exact-match lookups on encrypted fields use a deterministic blind index (a keyed hash of the normalized value) stored alongside the ciphertext, so search continues to work without exposing plaintext. Range or partial-match queries are not supported on encrypted fields.

### Backup / Restore with Encryption

1. Take a snapshot as described above; encrypted fields appear only as ciphertext plus their blind index.
2. Restore the snapshot into a database and start the application with the same `ENCRYPTION_KEY`.
3. Verify that the application can decrypt and display the sensitive fields and that exact-match search returns the expected records.
