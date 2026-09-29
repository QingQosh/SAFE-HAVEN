---
name: Technical Debt
about: Report a known technical debt item so it can be tracked, prioritized, and resolved
title: "debt: <short description>"
labels: tech-debt
assignees: ""
---

<!--
  Use this template to document technical debt discovered in SAFE-HAVEN.
  Fill in every section. If a field genuinely does not apply, write "N/A" rather than leaving it blank.
  After filing the issue, add a row to DEBT.md with this issue number in the "Linked Issue" column.
-->

## Description

<!-- What is the debt? Describe the problem concisely. Focus on what exists today,
     not what the fix would look like. -->


## Location

<!-- File path(s) and line number(s) where the debt lives.
     Use the format: `path/to/file.rs:LINE` or a range `path/to/file.rs:LINE_START–LINE_END`.
     List multiple locations if the debt spans several files. -->


## Severity

<!-- Choose one and delete the others. -->
- [ ] 🔴 **High** — Security or correctness risk; likely to cause a production incident or data loss if not addressed.
- [ ] 🟡 **Medium** — Degrades developer experience, correctness under edge cases, or blocks future features.
- [ ] 🟢 **Low** — Cosmetic, ergonomic, or future risk only; no immediate impact on correctness or users.

## Effort Estimate

<!-- Estimated hours for a single developer to resolve this item end-to-end,
     including tests and documentation updates. Give a single integer. -->

**Estimated effort:** ___ hours

## Impact

<!-- What goes wrong, or gets harder, if this debt is never resolved?
     Be specific: which users, operations, or code paths are affected? -->


## Proposed Fix

<!-- Describe the approach you would take to resolve the debt.
     This does not need to be a final design — a high-level direction is sufficient.
     If you are unsure, write "Unknown — investigation needed." -->


## Links to Related Issues / PRs

<!-- Reference any GitHub issues, PRs, or ADRs related to this item.
     Use `#NNN` syntax so GitHub creates automatic cross-links. -->

- Related issues: 
- Related PRs: 
- ADRs: 

## Additional Context

<!-- Screenshots, log excerpts, test output, or any other supporting information.
     Delete this section if not needed. -->
