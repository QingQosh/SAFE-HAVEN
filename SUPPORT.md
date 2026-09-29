# Maintenance and Support Matrix

This document defines the version lifecycle, support windows, SLA commitments, EOL
policy, and upgrade paths for SAFE-HAVEN.

---

## Table of Contents

1. [Version Lifecycle Stages](#1-version-lifecycle-stages)
2. [Support Matrix](#2-support-matrix)
3. [Support Scope](#3-support-scope)
4. [Bug Fix SLA by Severity](#4-bug-fix-sla-by-severity)
5. [End-of-Life Policy](#5-end-of-life-policy)
6. [Upgrade Paths](#6-upgrade-paths)
7. [Migration Guides](#7-migration-guides)
8. [How to Report an Issue](#8-how-to-report-an-issue)

---

## 1. Version Lifecycle Stages

Every SAFE-HAVEN release progresses through the following stages:

### Alpha

- **Purpose:** Early development and internal testing. Not intended for use with real
  funds.
- **Stability:** API surface and storage layout may change without notice between
  alpha builds.
- **Support:** Best-effort only. No SLA. No patch releases for non-critical bugs.
- **Identifier:** `0.x.0-alpha.N` (e.g. `0.2.0-alpha.1`)
- **Deployment:** Testnet only.

### Beta

- **Purpose:** Public testing and integration. Suitable for testnet exploration.
  Not recommended for mainnet use with real funds.
- **Stability:** API is stabilizing. Breaking changes are announced in advance.
  Storage layout changes trigger a migration path.
- **Support:** Issues triaged within 7 days. Critical bugs patched within 14 days.
- **Identifier:** `0.x.0-beta.N` (e.g. `0.2.0-beta.1`)
- **Deployment:** Testnet; mainnet at operator discretion with risk acknowledgement.

### Stable

- **Purpose:** Production-ready release. Suitable for mainnet deployment with real
  funds.
- **Stability:** API and storage layout are stable. No breaking changes within a
  minor version. Patch releases issued for bug fixes and security vulnerabilities.
- **Support:** Full SLA applies. See [Section 4](#4-bug-fix-sla-by-severity).
- **Identifier:** `MAJOR.MINOR.PATCH` (e.g. `0.1.0`, `1.0.0`)
- **Deployment:** Testnet and mainnet.

### Maintenance

- **Purpose:** A previous stable version that is still supported for security patches
  only. No new features backported.
- **Stability:** No API changes. Security patches only.
- **Support:** Critical and high security patches only. Non-security bugs are not
  fixed in maintenance releases; upgrade to the current stable.
- **Identifier:** Same semver scheme. Patch releases only (e.g. `0.1.1`, `0.1.2`).
- **Deployment:** Existing deployments remain; new deployments should use current stable.

### End of Life (EOL)

- **Purpose:** A version that is no longer supported. No patches, no security fixes.
- **Support:** None. Users must upgrade.
- **Action required:** Migrate funds to a contract deployed from the current stable
  release. See [Section 6](#6-upgrade-paths).

---

## 2. Support Matrix

### Current Status (as of 2026-08-30)

| Version | Stage | Released | Active Support Until | EOL Date | Notes |
|---|---|---|---|---|---|
| `0.1.0` | **Stable** | 2026-05-31 | TBD (≥ 12 months) | TBD | Current stable release |
| `0.2.0` | Planned | — | — | — | Under development |

> **Policy:** At any given time, the latest stable minor version receives full support.
> The immediately preceding stable minor version receives maintenance (security-only)
> support for 6 months after the next minor release. All earlier versions are EOL.

### Stellar Network Dependencies

SAFE-HAVEN depends on the Stellar protocol and Soroban runtime. Support for a
SAFE-HAVEN version is implicitly limited by Stellar's own support for the protocol
version it was compiled against.

| SAFE-HAVEN Version | Soroban SDK | Min Rust | Stellar Protocol |
|---|---|---|---|
| `0.1.0` | `v22` | `1.81` | Protocol 21+ |

If Stellar deprecates a protocol version that a SAFE-HAVEN release depends on, that
SAFE-HAVEN version enters EOL regardless of its normal lifecycle timeline.

---

## 3. Support Scope

### In Scope

The following are supported within the applicable SLA:

- Bugs in the SAFE-HAVEN smart contract logic (`contracts/safe-haven/src/`)
- Bugs in the React/TypeScript frontend (`frontend/src/`)
- Incorrect documentation that causes a user to take a harmful action
- Security vulnerabilities in SAFE-HAVEN code
- Build system failures (`Makefile`, CI workflows)
- Deployment script failures (`scripts/deploy.sh`, `scripts/deploy_testnet.sh`)
- Storage migration issues (`migrate()` function)
- Version compatibility check failures (`frontend/src/lib/versionCompat.ts`)

### Out of Scope

The following are **not** covered by the SAFE-HAVEN support SLA:

- Issues caused by bugs in `soroban-sdk` itself — report to
  [stellar/rs-soroban-sdk](https://github.com/stellar/rs-soroban-sdk)
- Stellar network outages, validator issues, or protocol-level failures — report to
  the [Stellar Development Foundation](https://stellar.org)
- Freighter wallet bugs — report to [stellar/freighter](https://github.com/stellar/freighter)
- RPC endpoint reliability issues with third-party providers
- User errors (e.g. setting incorrect penalty basis points, choosing a wrong
  lock duration)
- Requests for features not in the current stable release
- Supporting versions that have reached EOL
- Custom forks or modifications of SAFE-HAVEN code
- Tax, legal, or financial advice

### Critical vs. Non-Critical Issues

| Category | Examples |
|---|---|
| **Critical** | Loss of user funds, auth bypass, re-entrancy, emergency-withdraw sending to wrong address, contract completely non-functional |
| **High** | Security vulnerability without confirmed exploitation, withdrawal failures affecting multiple users, admin key management failures |
| **Medium** | Non-critical feature regression, elevated transaction error rate, frontend rendering defects affecting usability |
| **Low** | Minor UI cosmetic defects, documentation improvements, performance optimisations with no correctness impact |

---

## 4. Bug Fix SLA by Severity

These SLAs apply to **stable** versions. Beta versions have relaxed timelines.
Maintenance versions apply SLAs only to Critical and High security issues.

| Severity | Initial Response | Triage Complete | Fix Released | Notes |
|---|---|---|---|---|
| **Critical** | 4 hours | 24 hours | 72 hours | Immediate notification; patch release |
| **High** | 24 hours | 72 hours | 14 days | Security advisories issued where applicable |
| **Medium** | 3 business days | 7 days | Next patch or minor release | Batched into scheduled releases |
| **Low** | 7 business days | 14 days | Next minor release | Triaged and scheduled |

**SLA clock starts** when the issue is acknowledged by a maintainer (not at submission
time). Response times are measured in calendar time unless stated as business days.

**SLA exceptions:** SLAs may be suspended during:
- Stellar network-wide outages that block deployment of a fix.
- Legal or regulatory proceedings that restrict disclosure.
- Active incident response where stabilization takes priority over SLA compliance.

Any suspension is communicated on the GitHub repository.

### Patch Release Policy

- Patch releases (`MAJOR.MINOR.PATCH`) are issued for Critical and High severity fixes
  only. They contain only the targeted fix plus any directly related changes.
- Medium and Low severity fixes are collected into the next scheduled minor or patch
  release.
- A patch release does not change the contract storage layout. If a fix requires a
  storage schema change, it is a minor or major version bump, not a patch.

> **Smart contract constraint:** Because Soroban contracts are immutable, a "patch
> release" means deploying a new contract and providing a migration path. The old
> contract remains on-chain permanently. See [Section 6](#6-upgrade-paths).

---

## 5. End-of-Life Policy

### EOL Announcement

EOL dates are announced **at least 6 months** before a version becomes EOL. Announcements
are made via:

- A GitHub release note on the new stable version.
- An update to this document with the confirmed EOL date.
- A pinned issue in the GitHub repository.
- A banner in the frontend UI (if the deprecated version is still deployed).

### EOL Effects

Once a version reaches EOL:

- No further security patches are issued.
- The version is removed from the support matrix above with status `EOL`.
- The frontend may display an EOL warning when connected to an EOL contract version.
- The `docs/VERSION_COMPATIBILITY.md` compatibility matrix is updated to mark the
  version as no longer compatible with current frontend builds.

### User Obligations at EOL

Users with active deposits in an EOL contract are not automatically affected — the
contract continues to operate on-chain. However:

- Security vulnerabilities in an EOL contract will not be patched.
- The frontend may no longer support interaction with an EOL contract version.
- Users should withdraw their funds and re-deposit into a contract running the current
  stable version.

Operators who deployed an EOL version should communicate to their users and provide
a migration path. See [Section 6](#6-upgrade-paths).

---

## 6. Upgrade Paths

Because SAFE-HAVEN smart contracts are **immutable** on the Stellar blockchain,
"upgrading" means deploying a new contract and migrating user funds. There is no
in-place upgrade mechanism.

### Standard Upgrade Procedure

1. **Announce** the new version and EOL timeline of the old version with at least
   6 months' notice.
2. **Deploy** the new contract version to testnet; run full smoke tests.
3. **Deploy** to mainnet using `make deploy-mainnet`. Record the new contract ID.
4. **Update** the frontend: set `VITE_CONTRACT_ID` to the new contract ID, and
   update `VITE_EXPECTED_CONTRACT_VERSION` in `.env`.
5. **Verify** the version compatibility banner is clear in the frontend.
6. **Communicate** the new contract address to all users. Users must:
   - Wait for their existing locks to expire (or cancel with penalty).
   - Withdraw from the old contract.
   - Re-deposit into the new contract.
7. **Monitor** the old contract: track the remaining locked deposits so you know
   when all funds have been migrated.
8. When the old contract has zero active deposits, mark it as fully deprecated in
   your operational records. It will remain on-chain permanently but receives no
   further support.

See [`docs/CONTRACT_UPGRADE_AND_MIGRATION_PLAYBOOK.md`](./docs/CONTRACT_UPGRADE_AND_MIGRATION_PLAYBOOK.md)
for the detailed runbook.

### Emergency Migration (Critical Bug)

If a critical vulnerability requires urgent migration:

1. Follow [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md) for containment.
2. Pause deposits on the old contract immediately: `make` target or direct CLI call.
3. Deploy the patched contract to testnet; fast-track testing (at minimum: smoke
   tests + targeted regression tests for the vulnerability).
4. Deploy to mainnet.
5. Use `emergency_withdraw(admin, depositor, deposit_id)` for affected deposits if
   the vulnerability could result in fund loss and users cannot self-service.
6. Communicate immediately via GitHub, status channels, and any external dashboards.

The `emergency_withdraw` function always sends funds to the depositor, never to the
admin. This design ensures emergency migration cannot be used for admin theft.

---

## 7. Migration Guides

### 0.1.0 → 0.2.0 (Future)

> This section will be completed when v0.2.0 is released. See
> [`docs/VERSION_COMPATIBILITY.md`](./docs/VERSION_COMPATIBILITY.md) for a preview.

High-level steps:

1. Review `CHANGELOG.md` for all breaking changes.
2. Deploy the new contract: `make deploy-testnet` then `make deploy-mainnet`.
3. Update `VITE_CONTRACT_ID` and `VITE_EXPECTED_CONTRACT_VERSION` in `.env`.
4. If storage schema changed: update `VITE_EXPECTED_STORAGE_VERSION` and add a row
   to the `COMPATIBILITY_MATRIX` in `frontend/src/lib/versionCompat.ts`.
5. Communicate to users: the old contract remains active; deposits must be
   self-migrated.

---

## 8. How to Report an Issue

### Security Vulnerabilities

**Do not open a public GitHub issue for security vulnerabilities.**

Email: security@example.com *(update with production address before launch)*

Include: description, reproduction steps, potential impact, and (if available) a
proof-of-concept. Response within 72 hours. See [SECURITY.md](./SECURITY.md) for
full disclosure policy.

### Non-Security Bugs

1. Search [existing issues](https://github.com/kenedybok3/SAFE-HAVEN/issues) to avoid
   duplicates.
2. Open a new issue using the **Bug Report** template.
3. Include: SAFE-HAVEN version, contract ID and network, steps to reproduce, actual
   vs. expected behaviour, and any relevant logs or transaction hashes.

### Feature Requests

Open a **Feature Request** issue or start a GitHub Discussion. Large features should
follow the ADR process described in
[`docs/adr/README.md`](./docs/adr/README.md) before implementation begins.

---

*Last reviewed: 2026-08-30 | Next review due: 2027-08-30*
