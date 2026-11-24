# Staging Deployment - Quick Reference Card

## 🚀 One-Minute Quick Start

### First Time Setup
```bash
# 1. Follow setup guide
cat docs/SETUP_STAGING_SECRETS.md

# 2. Add GitHub secrets:
#    STAGING_SOROBAN_SECRET_KEY = secret key from stellar CLI
#    STAGING_ADMIN_ADDRESS = public address

# 3. Test deployment
git commit --allow-empty -m "test: trigger deployment"
git push origin main

# 4. Check GitHub Actions
# https://github.com/<org>/<repo>/actions
```

### Regular Deployment
```bash
# Just push to main - it deploys automatically!
git push origin main

# Monitor: GitHub Actions → Staging Deployment workflow
```

---

## 📊 Deployment Status

### View Recent Deployments
```bash
# List last 5 deployments
gh run list --workflow staging-deploy.yml --limit 5

# View specific deployment
gh run view <RUN_ID> --log

# Download logs
gh run download <RUN_ID> --name staging-deployment-log
```

### Check Contract on Explorer
```
https://explorer.stellar.org?resource=contract&resourceCode=<CONTRACT_ID>&network=staging
```

---

## ✅ Pre-Deployment Checks

These run automatically and must all pass:

| Check | Command | Status |
|-------|---------|--------|
| Formatting | `cargo fmt --all -- --check` | Blocks if fail |
| Linting | `cargo clippy -- -D warnings` | Blocks if fail |
| Tests | `cargo test --features testutils` | Blocks if fail |
| Security | `cargo audit` | Blocks if HIGH/CRITICAL |
| WASM size | Check < 65,536 bytes | Blocks if fail |
| Changelog | [Unreleased] section | Blocks if empty |

**All must pass before deployment proceeds.**

---

## 🏥 Health Checks

After deployment, these checks must pass:

```
✓ is_initialized = true
✓ get_admin = <configured address>
✓ get_constants = <returns data>
✓ Response latency < 5 seconds
```

All 4 must pass for successful deployment.

---

## ⏱️ Deployment Timeline

```
Pre-deploy checks      ~2 min  (gates deployment)
Build & optimize WASM  ~4 min  (compile + optimize)
Deploy to staging      ~2 min  (upload contract)
Initialize contract    ~1 min  (set admin)
Health checks          ~1 min  (4 checks)
Integration tests      ~1 min  (5 tests)
                       ─────────
Total                  ~10-15 min
```

---

## 🔄 Rollback

### Automatic Rollback
Triggered if health checks fail:
- Previous deployment identified
- Previous WASM redeployed
- Health checks rerun
- Exit code: 2

### Manual Rollback
```bash
gh workflow run staging-rollback.yml \
  -f reason='<reason>' \
  -f artifact-timestamp='<timestamp or latest>'
```

Or manually:
```bash
SOROBAN_SECRET_KEY=SB... \
ADMIN_ADDRESS=G... \
bash scripts/deploy.sh rollback staging \
  --artifact-dir deployments/staging/<timestamp>
```

---

## 📁 Artifacts

Each deployment creates:
```
deployments/staging/<YYYYMMDDTHHMMSSZ>/
├─ manifest.env                # Deployment metadata
├─ contract-id.txt             # Contract ID
├─ safe_haven.optimized.wasm   # Deployed binary
├─ deployment.log              # Full log with timestamps
└─ health-check-log.txt        # Health check results
```

**View latest:**
```bash
cat deployments/staging/*/manifest.env | tail -10
```

---

## 🚨 Common Issues & Fixes

### Pre-Deploy Checks Failed

**Formatting:**
```bash
cargo fmt --all
git add -A && git commit -m "chore: format" && git push
```

**Tests failed:**
```bash
cargo test --features testutils
# Fix issues
git add -A && git commit -m "fix: tests" && git push
```

**Security vulnerabilities:**
```bash
cargo audit
cargo update -p <crate>
git add Cargo.lock && git commit -m "security: update" && git push
```

### Deployment Failed

**RPC authentication error:**
- Verify `STAGING_SOROBAN_SECRET_KEY` is set in GitHub Secrets
- Verify account is funded

**WASM too large:**
- Review code for bloat
- Consider splitting contract
- Check dependencies

**Health check timeout:**
- Check RPC endpoint availability
- Try deployment again (may be transient)

---

## 📝 Documentation

| Document | Purpose |
|----------|---------|
| [DEPLOYMENT_AUTOMATION.md](DEPLOYMENT_AUTOMATION.md) | Main guide (start here) |
| [docs/STAGING_DEPLOYMENT.md](docs/STAGING_DEPLOYMENT.md) | Complete reference |
| [docs/SETUP_STAGING_SECRETS.md](docs/SETUP_STAGING_SECRETS.md) | Secret setup guide |
| [docs/MONITORING_DASHBOARD.md](docs/MONITORING_DASHBOARD.md) | Health monitoring |

---

## 🔑 GitHub Secrets Required

```
STAGING_SOROBAN_SECRET_KEY    → Secret key (SB...)
STAGING_ADMIN_ADDRESS          → Public address (G...)
STAGING_RPC_URL (optional)     → RPC endpoint
```

**Set in:** Settings → Secrets and variables → Actions

---

## 📱 Monitor & Alerts

### GitHub Actions Dashboard
```
https://github.com/<org>/<repo>/actions/workflows/staging-deploy.yml
```

### Local Monitoring
```bash
# Watch deployments
watch -n 5 'gh run list --workflow staging-deploy.yml --limit 3'

# View live logs
gh run view <RUN_ID> --log --tail 50

# Check health
cat deployments/staging/*/health-check-log.txt
```

### Contract Health
```bash
# Check if contract is live
stellar contract invoke \
  --id C... \
  --source <KEY> \
  --network staging \
  -- is_initialized
```

---

## 🛠️ Useful Commands

### Deploy Manually
```bash
SOROBAN_SECRET_KEY=SB... \
bash scripts/deploy_staging.sh
```

### Run Integration Tests
```bash
STAGING_CONTRACT_ID=C... \
SOROBAN_SECRET_KEY=SB... \
bash scripts/integration_tests_staging.sh
```

### Check Pre-Deploy
```bash
bash scripts/pre_deploy_check.sh
```

### List All Deployments
```bash
ls -la deployments/staging/
```

### View Deployment Info
```bash
cat deployments/staging/<timestamp>/manifest.env
cat deployments/staging/<timestamp>/deployment.log
cat deployments/staging/<timestamp>/health-check-log.txt
```

---

## ⚡ Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Total time | < 15 min | ✓ |
| Deploy time | < 3 min | ✓ |
| Health check latency | < 500ms | ✓ |
| Success rate | > 95% | ✓ |
| Tests pass rate | 100% | ✓ |

---

## 📞 Emergency Contacts

**Deployment Failed?**
1. Check GitHub Actions log
2. Review `deployment.log` in artifacts
3. See [Troubleshooting](docs/STAGING_DEPLOYMENT.md#troubleshooting)

**Emergency Rollback?**
```bash
gh workflow run staging-rollback.yml -f reason='Emergency'
```

---

## 🎯 Before Going to Production

- [ ] 24+ hours stable operation on staging
- [ ] Zero critical errors
- [ ] Health checks consistently passing
- [ ] Team trained on procedures
- [ ] Rollback tested and working
- [ ] Monitoring configured
- [ ] Alerts set up

---

## 📚 Full Documentation

For complete details, see:
- [DEPLOYMENT_AUTOMATION.md](DEPLOYMENT_AUTOMATION.md) - Main reference
- [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) - Technical overview

