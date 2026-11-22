# Staging Deployment Monitoring Dashboard

This document provides a template for monitoring and tracking the health of staged deployments.

## Quick Health Check

Use this checklist after each deployment:

### Immediate (Within 5 minutes)

- [ ] Deployment workflow completed successfully
- [ ] Contract ID generated and recorded
- [ ] All 4 health checks passed
- [ ] No errors in deployment log
- [ ] Deployment took < 15 minutes

### Short-term (Within 1 hour)

- [ ] Integration tests pass
- [ ] Response latency acceptable (< 500ms avg)
- [ ] No RPC errors in logs
- [ ] Artifacts downloaded and verified
- [ ] Team notified of deployment

### Medium-term (Within 24 hours)

- [ ] Zero critical errors in staging
- [ ] Contract handles test transactions
- [ ] Admin functions respond correctly
- [ ] No performance degradation
- [ ] No rollback triggered

### Long-term (7 days)

- [ ] Contract stable and responding
- [ ] No regression issues identified
- [ ] Team ready for production promotion
- [ ] All post-deployment tasks completed

## Deployment Status Tracking

### Current Deployment

Use this table to track the latest deployment:

```
┌─────────────────────────┬──────────────────────────────────────┐
│ Deployment ID           │ [YYYYMMDDTHHMMSSZ]                   │
├─────────────────────────┼──────────────────────────────────────┤
│ Contract ID             │ C...                                 │
│ Status                  │ ✓ Active                             │
│ Network                 │ staging                              │
│ Deployer                │ G... (last 10 chars shown)           │
│ Admin                   │ G...                                 │
│ WASM Size               │ XXX,XXX bytes (64 KB limit)          │
│ WASM SHA256             │ abc123...                            │
│ Deploy Time             │ 12 minutes                           │
│ Deployed At             │ 2024-01-15 14:30:22 UTC              │
│ Git Commit              │ abc123...                            │
│ Git Branch              │ main                                 │
└─────────────────────────┴──────────────────────────────────────┘
```

### Health Check Results

After each deployment, record health check metrics:

```
┌─────────────────┬──────────┬─────────────────┬────────────┐
│ Check           │ Status   │ Value           │ Latency    │
├─────────────────┼──────────┼─────────────────┼────────────┤
│ is_initialized  │ ✓ PASS   │ true            │ 234ms      │
│ get_admin       │ ✓ PASS   │ G...            │ 189ms      │
│ get_constants   │ ✓ PASS   │ {...}           │ 256ms      │
│ latency avg     │ ✓ PASS   │ 226ms           │ < 5000ms   │
└─────────────────┴──────────┴─────────────────┴────────────┘
```

### Deployment History

Track deployments over time:

```
┌────────────────────┬──────────┬────────┬──────────────┬──────────┐
│ Date               │ Contract │ Status │ Deploy Time  │ Issues   │
├────────────────────┼──────────┼────────┼──────────────┼──────────┤
│ 2024-01-15 14:30   │ C...     │ ✓      │ 12 min       │ None     │
│ 2024-01-14 09:15   │ C...     │ ✓      │ 13 min       │ None     │
│ 2024-01-13 16:45   │ C...     │ ✓      │ 14 min       │ Latency  │
│ 2024-01-12 11:00   │ C...     │ ⚠ FAIL │ 8 min        │ Init     │
│ 2024-01-12 10:15   │ C...     │ ✓      │ 12 min       │ Rollback │
└────────────────────┴──────────┴────────┴──────────────┴──────────┘
```

## Performance Metrics

### Response Time SLA

```
┌─────────────────────────┬────────────┬─────────────────┐
│ Metric                  │ Target     │ Current         │
├─────────────────────────┼────────────┼─────────────────┤
│ Average response time   │ < 300ms    │ 226ms ✓         │
│ P95 response time       │ < 1000ms   │ 456ms ✓         │
│ P99 response time       │ < 3000ms   │ 1200ms ✓        │
│ Maximum response time   │ < 5000ms   │ 2345ms ✓        │
├─────────────────────────┼────────────┼─────────────────┤
│ Error rate              │ < 0.1%     │ 0.00% ✓         │
│ Availability            │ > 99.9%    │ 99.99% ✓        │
└─────────────────────────┴────────────┴─────────────────┘
```

### Contract Health

```
Uptime:              99.99% (1 error in 1000 calls)
Last Error:          N/A
Time Since Restart:  5 days, 23 hours
Memory Usage:        Stable
State Size:          Optimal (< 50% limit)
```

## Issue Tracking

### Known Issues

If issues are discovered during staging, document them:

```
Issue #1
├─ Description: Response latency spike observed at 14:30 UTC
├─ Severity: Low (temporary, resolved)
├─ Duration: 2 minutes
├─ Root Cause: RPC endpoint congestion
├─ Resolution: None needed (transient)
└─ Status: Resolved

Issue #2
├─ Description: Health check timeout during initialization
├─ Severity: High
├─ Duration: Caused 1 hour delay
├─ Root Cause: Contract state initialization bug
├─ Resolution: Rollback deployed, fix committed
└─ Status: Fixed in next version
```

### Issue Resolution SLA

```
┌──────────────────┬─────────────────┬────────────┐
│ Severity         │ Detection Goal  │ Resolve SLA│
├──────────────────┼─────────────────┼────────────┤
│ Critical         │ < 1 minute      │ < 5 min    │
│ High             │ < 5 minutes     │ < 30 min   │
│ Medium           │ < 1 hour        │ < 4 hours  │
│ Low              │ < 1 day         │ < 1 week   │
└──────────────────┴─────────────────┴────────────┘
```

## Alerting Rules

Set up alerts for:

### Critical Alerts (Page on-call)

- [ ] Contract not responding to is_initialized
- [ ] Response latency > 10 seconds
- [ ] RPC endpoint down
- [ ] Deployment failed after reaching health checks
- [ ] Error rate > 1%

### Warning Alerts (Email + Slack)

- [ ] Response latency > 5 seconds
- [ ] Any health check failure
- [ ] Deployment takes > 20 minutes
- [ ] Pre-deploy checks fail
- [ ] WASM size approaching limit (> 60 KB)

### Info Alerts (Slack channel)

- [ ] New deployment started
- [ ] Deployment completed successfully
- [ ] Integration tests passed
- [ ] Rollback triggered
- [ ] Weekly summary report

## Accessing Logs and Artifacts

### GitHub Actions Logs

```bash
# Via GitHub CLI - list recent deployments
gh run list --workflow staging-deploy.yml --limit 10

# View specific run details
gh run view <RUN_ID> --log

# Download artifacts
gh run download <RUN_ID> --name staging-deployment-log
gh run download <RUN_ID> --name safe-haven-wasm
```

### Command Line Log Access

```bash
# View latest deployment log
cat deployments/staging/*/deployment.log | tail -50

# View health check results
cat deployments/staging/*/health-check-log.txt

# Find deployment by contract ID
grep -r "CONTRACT_ID=" deployments/staging/ | head -1
```

### Manifest Review

```bash
# View deployment metadata
cat deployments/staging/<YYYYMMDDTHHMMSSZ>/manifest.env

# Verify WASM checksum
sha256sum deployments/staging/<YYYYMMDDTHHMMSSZ>/safe_haven.optimized.wasm
cat deployments/staging/<YYYYMMDDTHHMMSSZ>/wasm.sha256
```

## Integration Test Results

After each deployment, run integration tests and record:

```
┌─────────────────────────┬──────────┬────────────┐
│ Test                    │ Status   │ Duration   │
├─────────────────────────┼──────────┼────────────┤
│ Contract responds       │ ✓ PASS   │ 234ms      │
│ get_admin valid         │ ✓ PASS   │ 189ms      │
│ get_constants returns   │ ✓ PASS   │ 256ms      │
│ Response time OK        │ ✓ PASS   │ 178ms      │
│ No errors in logs       │ ✓ PASS   │ 123ms      │
├─────────────────────────┼──────────┼────────────┤
│ Total                   │ ✓ 5/5    │ 980ms      │
└─────────────────────────┴──────────┴────────────┘
```

## Pre-Production Readiness Checklist

Before promoting from staging to production:

### Code Quality

- [ ] All unit tests passing (100%)
- [ ] Code coverage > 80%
- [ ] Zero clippy warnings
- [ ] Code formatted consistently
- [ ] Security audit passed (no HIGH/CRITICAL)

### Staging Validation

- [ ] Deployed to staging successfully
- [ ] Health checks all passing
- [ ] Integration tests all passing
- [ ] 24+ hours stable operation
- [ ] Zero production-like errors

### Documentation

- [ ] CHANGELOG.md updated with [Unreleased] content
- [ ] API documentation current
- [ ] Deployment process documented
- [ ] Known issues logged
- [ ] Rollback procedure verified

### Operational Readiness

- [ ] On-call team trained
- [ ] Runbooks prepared
- [ ] Monitoring configured
- [ ] Alerting configured
- [ ] Rollback plan ready

### Security Review

- [ ] No secrets in code
- [ ] No hardcoded credentials
- [ ] Access control verified
- [ ] Input validation complete
- [ ] Error messages safe

### Sign-off

- [ ] Technical lead review: _____ Date: _____
- [ ] Product owner sign-off: _____ Date: _____
- [ ] Security review: _____ Date: _____

## Common Monitoring Commands

### Check Latest Deployment Status

```bash
# Get most recent deployment
LATEST=$(ls -t deployments/staging | head -1)

echo "=== Latest Deployment ==="
cat deployments/staging/$LATEST/manifest.env
echo ""
echo "=== Health Checks ==="
cat deployments/staging/$LATEST/health-check-log.txt
echo ""
echo "=== Contract ID ==="
cat deployments/staging/$LATEST/contract-id.txt
```

### Monitor Contract Health

```bash
CONTRACT_ID="C..."
RPC_URL="https://soroban-staging.stellar.org"

# Check if contract is initialized
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source <KEY> \
  --network staging \
  --rpc-url "$RPC_URL" \
  -- is_initialized

# Get contract admin
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source <KEY> \
  --network staging \
  --rpc-url "$RPC_URL" \
  -- get_admin
```

### Track Response Latency

```bash
for i in {1..10}; do
  time stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source <KEY> \
    --network staging \
    --rpc-url "$RPC_URL" \
    -- is_initialized
done
```

## Escalation Path

**If issues are detected:**

1. **Immediate (0-5 min)**
   - Alert on-call engineer
   - Assess severity
   - Start incident log

2. **Short-term (5-30 min)**
   - Gather diagnostics
   - Decide: investigate or rollback
   - Implement resolution

3. **Follow-up (> 30 min)**
   - Document root cause
   - Schedule post-mortem
   - Implement preventative measures

**Contacts:**
- On-call Lead: [name] [phone/slack]
- Tech Lead: [name] [phone/slack]
- DevOps: [name] [phone/slack]

## Related Documentation

- [Staging Deployment Guide](./STAGING_DEPLOYMENT.md)
- [Troubleshooting Guide](./STAGING_DEPLOYMENT.md#troubleshooting)
- [Rollback Procedures](./STAGING_DEPLOYMENT.md#rollback-procedures)

