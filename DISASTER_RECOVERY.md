# SAFE-HAVEN Disaster Recovery Runbook

This runbook covers incidents affecting a deployed SAFE-HAVEN Soroban contract and its operators. It is an operational procedure, not a substitute for an independent smart-contract audit.

Use [MONITORING.md](MONITORING.md) for the recurring health checks and alerts that should detect these incidents early.

## Scope and Assumptions

- The contract is deployed on Stellar/Soroban and identified by a contract ID.
- The deployed contract exposes `pause`, `unpause`, `emergency_withdraw`, and two-step admin transfer.
- The contract has no upgrade entry point. A fixed implementation must be deployed as a new contract and users must move funds through supported contract calls.
- `emergency_withdraw` sends the recorded deposit amount to the depositor. It does not send funds to the admin.
- Stellar ledger history is append-only. “Data corruption” means an application, indexer, RPC, or storage-observation inconsistency unless ledger evidence shows otherwise.
- Never put secret keys, seed phrases, signed XDR, or unredacted user data in tickets, chat, logs, or Git.

## Roles and Responsibilities

| Role | Responsibilities | Decision authority |
|---|---|---|
| Incident commander | Opens the incident, assigns tasks, maintains the timeline, chooses severity, and approves closure. | Coordinates all response decisions. |
| Contract operator | Controls the admin signer, pauses the contract, runs read-only checks, and submits emergency recovery transactions. | Contract state changes, subject to approval policy. |
| Security lead | Validates exploitability, preserves evidence, coordinates disclosure, and approves containment and remediation. | Security severity and disclosure path. |
| Release/deployment engineer | Builds, tests, audits, deploys, and verifies a replacement contract and frontend configuration. | Release readiness and deployment execution. |
| Data/reconciliation owner | Compares ledger events, contract queries, RPC responses, and application indexes; produces the affected-deposit ledger. | Reconciliation sign-off. |
| Communications owner | Sends user, partner, and status updates using approved channels. | External messaging. |
| Depositor support | Handles identity and transaction questions without requesting private keys; tracks claims and returned funds. | Case intake and user follow-up. |

For a small team, one person may hold multiple roles, but the incident commander, security lead, and transaction approver should be separate people for a material incident whenever possible.

## Severity and Escalation

| Severity | Trigger | Initial response | Update cadence |
|---|---|---|---|
| SEV-1 | Active or likely fund loss, private admin key compromise, or contract-wide integrity failure. | Pause immediately; page security lead and incident commander. | Every 30 minutes until contained. |
| SEV-2 | Contract vulnerability with no confirmed loss, repeated failed withdrawals, or material data inconsistency. | Restrict deposits, preserve evidence, and start reconciliation. | Every 2 hours. |
| SEV-3 | Isolated UI, RPC, indexer, or documentation issue with no evidence of incorrect on-chain state. | Route to the owning engineer and monitor. | Daily until resolved. |

Escalate immediately to SEV-1 when any of the following is observed: an unauthorized admin transaction, a token balance decrease not explained by a valid withdrawal, a compromised signer, or evidence that users cannot recover funds through the documented contract paths. Notify the security contact in [SECURITY.md](SECURITY.md) privately; replace its placeholder address before production use.

The incident commander may declare containment only after the security lead confirms that no new unauthorized state changes are occurring and the data owner records the last verified ledger. Public disclosure, user remediation, and unpausing require security and incident-commander approval.

## Common First Response

1. Declare the incident in the private incident channel and record UTC time, reporter, contract ID, network, symptoms, and suspected first affected ledger.
2. Freeze unrelated deployments and frontend configuration changes.
3. Preserve evidence: transaction hashes, operation XDR where safe, RPC responses, contract events, logs, frontend error output, current WASM checksum, and the deployed contract ID. Store artifacts read-only with restricted access.
4. Verify the network and contract ID before signing anything. Use read-only queries first.
5. Confirm the current admin, pending admin, paused state, contract initialization state, and contract/token balances from more than one trusted RPC or explorer when possible.
6. Require a second operator to review every recovery transaction’s destination, amount, contract ID, network passphrase, and simulation result.
7. Keep a transaction ledger containing: UTC time, operator, reviewer, function, depositor, deposit ID, token, amount, transaction hash, result, and evidence link.

## Scenario A: Contract Vulnerability or Active Exploit

### Containment

1. Set severity to SEV-1 unless the security lead confirms there is no plausible fund-loss path.
2. Stop frontend deposits and any automated submissions. If the admin signer is available, call `pause(admin)` and verify `is_paused() == true` from the network.
3. Do not call the suspected vulnerable function again except as part of an approved reproduction in an isolated environment. Do not attempt an unreviewed patch against production state.
4. Identify the vulnerable version, affected entry points, token addresses, affected depositor IDs, first and last suspect ledgers, and the remaining contract balance for each token.
5. Capture all relevant events and transactions before changing state. Keep the original contract ID active for evidence and read-only reconciliation.

### Recovery and remediation

1. The security lead writes a short impact assessment and an exploit/reproduction test. The release engineer adds a regression test and runs `make fmt-check`, `make lint`, and `make test`.
2. If the admin is trusted and the contract’s recovery path is safe, create an affected-deposit list from on-chain state. For each confirmed deposit, independently verify depositor, ID, token, and amount before calling `emergency_withdraw(admin, depositor, deposit_id)`.
3. Submit recovery calls in small reviewed groups. After each transaction, verify the event, recipient balance, removed deposit state, and remaining contract token balance. Never substitute an admin-controlled destination: the contract is designed to return funds to the depositor.
4. If emergency withdrawal is unsafe or the admin is compromised, do not sign recovery transactions. Escalate to the security lead and incident commander for a coordinated disclosure and a chain-specific legal/technical response; the contract cannot be upgraded in place.
5. Build and deploy a reviewed replacement contract with a new contract ID. Initialize it with the approved admin and fee recipient, then verify `get_admin()`, `get_constants()`, `is_initialized()`, and `is_paused()`.
6. Update the frontend contract ID only after deployment verification. Keep the replacement paused until smoke tests, reconciliation, and communications are complete.
7. Publish migration instructions that require users to sign their own deposits or withdrawals. Do not ask users for secret keys. Record old and new transaction hashes.
8. Leave the vulnerable contract paused when possible. Unpause it only if the security lead documents why it is safe and the incident commander approves it.

### Exit criteria

- No unauthorized transactions after the containment ledger.
- Impacted deposits have a disposition: returned, migrated, or explicitly unresolved with an owner.
- Regression tests and deployment verification pass.
- Users have received an accurate status and recovery path.

## Scenario B: Data Corruption or State Mismatch

### Triage

1. Treat an indexer or frontend mismatch as untrusted application data until independently confirmed. Do not rewrite an index or delete records during investigation.
2. Compare, for the affected contract and time window: `get_deposit_ids`, `get_vault` or `get_ledger_vault`, `get_depositor_count`, token balances, contract events, transaction results, and ledger sequence/timestamp.
3. Check both timestamp-based and ledger-based deposits. Ledger-based deposits are read with `get_ledger_vault`; `get_vault` intentionally returns no value for them.
4. Determine whether the issue is stale RPC data, a missed event, an expired read cache, a frontend parsing error, Soroban persistent-entry TTL behavior, or an actual contract transaction. Record the last consistent ledger and a reproducible query.

### Recovery

1. If only the application view is wrong, pause writes to the affected indexer or frontend workflow, replay events from the last verified ledger, and rebuild derived state from on-chain results. Keep the contract paused only if user actions could be misdirected.
2. If contract storage is missing or inconsistent on-chain, preserve the ledger evidence and escalate to the security lead as SEV-1/SEV-2. Do not fabricate state or credit a user based only on the indexer.
3. Reconcile every affected `(depositor, deposit_id)` against the token transfer events and current contract balance. Flag any amount that cannot be proven from both sides.
4. For a verified active deposit with a functioning contract, use the normal `withdraw`/`withdraw_to` path when unlocked, or the reviewed admin `emergency_withdraw` path when emergency recovery is authorized.
5. For expired or missing persistent entries, document the exact ledger evidence, contract response, RPC endpoint, and attempted remediation. Escalate unresolved cases rather than silently marking them paid.
6. Restore the indexer from a clean checkpoint, replay through the current ledger, compare totals, and have the data owner sign off before resuming writes or unpausing.

### Exit criteria

- Derived data matches independently queried on-chain state at a named ledger.
- Token inflows, outflows, and remaining balances reconcile per asset.
- A backup/checkpoint and replay procedure are tested and documented.
- Any unresolved deposits have named owners and user communications.

## Scenario C: Admin Key Loss, Compromise, or Lockout

### Determine the state

1. Set severity to SEV-1 for suspected compromise and SEV-2 for accidental loss without evidence of misuse.
2. Query `get_admin()` and `get_pending_admin()` from a trusted RPC. Confirm whether the current signer is the stored admin, whether the contract is paused, and whether a transfer is pending.
3. Check recent admin events and transactions for `pause`, `unpause`, `emergency_withdraw`, `transfer_admin`, `accept_admin`, `cancel_transfer_admin`, and `renounce_admin`.

### Recovery paths

1. **Known-good current admin available:** rotate immediately with `transfer_admin(admin, new_admin)`. Have the new admin call `accept_admin(new_admin)`. Verify both `get_admin()` and `get_pending_admin()` before and after acceptance.
2. **Pending admin available:** have the intended new admin call `accept_admin(new_admin)`, then rotate again to the approved operational signer if needed. If the pending address is wrong and the current admin is available, call `cancel_transfer_admin(admin)` first.
3. **Key suspected compromised:** from the known-good signer, call `pause(admin)` first, then transfer admin to a clean signer. Revoke or quarantine the compromised key, inspect all transactions it signed, and treat any unexplained emergency withdrawal as a security incident.
4. **Key lost but an approved backup signer exists:** use the backup only if it is the actual stored admin address or is already the pending admin. A separate backup key cannot take control by itself; the contract has no recovery override.
5. **Key lost and no pending/backup authority exists:** the deployed contract cannot be administered. Users can still use normal permissionless withdrawal paths when their deposits unlock, but emergency withdrawal, pause, and admin rotation are unavailable. Deploy a new contract for future deposits and communicate the limitation.
6. **Admin renounced:** treat admin recovery as impossible by design. `renounce_admin` permanently removes the admin; no new admin can be installed. Use permissionless withdrawals and a new deployment for future deposits.

### Prevention and readiness

- Keep the operational admin in a hardware-backed or multisignature account where the deployment model supports it.
- Store encrypted offline backups and test restoration without exposing production secrets.
- Use two-person approval for emergency withdrawals, renunciation, admin transfer, and unpause.
- Record the current admin, pending admin, contract ID, network, and verified signer ownership in the restricted operations inventory.

## Communications and Closure

The communications owner should publish only verified facts: affected network and contract ID, incident start/containment ledger, whether deposits are paused, user action required, and the next update time. Never publish private keys or speculate about individual balances.

Close the incident only after the incident commander has accepted the security assessment, reconciliation report, transaction ledger, deployment or indexer verification, user communication record, and a follow-up list with owners and due dates. The security lead should also decide whether coordinated vulnerability disclosure or regulator/legal notification is required.

## Useful Read-Only Checks

Use the project’s configured Soroban RPC and network passphrase. Substitute the real contract ID and address values; do not put secrets in shell history.

```bash
soroban contract invoke --id "$CONTRACT_ID" --network testnet --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_admin
soroban contract invoke --id "$CONTRACT_ID" --network testnet --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_pending_admin
soroban contract invoke --id "$CONTRACT_ID" --network testnet --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- is_paused
soroban contract invoke --id "$CONTRACT_ID" --network testnet --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" -- get_deposit_ids --depositor "$DEPOSITOR"
```

Before production, replace the placeholder security email in [SECURITY.md](SECURITY.md), document the production RPC and custody arrangement in restricted operator records, and test this runbook on testnet with an approved incident exercise.