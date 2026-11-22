# Staging Deployment Automation

This document describes the automated staging deployment pipeline for SAFE-HAVEN, including CI/CD workflows, validation checks, health monitoring, and rollback procedures.

## Overview

The staging deployment system provides:

- **Automatic deployments** on main branch merge
- **Pre-deployment validation** (formatting, linting, tests, security audit)
- **Optimized build process** with WASM size verification
- **Comprehensive health checks** post-deployment
- **Automatic rollback** on health check failure
- **Complete audit trail** with deployment logs and artifacts

## Architecture

```
┌─────────────────┐
│  Main branch    │
│  merge/push     │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  staging-deploy.yml GitHub Action  │
├─────────────────────────────────────┤
│ ✓ Pre-deploy validation             │
│ ✓ Build & optimize WASM             │
│ ✓ Deploy to staging                 │
│ ✓ Integration tests                 │
│ ✓ Rollback ready                    │
└────────┬────────────────────────────┘
         │
    ┌────┴─────────┐
    │ Success      │ Failure
    ▼              ▼
┌─────────┐    ┌──────────────────┐
│ Live    │    │ Trigger manual   │
│         │    │ rollback workflow│
└─────────┘    └──────────────────┘
```

## Workflows

### 1. Staging Deployment Workflow (`staging-deploy.yml`)

Triggered automatically when commits are pushed to `main` branch.

**Jobs:**
- `pre-deploy-validation`: Runs linting, tests, security audit
- `build`: Compiles WASM to wasm32-unknown-unknown and optimizes
- `deploy-staging`: Deploys contract to staging network
- `integration-tests`: Runs post-deployment validation
- `rollback-ready`: Publishes rollback instructions

**Duration:** ~10-15 minutes total

**Success Indicators:**
- All pre-deploy checks pass
- WASM size < 64 KB
- Contract deploys successfully
- Contract initializes with correct admin
- Health checks pass within 5s response time

### 2. Staging Rollback Workflow (`staging-rollback.yml`)

Manual workflow for emergency rollback after staging deployment.

**Trigger:**
```bash
gh workflow run staging-rollback.yml \
  -f reason='<reason>' \
  -f artifact-timestamp='<timestamp or latest>'
```

**Actions:**
- Validates previous deployment artifact
- Redeployes previous WASM
- Publishes rollback audit log

## CI/CD Pipeline Details

### Pre-Deployment Validation

Located in `scripts/pre_deploy_check.sh`, runs:

1. **Formatting** (`cargo fmt --check`)
   - Ensures consistent code style
   - Blocks if unformatted files detected

2. **Linting** (`cargo clippy -D warnings`)
   - Checks for code issues and anti-patterns
   - Treats warnings as errors

3. **Unit Tests** (`cargo test`)
   - All Rust unit and doc tests must pass
   - Required before any deployment

4. **Security Audit** (`cargo audit`)
   - Scans dependencies for known vulnerabilities
   - Blocks on HIGH/CRITICAL advisories
   - Warns on low/medium severity

5. **WASM Size Check** (65,536 bytes limit)
   - Optimized WASM must fit in 64 KB
   - Checked after `stellar contract optimize`

6. **CHANGELOG Verification**
   - `[Unreleased]` section must have content
   - Documents changes for release

### Build & Optimization

```bash
# Raw build
cargo build --target wasm32-unknown-unknown --release

# Optimize with stellar-cli
stellar contract optimize \
  --wasm target/wasm32-unknown-unknown/release/safe_haven.wasm \
  --wasm-out target/safe_haven.optimized.wasm
```

**Verification:**
- Raw WASM: checked
- Optimized WASM: < 65,536 bytes
- SHA256: recorded for audit

### Deployment Steps

1. **Deploy contract** to staging network
   - Uploads optimized WASM
   - Registers contract
   - Returns contract ID

2. **Initialize contract**
   - Sets admin address
   - Initializes default state
   - Applies configured parameters

3. **Health checks** (must pass within 60 seconds)
   - `is_initialized` → must be `true`
   - `get_admin` → must match configured admin
   - `get_constants` → must return valid data
   - Response latency → must be < 5 seconds

4. **Post-deployment**
   - Write artifacts to `deployments/staging/<timestamp>/`
   - Generate deployment manifest
   - Create health check log

## Configuration

### GitHub Secrets (Required)

Set these in your repository settings under **Settings → Secrets and variables → Actions**:

```env
# Staging network credentials
STAGING_SOROBAN_SECRET_KEY=SBXXXX...     # Funded deployer key
STAGING_ADMIN_ADDRESS=GXXXX...           # Contract admin address
STAGING_RPC_URL=https://soroban-staging.stellar.org  # RPC endpoint
```

### Environment Variables (Optional)

Set in workflow or `scripts/deploy_staging.sh`:

```env
HEALTH_CHECK_TIMEOUT=60         # Seconds to wait for health checks
ROLLBACK_ON_FAILURE=true        # Auto-rollback if health checks fail
KEEP_PREV_ARTIFACT=true         # Retain previous deployment artifacts
DEPLOYMENTS_DIR=./deployments   # Artifact storage location
```

## Deployment Artifacts

Each deployment creates a timestamped directory at:
```
deployments/staging/<YYYYMMDDTHHMMSSZ>/
```

**Contents:**

```
manifest.env                      # Deployment metadata
├─ DEPLOYMENT_ID=20240115T143022Z
├─ NETWORK=staging
├─ CONTRACT_ID=C...
├─ DEPLOYER_ADDRESS=G...
├─ ADMIN_ADDRESS=G...
├─ WASM_SHA256=abc123...
├─ GIT_SHA=abc123...
└─ GIT_BRANCH=main

safe_haven.optimized.wasm        # Deployed WASM binary (64 KB limit)
safe_haven.wasm                  # Raw WASM (for reference)
wasm.sha256                       # SHA256 checksum
contract-id.txt                  # Contract ID (for easy reference)
deployment.log                   # Full deployment log with timestamps
health-check-log.txt             # Post-deployment health check results
```

### Using Artifacts

**Verify deployment:**
```bash
cd deployments/staging/20240115T143022Z
sha256sum safe_haven.optimized.wasm
# Compare output with wasm.sha256
```

**Manual rollback:**
```bash
bash scripts/deploy.sh rollback staging \
  --artifact-dir deployments/staging/20240115T143022Z
```

## Health Checks

The deployment performs automatic post-deployment validation:

### Check 1: is_initialized
- **Test:** Contract has been initialized
- **Expected:** `true`
- **Timeout:** 10 seconds
- **Latency:** Recorded

### Check 2: get_admin
- **Test:** Admin address matches configuration
- **Expected:** Matches `STAGING_ADMIN_ADDRESS`
- **Timeout:** 10 seconds
- **Latency:** Recorded

### Check 3: get_constants
- **Test:** Configuration has been stored
- **Expected:** Non-empty response
- **Timeout:** 10 seconds
- **Latency:** Recorded

### Check 4: Response Latency
- **Test:** Average response time acceptable
- **Expected:** < 5 seconds (avg of 3 calls)
- **Threshold:** 5000ms (warning at higher values)

### Health Check Output

Results are logged to `health-check-log.txt`:

```
=== Health Check Log ===
Timestamp: 2024-01-15T14:30:22Z
Contract: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4

Check 1: is_initialized
PASSED: is_initialized=true (234ms)

Check 2: get_admin
PASSED: get_admin matches (G...)

Check 3: get_constants
PASSED: get_constants returned data

Check 4: response latency
PASSED: average latency 256ms (threshold: 5000ms)

Health Check Summary: 4 passed, 0 failed
```

## Integration Tests

Post-deployment integration tests (`scripts/integration_tests_staging.sh`) verify:

1. **Contract responds** to contract invocations
2. **get_admin** returns valid Stellar address format
3. **get_constants** returns configuration
4. **Response time** is acceptable (< 5s)
5. **No errors** in system logs (placeholder)

**Run manually:**
```bash
STAGING_CONTRACT_ID=C... \
SOROBAN_SECRET_KEY=SB... \
bash scripts/integration_tests_staging.sh
```

**Output:**
```
✓ Contract responds (is_initialized = true)
✓ get_admin returned valid address
✓ get_constants returned data
✓ Response time acceptable: 234ms
ℹ Placeholder: no log analysis configured (check manually)

═══════════════════════════════════════════════════
  Test Summary
═══════════════════════════════════════════════════

Total:  5
Passed: 5
```

## Rollback Procedures

### Automatic Rollback

Triggered when post-deployment health checks fail:

1. Health check fails (e.g., contract not initialized)
2. Automatic rollback triggered if `ROLLBACK_ON_FAILURE=true`
3. Previous successful deployment identified
4. Previous WASM redeployed
5. New health checks run on rolled-back version
6. Audit log created

**Status:** Exit code `2` = rollback in progress

### Manual Rollback

For deliberate rollback after staging deployment:

```bash
# Via GitHub CLI
gh workflow run staging-rollback.yml \
  -f reason='Critical bug found in deployment' \
  -f artifact-timestamp='20240115T143022Z'

# Or manually with scripts
cd /workspaces/SAFE-HAVEN
SOROBAN_SECRET_KEY=SB... \
ADMIN_ADDRESS=G... \
bash scripts/deploy.sh rollback staging \
  --artifact-dir deployments/staging/20240115T143022Z
```

**Rollback Checklist:**
- [ ] Identified root cause
- [ ] Confirmed previous artifact exists and is healthy
- [ ] Notified team of rollback
- [ ] Executed rollback
- [ ] Verified health checks pass on rolled-back version
- [ ] Documented incident
- [ ] Scheduled post-mortem

### Rollback Audit

Each rollback creates:

```
deployments/staging/rollback-<YYYYMMDDTHHMMSSZ>/
├─ rollback.log                  # Rollback execution details
├─ manifest.env                  # Previous deployment metadata
├─ safe_haven.optimized.wasm
└─ contract-id.txt
```

## Monitoring & Alerts

### Deployment Status Dashboard

GitHub Actions provides visibility into deployments:
- **Actions tab** → `staging-deploy` workflow
- **Job logs** for each stage
- **Artifact downloads** for logs and WASM

### Key Metrics to Monitor

1. **Deployment Time**
   - Target: < 15 minutes
   - Includes build, test, deploy, health checks

2. **Health Check Latency**
   - Target: < 500ms average
   - Alert if > 5 seconds

3. **Pre-Deploy Check Pass Rate**
   - Target: 100%
   - Investigate failures immediately

4. **Rollback Frequency**
   - Target: < 1 per week
   - Investigate root causes

### Notifications

GitHub Actions can send notifications via:
- Email (default)
- Slack integration
- Custom webhooks

**Configure in:** Repository Settings → Actions → Notifications

## Troubleshooting

### Pre-Deploy Checks Failed

**Issue:** Formatting check failed

**Resolution:**
```bash
cargo fmt --all
git add -A
git commit -m "chore: format code"
git push
```

**Issue:** Tests failed

**Resolution:**
```bash
cargo test --features testutils
# Fix failing tests
git add -A
git commit -m "fix: failing tests"
git push
```

**Issue:** Security audit found HIGH/CRITICAL vulnerability

**Resolution:**
```bash
cargo audit
# Update vulnerable dependency
cargo update -p <crate>
cargo test
git add Cargo.lock
git commit -m "security: update vulnerable dependency"
git push
```

### Deployment Failed

**Issue:** Contract deploy returned empty contract ID

**Resolution:**
- Check network connectivity to RPC endpoint
- Verify `STAGING_SOROBAN_SECRET_KEY` is funded
- Check RPC endpoint availability
- Review full deployment log in GitHub Actions

**Issue:** Contract initialization failed

**Resolution:**
- Verify `STAGING_ADMIN_ADDRESS` is valid
- Check contract storage capacity
- Review RPC logs for error details

### Health Checks Failed

**Issue:** is_initialized returned false

**Resolution:**
- Check that initialization step completed successfully
- Verify contract state on blockchain
- Check for initialization errors in deployment log

**Issue:** Response latency > 5 seconds

**Resolution:**
- Check RPC endpoint health
- Check network latency
- Consider RPC rate limiting
- Verify staging network congestion

### Rollback Issues

**Issue:** Previous artifact not found

**Resolution:**
```bash
# List available deployments
ls -la deployments/staging/

# Use manual deployment if needed
SOROBAN_SECRET_KEY=... \
ADMIN_ADDRESS=... \
bash scripts/deploy.sh deploy staging
```

**Issue:** Rolled-back contract still failing

**Resolution:**
- Contact Stellar team if infrastructure issue
- Deploy new fixed version to staging
- Skip staging and deploy directly (not recommended)

## Best Practices

### Before Each Deployment

1. **Review changes**
   ```bash
   git log --oneline main^..main
   ```

2. **Check test coverage**
   ```bash
   cargo tarpaulin --out Html
   ```

3. **Update CHANGELOG**
   ```
   ## [Unreleased]
   ### Added
   - New feature X
   
   ### Fixed
   - Bug Y
   
   ### Changed
   - Behavior Z
   ```

4. **Run pre-deploy checks locally**
   ```bash
   bash scripts/pre_deploy_check.sh
   ```

### During Deployment

1. **Monitor GitHub Actions** workflow in real-time
2. **Check health check logs** once deployment completes
3. **Run integration tests** manually
4. **Monitor staging contract** for 24 hours

### After Deployment

1. **Document contract ID** for reference
2. **Announce deployment** to team
3. **Monitor error rates** and performance
4. **Plan maintenance** if needed
5. **Prepare for rollback** if issues found

### Preventing Issues

1. **Automated testing** catches regressions early
2. **Code review** before merging to main
3. **Staging deployment** validates in realistic environment
4. **Health checks** catch initialization failures
5. **Rollback readiness** enables quick recovery

## Security Considerations

### Secret Management

- `STAGING_SOROBAN_SECRET_KEY`: Keep secret, rotate regularly
- Store in GitHub Secrets (not in code or logs)
- Use separate key from production
- Rotate after team member departure

### Audit Trail

- All deployments logged in `deployments/staging/`
- GitHub Actions audit log records workflow executions
- Deployment logs include timestamps, git SHAs, admins
- Rollback history preserved separately

### Access Control

- GitHub repository settings restrict deployment permissions
- Staging environment limited in scope/blast radius
- Rollback requires manual trigger (not automatic)
- Can be protected by branch rules and required reviews

## Related Documentation

- [Deployment Scripts](../scripts/deploy.sh) - Core deployment logic
- [Stellar CLI Documentation](https://stellar.org/developers/reference/softwares/stellar-cli) - Tool reference
- [Soroban Smart Contract Development](https://developers.stellar.org/learn/fundamentals) - Contract basics
- [Production Deployment](./PRODUCTION_DEPLOYMENT.md) - Mainnet procedures

