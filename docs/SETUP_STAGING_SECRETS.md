# Setting Up Staging Deployment Secrets

This guide walks through configuring GitHub repository secrets required for automated staging deployments.

## Prerequisites

- GitHub repository admin access
- A funded Stellar account on staging network (from Friendbot or direct funding)
- Stellar CLI (`stellar` command available)

## Step 1: Generate Staging Deployer Identity

If you don't have a staging identity, generate one:

```bash
# Generate a new keypair
stellar keys generate staging-deployer --network staging

# Get the public address
stellar keys address staging-deployer --network staging
# Output: GXXXX...

# Get the secret key (needed for GitHub Secrets)
stellar keys show staging-deployer --network staging
# Output: SBXXXX...
```

## Step 2: Fund the Deployer Account

Fund your deployer account with XLM on the Stellar staging network:

```bash
# Fund through Friendbot (staging only)
curl "https://friendbot-staging.stellar.org?addr=$(stellar keys address staging-deployer --network staging)"

# Or fund with existing account
stellar payment send deployer-account 10 staging-deployer \
  --network staging \
  --rpc-url https://soroban-staging.stellar.org
```

## Step 3: Verify Funding

Check that the account is funded:

```bash
stellar account info staging-deployer --network staging
```

Expected output:
```
Account ID:      GXXXX...
Balance:         100 XLM
Sequence:        1
Flags:
  auth_required: false
...
```

## Step 4: Add GitHub Secrets

Navigate to your repository and add these secrets:

### Repository Settings Path

1. Go to your repository on GitHub
2. Navigate to **Settings**
3. Go to **Secrets and variables** → **Actions**
4. Click **New repository secret**

### Secret 1: STAGING_SOROBAN_SECRET_KEY

**Name:** `STAGING_SOROBAN_SECRET_KEY`

**Value:** The secret key from Step 1

```bash
# Get the secret key
stellar keys show staging-deployer --network staging
# Starts with SB...
```

Paste the full secret key (starting with `SB`) as the value.

### Secret 2: STAGING_ADMIN_ADDRESS

**Name:** `STAGING_ADMIN_ADDRESS`

**Value:** The public address of the contract admin

```bash
# Use the same address as the deployer for simplicity
stellar keys address staging-deployer --network staging
# Starts with G...
```

Or use a different address if you prefer admin and deployer to be separate:

```bash
# If using a different admin account
stellar keys address admin-account --network staging
```

Paste the full public address (starting with `G`) as the value.

### Secret 3: STAGING_RPC_URL (Optional)

**Name:** `STAGING_RPC_URL`

**Value:** The RPC endpoint URL (optional, defaults to https://soroban-staging.stellar.org)

```
https://soroban-staging.stellar.org
```

Or use a custom endpoint if available.

## Step 5: Verify Secrets in GitHub

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. You should see three secrets listed:
   - `STAGING_SOROBAN_SECRET_KEY` (with value masked)
   - `STAGING_ADMIN_ADDRESS` (with value visible)
   - `STAGING_RPC_URL` (with value visible, if added)

## Step 6: Test the Configuration

Trigger a test deployment:

```bash
# Option 1: Push to main branch
git commit --allow-empty -m "test: trigger staging deployment"
git push origin main

# Option 2: Manually trigger via GitHub CLI
gh workflow run staging-deploy.yml
```

Monitor the deployment in GitHub Actions:

1. Go to your repository
2. Click **Actions**
3. Select the **Staging Deployment** workflow
4. Watch the job execution

Expected successful output:
```
✓ Pre-Deployment Validation
✓ Build & Optimize WASM
✓ Deploy to Staging
✓ Integration Tests
✓ Rollback Ready
```

## Step 7: Verify Deployed Contract

Once deployment succeeds, verify the contract on staging:

```bash
# Get the contract ID from the deployment summary
CONTRACT_ID="C..."

# Check contract state
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source staging-deployer \
  --network staging \
  --rpc-url https://soroban-staging.stellar.org \
  -- is_initialized

# Expected output: true
```

## Troubleshooting

### Secret not found error

**Error:** `Error: secret STAGING_SOROBAN_SECRET_KEY not found`

**Resolution:**
1. Check secret is added in Settings → Secrets and variables → Actions
2. Verify exact secret name matches (case-sensitive)
3. Ensure you're in the correct repository

### Authentication failed during deployment

**Error:** `Error: RPC error: authentication failed`

**Resolution:**
1. Verify `STAGING_SOROBAN_SECRET_KEY` value is correct
2. Check that secret account is funded (has XLM balance)
3. Regenerate if needed:
   ```bash
   stellar keys generate staging-deployer --network staging --force
   stellar keys fund staging-deployer --network staging
   # Update STAGING_SOROBAN_SECRET_KEY secret in GitHub
   ```

### Deployment times out

**Error:** `Timeout waiting for health checks`

**Resolution:**
1. Check RPC endpoint status (may be slow)
2. Try deployment again (may be temporary network issue)
3. Use different RPC endpoint if available

### Contract deployment fails

**Error:** `Contract deployment failed`

**Resolution:**
1. Check deployer account balance (needs XLM)
2. Verify WASM artifact size < 64 KB
3. Check RPC endpoint connectivity
4. Review full deployment log in GitHub Actions

## Monitoring Deployments

### GitHub Actions Dashboard

Access deployment history:
1. Repository → **Actions**
2. Select **Staging Deployment** workflow
3. Review run history with dates and status

### Deployment Artifacts

Download artifacts from completed runs:
1. Go to successful workflow run
2. Scroll to **Artifacts** section
3. Download:
   - `safe-haven-wasm` - The deployed contract binary
   - `staging-deployment-log` - Full deployment log
   - `staging-rollback-log` (if rollback occurred)

### Contract Explorer

View deployed contract on Stellar network:

```
https://explorer.stellar.org?resource=contract&resourceCode=<CONTRACT_ID>&network=staging
```

## Rotating Secrets

Rotate deployer credentials regularly (recommended: every 90 days):

### Generate new identity

```bash
stellar keys generate staging-deployer-v2 --network staging
stellar keys fund staging-deployer-v2 --network staging --rpc-url https://soroban-staging.stellar.org
```

### Update GitHub Secret

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **STAGING_SOROBAN_SECRET_KEY**
3. Click **Update**
4. Paste new secret key
5. Click **Update secret**

### Verify new secret works

```bash
git commit --allow-empty -m "test: verify rotated secret"
git push origin main
# Monitor GitHub Actions for success
```

## Related Documentation

- [Staging Deployment Guide](./STAGING_DEPLOYMENT.md) - Complete deployment documentation
- [Stellar Testnet Faucet](https://developers.stellar.org/tools/testnet-deploy-fund) - Getting testnet XLM
- [Soroban CLI Documentation](https://developers.stellar.org/learn/fundamentals#soroban-cli)

