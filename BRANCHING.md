# Branching and Release Strategy

This document defines the branching model, naming conventions, branch protection rules,
release cadence, release checklist, and rollback procedures for SAFE-HAVEN.

---

## Table of Contents

1. [Branching Strategy](#1-branching-strategy)
2. [Branch Types and Naming Conventions](#2-branch-types-and-naming-conventions)
3. [Branch Protection Rules](#3-branch-protection-rules)
4. [Release Branching and Cadence](#4-release-branching-and-cadence)
5. [Pre-Release Testing](#5-pre-release-testing)
6. [Release Checklist](#6-release-checklist)
7. [Hotfix Procedure](#7-hotfix-procedure)
8. [Rollback Procedures](#8-rollback-procedures)
9. [Tagging Convention](#9-tagging-convention)

---

## 1. Branching Strategy

SAFE-HAVEN uses a **simplified Git Flow** model adapted for a project where the
primary artifact is an immutable blockchain contract. The key constraint that shapes
the strategy is: **there is no in-place upgrade**. Every release that changes the
contract is a new on-chain deployment.

### Branch Hierarchy

```
main            ← production-ready code; always matches the latest stable release
│
├── develop     ← integration branch; all feature work merges here first
│   ├── feat/...
│   ├── fix/...
│   └── chore/...
│
├── release/x.y ← release preparation; created from develop when x.y enters freeze
│
└── hotfix/...  ← emergency patches on main; merged back into main AND develop
```

### Rules Summary

| Branch | Purpose | Merge target | Direct commits |
|---|---|---|---|
| `main` | Latest stable release | — | Never; PRs only |
| `develop` | Ongoing integration | — | Never; PRs only |
| `feat/*` | New features | `develop` | Yes (by branch author) |
| `fix/*` | Bug fixes | `develop` (or `release/*`) | Yes (by branch author) |
| `chore/*` | Maintenance (deps, docs, CI) | `develop` | Yes (by branch author) |
| `release/*` | Release stabilisation | `main` + back to `develop` | Fixes only; no new features |
| `hotfix/*` | Emergency production patch | `main` + back to `develop` | Yes (by patch author) |

---

## 2. Branch Types and Naming Conventions

All branch names must use lowercase `kebab-case`. No spaces, no special characters
other than `/` and `-`.

### Feature Branches

Format: `feat/<short-description>` or `feat/<issue-number>-<short-description>`

```
feat/staker-registry
feat/330-multi-token-deposit
feat/ledger-based-deposits
```

- Created from `develop`.
- Merged back to `develop` via a reviewed pull request.
- Deleted after merge.
- Maximum lifespan: 30 days. Long-running features should use a feature flag or be
  broken into smaller increments.

### Bug Fix Branches

Format: `fix/<short-description>` or `fix/<issue-number>-<short-description>`

```
fix/depositor-pagination-overflow
fix/140-offset-limit-overflow
fix/accept-admin-after-cancel
```

- Created from `develop` for non-release bugs.
- Created from the relevant `release/*` branch for release-cycle fixes.
- Merged to the branch they were created from via reviewed PR.
- Deleted after merge.

### Chore Branches

Format: `chore/<short-description>`

```
chore/bump-soroban-sdk-v23
chore/add-deny-toml
chore/update-ci-checkout-v4
chore/docs-legal
```

- Created from `develop`.
- Used for dependency updates, CI configuration, documentation, and tooling changes.
- Merged to `develop` via reviewed PR.
- Deleted after merge.

### Release Branches

Format: `release/<major>.<minor>`

```
release/0.1
release/0.2
release/1.0
```

- Created from `develop` when the release enters feature-freeze.
- Only bug fixes and release-preparation commits are allowed on this branch.
  No new features.
- Merged into `main` (creating the release tag) AND back into `develop`
  (to carry forward any release fixes).
- Not deleted — retained as the long-lived maintenance branch for that minor version.

### Hotfix Branches

Format: `hotfix/<short-description>` or `hotfix/<issue-number>-<short-description>`

```
hotfix/critical-penalty-calc
hotfix/283-auth-bypass
```

- Created from `main` (the current production tag), **not** from `develop`.
- Contains only the minimal fix. No unrelated changes.
- Merged into `main` via reviewed PR. Tagged as a patch release.
- Also merged back into `develop` (and the current `release/*` branch if one is open).
- Deleted after merge.

---

## 3. Branch Protection Rules

Apply the following rules to the GitHub repository under **Settings → Branches**.

### `main`

| Rule | Setting |
|---|---|
| Require pull request before merging | ✅ Enabled |
| Required approvals | 1 (raise to 2 for mainnet-affecting changes) |
| Dismiss stale reviews when new commits are pushed | ✅ Enabled |
| Require review from code owners (`CODEOWNERS`) | ✅ Enabled |
| Require status checks to pass before merging | ✅ Enabled |
| Required status checks | `ci / lint`, `ci / test`, `ci / build`, `ci / check-wasm-size` |
| Require branches to be up to date before merging | ✅ Enabled |
| Restrict who can push to matching branches | Maintainers and admins only |
| Allow force pushes | ❌ Disabled |
| Allow deletions | ❌ Disabled |

### `develop`

| Rule | Setting |
|---|---|
| Require pull request before merging | ✅ Enabled |
| Required approvals | 1 |
| Require status checks to pass before merging | ✅ Enabled |
| Required status checks | `ci / lint`, `ci / test`, `ci / build` |
| Allow force pushes | ❌ Disabled |
| Allow deletions | ❌ Disabled |

### `release/*`

Same rules as `main`. Release branches are treated as production-quality branches.

---

## 4. Release Branching and Cadence

### Release Cadence

| Release Type | Cadence | Trigger |
|---|---|---|
| **Minor release** (x.Y.0) | Quarterly (approximately) | Feature milestone complete |
| **Patch release** (x.y.Z) | As needed | Critical or High severity bug fix |
| **Major release** (X.0.0) | As needed; rare | Breaking API or storage layout changes |

Exact release dates are announced in the GitHub repository at least 4 weeks in advance
for minor releases. Patch releases may be unscheduled when driven by Critical severity
issues.

### Release Preparation Timeline

```
T - 4 weeks   Feature freeze: no new features merge to develop
T - 4 weeks   Create release/x.y branch from develop
T - 4 weeks   Begin release testing (see Section 5)
T - 2 weeks   Release candidate RC1 tagged on release/x.y
T - 1 week    RC1 deployed to testnet for integration validation
T - 3 days    Final go/no-go review
T             Release: merge release/x.y → main, tag vX.Y.0, deploy
```

### Semantic Versioning

SAFE-HAVEN follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR**: Breaking changes to the public contract ABI, storage layout, or
  frontend API that require user action or coordinator migration.
- **MINOR**: New backward-compatible functionality. New contract entry points that
  do not change existing ones. Frontend features.
- **PATCH**: Backward-compatible bug fixes. Security fixes. Documentation corrections.

The contract's `version()` function returns the semver string from `Cargo.toml` and
must match the Git tag for every release.

---

## 5. Pre-Release Testing

All of the following must pass before `release/*` is merged to `main`:

### Automated Checks (CI)

```bash
make check          # fmt + lint + test + audit + deny
make build          # WASM compilation succeeds
make check-wasm-size  # WASM ≤ 64 KB
make smoke-test-local  # end-to-end smoke test on local Stellar node
```

The full CI suite (`ci.yml`) must be green on the release branch with no failing or
skipped required checks.

### Manual Testnet Validation

1. Deploy the release candidate to Stellar testnet:
   ```bash
   export SOROBAN_SECRET_KEY=S...
   make deploy-testnet
   ```
2. Run the full smoke test against testnet:
   ```bash
   bash scripts/smoke_test_local.sh  # adapted for testnet endpoint
   ```
3. Verify the `version()` query returns the expected semver string.
4. Verify the frontend version compatibility banner shows no warning when
   pointed at the testnet contract.
5. Test all major user paths manually:
   - Deposit (timestamp-based)
   - Deposit (ledger-based)
   - Withdraw after lock expiry
   - Cancel deposit with penalty
   - Register staker / claim rewards
   - Admin pause / unpause
   - Admin transfer (two-step)

### Security Review Checklist

- [ ] No new `unsafe` Rust code introduced.
- [ ] `cargo audit` reports zero vulnerabilities.
- [ ] `cargo deny check` passes (license and ban policy).
- [ ] All new public entry points call `require_auth()` as the first statement.
- [ ] No new storage keys bypass TTL bumping.
- [ ] `STORAGE_VERSION` bumped if any `contracttype` struct changed.
- [ ] `CHANGELOG.md` fully updated with all changes.

### Frontend Checks

```bash
cd frontend
npm run build     # TypeScript build clean
npm run test      # Vitest unit tests
npm run lint      # ESLint clean
npx playwright test  # E2E smoke tests
```

---

## 6. Release Checklist

Use this checklist for every release. Create a GitHub issue or PR description
from this template.

### Pre-Merge

- [ ] `CHANGELOG.md` updated with all changes under the new version header.
- [ ] Version in `contracts/safe-haven/Cargo.toml` matches the target release tag.
- [ ] `STORAGE_VERSION` bumped in `types.rs` if storage layout changed.
- [ ] `COMPATIBILITY_MATRIX` in `frontend/src/lib/versionCompat.ts` updated.
- [ ] `docs/VERSION_COMPATIBILITY.md` updated.
- [ ] `SUPPORT.md` support matrix updated with new version row.
- [ ] All CI checks green on the release branch.
- [ ] WASM size check passes (≤ 64 KB).
- [ ] Testnet smoke tests passed (record testnet contract ID: \_\_\_\_\_\_\_\_\_).
- [ ] Security review checklist completed (see Section 5).
- [ ] At least one reviewer approved the release PR.
- [ ] Release PR title follows format: `release: vX.Y.Z`.

### Merge and Tag

- [ ] Merge `release/x.y` into `main` (squash or merge commit — maintainer decision).
- [ ] Tag the merge commit: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`.
- [ ] Push tag: `git push origin vX.Y.Z`.
- [ ] Merge `release/x.y` back into `develop` to carry forward any release fixes.
- [ ] Verify CI runs successfully on the tagged commit.

### Deployment

- [ ] Deploy to mainnet:
  ```bash
  export SOROBAN_SECRET_KEY=S...
  make deploy-mainnet
  ```
- [ ] Record new contract ID: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
- [ ] Record deployment artifact path: `deployments/mainnet/<timestamp>/`
- [ ] Verify `version()` query on mainnet returns `"X.Y.Z"`.
- [ ] Verify `is_initialized()` returns `true`.
- [ ] Update frontend `.env` with new `VITE_CONTRACT_ID`.
- [ ] Deploy frontend to production.
- [ ] Verify frontend version compatibility banner shows `severity: 'ok'`.
- [ ] Run post-deployment smoke test:
  ```bash
  bash scripts/smoke_test_local.sh  # pointed at mainnet endpoint
  ```

### Post-Deployment

- [ ] Create GitHub Release: attach optimized WASM, manifest, and checksum.
- [ ] Update `SUPPORT.md` with the active support status of the new version.
- [ ] Announce the release on relevant channels (GitHub Discussions, status page, etc.).
- [ ] If a previous version is now entering Maintenance or EOL, publish an EOL notice
  per the policy in `SUPPORT.md`.
- [ ] Archive the release artifacts to the backup location per `BACKUP.md`.
- [ ] Close all GitHub issues resolved in this release.

---

## 7. Hotfix Procedure

Use this procedure for Critical or High severity bugs discovered in production.

### Steps

1. **Confirm severity.** A hotfix bypasses normal release cadence and must be justified
   by a Critical or High severity classification. Do not use hotfixes for Medium or
   Low issues; schedule them in the next regular release.

2. **Create the hotfix branch** from the current production tag (not from `develop`):
   ```bash
   git checkout -b hotfix/<description> vX.Y.Z
   ```

3. **Apply the minimal fix.** Hotfix branches must contain only the targeted change.
   Do not add unrelated improvements, refactors, or new features.

4. **Test locally:**
   ```bash
   make check
   make build
   make check-wasm-size
   make smoke-test-local
   ```

5. **Open a PR** from `hotfix/<description>` targeting `main`. Request at least one
   maintainer review. Include:
   - A clear description of the vulnerability or bug.
   - The root cause (brief).
   - Steps to reproduce.
   - Confirmation that the fix resolves the issue without regressions.

6. **Deploy to testnet** and verify. Do not skip testnet validation even for urgent fixes.

7. **Merge the PR to `main`.** Tag immediately:
   ```bash
   git tag -a vX.Y.(Z+1) -m "Hotfix: <description>"
   git push origin vX.Y.(Z+1)
   ```

8. **Deploy to mainnet** per the deployment steps in Section 6.

9. **Merge the hotfix back to `develop`** (and to the current `release/*` branch if one
   is open):
   ```bash
   git checkout develop
   git merge --no-ff hotfix/<description>
   ```

10. **Publish a security advisory** if the fix addresses a security vulnerability.
    Use GitHub Security Advisories. Coordinate disclosure per `SECURITY.md`.

11. **Delete the hotfix branch** after all merges are confirmed.

### Hotfix SLA Targets

| Severity | Merge to `main` | Mainnet deployment | User notification |
|---|---|---|---|
| Critical | Within 24 hours of fix validation | Within 4 hours of merge | Within 1 hour of deployment |
| High | Within 72 hours of fix validation | Within 24 hours of merge | Within 24 hours of deployment |

---

## 8. Rollback Procedures

Because SAFE-HAVEN contracts are **immutable** on the Stellar blockchain, "rollback"
means deploying the previous WASM as a new contract and switching the frontend to
point at it. The Makefile provides a dedicated target for this.

### Contract Rollback

1. Locate the deployment artifact directory for the version to restore:
   ```
   deployments/mainnet/<timestamp>/
   ```
   The directory contains the raw WASM, optimized WASM, manifest, and checksum.

2. Run the rollback target:
   ```bash
   make rollback NETWORK=mainnet ARTIFACT_DIR=deployments/mainnet/<timestamp>
   ```
   This deploys the retained WASM as a new contract ID. It does **not** destroy or
   alter the previous (buggy) contract — that remains on-chain permanently.

3. Update the frontend `.env` with the new contract ID and redeploy.

4. Verify the rolled-back deployment:
   ```bash
   stellar contract invoke --id <NEW_CONTRACT_ID> --source <KEY> --network mainnet \
     -- version
   # Should return the previous version string
   stellar contract invoke --id <NEW_CONTRACT_ID> --source <KEY> --network mainnet \
     -- is_initialized
   # Should return true after re-initialization
   ```

5. Communicate the rollback and new contract address to users.

### Code Rollback (Git)

If a bug is introduced in `develop` or a `release/*` branch but has not yet been
deployed to mainnet, revert the offending commit(s):

```bash
git revert <commit-hash>   # creates a new revert commit — safe, non-destructive
```

Do not use `git reset --hard` or `git push --force` on protected branches. Revert
commits preserve history and are clearly visible in the log.

### Frontend Rollback

The frontend is a stateless React SPA. Rolling back means redeploying the previous
build artifact:

1. Identify the previous build (use CI artifacts or your deployment pipeline history).
2. Redeploy that build to the hosting environment.
3. Verify the version compatibility banner is consistent with the active contract.

### When Not to Rollback

Rollback is appropriate for bugs introduced in a recent release. It is **not**
appropriate as a substitute for a hotfix when the issue was present in multiple
versions. In that case, follow the hotfix procedure.

Also note: a rollback deploys a contract that was already known to have the bug that
prompted the release. Confirm the previous version does not also contain the issue
before rolling back.

---

## 9. Tagging Convention

All release tags follow the pattern `vMAJOR.MINOR.PATCH`, matching the semver string
returned by the contract's `version()` function.

```
v0.1.0   ← stable release
v0.1.1   ← patch release
v0.2.0   ← minor release
v0.2.0-beta.1  ← pre-release
v0.2.0-alpha.1 ← pre-release
v1.0.0   ← major release
```

Tags are annotated (`-a`) with a message summarizing the release. They are pushed
to GitHub and trigger the CI deployment workflow defined in `.github/workflows/ci.yml`.

```bash
# Creating and pushing a release tag
git tag -a v0.2.0 -m "Release v0.2.0: <one-line summary>"
git push origin v0.2.0
```

Tags on `main` must correspond to a merge commit from a `release/*` or `hotfix/*`
branch. Tags are never placed on `develop` or feature branches.

---

*Last reviewed: 2026-08-30 | Next review due: 2027-08-30*
