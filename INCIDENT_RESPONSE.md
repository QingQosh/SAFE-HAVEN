# Incident Response

This document defines how SAFE-HAVEN incidents are classified, managed, and documented.

## Severity Levels

| Severity | Definition | Examples | Response target |
|---|---|---|---|
| **SEV1 — Critical** | Users are currently affected, or funds or security are at immediate risk. | Loss of access to funds, active exploit, or a production-wide outage. | Acknowledge within 5 minutes; incident lead assigned immediately; continuous response until mitigated. |
| **SEV2 — High** | There is potential for significant user, operational, or security impact even if users are not yet affected. | A confirmed vulnerability without evidence of exploitation, or a failure likely to affect withdrawals. | Acknowledge within 15 minutes; incident lead assigned within 30 minutes. |
| **SEV3 — Medium** | The service is degraded, but the primary user workflow remains available or a workaround exists. | Elevated transaction failures, slow RPC responses, or partial feature unavailability. | Acknowledge within 1 hour; triage during business hours. |
| **SEV4 — Low** | A minor issue with limited scope, little or no user impact, and no material security risk. | Cosmetic defects, isolated non-blocking errors, or documentation issues. | Acknowledge within 1 business day; handle through normal backlog. |

When in doubt, use the higher severity until triage establishes the actual impact. Severity may be raised or lowered as evidence changes.

## Incident Response Workflow

1. **Detect**
   - Identify the signal from monitoring, alerts, user reports, security reports, or team observation.
   - Record the detection time, affected component, initial symptoms, and supporting links or logs.
   - Open an incident record and assign an incident lead.

2. **Triage**
   - Confirm whether the event is an incident and assign an initial severity.
   - Determine the scope, affected users or funds, start time, and current risk.
   - Identify the incident lead, technical owner, communications owner, and any required security or infrastructure reviewers.

3. **Mitigate**
   - Contain the incident and protect users and funds before pursuing the full root cause.
   - Choose the least risky mitigation available, such as disabling a feature, pausing affected operations, rolling back a release, or applying a verified fix.
   - Record actions, owners, timestamps, and observed results. Validate recovery before declaring the incident resolved.

4. **Resolve**
   - Confirm that the mitigation has restored service and that monitoring is stable.
   - Downgrade or close the incident only after recovery is validated by the technical owner.
   - Notify stakeholders that the incident is resolved and summarize the current state.

5. **Communicate**
   - Keep stakeholders updated with the current impact, severity, mitigation status, and next update time.
   - Use the incident report as the source of truth and avoid sharing sensitive exploit details publicly before coordinated disclosure.
   - Notify affected users when impact is confirmed, when service is restored, and when follow-up actions materially affect them.

6. **Post-mortem**
   - Complete a blameless review after recovery, led by the incident owner.
   - Document the timeline, root cause, contributing factors, impact, what worked, and what did not.
   - Create tracked corrective actions with owners and due dates, then review completion with the relevant team.

## Roles and Responsibilities

| Role | Responsibilities |
|---|---|
| **Incident Lead** | Owns the incident end to end. Assigns severity, coordinates responders, makes mitigation decisions, and drives the incident to resolution. |
| **Technical Owner** | Investigates the root cause, proposes and applies fixes, and validates recovery. |
| **Communications Owner** | Maintains stakeholder and user updates, owns the incident report, and coordinates disclosure. |
| **Scribe** | Records the timeline, decisions, actions, and owners during the incident. |
| **Security Reviewer** | Required for any security or funds-related incident; assesses exposure and coordinates private disclosure. |
| **Executive Sponsor** | Informed for SEV1/SEV2; approves high-impact decisions such as pausing operations or public disclosure. |

For small incidents, one person may hold multiple roles, but the Incident Lead role must always be explicitly assigned.

## Escalation Procedures

Escalation follows a clear chain. Escalate immediately when severity increases, when the incident lead is unavailable, or when a decision exceeds the current responder's authority.

1. **On-call responder** — first to acknowledge the alert or report; performs initial triage and assigns severity.
2. **Incident Lead** — engaged for SEV1/SEV2 immediately, and for SEV3 when the on-call responder cannot mitigate.
3. **Technical Owner / Team Lead** — engaged when the fix requires domain expertise or cross-team coordination.
4. **Security Reviewer** — engaged for any security or funds-related incident regardless of severity.
5. **Executive Sponsor** — engaged for SEV1/SEV2, or when pausing operations or public disclosure is required.

Escalation triggers:

- Severity is raised to SEV1 or SEV2.
- No acknowledgement within the severity's response target.
- The incident is not mitigated within 30 minutes for SEV1, or 2 hours for SEV2.
- Funds, private keys, or user data may be at risk.
- A decision requires authority beyond the current responder.

## Runbooks

Runbooks for the top five common incidents. Each runbook lists the first actions to take; follow the workflow above for full handling.

### 1. Loss of Access to Funds (SEV1)

1. Confirm the report and scope: which users, which operations, and since when.
2. Assign an Incident Lead and engage the Security Reviewer immediately.
3. Pause affected operations if a safe pause mechanism exists; do not make unverified changes to contracts or keys.
4. Preserve logs, transaction hashes, and evidence before any remediation.
5. Communicate confirmed impact to affected users and stakeholders.
6. Escalate to the Executive Sponsor and coordinate disclosure per [SECURITY.md](./SECURITY.md).

### 2. Suspected Security Vulnerability or Exploit (SEV1/SEV2)

1. Treat as SEV1 until triage proves otherwise; engage the Security Reviewer.
2. Contain: disable the affected feature or apply the least risky mitigation.
3. Preserve evidence and avoid public disclosure of exploit details.
4. Follow the private reporting process in [SECURITY.md](./SECURITY.md).
5. Coordinate a verified fix, validate recovery, and plan coordinated disclosure.

### 3. Production Outage or Service Unavailability (SEV1/SEV2)

1. Confirm the outage scope from monitoring and health checks.
2. Assign an Incident Lead and identify the affected component.
3. Check recent deployments and roll back if a release correlates with the outage.
4. Apply the least risky mitigation and validate recovery with health checks.
5. Communicate status and next update time to stakeholders.

### 4. Elevated Transaction or RPC Failures (SEV3)

1. Confirm the failure rate and affected operations from monitoring.
2. Check upstream RPC providers, dependencies, and recent configuration changes.
3. Apply a workaround such as failing over to a healthy provider or reverting a config change.
4. Validate that failure rates return to baseline before resolving.

### 5. Failed Deployment or Release Regression (SEV2/SEV3)

1. Confirm the regression correlates with the most recent deployment.
2. Roll back to the last known good release.
3. Validate recovery with health checks and smoke tests.
4. Record the failing change and open a corrective action before re-attempting the release.

## Incident Report Template

Copy this template for each incident.

```markdown
# Incident: <short descriptive title>

## Summary

- **Incident ID:** <ID or link>
- **Status:** <investigating | mitigating | monitoring | resolved>
- **Severity:** <SEV1 | SEV2 | SEV3 | SEV4>
- **Incident lead:** <name or handle>
- **Technical owner:** <name or handle>
- **Detected:** <UTC timestamp>
- **Resolved:** <UTC timestamp or TBD>

## Impact

- **Affected users:** <who and how many, if known>
- **Affected components:** <contract, frontend, infrastructure, dependency, or other>
- **User impact:** <what users could not do or what they experienced>
- **Security or funds impact:** <actual or potential impact, or none known>
- **Duration:** <start and end time, or ongoing>

## Timeline

| Time (UTC) | Event | Owner or source |
|---|---|---|
| <timestamp> | <detection or observation> | <name, alert, or link> |
| <timestamp> | <triage finding or decision> | <name or link> |
| <timestamp> | <mitigation action> | <name or link> |
| <timestamp> | <communication or status update> | <name or link> |
| <timestamp> | <recovery validation or resolution> | <name or link> |

## Resolution and Follow-up

- **Root cause:** <what caused the incident>
- **Mitigation:** <what restored service or reduced risk>
- **Validation:** <checks confirming recovery>
- **Corrective actions:**
  - [ ] <action> - <owner> - <due date>
  - [ ] <action> - <owner> - <due date>
- **Related links:** <alerts, issues, deployments, logs, or communications>
```

Security vulnerabilities must follow the private reporting process in [SECURITY.md](./SECURITY.md). Do not include secrets, private keys, or sensitive exploit details in an incident report that is accessible to the public.