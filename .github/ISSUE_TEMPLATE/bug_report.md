---
name: Bug report
about: Report unexpected behaviour, a crash, or incorrect output
title: "fix: "
labels: "bug, status: needs-reproduction"
assignees: ""
---

<!--
  Before opening a new bug report, please search existing issues to avoid duplicates.
  For security vulnerabilities (fund loss, auth bypass, storage manipulation) — do NOT use
  this template. Follow the private disclosure process in SECURITY.md instead.
-->

## Summary

<!-- A clear, one-sentence description of the bug. -->

## Steps to Reproduce

<!-- Provide the minimal steps to reproduce the issue. The clearer this is, the faster it gets fixed. -->

1. 
2. 
3. 

## Expected Behaviour

<!-- What should have happened. -->

## Actual Behaviour

<!-- What actually happened. Paste error output, panic messages, or incorrect values here. -->

## Environment

| Field | Value |
|---|---|
| Version / commit | <!-- e.g. 0.1.0 or git sha --> |
| Rust toolchain | <!-- run: rustup show active-toolchain --> |
| soroban-cli version | <!-- run: stellar --version --> |
| Node / npm (if frontend) | <!-- run: node --version && npm --version --> |
| OS | <!-- e.g. Ubuntu 24.04, macOS 14 --> |
| Network | <!-- local / testnet / mainnet --> |

## Affected Component

<!-- Check all that apply -->

- [ ] Smart contract (Rust / Soroban)
- [ ] Frontend (React / TypeScript)
- [ ] CI / build tooling
- [ ] Documentation

## Severity (self-assessment)

<!-- The triage team will confirm this. Pick the highest that applies. -->

- [ ] **S1 — Critical** — Funds at risk, auth bypass, or contract panic
- [ ] **S2 — High** — Major incorrect behaviour or storage risk
- [ ] **S3 — Medium** — Incorrect output, minor UX failure
- [ ] **S4 — Low** — Cosmetic, docs, or unreachable-code issue

## Additional Context

<!-- Logs, screenshots, XDR blobs, transaction hashes, or links to related issues. -->

---

<!-- 
  ─────────────────────────────────────────────────────────────
  FOR MAINTAINERS — complete this section during triage
  ─────────────────────────────────────────────────────────────
-->

<details>
<summary>Triage Checklist (maintainers only)</summary>

### Triage Checklist

**Reproducibility**
- [ ] Bug reproduced locally with the steps provided
- [ ] Reproduction steps added to the issue if missing
- [ ] Minimal failing test or code snippet attached (or noted as unnecessary)

**Classification**
- [ ] Severity label applied (`severity: critical / high / medium / low`)
- [ ] Priority label applied (`priority: urgent / high / medium / low`)
- [ ] Component label applied (`component: contract / frontend / ci / docs`)
- [ ] Regression label applied if this was working in a previous release

**Ownership & Planning**
- [ ] Owner assigned (GitHub Assignee field)
- [ ] Effort estimated and added to the issue (XS / S / M / L / XL)
- [ ] Milestone / sprint set

**Security Gate**
- [ ] Confirmed this is NOT a security vulnerability requiring private disclosure
  (If it is, move to private channel immediately — see SECURITY.md)

**Context**
- [ ] Affected versions listed
- [ ] Workaround documented in the issue if one exists
- [ ] Linked to related issues or PRs

See [BUG_TRIAGE.md](../../BUG_TRIAGE.md) for the full triage process and SLA targets.

</details>
