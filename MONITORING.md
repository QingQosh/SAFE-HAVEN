# SAFE-HAVEN Monitoring and Alerting

This document defines the minimum production monitoring for a deployed SAFE-HAVEN contract. It is a monitoring specification and operator runbook; the repository does not currently include a metrics collector or alertmanager configuration.

## Monitored Resources

Keep a versioned inventory for every deployment containing:

- Network, Soroban RPC URL, Horizon URL, contract ID, WASM checksum, and deployment ledger.
- Expected admin address, expected fee recipient, and native asset used to pay transaction fees.
- Token contract addresses accepted by the deployment.
- Alert destinations and on-call ownership.

All checks must record the network, contract ID, RPC endpoint, observed ledger sequence, observation timestamp, and query result. Use at least two independent RPC endpoints for a critical alert before paging an operator, when available.

## Collection Schedule

| Check | Collection interval | Retention | Owner |
|---|---:|---:|---|
| Initialization, pause state, admin identity | 1 minute | 90 days | Contract operator |
| Admin native balance | 5 minutes | 90 days | Contract operator |
| Contract token balances and active deposit totals | 5 minutes | 1 year | Reconciliation owner |
| Storage entries and TTLs | 15 minutes | 1 year | Data/reconciliation owner |
| Transactions and contract events | Continuous, or every 1 minute | 1 year | Observability owner |
| Synthetic deposit/withdrawal probe | Every 15 minutes on testnet; daily on production | 1 year | Contract operator |

Pollers should use read-only simulation or RPC queries. Never use the production admin signer for a health check. Synthetic probes must use a dedicated test account and token, and must never interact with production user funds.

## Core Metrics

Use labels with bounded cardinality: `network`, `contract_id`, `token`, `method`, and `error_code`. Do not use raw depositor addresses or transaction hashes as metric labels; keep those in structured logs or traces.

### Contract health

Record the following on every poll:

- `safe_haven_contract_initialized` as `0` or `1` from `is_initialized()`.
- `safe_haven_contract_paused` as `0` or `1` from `is_paused()`.
- `safe_haven_admin_match` as `0` or `1` by comparing `get_admin()` with the approved inventory address.
- `safe_haven_pending_admin_present` as `0` or `1` from `get_pending_admin()`.
- `safe_haven_storage_version` from `get_storage_version()` when supported.
- `safe_haven_contract_ledger` and `safe_haven_rpc_observation_age_seconds`.

The expected initialized state is `1`. The expected paused state is normally `0`, except during a declared incident, planned maintenance window, or migration. A missing admin after `renounce_admin` is valid only when the deployment inventory explicitly marks the contract as trustless.

### Balances and solvency signals

Track the admin account’s native asset balance through Horizon or the network’s account endpoint. Alert before the account can no longer pay transaction fees:

- Warning: below `100 XLM` or below 30 days of observed average operating fees, whichever is higher.
- Critical: below `25 XLM` or below 7 days of observed average operating fees, whichever is higher.
- Page immediately when a balance falls by more than `20%` between polls without a recorded approved transaction.

Thresholds must be configured per network and adjusted after observing real fees. Never treat a low admin balance as a contract fund loss: the admin account and contract token balances are separate.

For each supported token, record the contract balance and the sum of active deposit amounts reconstructed from on-chain state. Alert on either:

- Contract balance below the reconciled liability total, after accounting for documented fees and rounding.
- A balance change without a matching `deposit`, `withdraw`, `withdraw_to`, `cancel_deposit`, or approved `emergency_withdraw` event.

These are reconciliation alerts and should page the security lead until explained.

### Storage growth and TTL

Track:

- Active depositor count from `get_depositor_count()`.
- Active IDs per depositor from `get_deposit_ids(depositor)`.
- Timestamp-based entries from `get_vault(depositor, id)`.
- Ledger-based entries from `get_ledger_vault(depositor, id)`.
- Contract persistent-entry count, byte size, and expiration ledger from Soroban RPC ledger-entry inspection.
- Counts of missing entries, stale active IDs, and entries whose TTL is below the configured safety window.

The contract bumps persistent entries to a target derived from the maximum lock duration when entries are written or accessed through mutating contract reads. The public read-only methods intentionally do not extend TTL. Therefore, monitoring must not assume that polling `get_vault` keeps data alive.

Alert when:

- Any active ID has neither a timestamp-based nor ledger-based entry.
- Any entry’s expiration is earlier than its unlock ledger/time plus a documented safety margin.
- Persistent-entry count or byte size grows faster than the approved baseline for two consecutive collection periods.
- The active depositor count or active ID count drops without matching withdrawal/cancellation events.

For an apparent TTL expiration, preserve the entry key, expiration ledger, last successful query, relevant transaction events, and RPC response before attempting a write or migration. Escalate missing on-chain entries as a reconciliation/security incident; do not recreate them from an index alone.

## Initialization Monitoring

1. Query `is_initialized()` and `get_admin()` at every health interval.
2. Compare the result with the deployment inventory and the deployment ledger recorded at release time.
3. Page SEV-1 if an initialized production contract reports `false`, if an uninitialized contract is detected at a published contract ID, or if the admin differs from the approved address without an approved admin-transfer record.
4. Page SEV-2 if the query fails for three consecutive intervals or observations disagree across trusted RPC endpoints.
5. Do not call `initialize` in response to an alert. Initialization is one-time and requires an approved deployment decision.

## Unexpected Pause Monitoring

1. Query `is_paused()` every minute and subscribe to or poll for `paused` and `unpaused` events.
2. Compare every state transition with the maintenance and incident calendar.
3. Page SEV-1 for a pause without an approved operator, incident, or deployment change. Capture the event ledger, transaction hash, signer, and current admin state.
4. Page SEV-2 when the contract remains paused beyond the approved window or when `unpaused` is observed without approval.
5. For an unexpected pause, stop new deposits at the frontend, preserve evidence, notify the incident commander, and follow [DISASTER_RECOVERY.md](DISASTER_RECOVERY.md). Do not unpause until the security lead and incident commander approve it.

## Failed Transactions and Error Rates

Ingest submitted transaction status from Soroban RPC and classify failures by contract method, error code, transport/RPC error, simulation error, and user cancellation. Keep the raw transaction hash and response in restricted structured logs.

Record:

- `safe_haven_transactions_submitted_total`.
- `safe_haven_transactions_confirmed_total`.
- `safe_haven_transactions_failed_total`.
- `safe_haven_transaction_latency_seconds`.
- `safe_haven_simulation_failures_total`.
- `safe_haven_rpc_errors_total` and `safe_haven_rpc_latency_seconds`.
- `safe_haven_contract_error_rate` over 5-minute and 1-hour windows.

Recommended initial alerts, to be tuned from baseline:

- Warning: more than 5% of transactions fail over 15 minutes and at least 20 transactions were attempted.
- Critical: more than 20% fail over 5 minutes and at least 10 transactions were attempted.
- Critical: 5 or more failures of the same contract method and error code in 10 minutes.
- Warning: no successful read-only health check for 5 minutes.
- Critical: no successful health check for 15 minutes, or RPC errors from all configured endpoints.

Separate expected user errors such as `FundsStillLocked`, `InvalidAmount`, and invalid input from infrastructure or contract-integrity failures. A spike in expected user errors can still indicate a frontend regression and should be reported to the frontend owner, but should not page the security on-call by itself.

## Alert Routing and Response

| Alert | Route | First action |
|---|---|---|
| Initialization/admin mismatch | Security lead and incident commander | Verify ledger and admin events from a second RPC; declare SEV-1 if confirmed. |
| Unexpected pause/unpause | Contract operator and incident commander | Stop deposits, preserve event evidence, and follow disaster recovery. |
| Admin balance critical | Contract operator | Fund the approved admin account through the documented treasury process; never use an unapproved signer. |
| Liability/balance mismatch | Security lead and reconciliation owner | Freeze affected workflows and reconcile from events and ledger state. |
| TTL or storage anomaly | Data owner and security lead | Preserve ledger-entry evidence; do not rebuild state from the indexer. |
| Error-rate or RPC outage | Observability owner | Compare endpoints, classify errors, and run a read-only synthetic check. |

Every page must create an incident record containing alert name, first/last observed ledger, contract ID, network, query responses, related transaction hashes, assigned owner, and next update time. Escalate according to the severity rules in [DISASTER_RECOVERY.md](DISASTER_RECOVERY.md).

## Read-Only Probe Examples

Set `CONTRACT_ID`, `RPC_URL`, and `NETWORK_PASSPHRASE` from the deployment inventory. These commands do not require the admin secret key.

```bash
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- is_initialized
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- is_paused
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_admin
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_depositor_count
soroban contract invoke --id "$CONTRACT_ID" --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_storage_version
```

The CLI checks above cover contract state, not account balances, transaction history, or TTL. Those require the configured Horizon/Soroban RPC collector and must be stored with the same ledger sequence as the contract responses.

## Readiness Checklist

- [ ] Production contract inventory has an approved admin, network, RPC endpoints, token list, and expected pause mode.
- [ ] Pollers survive RPC errors, rate limits, restarts, and duplicate events.
- [ ] Alerts have tested routes and named owners.
- [ ] Admin balance thresholds are configured per network.
- [ ] Storage/TTL collection can inspect both timestamp and ledger-based deposits.
- [ ] Failed transactions are retained with method and error classification.
- [ ] A testnet exercise has triggered initialization, pause, low-balance, TTL, and error-rate alerts.
- [ ] On-call responders can follow [DISASTER_RECOVERY.md](DISASTER_RECOVERY.md) without requesting or exposing private keys.