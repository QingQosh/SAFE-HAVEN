# SAFE-HAVEN Production Rollout Checklist

**Issue:** #402  
**Last updated:** 2026-08-28  
**Related docs:** [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) · [MONITORING.md](./MONITORING.md) · [CHANGELOG.md](./CHANGELOG.md)

---

## Overview

This checklist is the authoritative runbook for every production deployment of the SAFE-HAVEN smart contract and frontend. Work through it sequentially. Every item must be checked off or explicitly deferred (with written justification) before moving to the next phase.

### Why this matters

Soroban contracts are **immutable**. There is no upgrade path — a "rollback" deploys the previous WASM as an entirely new contract and requires updating the frontend to point at the new contract ID. Users with active deposits in the replaced contract must wait for their lock to expire and withdraw manually. Take pre-deployment verification seriously.

### Roles

| Role | Responsibilities |
|---|---|
| **Dev Lead** | Code review, CI sign-off, version tag, release notes |
| **Dev** | WASM build/size, env var prep, frontend build |
| **Security** | `cargo audit`, vulnerability triage |
| **DevOps** | Deployment execution, artifact backup, monitoring |
| **QA** | Smoke tests, post-deploy validation |
| **Admin** | Contract initialization, admin key management |
| **Communications** | User notifications, status page updates |
| **Project Manager** | Scheduling, on-call coordination, debrief |
| **Support** | Post-deploy user issue triage |

### Time estimate

| Phase | Duration |
|---|---|
| Phase 1 — Pre-Rollout (T-48h to T-24h) | 2–4 hours spread over 2 days |
| Phase 2 — Pre-Rollout (T-2h) | 30–60 minutes |
| Phase 3 — Deployment (T=0) | 30–60 minutes |
| Phase 4 — Post-Deployment Validation | 30–60 minutes |
| Phase 5 — 24h Follow-up | 30 minutes |
| **Total active time** | **~2–4 hours** |

### Preferred rollout window

**Weekday 02:00–06:00 UTC** (low-traffic period for Stellar mainnet). Avoid Fridays and days preceding public holidays when on-call coverage may be reduced.

---

## Phase 1 — Pre-Rollout (T-48h to T-24h)

Complete all items below at least 24 hours before the deployment window. Run `bash scripts/pre_deploy_check.sh` to automate the technical checks (items marked 🤖).

### Code Quality & CI

- [ ] **Code review approved by ≥ 2 reviewers** — both the contract and frontend changes reviewed. _(Owner: Dev Lead)_
- [ ] **All CI checks pass on the release branch** — security-audit, lint (`clippy`), unit tests, frontend build (`npm run build`), and `cargo deny` must all be green. _(Owner: Dev Lead)_
- [ ] 🤖 **`cargo fmt --check` passes** — no unformatted Rust source files. _(Owner: Dev)_
- [ ] 🤖 **`cargo clippy --all-targets --features testutils -- -D warnings` passes** — zero warnings treated as errors. _(Owner: Dev)_
- [ ] 🤖 **`cargo test --features testutils` passes** — all 48+ unit tests green. _(Owner: Dev)_
- [ ] 🤖 **`cargo audit` passes** — no HIGH or CRITICAL vulnerabilities in the dependency tree. _(Owner: Security)_
- [ ] 🤖 **`make check-wasm-size` passes** — optimized WASM is within the 64 KB limit. _(Owner: Dev)_
- [ ] **WASM size noted** — record the exact byte count from `make check-wasm-size` output: `_______ bytes`. _(Owner: Dev)_

### Documentation & Versioning

- [ ] **CHANGELOG.md `[Unreleased]` section has content** — all changes for this release documented before the tag is created. _(Owner: Dev Lead)_
- [ ] **`docs/API.md` is current** — reflects any new or changed contract functions, parameters, or error codes. _(Owner: Dev)_
- [ ] **Semantic version tag created and pushed:**
  ```bash
  git tag v0.X.Y
  git push origin v0.X.Y
  ```
  Record the tag: `___________` _(Owner: Dev Lead)_
- [ ] **CI testnet deployment triggered by tag** — verify that the CI workflow triggered and deployed successfully to testnet after the tag push. _(Owner: Dev Lead)_

### Artifacts & Rollback Readiness

- [ ] **Previous deployment artifact backed up:**
  ```bash
  make backup
  ```
  Note the backup location: `deployments/mainnet/___________/` _(Owner: DevOps)_
- [ ] **Rollback plan confirmed** — the previous WASM artifact path is recorded and verified to exist:
  ```
  Previous artifact dir: deployments/mainnet/___________/
  Previous contract ID:  C___________
  ```
  _(Owner: DevOps)_
- [ ] **Previous WASM checksum matches** — compare `sha256sum` of the backup WASM against the checksum in the artifact manifest. _(Owner: DevOps)_

### Testnet Validation

- [ ] **Full smoke test suite passes on testnet** — run the equivalent of `make smoke-test-local` against the testnet deployment:
  ```bash
  NETWORK=testnet CONTRACT_ID=<testnet-contract-id> bash scripts/smoke_test_local.sh
  ```
  _(Owner: QA)_
- [ ] **`migrate()` tested on testnet** — if this release includes a storage version bump, the migration was run and verified on testnet. _(Owner: Dev)_
- [ ] **Frontend connects to testnet contract** — manually verify the frontend build points at the testnet contract ID and the dashboard loads without console errors. _(Owner: QA)_

### Coordination

- [ ] **Admin notified of upcoming deployment** — admin key holder confirms availability during the rollout window. _(Owner: Project Manager)_
- [ ] **On-call schedule confirmed** — at least one engineer on call throughout the rollout window and for 4 hours after. _(Owner: DevOps)_

---

## Phase 2 — Pre-Rollout (T-2h)

Complete these items approximately 2 hours before the scheduled deployment window.

### Communications

- [ ] **User-facing maintenance notification sent** — post to status page, Discord, and any other user-facing channels:
  > "Scheduled maintenance: SAFE-HAVEN will deploy a contract upgrade on [DATE] at approximately [TIME] UTC. Existing deposits are unaffected. New deposits will be temporarily unavailable during the ~30-minute deployment window."
  _(Owner: Communications)_

### System Health

- [ ] **Monitoring dashboards reviewed** — check the past 24 hours in MONITORING.md dashboards. No anomalies (error spikes, RPC timeouts, unusual transaction failure rates) in the pre-deployment window. Note any open alerts: `___________` _(Owner: DevOps)_
- [ ] **Stellar network health checked** — verify Stellar mainnet has no active incidents at [https://status.stellar.org](https://status.stellar.org). _(Owner: DevOps)_
- [ ] **RPC endpoint reachable:**
  ```bash
  curl -s https://soroban.stellar.org/health | jq .
  ```
  _(Owner: DevOps)_

### Deployment Readiness

- [ ] **On-call engineer confirmed available** — engineer is reachable and ready for the rollout window. _(Owner: DevOps)_
- [ ] **Rollback decision criteria agreed upon:** "Roll back immediately if smoke tests fail within 30 minutes of deployment, or if any user reports inability to withdraw existing funds." _(Owner: Dev Lead)_
- [ ] **`SOROBAN_SECRET_KEY` available** — deployer key is accessible (hardware wallet unlocked, or CI secret confirmed), not expired. _(Owner: DevOps)_
- [ ] **Frontend env vars prepared** — the following values are ready to paste into `frontend/.env` after deployment:
  ```
  VITE_CONTRACT_ID=           <new — to fill in post-deploy>
  VITE_EXPECTED_CONTRACT_VERSION=  <e.g. 0.2.0>
  VITE_NETWORK_PASSPHRASE=    Public Global Stellar Network ; September 2015
  VITE_RPC_URL=               https://soroban.stellar.org
  VITE_HORIZON_URL=           https://horizon.stellar.org
  VITE_EXPLORER_URL=          https://stellar.expert/explorer
  ```
  _(Owner: Dev)_
- [ ] **Storage migration script tested** — if this release bumps `STORAGE_VERSION`, confirm `migrate(admin)` was tested against a copy of production state on testnet and returns `true`. If no migration, mark N/A. _(Owner: Dev)_
- [ ] **Deployment command confirmed** — dry-run the deploy script in `--help` mode to confirm the correct network flags:
  ```bash
  bash scripts/deploy.sh --help
  ```
  _(Owner: DevOps)_

---

## Phase 3 — Deployment (T=0)

Work through these steps in order. Do not skip ahead. Record timestamps for every step.

**Start time (UTC):** `__________`

### Contract Deployment

- [ ] **Deploy contract to mainnet:**
  ```bash
  export SOROBAN_SECRET_KEY=S...
  export FEE_RECIPIENT=G...          # mainnet fee recipient address
  make deploy-mainnet
  ```
  If the script fails, do NOT retry until you understand the failure. Check `deployments/mainnet/` for a partial artifact before re-running. _(Owner: DevOps)_

- [ ] **Record new Contract ID** from deployment output:
  ```
  New contract ID: C___________
  Artifact dir:    deployments/mainnet/___________/
  ```
  _(Owner: DevOps)_

- [ ] **Verify WASM hash matches expected** — compare the hash in the new artifact manifest against `sha256sum` of the locally built optimized WASM:
  ```bash
  sha256sum target/safe_haven.optimized.wasm
  cat deployments/mainnet/<timestamp>/manifest.json | jq .wasm_sha256
  ```
  _(Owner: DevOps)_

### Contract Initialization

- [ ] **Verify contract is live:**
  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --network mainnet \
    -- is_initialized
  ```
  Expected: `false` (new contract, not yet initialized). _(Owner: DevOps)_

- [ ] **Run `initialize()` with correct admin and fee_recipient:**
  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --source <DEPLOYER_IDENTITY> \
    --network mainnet \
    -- initialize \
    --admin <ADMIN_ADDRESS> \
    --fee_recipient <FEE_RECIPIENT_ADDRESS>
  ```
  _(Owner: Admin)_

- [ ] **Verify initialization succeeded:**
  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --network mainnet \
    -- is_initialized
  # Expected: true

  stellar contract invoke \
    --id <CONTRACT_ID> \
    --network mainnet \
    -- get_admin
  # Expected: <ADMIN_ADDRESS>

  stellar contract invoke \
    --id <CONTRACT_ID> \
    --network mainnet \
    -- version
  # Expected: current contract version string
  ```
  _(Owner: Admin)_

- [ ] **Run storage migration if applicable:**
  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --source <ADMIN_IDENTITY> \
    --network mainnet \
    -- migrate \
    --admin <ADMIN_ADDRESS>
  ```
  If no migration needed for this release, mark N/A. _(Owner: Admin)_

### Frontend Deployment

- [ ] **Update `frontend/.env` with new contract ID:**
  ```bash
  # Edit frontend/.env:
  VITE_CONTRACT_ID=<NEW_CONTRACT_ID>
  VITE_EXPECTED_CONTRACT_VERSION=<e.g. 0.2.0>
  # (ensure mainnet URLs — not testnet — for all other vars)
  ```
  _(Owner: Dev)_

- [ ] **Build frontend:**
  ```bash
  cd frontend && npm run build
  ```
  Build must complete with zero errors. _(Owner: Dev)_

- [ ] **Deploy frontend to hosting** — upload `frontend/dist/` to the production hosting environment (Vercel / Netlify / S3+CloudFront / etc.). _(Owner: DevOps)_

- [ ] **Verify frontend connects to new contract** — open the production URL in a browser:
  - Dashboard loads without console errors
  - Version shown in the UI matches the deployed contract version
  - No version-mismatch warning banner visible
  - Wallet connection prompt functions correctly
  _(Owner: QA)_

**Deployment complete time (UTC):** `__________`

---

## Phase 4 — Post-Deployment Validation (T+15min to T+30min)

These checks confirm the deployment is healthy. Begin within 15 minutes of Phase 3 completion.

### Smoke Tests

- [ ] **Smoke test: deposit** — using a test wallet on mainnet, deposit a minimal amount (e.g., 1 unit of a test token if available, or smallest practical amount) and verify:
  - Transaction confirmed on-chain
  - Vault entry appears on the dashboard
  - Deposit ID and unlock time displayed correctly
  _(Owner: QA)_

- [ ] **Smoke test: `get_vault` query** — query the new deposit via the CLI and confirm the `VaultEntry` fields match the deposit inputs:
  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --network mainnet \
    -- get_vault \
    --depositor <TEST_ADDRESS> \
    --id 0
  ```
  _(Owner: QA)_

- [ ] **Smoke test: `time_remaining` query** — confirm it returns a non-zero value for the test deposit. _(Owner: QA)_

- [ ] **Smoke test: version banner absent** — the frontend does NOT display a "contract version mismatch" warning. _(Owner: QA)_

- [ ] **Smoke test: admin functions accessible** — from the Admin page:
  - `get_admin()` returns the correct admin address
  - `is_paused()` returns `false`
  - `pause()` / `unpause()` round-trip succeeds (optional, use caution)
  _(Owner: Admin)_

- [ ] **Smoke test: `deposit_for` works** — if the release includes or touches `deposit_for`, verify a deposit for a second address completes successfully. _(Owner: QA)_

### Monitoring

- [ ] **Transaction success rate normal** — check monitoring dashboards (see MONITORING.md). No unusual failure rate spike in the first 15 minutes post-deploy. _(Owner: DevOps)_
- [ ] **No RPC error spikes** — check RPC provider logs or Stellar network status for abnormal error rates. _(Owner: DevOps)_
- [ ] **No frontend JS errors** — check browser console and any error-reporting service (Sentry etc.) for new error classes introduced by this release. _(Owner: QA)_

### Record-Keeping

- [ ] **Update deployment log** — append the following to `deploy_mainnet.log`:
  ```
  DATE=<ISO8601>  CONTRACT_ID=<NEW>  VERSION=<TAG>  DEPLOYER=<IDENTITY>  STATUS=SUCCESS
  ```
  _(Owner: DevOps)_

- [ ] **Tag GitHub release** — create a GitHub release from the version tag with release notes copied from CHANGELOG.md:
  ```bash
  gh release create v0.X.Y --notes-file <(sed -n '/## \[0\.X\.Y\]/,/## \[/p' CHANGELOG.md | head -n -1)
  ```
  _(Owner: Dev Lead)_

- [ ] **Announce deployment complete** — send completion announcement (see Communication Plan below). _(Owner: Communications)_

---

## Phase 5 — 24h Follow-up

Complete these items within 24 hours of deployment.

- [ ] **Review error logs and monitoring dashboards** — look for any new error patterns, unusual transaction volumes, or RPC anomalies not caught in Phase 4. _(Owner: DevOps)_
- [ ] **Confirm no user-reported issues** — check support channels (Discord, GitHub Issues, email) for reports of failed withdrawals, missing deposits, or UI errors. _(Owner: Support)_
- [ ] **Confirm existing deposits are intact** — spot-check several pre-existing deposits from before the deployment and verify they are still queryable and show correct state. _(Owner: QA)_
- [ ] **Close rollout ticket / GitHub milestone** — mark issue #402 resolved and close the milestone if applicable. _(Owner: Dev Lead)_
- [ ] **Schedule rollout debrief** — schedule a 30-minute debrief within 48 hours using the template below. _(Owner: Project Manager)_

---

## Rollback Procedure

> **IMPORTANT:** Soroban contracts are immutable. Rolling back means deploying the **previous WASM as a brand-new contract** with a new contract ID. Users with active deposits in the new contract are NOT automatically migrated — they retain access to their funds via the new contract's existing state. A rollback requires careful coordination.

Before triggering a rollback, confirm:
1. The failure is confirmed (not a transient RPC error).
2. The decision criteria from Phase 2 are met: smoke tests failed within 30 minutes, or users cannot withdraw funds.
3. The incident commander has approved the rollback.

For severe incidents, consult [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) first.

### Step-by-step rollback

**Step 1 — Pause the new contract** (if admin is available and contract is functional enough to accept the call):
```bash
stellar contract invoke \
  --id <NEW_CONTRACT_ID> \
  --source <ADMIN_IDENTITY> \
  --network mainnet \
  -- pause \
  --admin <ADMIN_ADDRESS>
```
Verify: `is_paused()` returns `true`.

**Step 2 — Identify the previous artifact directory:**
```bash
ls -lt deployments/mainnet/       # most recent is the new one; second is previous
# Example: deployments/mainnet/20260828T020000Z/
```

**Step 3 — Deploy the previous WASM as a new contract:**
```bash
export SOROBAN_SECRET_KEY=S...
make rollback NETWORK=mainnet ARTIFACT_DIR=deployments/mainnet/<PREV_TIMESTAMP>
```
This calls `scripts/deploy.sh rollback mainnet --artifact-dir <dir>` which deploys the retained previous WASM and records a new contract ID in the artifact directory.

**Step 4 — Record the rollback contract ID** from the output. Verify it is live:
```bash
stellar contract invoke \
  --id <ROLLBACK_CONTRACT_ID> \
  --network mainnet \
  -- is_initialized
# Expected: false (needs initialization)
```

**Step 5 — Initialize the rollback contract:**
```bash
stellar contract invoke \
  --id <ROLLBACK_CONTRACT_ID> \
  --source <ADMIN_IDENTITY> \
  --network mainnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --fee_recipient <FEE_RECIPIENT>
```

**Step 6 — Revert frontend `.env` to point at the rollback contract:**
```bash
# Edit frontend/.env:
VITE_CONTRACT_ID=<ROLLBACK_CONTRACT_ID>
VITE_EXPECTED_CONTRACT_VERSION=<PREVIOUS_VERSION>
```

**Step 7 — Rebuild and redeploy the frontend:**
```bash
cd frontend && npm run build
# Deploy frontend/dist/ to hosting
```

**Step 8 — Run smoke tests against the rollback contract** — confirm deposits and withdrawals work.

**Step 9 — Send rollback announcement** (see Communication Plan below).

**Step 10 — Open a post-mortem** — document what failed, why the rollback was triggered, and what must be fixed before the next deploy attempt. Reference [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) for the incident timeline template.

> **Note on deposits made to the failed contract:** Any deposits made to the new (failed) contract between its deployment and the rollback remain in that contract. Users can still withdraw from the paused contract once the lock expires (pausing only blocks new deposits, not withdrawals). Document the new contract ID and communicate this clearly to affected users.

---

## Communication Plan

### Channels

| Channel | Audience | Used for |
|---|---|---|
| Status page | All users | Maintenance windows, incidents, resolution |
| Discord `#announcements` | Community | Planned deploys, completion, rollbacks |
| Discord `#support` | Users needing help | Rollback user impact, withdrawal guidance |
| GitHub Releases | Developers / integrators | Release notes, contract ID changes |
| Internal Slack/chat | Team | Operational coordination |
| Email (mailing list) | Subscribed users | Major releases or incidents only |

### Notification timing

| Event | When | Channels |
|---|---|---|
| Deployment planned | T-24h | Status page, Discord `#announcements` |
| Deployment starting | T-5min | Discord `#announcements` |
| Deployment complete | T+15min (after smoke tests pass) | Status page, Discord `#announcements`, GitHub Release |
| Rollback triggered | Immediately | Status page, Discord `#announcements`, Discord `#support` |
| Rollback complete | After smoke tests pass | Status page, Discord `#announcements` |
| Incident resolved | After 24h monitoring | Status page, Discord `#announcements` |

### Message templates

**Deployment starting:**
```
🔧 Maintenance starting now: SAFE-HAVEN is deploying contract v[VERSION].
New deposits are temporarily unavailable. Existing deposits and withdrawals are unaffected.
Expected duration: ~30 minutes. Updates to follow.
```

**Deployment complete:**
```
✅ SAFE-HAVEN v[VERSION] is live on Stellar mainnet.
New contract ID: C[CONTRACT_ID]
All features are available. If you encounter any issues, please let us know in #support.
Full release notes: [GITHUB_RELEASE_URL]
```

**Rollback triggered:**
```
⚠️ We detected an issue with the v[VERSION] deployment and have initiated a rollback.
Deposits made in the last [N] minutes to contract C[NEW_CONTRACT_ID] remain safe — those funds are accessible via withdrawal once your lock expires.
We are restoring the previous version. Updates to follow.
```

**Rollback complete:**
```
✅ Rollback complete. SAFE-HAVEN is operating on the previous stable version (v[PREV_VERSION]).
Contract ID: C[ROLLBACK_CONTRACT_ID]
If you made a deposit during the failed deployment window, your funds are safe in contract C[FAILED_CONTRACT_ID] and will be accessible when your lock expires. Contact #support if you need help.
```

---

## Rollout Debrief Template

Schedule within 48 hours of deployment. Fill in and save as `docs/debrief/rollout-v<VERSION>-<DATE>.md`.

```
# Rollout Debrief — SAFE-HAVEN v[VERSION]

**Date:** [YYYY-MM-DD]
**Facilitator:** [Name]
**Participants:** [Name, Role], [Name, Role], ...

## Timeline

| Time (UTC) | Event |
|---|---|
| | Phase 1 complete |
| | Phase 2 complete |
| | Deployment started |
| | Contract initialized |
| | Frontend deployed |
| | Smoke tests passed |
| | Deployment declared complete |

## Metrics

| Metric | Value |
|---|---|
| Total deploy time (T=0 to smoke tests pass) | |
| Downtime (new deposits unavailable) | |
| Rollback triggered? | Yes / No |
| Rollback duration (if applicable) | |
| User-reported issues within 24h | |
| Tests failed during deploy | |
| Contract ID | |
| WASM size (bytes) | |

## What Went Well

-
-

## What Didn't Go Well

-
-

## Action Items

| Item | Owner | Due date |
|---|---|---|
| | | |

## Notes

[Free-form notes, incident links, etc.]
```
