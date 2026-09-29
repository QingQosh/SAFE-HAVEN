# SAFE-HAVEN Contract Upgrade and Migration Playbook

## Purpose

Soroban smart contracts are immutable — once deployed, the code cannot be changed. This playbook defines the procedure for upgrading SAFE-HAVEN, including risk assessment, pre-upgrade testing, migration paths, rollback procedures, and post-upgrade validation.

---

## Table of Contents

1. [Overview and Immutability Model](#overview-and-immutability-model)
2. [Roles and Decision Authority](#roles-and-decision-authority)
3. [Pre-Upgrade Process](#pre-upgrade-process)
4. [Upgrade Procedure](#upgrade-procedure)
5. [Data Migration Strategy](#data-migration-strategy)
6. [Stakeholder Communication Plan](#stakeholder-communication-plan)
7. [Testing Requirements](#testing-requirements)
8. [Rollback Procedures](#rollback-procedures)
9. [Post-Upgrade Validation](#post-upgrade-validation)
10. [Lessons Learned](#lessons-learned)

---

## Overview and Immutability Model

### Why Upgrade?

- **Security fix:** Bug identified in production
- **Feature addition:** New functionality needed (multi-token support, governance, etc.)
- **Optimization:** Gas efficiency or performance improvement
- **Regulatory compliance:** Legal requirements
- **Dependency update:** Soroban SDK or ecosystem change

### How Upgrades Work on Soroban

**Soroban contracts are immutable.** Once deployed, the bytecode cannot be modified. To "upgrade," you must:

1. **Deploy a new contract** with the updated code
2. **Communicate the new contract ID** to the frontend and all integrators
3. **Migrate users and data** from the old contract to the new one (if state transfer is needed)
4. **Retire the old contract** (pause it, or leave it archived for audit trail)

### State Transfer Implications

| Scenario | State Transfer Needed? | Approach |
|---|---|---|
| Bug fix, same storage layout | No | Users call `withdraw` on old contract, then `deposit` on new contract |
| Feature addition (new fields) | Yes | Implement `migrate()` function or manual data rebuild |
| Breaking API change | Varies | May require manual intervention per depositor |
| Pause and deprecate old | No | Pause old contract; users migrate manually |

---

## Roles and Decision Authority

| Role | Responsibilities | Authority |
|---|---|---|
| **Incident Commander** | Declares upgrade urgency; authorizes pausing old contract; approves communication | GO/NOGO for upgrades; defines severity |
| **Security Lead** | Assesses security impact; reviews exploit patches; approves risk mitigation | Security sign-off; disclosure coordination |
| **Release Engineer** | Builds, tests, deploys WASM; verifies checksums; runs smoke tests | Release execution; deployment reversal |
| **Data/Migration Owner** | Maps old state to new state; runs reconciliation; audits migration results | Migration sign-off; data integrity |
| **Frontend Lead** | Updates contract ID in frontend; tests UI against new contract; monitors for breaking changes | UX sign-off; frontend deployment |
| **Communications Owner** | Writes user announcements; manages status page; handles support escalations | External messaging |
| **Sponsor/Stakeholder** | Approves release schedule and resource allocation | Project-level go/no-go |

For small teams, one person may hold multiple roles, but **Security Lead, Release Engineer, and Data Owner should be different people.**

---

## Pre-Upgrade Process

### Phase 1: Decision and Planning (Weeks 1–2)

#### 1.1 Upgrade Proposal

**Document:** `/docs/upgrades/YYYYMMDD-upgrade-proposal.md`

```markdown
## Upgrade Proposal: [Feature/Fix/Optimization]

**Date:** YYYY-MM-DD
**Urgency:** [Critical / High / Medium / Low]
  - Critical: Security exploit in progress; immediate action needed
  - High: Known security issue; no active exploit; deploy within 7 days
  - Medium: Feature request or optimization; deploy within 30 days
  - Low: Nice-to-have improvement; defer if needed

### What's changing?
- Feature/Fix: [Brief description]
- Code changes: [Files modified, new functions, etc.]
- Storage changes: [New fields, removed fields, layout changes?]
- API changes: [New functions, deprecated functions, changed signatures?]
- Breaking changes: [Will old contracts stop working?]

### Why?
- [User feedback, security, performance, etc.]

### Risk assessment
- Security risk: [Low / Medium / High]
- Storage risk: [Low / Medium / High]
- User impact: [Number of users affected, tokens at risk]
- Difficulty: [Low / Medium / High]

### Migration strategy
- [Manual user migration, automatic migration, two-contract period, etc.]

### Timeline
- Code review: [Date]
- Testnet deployment: [Date]
- Mainnet deployment: [Date]

### Approvals needed
- [ ] Security lead sign-off
- [ ] Release engineer feasibility check
- [ ] Sponsor approval
- [ ] Incident commander (if critical)

### Rollback plan (brief)
- Old contract: [Contract ID or TBD]
- Rollback trigger: [What would prompt rollback?]
- Rollback effort: [Hours to reverse]
```

#### 1.2 Risk Assessment

**Complete a risk matrix:**

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Data corruption during migration | Medium | Critical | Dry run on testnet; audit queries |
| New code has a bug on mainnet | Medium | Critical | Comprehensive testing; staged rollout |
| Users confused by new contract ID | High | Low | Clear communication; DNS alias (future) |
| Depositor loses access to funds | Low | Critical | Emergency withdrawal path; manual recovery |
| API breaking change breaks integrations | Medium | Medium | Backwards-compatible design; deprecation window |

#### 1.3 Approval

**Gate:** Before proceeding to development, secure:
- [ ] Security lead: "Risk acceptable given mitigations"
- [ ] Release engineer: "Deployment plan is feasible"
- [ ] Sponsor: "Timeline and resources approved"

---

### Phase 2: Development and Testing (Weeks 3–4)

#### 2.1 Code Changes

**Standard process:**
1. Create a branch: `feat/upgrade-description`
2. Implement changes in `contracts/safe-haven/src/`
3. Add unit tests for new behavior
4. Run `make check` locally (fmt, clippy, tests)
5. Open PR with detailed description

#### 2.2 Migration Function (if needed)

If the upgrade adds new storage fields or changes layout, implement a `migrate()` function:

```rust
pub fn migrate(env: Env) -> Result<(), VaultError> {
    // Check admin authorization
    let admin = storage::get_admin(&env)
        .ok_or(VaultError::Unauthorized)?;
    admin.require_auth();

    // Check if already migrated
    let current_version = storage::get_storage_version(&env)
        .unwrap_or(0);
    if current_version >= TARGET_VERSION {
        return Ok(());  // Already up to date
    }

    // Migrate data
    // Example: add new field with default value
    // for each depositor, ensure new_field is initialized

    // Record migration
    storage::set_storage_version(&env, TARGET_VERSION)?;
    events::emit_migration_complete(&env, TARGET_VERSION);

    Ok(())
}
```

**Key properties of `migrate()`:**
- **Idempotent:** Calling it twice should be safe
- **Admin-only:** Requires `admin.require_auth()`
- **Versioned:** Tracks storage version to detect stale state
- **Observable:** Emits an event
- **Tested:** Run against testnet with real data patterns

#### 2.3 Testing

**Unit tests:**
- [ ] New functions work correctly
- [ ] Migration function is idempotent
- [ ] No storage regressions
- [ ] All existing tests still pass
- Run: `make test`

**Integration tests:**
- [ ] Deploy old contract on local Soroban
- [ ] Create deposits
- [ ] Deploy new contract
- [ ] Call migrate()
- [ ] Verify data still accessible
- [ ] Verify new functionality works
- Run: `make smoke-test-local`

**Testnet dry run:**
- [ ] Deploy to testnet
- [ ] Create test deposits
- [ ] Call migrate() on testnet
- [ ] Query data; verify correctness
- [ ] Manual user acceptance test
- Timeline: 3–5 days

---

### Phase 3: Pre-Upgrade Checklist (Week 4)

#### 3.1 Code Review Checklist

- [ ] Code builds without warnings (`make build`)
- [ ] All tests pass (`make test`)
- [ ] No clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Dependencies are audited (`cargo audit`)
- [ ] License compliance checked (`cargo deny`)
- [ ] PR reviewed by ≥1 CODEOWNER
- [ ] Any security implications documented

#### 3.2 Artifact Preparation

**Build and archive:**

```bash
# Build optimized WASM
make build
make optimize

# Record checksums
sha256sum contracts/safe-haven/target/wasm32-unknown-unknown/release/safe_haven_optimized.wasm

# Archive artifacts
mkdir -p deployments/mainnet/YYYYMMDD-upgrade-name/
cp contracts/safe-haven/target/wasm32-unknown-unknown/release/safe_haven_optimized.wasm \
   deployments/mainnet/YYYYMMDD-upgrade-name/
cp contracts/safe-haven/target/wasm32-unknown-unknown/release/safe_haven.wasm \
   deployments/mainnet/YYYYMMDD-upgrade-name/
echo "[checksum details]" > deployments/mainnet/YYYYMMDD-upgrade-name/CHECKSUMS.txt
```

**Verify WASM size:**
- [ ] WASM is < 65 KB (Soroban limit)
- [ ] Size did not increase unexpectedly

#### 3.3 Migration Plan Finalization

**For data-changing upgrades:**

1. **Define migration steps:**
   - [ ] Which data needs to move?
   - [ ] Which data is obsolete?
   - [ ] How will new fields be initialized?

2. **Test migration on testnet:**
   - [ ] Create representative test data (depositors, deposits, events)
   - [ ] Deploy old contract, populate data
   - [ ] Deploy new contract
   - [ ] Call migrate()
   - [ ] Query data; verify correctness
   - [ ] Check no data loss

3. **Document migration:**
   - [ ] User-facing migration guide (if users take action)
   - [ ] Operator playbook (for us)
   - [ ] Reconciliation queries (to verify success)

#### 3.4 Stakeholder Briefing

**Timeline:** 1 week before mainnet deployment

**Communication:**
- Email to team + sponsor: "Upgrade planned for [date]. Here's what's changing and why."
- Discord announcement: "We're upgrading SAFE-HAVEN on [date]. No action needed from users, but we wanted you to know."
- Status page: "Scheduled maintenance: [date] [time] UTC. Expected downtime: X minutes."

---

## Upgrade Procedure

### Timing and Coordination

**Recommended:**
- **Deploy on Tuesday–Thursday** (if something breaks, response team is available; weekends avoided)
- **Deploy at off-peak times** (early morning UTC, outside typical trading hours)
- **Announce 48 hours in advance** (gives integrators time to test against new contract)

### Mainnet Deployment Steps

#### Step 1: Final Verification (T-30 minutes)

On the release engineer's machine:

```bash
# Verify checksums
sha256sum deployments/mainnet/YYYYMMDD-upgrade-name/safe_haven_optimized.wasm
# Compare against archived checksum

# Verify network passphrase (CRITICAL!)
echo $SOROBAN_NETWORK_PASSPHRASE
# Should output: "Public Global Stellar Network ; September 2015"

# Dry-run deployment on testnet (if not already done)
soroban contract deploy \
  --wasm deployments/mainnet/YYYYMMDD-upgrade-name/safe_haven_optimized.wasm \
  --source $SOROBAN_SOURCE_KEY \
  --network testnet
```

**Verification gate:**
- [ ] Checksum matches archived value
- [ ] Network is testnet (not mainnet)
- [ ] Deployer has key access
- [ ] Two people have reviewed the deployment command

#### Step 2: Mainnet Deployment (T-0)

```bash
# Deploy to mainnet
soroban contract deploy \
  --wasm deployments/mainnet/YYYYMMDD-upgrade-name/safe_haven_optimized.wasm \
  --source $SOROBAN_SOURCE_KEY \
  --network public

# Output: New contract ID
# Example: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4
```

**Record:**
- [ ] Deployment timestamp (UTC)
- [ ] New contract ID
- [ ] Transaction hash
- [ ] Deployer name

#### Step 3: Initialization / Migration (T+5 minutes)

If needed, call `initialize()` or `migrate()`:

```bash
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --source $SOROBAN_ADMIN_KEY \
  --network public \
  -- migrate
```

**Or, if this is a fresh contract:**

```bash
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --source $SOROBAN_ADMIN_KEY \
  --network public \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --fee-recipient $FEE_RECIPIENT_ADDRESS
```

**Record:**
- [ ] Function called (migrate / initialize)
- [ ] Transaction hash
- [ ] Result (success / error)

#### Step 4: Verification (T+10 minutes)

Run read-only queries to verify the new contract:

```bash
# Check admin
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --network public \
  -- get_admin

# Check initialization state
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --network public \
  -- is_initialized

# Check constants
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --network public \
  -- get_constants

# Verify paused state (should be unpaused for normal operation)
soroban contract invoke \
  --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4 \
  --network public \
  -- is_paused
```

**Verification gate:**
- [ ] Admin is correct
- [ ] Contract is initialized
- [ ] Constants are as expected
- [ ] Contract is not paused (unless intentionally paused for migration)

#### Step 5: Frontend Update (T+15 minutes)

Update the frontend to point to the new contract:

**File:** `frontend/src/config.ts`

```typescript
// Before
export const CONTRACT_ID = 'COLD_CONTRACT_ID_OLD';

// After
export const CONTRACT_ID = 'CNEW_CONTRACT_ID_NEW';
```

**Process:**
1. [ ] Update config.ts
2. [ ] Rebuild frontend: `npm run build`
3. [ ] Test locally: `npm run preview`
4. [ ] Deploy to hosting (Vercel, GitHub Pages, etc.)

**Deployment gate:**
- [ ] Two people have reviewed the config change
- [ ] Frontend build succeeds
- [ ] Old contract ID still works (users are still using it; don't break them immediately)

#### Step 6: Smoke Tests (T+30 minutes)

Run smoke tests against the new contract:

```bash
# Deposit test
soroban contract invoke \
  --id CNEW_CONTRACT_ID \
  --source $TEST_ACCOUNT_KEY \
  --network public \
  -- deposit \
  --depositor $TEST_ACCOUNT \
  --token $TEST_TOKEN \
  --amount 1000000000 \
  --unlock-time $(( $(date +%s) + 3600 )) \
  --penalty-bps 1000

# Query test
soroban contract invoke \
  --id CNEW_CONTRACT_ID \
  --network public \
  -- get_vault \
  --depositor $TEST_ACCOUNT \
  --id 0
```

**Verification gate:**
- [ ] Deposit succeeds
- [ ] Query returns correct data
- [ ] No errors or exceptions

---

## Data Migration Strategy

### Scenario A: No State Transfer Needed

**When:** Bug fix, optimization, or feature that doesn't change storage layout

**Process:**
1. Deploy new contract with new contract ID
2. Users withdraw from old contract (or leave funds if old contract is retired)
3. Users deposit in new contract
4. No data transfer—each user controls their own funds

**Timeline:** Immediate post-deployment

**User action required:** Yes (manual migration)

### Scenario B: Automatic Migration via `migrate()` Function

**When:** New fields added with sensible defaults; layout changes; versioning introduced

**Process:**
1. Deploy new contract with same contract ID (NOT possible—Soroban immutability)
   - **Actually:** Deploy new contract with new ID
2. Admin calls `migrate()` function once to update all stored data
3. Function updates storage version and initializes new fields

**Prerequisites:**
- `migrate()` is idempotent (safe to call multiple times)
- Migration is tested on testnet with real data patterns
- Migration is admin-only

**Timeline:**
- Deploy new contract (T+0)
- Call migrate() (T+5 minutes)
- Verify all queries still work (T+10 minutes)

**User action required:** No (automatic after admin calls migrate)

### Scenario C: Two-Contract Migration Window

**When:** Breaking API change; major architecture shift; users need time to move funds

**Process:**
1. Deploy new contract
2. Keep old contract paused or active for a defined period (e.g., 30 days)
3. Users withdraw from old contract and deposit in new one at their pace
4. Operator reconciles: total locked in old + new should match pre-migration total
5. Retire old contract after migration window closes

**Timeline:**
- Deploy new contract (T+0)
- Migration window open (30 days)
- Retirement deadline (T+30 days)

**User action required:** Yes, within the migration window

### Scenario D: Manual Recovery (Emergency)

**When:** Data corruption; significant bugs; users unable to migrate normally

**Process:**
1. Deploy new, fixed contract
2. Admin iterates over old deposits using `get_deposits_page()`
3. For each affected depositor, call `emergency_withdraw(depositor, deposit_id)` on old contract
4. Manually deposit funds in new contract on behalf of depositor (if needed)
5. Reconcile and verify

**Timeline:** Variable; depends on number of affected depositors

**User action required:** None (admin handles recovery)

---

## Stakeholder Communication Plan

### Pre-Upgrade (48 hours before)

**Channels:** Discord + Email + Status page

**Message template:**
```
📢 **Scheduled Upgrade: SAFE-HAVEN [Date]**

We'll be upgrading SAFE-HAVEN on [Date] at [Time] UTC.

What's changing?
- [Describe the upgrade: feature, fix, or optimization]
- [List any user-visible changes]

What do you need to do?
- [If no action needed]: Nothing—the upgrade is automatic.
- [If action needed]: See migration guide [link]

Expected downtime: ~X minutes

Questions? Ask in #support or reply to this thread.

[More info](link)
```

### During Upgrade (Real-time updates every 10 minutes)

**Status page updates:**

```
🔄 Upgrade in progress
- 14:32 UTC: New contract deployed
- 14:37 UTC: Verification in progress
- 14:42 UTC: Migration complete
- 14:47 UTC: Frontend updated
```

### Post-Upgrade (Immediate)

**Announcement:**

```
✅ **Upgrade Complete!**

SAFE-HAVEN has been successfully upgraded. 

New features: [List]
Bug fixes: [List]

Deposits are resuming. No further action needed.

Feedback? We'd love to hear from you in #general.
```

### Post-Upgrade (24 hours later)

**Incident-free confirmation:**

```
✨ **24 hours later, all systems healthy**

The SAFE-HAVEN upgrade was successful. No issues reported. 

Metrics:
- [Deposits processed]
- [Users upgraded]
- [Funds secured]

Thank you for your patience!
```

### If Issues Arise

**Escalation protocol:**

1. **T+0-15 min:** Issue detected; escalate to incident commander
2. **T+15 min:** Decision to rollback or investigate
3. **T+30 min:** User announcement of incident + ETA for resolution
4. **T+60 min:** Resolution (rollback or fix) + all-clear announcement

---

## Testing Requirements

### Pre-Mainnet Testing Checklist

#### Unit Tests

```bash
make test
```

**Coverage:**
- [ ] All new functions have tests
- [ ] Migration function is tested (idempotence, correctness)
- [ ] Existing functionality unaffected

#### Integration Tests (Local Soroban)

```bash
# Start local Soroban standalone
soroban network add --rpc-url http://localhost:8000 --network-passphrase \
  "Test SDF Network ; September 2015" local

# Deploy old contract; create test data
soroban contract deploy --wasm old_contract.wasm --source deployer --network local

# Create deposits, events, state
soroban contract invoke --id OLD_ID --source test_user --network local \
  -- deposit --amount 1000000000 ...

# Deploy new contract
soroban contract deploy --wasm new_contract.wasm --source deployer --network local

# Call migrate() if applicable
soroban contract invoke --id NEW_ID --source admin --network local \
  -- migrate

# Verify state
soroban contract invoke --id NEW_ID --network local \
  -- get_vault --depositor test_user --id 0
```

**Verification:**
- [ ] Old state migrates correctly
- [ ] All queries still work
- [ ] New functionality works
- [ ] No data loss

#### Testnet Dry Run

**Duration:** 3–5 days before mainnet

**Process:**
1. Deploy to testnet
2. Create representative test data:
   - 10+ deposits across different token types
   - Mix of active, recently unlocked, and expired deposits
   - High and low penalty rates
   - Early exits if testing penalty paths
3. Call migrate() (if applicable)
4. Query state; compare against expectations
5. User acceptance testing (if users available)

**Sign-off:**
- [ ] Release engineer: "Testnet deployment is stable"
- [ ] Data owner: "Data integrity verified"
- [ ] Security lead: "No new security risks"

#### Smoke Tests

**Post-mainnet deployment:**

```bash
# Create a test deposit
soroban contract invoke \
  --id NEW_MAINNET_CONTRACT_ID \
  --source test_account \
  --network public \
  -- deposit \
  --amount 1000000000 \
  --unlock-time $(( $(date +%s) + 7200 )) \
  --penalty-bps 1000

# Verify deposit is recorded
soroban contract invoke \
  --id NEW_MAINNET_CONTRACT_ID \
  --network public \
  -- get_vault --depositor test_account --id 0

# Verify event was emitted (via Stellar Expert or RPC)
# [Manual check: navigate to explorer]
```

---

## Rollback Procedures

### When to Rollback

Rollback is triggered if:
- [ ] Critical bug discovered post-deployment (fund loss, security exploit)
- [ ] New contract fails to initialize or respond
- [ ] Migration corrupts data
- [ ] Users report widespread failures

**Decision:** Incident commander + security lead agree to rollback.

### Rollback Steps

#### Step 1: Pause New Contract (T+0)

```bash
soroban contract invoke \
  --id NEW_CONTRACT_ID \
  --source admin_key \
  --network public \
  -- pause
```

**Rationale:** Stop new deposits while we assess the issue.

#### Step 2: Communicate (T+5 minutes)

```
🚨 **Incident: SAFE-HAVEN Upgrade Issue**

We detected an issue with the new contract and have paused deposits.

Status: [Investigating / Rolling back]
ETA: [X minutes / X hours]

Your deposits are safe. We're working on a fix.

[Support email / Discord link]
```

#### Step 3: Decide: Rollback vs. Fix

**Rollback** (if bug is unfixable without new deployment):
- Revert frontend to old contract ID
- Announce rollback to users
- Plan post-mortem

**Fix** (if issue is in initialization):
- Call `unpause()` on new contract
- Run verification again
- Announce all-clear

#### Step 4: If Rollback Needed

**Revert frontend:**

```typescript
// frontend/src/config.ts
export const CONTRACT_ID = 'OLD_CONTRACT_ID';  // Back to old
```

**Redeploy frontend:**

```bash
cd frontend
npm run build
npm run deploy  # or git push to deploy trigger
```

**Announce:**

```
✅ **Rollback Complete**

We've rolled back to the previous version while we investigate the issue.

What happened: [Brief explanation]
Next steps: [Will retry on [date] after [fix]]

Thank you for your patience.
```

#### Step 5: Post-Mortem (Within 24 hours)

**Document:**
- What went wrong?
- Why didn't testing catch it?
- What changes to prevent recurrence?
- Timeline for retry?

---

## Post-Upgrade Validation

### Day 1 Post-Upgrade

**Checklist:**

- [ ] Contract is live and responding to queries
- [ ] Deposits are processing normally
- [ ] Withdrawals work
- [ ] Early exits work (if applicable)
- [ ] Admin functions work
- [ ] Events are emitting correctly
- [ ] No errors in logs or user reports
- [ ] Gas costs are as expected

**Queries to run:**

```bash
# Query contract info
soroban contract invoke --id NEW_ID --network public -- get_admin
soroban contract invoke --id NEW_ID --network public -- is_paused
soroban contract invoke --id NEW_ID --network public -- get_depositor_count
soroban contract invoke --id NEW_ID --network public -- get_constants

# Check recent events (via Stellar Expert or RPC)
# [Navigate to explorer; search for contract address]
```

### Week 1 Post-Upgrade

**Monitoring:**

- [ ] Average response time from RPC
- [ ] Transaction success rate
- [ ] No spike in error logs
- [ ] User support tickets (any upgrade-related issues?)
- [ ] TVL trend (should be continuous or growing)

**Data validation:**

- [ ] Total deposits in new contract matches expected
- [ ] Total value locked matches expected
- [ ] Old contract: any remaining deposits? (Reconcile if migration window is open)
- [ ] Event history: all events recorded correctly?

**Sample queries:**

```bash
# Count deposits
soroban contract invoke --id NEW_ID --network public -- get_depositor_count

# Sample depositor state
soroban contract invoke --id NEW_ID --network public \
  -- get_deposits_page --offset 0 --limit 10

# Verify staker pool (if applicable)
soroban contract invoke --id NEW_ID --network public -- get_staker_list
```

### Month 1 Post-Upgrade

**Comprehensive validation:**

- [ ] Audit trail complete (all transactions in explorer)
- [ ] No unexplained fund discrepancies
- [ ] User migration complete (if applicable)
- [ ] Security: no new vulnerabilities reported
- [ ] Performance: meets or exceeds pre-upgrade baseline
- [ ] Retire old contract (pause or archive)

**Retirement of old contract:**

```bash
soroban contract invoke \
  --id OLD_CONTRACT_ID \
  --source admin_key \
  --network public \
  -- pause

# Announce
# "The previous SAFE-HAVEN contract (OLD_ID) is now archived. 
#  All active deposits have been migrated to NEW_ID."
```

---

## Lessons Learned

### Post-Upgrade Retrospective (Within 1 week)

**Meeting:** 1 hour, full team + sponsor

**Agenda:**

1. **What went well?** (15 min)
   - Good communication?
   - Testing found issues before mainnet?
   - Smooth deployment process?

2. **What could be better?** (20 min)
   - Gaps in testing?
   - Communication breakdowns?
   - Unexpected risks?
   - Tooling issues?

3. **Process improvements** (15 min)
   - Update this playbook?
   - Add new checklist items?
   - Change deployment timing?

4. **Action items** (10 min)
   - Assign owners
   - Prioritize for next upgrade

**Output:** Update this playbook with lessons learned.

### Documentation Updates

After each upgrade, update:

- **This playbook** with new edge cases or procedures
- **Upgrade decision log** (decision + rationale + outcome)
- **Runbook** for incident response (if issues occurred)
- **Team wiki** (if new patterns emerge)

---

## Appendix A: Upgrade Decision Log Template

**Location:** `/docs/upgrades/YYYYMMDD-upgrade-decision.md`

```markdown
## Upgrade Decision: [Title]

**Date:** YYYY-MM-DD
**Severity:** [Critical / High / Medium / Low]
**Status:** [Planning / Testing / Deployed / Rolled Back]

### What changed?
- [Feature, fix, or optimization]

### Why?
- [User request, security, performance, etc.]

### Decision makers
- [ ] Security lead: [Name] — Approved [date]
- [ ] Release engineer: [Name] — Approved [date]
- [ ] Sponsor: [Name] — Approved [date]

### Outcome
- **Deployment date:** YYYY-MM-DD
- **Result:** Success / Rollback / Partial success
- **Issues encountered:** [If any]
- **Resolution:** [How we fixed it, if needed]

### Lessons learned
- [What we learned from this upgrade]

### Sign-off
- Data owner confirmed data integrity: [Date]
- All systems healthy at T+24 hours: [Date]
```

---

## Appendix B: Pre-Deployment Verification Checklist

**Print or save this checklist; complete before clicking "deploy"**

### 48 Hours Before Deployment

- [ ] Code reviewed by ≥1 CODEOWNER
- [ ] All tests pass: `make test`
- [ ] No Clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Code formatted: `cargo fmt --all`
- [ ] Dependencies audited: `cargo audit` (no vulnerabilities)
- [ ] License compliance: `cargo deny` (no blocked licenses)
- [ ] Testnet deployment successful
- [ ] Migration tested on testnet (if applicable)
- [ ] User communications drafted (Discord, email, status page)

### 24 Hours Before Deployment

- [ ] WASM built and archived: `make build && make optimize`
- [ ] WASM size verified: < 65 KB
- [ ] Checksums recorded: `sha256sum [file]`
- [ ] Deployment commands reviewed by 2 people
- [ ] Network passphrase verified: mainnet or testnet?
- [ ] Admin key is ready and available
- [ ] Rollback plan documented
- [ ] On-call team notified

### 1 Hour Before Deployment

- [ ] Network status normal (check Stellar Expert)
- [ ] Release engineer is ready
- [ ] Communications owner is ready
- [ ] Incident commander is on standby
- [ ] Slack/Discord is monitored by team
- [ ] All browser tabs close except Stellar Expert and RPC endpoint

### Deployment

- [ ] Deployer confirms: "This is [testnet / **mainnet**]"
- [ ] Deployer confirms: "Admin address is [correct address]"
- [ ] Deployer confirms: "WASM checksum is [correct value]"
- [ ] Second person approves: "Approved for deployment"
- [ ] **Execute deployment**
- [ ] Record transaction hash
- [ ] Record new contract ID
- [ ] Post in #deployments: "New contract ID: [ID] | Tx: [hash]"

### Post-Deployment (T+30 min)

- [ ] Smoke tests pass
- [ ] Frontend updated
- [ ] Verification queries run successfully
- [ ] No error logs
- [ ] Post update to status page: "Upgrade complete"

---

## Related Documents

- [DISASTER_RECOVERY.md](../DISASTER_RECOVERY.md) — Incident response playbook
- [MONITORING.md](../MONITORING.md) — Health checks and alerting
- [CONTRIBUTING.md](../CONTRIBUTING.md) — Development guidelines
- [README.md](../README.md) — Technical overview
