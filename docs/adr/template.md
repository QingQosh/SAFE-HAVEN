# ADR-NNN — Title

| Field | Value |
|---|---|
| **Status** | Proposed / Accepted / Superseded / Deprecated |
| **Date** | YYYY-MM-DD |
| **Deciders** | Names or GitHub handles |
| **Issue / PR** | #number or URL |
| **Supersedes** | ADR-NNN (if applicable) |
| **Superseded by** | ADR-NNN (fill in when this ADR is superseded) |

---

## Context

Describe the problem or situation that prompted this decision. Include:

- What is the current behavior and why it is insufficient or problematic?
- What constraints apply (budget, compatibility, security, team size)?
- What assumptions are you making? Name them explicitly — a future reader
  needs to know which ones changed when this ADR is revisited.
- Reference any related issues, PRs, or prior ADRs.

Keep this section factual and neutral. Save the justification for the
**Decision** section.

---

## Decision

State the choice clearly in one or two sentences, then expand.

- What exactly is being changed?
- What is explicitly **not** changing (important for scope)?
- If this is a pattern or convention (not just a one-off change), describe
  the rule.

Include relevant code snippets, data-model diagrams, or API signatures if
they help clarify the decision.

---

## Consequences

### Positive

- List concrete benefits: performance, safety, correctness, simplicity.
- Be specific. "Easier to read" is weaker than "removes 40 lines of
  indirection in the withdraw hot path."

### Negative / Risks

- List tradeoffs, known limitations, and open questions.
- Note any migration cost or compatibility impact for existing deployments.
- Describe what would need to happen to reverse this decision if it turns
  out to be wrong.

---

## Alternatives Considered

For each meaningful alternative you evaluated, document:

### Alternative A — Short title

What it is, why it was considered, and why it was **not** selected.
Be honest about the tradeoffs: an alternative that was genuinely close should
say so. One-line dismissals ("too complex") are not useful to a future reader.

### Alternative B — Short title

(Repeat as needed. Omit this section only if no meaningful alternatives
existed.)

---

## References

- Relevant source files (e.g., `contracts/safe-haven/src/storage.rs`)
- Related issues or PRs
- External specifications or prior art
- Any ADRs this one depends on or supersedes
