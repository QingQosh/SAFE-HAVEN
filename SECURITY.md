# Security Policy

For handling and documenting confirmed incidents, see [INCIDENT_RESPONSE.md](./INCIDENT_RESPONSE.md).

## Supported Versions

---

## Table of Contents

1. [Supported Versions](#1-supported-versions)
2. [Reporting a Vulnerability](#2-reporting-a-vulnerability)
3. [What to Include in a Report](#3-what-to-include-in-a-report)
4. [Response Timeline](#4-response-timeline)
5. [Embargo and Coordinated Disclosure](#5-embargo-and-coordinated-disclosure)
6. [Vulnerability Handling Process](#6-vulnerability-handling-process)
7. [Scope](#7-scope)
8. [Out of Scope](#8-out-of-scope)
9. [Credit and Acknowledgement](#9-credit-and-acknowledgement)
10. [Bug Bounty](#10-bug-bounty)
11. [Security Advisory Template](#11-security-advisory-template)

---

## 1. Supported Versions

| Version | Supported | Notes |
|---------|-----------|-------|
| `0.1.x` (latest) | ✅ Yes | Active development; patches released as needed |
| `< 0.1.0` | ❌ No | Pre-release; unsupported |

Security patches are only issued for the current latest release. If you are running an older deployment, upgrade before assuming you are protected.

For operational response to contract vulnerabilities, state mismatches, and admin-key incidents, see the [Disaster Recovery Runbook](DISASTER_RECOVERY.md).

The following are considered in-scope vulnerabilities:

## 2. Reporting a Vulnerability

**Do not open a public GitHub issue for any security vulnerability.**

Public disclosure before a fix is deployed puts every user's funds at risk.

### How to report

**Email:** security@safe-haven.example.com
*(Replace with a monitored address before going to production. Consider a PGP-encrypted channel for critical reports.)*

**PGP key:** *(publish your PGP public key fingerprint here once available)*

**GitHub Security Advisories (alternative):** Use the [Private Security Advisory](https://github.com/kenedybok3/SAFE-HAVEN/security/advisories/new) feature to report directly through GitHub. This keeps the report private until a fix is ready.

Do not:

- Open a public issue
- Share details in the public Discord, Telegram, or any other community channel
- Commit a proof-of-concept to a public fork of this repository
- Contact individual maintainers on personal social media

---

## 3. What to Include in a Report

A complete report speeds up triage and reduces back-and-forth. Please include:

- **Summary** — A concise description of the vulnerability
- **Affected component** — Smart contract function(s), frontend path, or infrastructure
- **Affected version(s)** — The contract version or commit hash
- **Vulnerability class** — e.g., integer overflow, auth bypass, re-entrancy, storage corruption
- **Steps to reproduce** — The minimum sequence of calls or actions that trigger the issue
- **Proof of concept** — A failing test, CLI invocation, or transaction XDR (if available; not required)
- **Impact assessment** — Funds at risk? Which users? Estimated blast radius
- **Suggested fix** (optional) — If you have a patch idea, include it — but a report without a fix is still valuable

You do not need to provide a complete exploit to file a report.

---

## 4. Response Timeline

| Milestone | Target |
|---|---|
| **Initial acknowledgement** | Within **48 hours** of receipt |
| **Triage complete** (severity confirmed) | Within **5 business days** |
| **Fix target communicated** | Within **7 days** for S1; within **14 days** for S2 |
| **Fix delivered (patch PR merged)** | Within **14 days** for S1 (critical); **30 days** for S2 (high) |
| **Public disclosure** | After fix is deployed AND embargo window has passed (see §5) |

If we cannot meet a target, we will communicate proactively on the private thread.

Silence past the 48-hour acknowledgement window is a bug in our process, not a signal that the report was ignored. Re-ping us.

---

## 5. Embargo and Coordinated Disclosure

### Embargo period

We follow coordinated disclosure. The default embargo is:

- **S1 (Critical):** 14 days from fix deployment, or earlier by mutual agreement
- **S2 (High):** 30 days from fix deployment, or earlier by mutual agreement
- **S3/S4:** Disclosed at the maintainer's discretion, usually with the release

During the embargo window, the vulnerability details are shared only with:

- The reporter
- The core maintainer team
- Any auditor or security reviewer directly involved in the fix
- Downstream partners directly affected (e.g., if a custodian relies on the contract)

### Reporter obligations during embargo

- Do not disclose the vulnerability publicly until the embargo has lifted or you have received written clearance from the maintainer team
- Do not demonstrate the vulnerability on mainnet or testnet against funds you do not own
- Do not contact third parties (exchanges, custodians) about the issue without coordinating with the maintainer team first

### What happens when embargo lifts

1. A GitHub Security Advisory is published (see §11 for the template)
2. A `CHANGELOG.md` entry is added under `### Security`
3. A new patch version is tagged and released
4. A brief public post-mortem is published (optional, for high-impact vulnerabilities)

### Expedited disclosure

If an S1 vulnerability is already being actively exploited on mainnet, the embargo is waived and disclosure happens immediately alongside or after emergency patching.

---

## 6. Vulnerability Handling Process

```
Reporter sends email / GitHub Advisory
            │
            ▼
  [Acknowledged within 48h]
  Maintainer confirms receipt, opens a private tracking issue,
  assigns a severity, and names an incident lead.
            │
            ▼
  [Triage — within 5 business days]
  Incident lead reproduces the issue.
  Severity confirmed: S1 / S2 / S3 / S4.
  Fix deadline set.
            │
            ▼
  [Patching]
  Fix developed on a private branch.
  Internal review by at least one other maintainer.
  For contract changes: smart-contract security review required.
            │
            ▼
  [Staging verification]
  Fix tested on local node and testnet.
  All existing tests pass.
  New regression test added that directly covers the reported scenario.
            │
            ▼
  [Deployment]
  New contract version deployed to testnet, then mainnet.
  Frontend updated if applicable.
  New release tagged.
            │
            ▼
  [Disclosure — after embargo period]
  GitHub Security Advisory published.
  CHANGELOG.md updated.
  Reporter credited (with their permission).
  Post-mortem published for critical issues.
```

### Tracking and communication

- Every accepted report gets a private GitHub Security Advisory entry as the canonical tracking document.
- The reporter receives a status update at each stage transition.
- If an incident lead becomes unavailable, the triage facilitator (see BUG_TRIAGE.md) takes over.

---

## 7. Scope

The following are considered **in-scope** vulnerabilities for this project:

### Smart contract (highest priority)

- Any bug that can result in **fund loss or theft** from depositors
- **Auth bypass:** circumventing `require_auth()` in any mutating function
- **Re-entrancy** or checks-effects-interactions violations
- **Storage manipulation:** modifying another depositor's `VaultEntry` or `LedgerVaultEntry`
- **Admin privilege escalation:** gaining admin rights without the current admin's consent
- **Contract re-initialization** after `renounce_admin()` or `initialize()` has run
- **Penalty miscalculation** that over-charges or under-charges depositors
- **Incorrect TTL handling** that causes live deposits to expire prematurely
- **`deposit_by_ledger` missing upper-bound check** (known gap — tracked in Known Limitations)
- **Integer overflow / underflow** in amount arithmetic or ledger sequence comparisons

### Frontend

- XSS vulnerabilities that could lead to wallet compromise
- Incorrect transaction construction that submits wrong parameters to the contract
- Network passphrase mismatch that silently executes on the wrong network

### Infrastructure

- Secrets exposed in CI logs or GitHub Actions environment
- Dependency confusion or supply-chain attack via `package.json` or `Cargo.toml`

---

## 8. Out of Scope

The following are **not** in scope for this security policy:

- Stellar network-level vulnerabilities — report to [Stellar Bug Bounty](https://www.stellar.org/bug-bounty-program)
- Bugs in `soroban-sdk` itself — report to [stellar/rs-soroban-sdk](https://github.com/stellar/rs-soroban-sdk/security)
- Issues requiring physical access to a user's private key or seed phrase
- Social engineering attacks targeting maintainers or users
- Theoretical vulnerabilities without a realistic attack path
- Missing security headers on a static frontend with no server-side processing
- Rate limiting or DoS on the Stellar RPC — this is a network-level concern
- Vulnerabilities in Freighter wallet internals — report to the Freighter team

If you are unsure whether your finding is in scope, report it anyway. We would rather triage an out-of-scope report than miss a real issue.

---

## 9. Credit and Acknowledgement

We believe in crediting security researchers who responsibly disclose vulnerabilities.

By default, when a security advisory is published we will credit the reporter as:

> Reported by **[Name / Handle]** ([link to profile], [date])

If you prefer to remain anonymous, tell us in your initial report and we will publish the advisory without attribution.

We also maintain a **Hall of Fame** section in this file (below) for past reports. Entry into the Hall of Fame is opt-in.

### Hall of Fame

| Researcher | Vulnerability | Severity | Date | Advisory |
|---|---|---|---|---|
| *(none yet — be the first)* | | | | |

---

## 10. Bug Bounty

There is currently **no active bug bounty program**. Vulnerability research is voluntary.

If a critical vulnerability is reported and leads to a fix that protects live user funds, the maintainer team will consider a discretionary reward on a case-by-case basis, subject to available project funds. This is not a guarantee.

This section will be updated if a formal bug bounty program is launched.

---

## 11. Security Advisory Template

Use this template when publishing a GitHub Security Advisory after an embargo lifts.

```markdown
## [SAFE-HAVEN-YYYY-NNN] <Vulnerability Title>

**CVE ID:** CVE-YYYY-NNNNN (if assigned)
**GHSA ID:** GHSA-xxxx-xxxx-xxxx
**Severity:** Critical / High / Medium / Low
**CVSS score:** N/A or X.X (if calculated)
**Affected versions:** < X.Y.Z
**Fixed in version:** X.Y.Z
**Published:** YYYY-MM-DD
**Reporter:** Name / Handle (or "Anonymous")

---

### Summary

A one-paragraph description of the vulnerability suitable for public consumption.
Avoid overly specific exploit details that could be used to attack unpatched deployments.

### Impact

Who is affected, what can an attacker do, and what is the worst-case outcome?

- Affected users: all depositors / depositors using `deposit_by_ledger` / admin only
- Worst case: fund loss / auth bypass / incorrect query result / DoS

### Patches

Fixed in commit [abc1234](https://github.com/kenedybok3/SAFE-HAVEN/commit/abc1234)
and released in [v X.Y.Z](https://github.com/kenedybok3/SAFE-HAVEN/releases/tag/vX.Y.Z).

### Workarounds

If no patch is available, document any mitigations here.
If a deployed contract is immutable and cannot be patched, clearly state that.

### Timeline

| Date | Event |
|---|---|
| YYYY-MM-DD | Vulnerability reported by [Reporter] |
| YYYY-MM-DD | Acknowledged; private tracking issue opened |
| YYYY-MM-DD | Triage complete; severity confirmed as [S1/S2/S3/S4] |
| YYYY-MM-DD | Fix developed and reviewed |
| YYYY-MM-DD | Fix deployed to mainnet; v X.Y.Z released |
| YYYY-MM-DD | Embargo lifted; advisory published |

### References

- [Related issue or PR](link)
- [Relevant section of contract code](link)
- [SAFE-HAVEN Known Limitations](../README.md#known-limitations)
```
