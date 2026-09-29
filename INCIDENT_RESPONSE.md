# Incident Response

This document defines how SAFE-HAVEN incidents are classified, managed, and documented.

## Severity Levels

| Severity | Definition | Examples |
|---|---|---|
| **Critical** | Users are currently affected, or funds or security are at immediate risk. | Loss of access to funds, active exploit, or a production-wide outage. |
| **High** | There is potential for significant user, operational, or security impact even if users are not yet affected. | A confirmed vulnerability without evidence of exploitation, or a failure likely to affect withdrawals. |
| **Medium** | The service is degraded, but the primary user workflow remains available or a workaround exists. | Elevated transaction failures, slow RPC responses, or partial feature unavailability. |
| **Low** | A minor issue with limited scope, little or no user impact, and no material security risk. | Cosmetic defects, isolated non-blocking errors, or documentation issues. |

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

4. **Communicate**
   - Keep stakeholders updated with the current impact, severity, mitigation status, and next update time.
   - Use the incident report as the source of truth and avoid sharing sensitive exploit details publicly before coordinated disclosure.
   - Notify affected users when impact is confirmed, when service is restored, and when follow-up actions materially affect them.

5. **Post-mortem**
   - Complete a blameless review after recovery, led by the incident owner.
   - Document the timeline, root cause, contributing factors, impact, what worked, and what did not.
   - Create tracked corrective actions with owners and due dates, then review completion with the relevant team.

## Incident Report Template

Copy this template for each incident.

```markdown
# Incident: <short descriptive title>

## Summary

- **Incident ID:** <ID or link>
- **Status:** <investigating | mitigating | monitoring | resolved>
- **Severity:** <critical | high | medium | low>
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