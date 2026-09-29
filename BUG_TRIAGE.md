# Bug Triage and Prioritization Process

This document defines how the SAFE-HAVEN team discovers, classifies, assigns, and resolves bugs. All maintainers and contributors are expected to follow this process.

---

## Table of Contents

1. [Triage Meeting](#1-triage-meeting)
2. [Severity Levels](#2-severity-levels)
3. [Priority Levels](#3-priority-levels)
4. [SLA — Resolution Targets](#4-sla--resolution-targets)
5. [Triage Checklist](#5-triage-checklist)
6. [Label Reference](#6-label-reference)
7. [Bug Lifecycle](#7-bug-lifecycle)
8. [Triage Report Template](#8-triage-report-template)
9. [Escalation Path](#9-escalation-path)

---

## 1. Triage Meeting

| Field | Detail |
|---|---|
| Cadence | Weekly, every Monday |
| Duration | 45 minutes |
| Facilitator | Rotating — any maintainer |
| Required attendees | At least one maintainer with smart-contract context |
| Input | All issues labelled `bug` opened since the last meeting |
| Output | Every new bug has severity + priority labels, an owner, and an effort estimate |

**Agenda (timeboxed)**

1. (5 min) Review last week's SLA compliance — any breached deadlines?
2. (30 min) Walk every untriaged bug:
   - Confirm reproducibility
   - Assign severity and priority (see §2 and §3)
   - Assign owner
   - Estimate effort (XS / S / M / L / XL)
   - Set milestone / target sprint
3. (10 min) Review in-flight bugs — status updates and blockers

---

## 2. Severity Levels

Severity describes the **technical impact** of the bug, independent of how many users are affected or how urgent a fix is. Assign the highest severity that applies.

### S1 — Critical

Criteria (any one is sufficient):

- Funds at risk: a bug that can cause loss, theft, or permanent locking of user tokens
- Auth bypass: `require_auth()` can be circumvented
- Re-entrancy or checks-effects-interactions violation
- Contract panics unconditionally on a reachable code path
- Data corruption in persistent storage
- The contract can be re-initialized after `renounce_admin()`

Examples in this codebase:

- `cancel_deposit` succeeds after the lock has already expired, paying a penalty on already-expired funds
- `emergency_withdraw` sends tokens to the admin instead of the depositor
- Overflow in `penalty_bps` calculation that allows token inflation

### S2 — High

Criteria:

- Incorrect business logic with material financial impact but not direct fund loss
- Certain error codes are emitted in wrong conditions, misleading callers
- TTL bump is skipped on a write path, risking storage expiry of live deposits
- A paginated query silently truncates or skips depositors

Examples:

- `time_remaining` returns a wildly incorrect estimate for ledger-based deposits
- `get_deposits_page` skips active deposits due to off-by-one in offset logic
- `deposit_by_ledger` accepts `unlock_ledger` values in the past

### S3 — Medium

Criteria:

- Incorrect output that does not directly affect funds
- UI/frontend behaviour that is wrong but not dangerous
- Non-critical event data is missing or incorrect
- A query returns a stale value after a concurrent state change

Examples:

- `get_vault` returns `None` for a timestamp-based deposit that should exist
- A toast notification shows the wrong token amount in the frontend
- An event carries a wrong `deposit_id` in its data payload

### S4 — Low

Criteria:

- Cosmetic, documentation, or copy errors
- Minor UI inconsistencies
- Code paths that are unreachable in production but would behave incorrectly if reached

Examples:

- README describes a function parameter incorrectly
- A comment in `contract.rs` references a closed issue number that no longer exists
- Unused import triggers a dead-code warning in a test helper

---

## 3. Priority Levels

Priority describes **when the fix must ship**, taking severity, blast radius, and current sprint capacity into account. Severity informs priority but does not dictate it — a low-severity bug on a hot path can warrant high priority.

| Priority | Label | When to use |
|---|---|---|
| **P1 — Urgent** | `priority: urgent` | Fix immediately; can block a hotfix release outside the normal sprint cycle |
| **P2 — High** | `priority: high` | Fix in the next sprint; should not roll over |
| **P3 — Medium** | `priority: medium` | Fix in the next 1–2 sprints; acceptable to defer once if scope is large |
| **P4 — Low** | `priority: low` | Fix when bandwidth allows; fine to defer to a backlog grooming session |

### Default severity → priority mapping

This mapping is a starting point. The triage facilitator may adjust priority up or down based on context.

| Severity | Default Priority |
|---|---|
| S1 — Critical | P1 — Urgent |
| S2 — High | P2 — High |
| S3 — Medium | P3 — Medium |
| S4 — Low | P4 — Low |

**Reasons to escalate priority** (e.g., S3 → P2):

- Affects the happy path used by the majority of depositors
- A workaround does not exist or is unreasonably complex
- The bug is already publicly reported or being discussed externally
- A partner integration depends on the broken behaviour

**Reasons to de-escalate priority** (e.g., S2 → P3):

- Only affects `deposit_by_ledger`, which has no frontend support and low usage
- A reliable workaround is documented and communicated to affected users
- The next planned release already addresses the underlying area

---

## 4. SLA — Resolution Targets

SLA is measured from the time the bug is triaged (labels applied and owner assigned), not from the time it was opened.

| Severity | Initial Triage | Fix PR Opened | Fix Merged & Released |
|---|---|---|---|
| S1 — Critical | Within 4 hours of report | Within 24 hours | Within 48 hours |
| S2 — High | At the next triage meeting (≤ 7 days) | Within 1 week of triage | Within 2 weeks |
| S3 — Medium | At the next triage meeting (≤ 7 days) | Within the next sprint (2 weeks) | Within 4 weeks |
| S4 — Low | Within 2 triage meetings (≤ 14 days) | No hard deadline | Next minor release |

**SLA breach protocol**

If an S1 or S2 fix PR is not opened within the target window, the owner must post a status update on the issue explaining the blocker and either resolve it or escalate to another maintainer.

---

## 5. Triage Checklist

Complete every item before marking a bug as triaged. Copy this checklist into the issue when triaging.

```markdown
### Triage Checklist

**Reproducibility**
- [ ] Bug reproduced locally with the steps provided
- [ ] Reproduction steps added to the issue if missing
- [ ] Minimal failing test or code snippet attached (or noted as unnecessary)

**Classification**
- [ ] Severity label applied (severity: critical / high / medium / low)
- [ ] Priority label applied (priority: urgent / high / medium / low)
- [ ] Component label applied (component: contract / frontend / ci / docs)
- [ ] Regression label applied if this was working in a previous release

**Ownership & Planning**
- [ ] Owner assigned (GitHub Assignee field)
- [ ] Effort estimated and added to the issue (XS / S / M / L / XL)
- [ ] Milestone / sprint set

**Security Gate**
- [ ] Confirmed this is NOT a security vulnerability requiring private disclosure
  (If it is, move to private channel immediately — see SECURITY.md)

**Context**
- [ ] Affected versions listed (e.g., 0.1.0, or Unreleased)
- [ ] Workaround documented in the issue if one exists
- [ ] Linked to related issues or PRs
```

---

## 6. Label Reference

Create these labels in the GitHub repository settings before using this process.

### Severity labels

| Label | Colour | Description |
|---|---|---|
| `severity: critical` | `#B60205` (red) | Fund loss, auth bypass, contract panic |
| `severity: high` | `#E4E669` (yellow) | Major incorrect behaviour, storage risk |
| `severity: medium` | `#FEF2C0` (light yellow) | Incorrect output, minor UX failure |
| `severity: low` | `#C5DEF5` (light blue) | Cosmetic, docs, unreachable code |

### Priority labels

| Label | Colour | Description |
|---|---|---|
| `priority: urgent` | `#B60205` (red) | Fix immediately |
| `priority: high` | `#E4E669` (yellow) | Fix next sprint |
| `priority: medium` | `#FEF2C0` (light yellow) | Fix in 1–2 sprints |
| `priority: low` | `#C5DEF5` (light blue) | Fix when bandwidth allows |

### Component labels

| Label | Colour | Description |
|---|---|---|
| `component: contract` | `#D4C5F9` (purple) | Smart contract (Rust / Soroban) |
| `component: frontend` | `#0075CA` (blue) | React / TypeScript UI |
| `component: ci` | `#E4E669` (yellow) | GitHub Actions, CI/CD |
| `component: docs` | `#CFDCEF` (light blue) | Documentation, README, ADRs |

### Status labels

| Label | Colour | Description |
|---|---|---|
| `status: needs-reproduction` | `#F9D0C4` (peach) | Cannot be confirmed yet |
| `status: confirmed` | `#0E8A16` (green) | Reproduced and ready to fix |
| `status: in-progress` | `#D93F0B` (orange) | Fix is actively being worked on |
| `status: blocked` | `#B60205` (red) | Waiting on external dependency |
| `status: wont-fix` | `#EEEEEE` (grey) | Intentional behaviour or out of scope |
| `regression` | `#EE0701` (red) | Worked in a previous release |

---

## 7. Bug Lifecycle

```
Reported (issue opened)
        │
        ▼
[Untriaged]
  Label: bug (auto-applied by template)
        │
        │  At triage meeting (or within 4h for S1)
        ▼
[Triaged]
  Labels: severity:*, priority:*, component:*
  Assignee set, milestone set, effort estimated
        │
        ├── needs-reproduction → reporter asked to provide steps
        │
        └── status: confirmed
              │
              ▼
        [In Progress]
          Branch opened: fix/<description>
          status: in-progress
              │
              ▼
        [PR Opened]
          Fix PR references the issue ("Closes #N")
          Must pass CI (fmt, clippy, tests, audit, deny)
          Requires smart-contract review for contract changes
              │
              ▼
        [Merged]
          CHANGELOG.md updated under [Unreleased] → ### Fixed
          Issue closed automatically by PR merge
              │
              ▼
        [Released]
          Version tagged and CHANGELOG entry promoted to a release section
```

---

## 8. Triage Report Template

Post this report as a comment on a dedicated weekly tracking issue or in the team communication channel after each triage meeting.

```markdown
## Bug Triage Report — YYYY-MM-DD

**Facilitator:** @username
**Attendees:** @username1, @username2

---

### New Bugs Triaged

| # | Title | Severity | Priority | Owner | Effort | SLA deadline |
|---|---|---|---|---|---|---|
| #N | Short title | S1/S2/S3/S4 | P1/P2/P3/P4 | @user | S | YYYY-MM-DD |

---

### SLA Status (open bugs)

| # | Title | Severity | Due | Status |
|---|---|---|---|---|
| #N | Short title | S1 | YYYY-MM-DD | On track / AT RISK / BREACHED |

---

### Closed Since Last Meeting

| # | Title | Time to close |
|---|---|---|
| #N | Short title | 3 days |

---

### Notes / Action Items

- [ ] Action item — @owner
```

---

## 9. Escalation Path

| Situation | Action |
|---|---|
| S1 bug identified outside triage meeting | Assign immediately; do not wait for Monday. Notify all maintainers via the team channel. |
| S1 fix PR not opened within 24 hours | Current owner escalates to another maintainer or requests help on the issue |
| Bug is actually a security vulnerability | Close the public issue immediately. Move to private disclosure per [SECURITY.md](./SECURITY.md). |
| Owner becomes unavailable | Facilitator re-assigns at the next triage meeting or immediately for P1 issues |
| Bug has no clear owner | Facilitator assigns themselves temporarily until a permanent owner is found |
| SLA breached twice in a row | Escalate to project lead; reassign if needed; root-cause why the SLA was missed |
