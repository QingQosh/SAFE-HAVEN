# Incident Post-Mortem Process

This document defines the SAFE-HAVEN post-mortem process: when post-mortems are
required, how they are conducted, the blameless culture we expect, how findings are
documented, how action items are tracked, and how we measure improvement over time.

Post-mortems are the primary mechanism by which SAFE-HAVEN learns from incidents.
They are **not** a forum for blame or punishment.

---

## Table of Contents

1. [What Is a Post-Mortem](#1-what-is-a-post-mortem)
2. [When a Post-Mortem Is Required](#2-when-a-post-mortem-is-required)
3. [Blameless Culture](#3-blameless-culture)
4. [Post-Mortem Owner and Participants](#4-post-mortem-owner-and-participants)
5. [Timeline and Scheduling](#5-timeline-and-scheduling)
6. [Post-Mortem Process](#6-post-mortem-process)
7. [Post-Mortem Template](#7-post-mortem-template)
8. [Action Item Tracking](#8-action-item-tracking)
9. [Follow-Up and Verification](#9-follow-up-and-verification)
10. [Metrics and Recurrence Prevention](#10-metrics-and-recurrence-prevention)
11. [Quarterly Review of Past Incidents](#11-quarterly-review-of-past-incidents)
12. [Sharing Lessons Learned](#12-sharing-lessons-learned)

---

## 1. What Is a Post-Mortem

A post-mortem (also called an incident review or retrospective) is a structured
analysis conducted after an incident is resolved. Its purpose is to:

- Build a shared, accurate understanding of what happened and why.
- Identify the conditions that allowed the incident to occur.
- Produce concrete, owned action items that reduce the risk of recurrence.
- Improve the team's detection, response, and recovery capabilities.

A post-mortem is **not**:
- A performance review or disciplinary process.
- An exercise in assigning fault.
- A box-ticking exercise — it should produce real, implemented improvements.

---

## 2. When a Post-Mortem Is Required

### Mandatory (any one condition is sufficient)

| Condition | Rationale |
|---|---|
| Incident severity was Critical or High (as classified in `INCIDENT_RESPONSE.md`) | Significant user or fund impact demands a formal review |
| Any user funds were temporarily or permanently inaccessible | Core safety property of the vault |
| A security vulnerability was confirmed in the smart contract or frontend | Must understand root cause and close gaps |
| An emergency withdrawal (`emergency_withdraw`) was used outside of a test | Admin-privileged action on user funds |
| The contract was paused in production | Service degradation event |
| A deployment caused a regression that reached mainnet | Deployment process failure |
| An SLA was missed | Service commitment failure |

### Recommended (post-mortem encouraged but not strictly required)

| Condition |
|---|
| Medium severity incident with potential to escalate |
| Near-miss: a condition that could have caused a mandatory incident but did not |
| An unexpected process failure that was caught before user impact |
| An incident that required more than 2 hours of on-call response |

### Not Required

- Low severity cosmetic or documentation issues with no operational impact.
- Planned maintenance activities that completed successfully.
- Incidents clearly caused by Stellar network or third-party dependency failures
  with no process improvement available to SAFE-HAVEN.

Even when a post-mortem is not required, a brief incident note (5–10 lines in the
incident channel) is always good practice.

---

## 3. Blameless Culture

SAFE-HAVEN post-mortems operate under a **blameless** model. This means:

### What Blameless Means

- **Focus on systems, processes, and conditions** — not on individuals' mistakes.
- **Assume good intent** — anyone who made a decision that contributed to the incident
  did so with the information they had at the time. Different information or tooling
  would have led to a different decision.
- **Ask "why did the system allow this?"** not "why did this person do this?".
- **Psychological safety** — participants must be able to speak honestly about what
  happened without fear of punishment. Honest post-mortems are only possible when
  people feel safe.

### What Blameless Does Not Mean

Blameless does not mean:
- Avoiding accountability for outcomes. Action items have named owners.
- Ignoring repeated poor decisions by the same person. That is a people management
  question, handled separately and privately.
- Preventing process improvement that includes clearer expectations.

### Facilitator Responsibility

The post-mortem facilitator (see Section 4) is responsible for keeping discussion
focused on systems and processes. If the conversation drifts toward individual blame,
the facilitator redirects it:

> "Let's think about what conditions made this the likely outcome — what could we
> change about the process or tooling so a different choice would be easier next time?"

---

## 4. Post-Mortem Owner and Participants

### Owner (Incident Lead)

The incident lead from the original incident (per `INCIDENT_RESPONSE.md`) owns the
post-mortem by default. Ownership may be transferred with agreement.

Responsibilities:
- Schedule the meeting within the required window (see Section 5).
- Ensure the post-mortem document is drafted before the meeting.
- Facilitate or designate a facilitator.
- Ensure action items are created and assigned.
- Publish the final document.

### Participants

| Role | Participation |
|---|---|
| Incident lead | Required |
| Technical owner (engineer who investigated) | Required |
| On-call responders | Required if they took any actions |
| Maintainers affected by corrective actions | Required |
| Security lead (for security incidents) | Required |
| Communications owner | Recommended |
| Observers / other team members | Welcome; non-voting |

Keep the meeting focused: too many participants reduces psychological safety and
increases noise. Core participants should not exceed 8 people.

---

## 5. Timeline and Scheduling

| Step | Deadline |
|---|---|
| Post-mortem document drafted (from template) | Within 3 days of incident resolution |
| Post-mortem meeting scheduled and held | Within 7 days of incident resolution |
| Draft document shared with participants | 24 hours before the meeting |
| Action items created in GitHub Issues | During or immediately after the meeting |
| Final document published | Within 2 days of the meeting |
| First action item progress check-in | 2 weeks after publication |

If the 7-day window cannot be met (e.g. due to holidays, extended on-call, or a
complex multi-phase incident), the incident lead must note the reason and reschedule
within 14 days. Post-mortems older than 14 days have significantly diminished recall
and value.

---

## 6. Post-Mortem Process

### Step 1: Prepare the Document

Before the meeting, the incident lead drafts the post-mortem document (see Section 7)
using available data sources:

- Incident timeline from the incident channel / war-room notes.
- On-chain events queried via Horizon or Stellar CLI.
- Monitoring alerts and dashboards (see `MONITORING.md`).
- Frontend error logs and CI build logs.
- Deployment manifests in `deployments/`.
- Communications sent during the incident.

The goal is to reconstruct a factual, neutral timeline. Do not editorialize or
attribute blame in the draft.

### Step 2: Share With Participants

Circulate the draft document at least 24 hours before the meeting so participants
can review and annotate. Encourage asynchronous comment on the timeline and
contributing factors before the meeting.

### Step 3: Conduct the Meeting

**Duration:** 60–90 minutes for a typical incident; up to 120 minutes for complex ones.

**Agenda:**
1. (5 min) Welcome and ground rules — restate the blameless principle.
2. (10 min) Walk through the timeline together. Correct inaccuracies.
3. (20 min) Discuss contributing factors: what made this outcome more likely?
4. (20 min) Discuss what went well: detection, response, communication, recovery.
5. (20 min) Identify action items: what changes would reduce risk or improve response?
6. (10 min) Assign owners and due dates to every action item.
7. (5 min) Confirm publication plan and who has access.

**Facilitator tips:**
- Keep the timeline walk-through factual. Stop "should have" language during this phase.
- For contributing factors, ask "five whys" to reach root causes, not symptoms.
- For action items, push for specificity: "improve monitoring" is not an action item;
  "add a Horizon polling alert for consecutive failed `withdraw` calls, threshold 3
  within 5 minutes" is.
- Distinguish between immediate fixes (done in the next sprint) and systemic
  improvements (may take longer but address root causes).

### Step 4: Finalize and Publish

After the meeting:
1. Update the document with any corrections and additions from the discussion.
2. Assign severity and impact values to confirm they were correctly classified.
3. Publish the document. See Section 7 for where to store it.
4. Create GitHub Issues for every action item.
5. Link the issues from the post-mortem document.
6. Notify relevant team members of their assigned action items.

---

## 7. Post-Mortem Template

Copy this template for each post-mortem. Store the completed document at:
`docs/postmortems/YYYY-MM-DD-<slug>.md`

---

```markdown
# Post-Mortem: <Short Descriptive Title>

**Date of incident:** YYYY-MM-DD
**Date of post-mortem meeting:** YYYY-MM-DD
**Post-mortem owner:** @<github-handle>
**Status:** Draft | In Review | Final

---

## Summary

One paragraph describing what happened, the impact, and the outcome. Write this
after the rest of the document is complete.

---

## Incident Details

| Field | Value |
|---|---|
| **Incident ID** | #<issue-number> or internal ID |
| **Severity** | Critical / High / Medium / Low |
| **Duration** | <start UTC> → <end UTC> (total: X hours Y minutes) |
| **Affected components** | Smart contract / Frontend / Infrastructure / Dependency |
| **Affected users** | <estimate or "unknown"> |
| **Funds at risk** | <amount or "none"> |
| **Incident lead** | @<github-handle> |
| **Technical owner** | @<github-handle> |

---

## Impact

Describe the concrete impact on users, the system, and the business:

- **User impact:** What could users not do? What errors did they see?
- **Financial impact:** Were funds inaccessible, lost, or at risk? Amounts?
- **Reputational impact:** Was this visible externally?
- **Security impact:** Was any security property violated or weakened?
- **Duration:** How long was the impact active?

---

## Timeline

All times in UTC. Be as precise as the data allows. Use "~" for estimates.

| Time (UTC) | Event | Source |
|---|---|---|
| 2026-XX-XX HH:MM | <First observable signal> | <monitoring alert / user report / team member> |
| 2026-XX-XX HH:MM | <Incident declared; lead assigned> | <name> |
| 2026-XX-XX HH:MM | <Triage finding> | <name> |
| 2026-XX-XX HH:MM | <Mitigation action taken> | <name> |
| 2026-XX-XX HH:MM | <User communication sent> | <name> |
| 2026-XX-XX HH:MM | <Service restored / incident resolved> | <name> |
| 2026-XX-XX HH:MM | <Post-mortem meeting held> | <name> |

---

## Root Cause Analysis

### Immediate Cause

What directly triggered the incident? (The thing that "broke".)

### Contributing Factors

List all conditions that made the outcome more likely or more severe. Use
"five whys" or Ishikawa analysis to work past symptoms to root causes.
Frame each factor as a system or process property, not a person's failing.

1. **[Factor 1]:** <description>
2. **[Factor 2]:** <description>
3. **[Factor n]:** <description>

### Root Cause

The deepest causal chain node that, if changed, would prevent this class of incident.
There may be more than one.

---

## Detection

- **How was the incident detected?** (monitoring, user report, team observation, etc.)
- **How long after the incident start was it detected?** (detection lag)
- **Could it have been detected earlier?** If yes, describe how.

---

## Response

### What Went Well

List things that worked as intended during detection, triage, mitigation, and
communication. Acknowledging successes reinforces good practices.

- Example: "The `is_paused()` check in the frontend immediately surfaced the pause
  state to all users without requiring a code change."
- Example: "The two-step admin transfer prevented an accidental key handoff."

### What Went Poorly

List gaps in process, tooling, documentation, or communication that made the
response harder than it needed to be. Frame as system gaps, not individual failures.

- Example: "No alert existed for consecutive failed `withdraw` transactions;
  detection relied on a user report."
- Example: "The runbook step for emergency withdrawal did not specify which deposit
  IDs to process first."

### Where We Got Lucky

List cases where the outcome could have been significantly worse but was not,
due to factors outside the team's control or design. These are highest-priority
action items — they represent gaps that chance protected us from.

- Example: "The bug only affected deposits made in the 2-hour window after the
  deployment. A longer window would have affected more users."

---

## Action Items

Each action item must have: a description, a GitHub Issue link, an owner, and a
due date. Action items without owners are not action items — they are wishes.

| # | Action Item | GitHub Issue | Owner | Due Date | Status |
|---|---|---|---|---|---|
| 1 | <Specific, measurable improvement> | #<issue> | @<handle> | YYYY-MM-DD | Open |
| 2 | <Specific, measurable improvement> | #<issue> | @<handle> | YYYY-MM-DD | Open |

### Action Item Categories (label GitHub Issues accordingly)

- `pm-detection` — Improve detection speed or accuracy.
- `pm-response` — Improve response process, runbook, or tooling.
- `pm-prevention` — Code or process change to prevent recurrence.
- `pm-communication` — Improve internal or external communication during incidents.
- `pm-documentation` — Correct or improve documentation that contributed to the incident.

---

## Lessons Learned

Key insights from this incident that the team should carry forward. Write for a
reader who was not involved in the incident.

1. <Lesson>
2. <Lesson>

---

## Related Links

- Incident issue: #<issue>
- Relevant contract events: <Horizon or explorer link>
- Deployment manifest: `deployments/<network>/<timestamp>/manifest.json`
- Related monitoring alert: <link>
- Prior related post-mortems: <link>

---

*Published: YYYY-MM-DD | Reviewed by: @<handles>*
```

---

## 8. Action Item Tracking

### GitHub Issues as Action Items

Every action item from a post-mortem must be tracked as a GitHub Issue with:

- **Title:** Prefixed with `[PM]` followed by a concise description.
  Example: `[PM] Add alert for consecutive withdraw failures on mainnet`
- **Labels:** One of `pm-detection`, `pm-response`, `pm-prevention`,
  `pm-communication`, `pm-documentation`; plus `post-mortem` and the incident severity
  label.
- **Assignee:** A named individual. Never unassigned.
- **Milestone:** The release or sprint in which it will be completed.
- **Linked post-mortem:** Reference the post-mortem document in the issue body.

### Tracking Dashboard

Maintain a GitHub Project (or equivalent) board with the following columns:

| Column | Meaning |
|---|---|
| **Open** | Action item identified; work not yet started |
| **In Progress** | Assignee is actively working on it |
| **In Review** | PR or change in review |
| **Done** | Merged / deployed / verified |
| **Deferred** | Postponed with documented reason and new due date |
| **Won't Fix** | Explicitly decided not to implement; reason documented |

Items must not linger in "Open" without a comment update for more than 2 weeks. If
a deadline will be missed, the assignee updates the due date and notes the reason.

### Escalation

If an action item is not completed by its due date and was not explicitly deferred:

1. The incident lead is notified automatically (via GitHub milestone or issue reminder).
2. The incident lead follows up with the assignee within 3 business days.
3. If still blocked, the incident lead escalates to a maintainer to unblock or
   reassign.

---

## 9. Follow-Up and Verification

Completing an action item and closing a GitHub Issue is not sufficient. The change
must be verified to be working as intended.

### Verification Steps by Category

| Category | Verification Method |
|---|---|
| `pm-detection` | Trigger the detection condition in a test environment and confirm the alert fires |
| `pm-response` | Run a tabletop exercise or drill using the updated runbook |
| `pm-prevention` | Write a regression test; confirm CI passes; deploy to testnet |
| `pm-communication` | Review the updated communication template with the communications owner |
| `pm-documentation` | Peer review the documentation change; confirm the scenario is covered |

### Two-Week Check-In

Two weeks after the post-mortem is published, the incident lead conducts a brief
(15 minute) check-in:

- Review the status of every action item.
- Confirm in-progress items have not stalled.
- Identify blockers.
- Update the post-mortem document status if all items are complete.

### Close-Out Criteria

A post-mortem is marked **Closed** when:

- All action items are either Done or explicitly Deferred/Won't Fix with documented
  reasons.
- The incident lead confirms no open questions remain.
- The document status is updated to `Final`.

---

## 10. Metrics and Recurrence Prevention

Track the following metrics across all post-mortems to measure the effectiveness of
the process and guide investment in prevention.

### Key Metrics

| Metric | Goal | How to Measure |
|---|---|---|
| Mean time to detect (MTTD) | Decreasing trend | `incident_start - first_detection_time` from timelines |
| Mean time to resolve (MTTR) | Decreasing trend | `resolution_time - incident_start` from timelines |
| Action item completion rate | > 80% on time | Closed issues / total issues within due date |
| Repeat incident rate | 0 for same root cause | Incidents with a prior post-mortem linking same root cause |
| Post-mortem coverage | 100% of mandatory incidents | Count of post-mortems / count of mandatory incidents |
| Detection lag | Decreasing trend | Time between incident start and detection |

### Recurrence Prevention Principles

1. **Address root causes, not symptoms.** A patch that masks the symptom without
   fixing the root cause will see the incident recur in a different form.
2. **Systemic fixes over procedural controls.** A code fix is stronger than a
   checklist item. A checklist item is stronger than a "be more careful" reminder.
3. **Prioritize "lucky" items.** Incidents where we got lucky represent the highest
   tail risk. Address these first even if the immediate impact was low.
4. **Regression tests are non-negotiable for bugs.** Every bug fix to the smart
   contract must be accompanied by a test that would have caught it. Add it to
   `contracts/safe-haven/src/test.rs`.
5. **Monitor the fix.** After deploying a prevention action, add a metric or alert
   that would confirm the fix is working and catch regressions.

---

## 11. Quarterly Review of Past Incidents

Once per quarter, the team conducts a 60-minute review of all incidents from the
preceding quarter (and any still-open action items from older incidents).

### Agenda

1. (10 min) Metrics review: MTTD, MTTR, action item completion rate, repeat incidents.
2. (20 min) Walk through each incident at a high level: was the root cause addressed?
   Are there open action items that are overdue?
3. (15 min) Identify patterns: are multiple incidents sharing a common root cause
   category (e.g. monitoring gaps, deployment process, admin key management)?
4. (10 min) Decide on any systemic investments to address patterns (e.g. a project
   to overhaul monitoring, a security audit).
5. (5 min) Confirm scheduling of the next quarterly review.

### Output

- Updated metrics table in this section (or linked dashboard).
- Any new high-priority GitHub Issues created for systemic patterns.
- A brief summary posted to the team channel.

### Historical Incidents Index

Maintain a running index of all post-mortems:

| Date | Title | Severity | MTTD | MTTR | Status |
|---|---|---|---|---|---|
| *(no incidents yet)* | — | — | — | — | — |

Add a row for every post-mortem as it is published. Store the completed documents in
`docs/postmortems/`.

---

## 12. Sharing Lessons Learned

Post-mortems are **internal only** by default. They must not be shared externally
without review, because they may contain:

- Information about vulnerabilities that have not been publicly disclosed.
- Details about admin key management or operational security.
- Transaction hashes or addresses that could assist an attacker.

### Internal Sharing

- Completed post-mortems are stored in `docs/postmortems/` and accessible to all
  repository contributors.
- The quarterly review summary is shared with the full team.
- Lessons learned that have broad applicability are promoted to the appropriate
  existing documentation (e.g. a runbook update to `INCIDENT_RESPONSE.md`, a new
  check in `MONITORING.md`, or a clarification in `DISASTER_RECOVERY.md`).

### External Sharing

If the team decides to share a post-mortem externally (e.g. as a transparency
report or community learning opportunity):

1. Remove or redact: private keys, seed phrases, unredacted depositor addresses,
   vulnerability details that are not yet patched, and any PII.
2. Review with the security lead and incident lead.
3. Obtain maintainer approval.
4. Publish in a separate public-facing document, not the internal post-mortem.

External publications should emphasize the systemic lessons and process improvements,
not the specific technical details of the vulnerability or failure.

---

*Last reviewed: 2026-08-30 | Next review due: 2027-08-30*
