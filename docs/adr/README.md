# Architecture Decision Records

Architecture Decision Records (ADRs) capture decisions that materially
affect the system's structure, on-chain storage, public contract interface,
security model, or long-term maintenance. They explain *why* a choice was
made so future contributors can change it deliberately rather than
accidentally.

## Index

| ADR | Title | Status |
|---|---|---|
| [ADR-001](ADR-001-dual-deposit-types.md) | Dual deposit types: timestamp vs ledger sequence | Accepted |
| [ADR-002](ADR-002-storage-layout.md) | Storage layout and `cancel_deposit` error semantics | Accepted |
| [ADR-003](ADR-003-admin-transfer-and-renunciation.md) | Two-step admin transfer and permanent renunciation | Accepted |
| [ADR-004](ADR-004-wasm-size-budget.md) | WASM size budget and optimization pipeline | Accepted |
| [ADR-005](ADR-005-early-exit-penalty-model.md) | Early-exit penalty model and fee recipient | Accepted |

[ADR template](template.md)

---

## When to write an ADR

An ADR is **required** when a PR:

- Changes the public contract interface (adds, removes, or renames
  functions or parameters)
- Changes the on-chain storage layout or key structure
- Changes the security model (auth, access control, or trust assumptions)
- Introduces a new architectural pattern that future contributors must follow
- Supersedes or reverts a previous ADR

An ADR is **not required** for routine fixes, tests, documentation, isolated
refactors, or dependency updates, unless one of the above categories applies.

If you are unsure, write a short draft and ask in the PR. It is easier to
discard a draft than to reconstruct the reasoning months later.

---

## Process

### Creating a new ADR

1. Assign the next available number from the index above.
2. Copy [template.md](template.md) to `ADR-NNN-short-title.md`.
3. Fill in the **Context**, **Decision**, **Consequences**, and
   **Alternatives Considered** sections. Be specific: vague ADRs provide
   little value.
4. Open the ADR alongside the implementation PR. The ADR and the code it
   documents should be reviewed and merged together.
5. After approval, set `Status` to `Accepted` and add the ADR to the index
   table above.

### Superseding an ADR

If a later decision overrides an existing one:

1. Create a new ADR using the normal process.
2. In the old ADR, set `Status` to `Superseded` and populate the
   `Superseded by` field with the new ADR number.
3. In the new ADR, populate the `Supersedes` field with the old ADR number.
4. Keep the old ADR file — do not delete it. Historical decisions are
   valuable context even when they are no longer active.

### Deprecating an ADR

If the feature or component an ADR describes is removed entirely, set its
`Status` to `Deprecated` and note the removal in the ADR file. Keep the
file in the repository.

---

## Referencing ADRs in code and PRs

When you write code that directly implements or depends on a decision
documented in an ADR, add a reference comment:

```rust
// Shared deposit-ID counter ensures IDs are globally unique per depositor
// regardless of deposit type. See docs/adr/ADR-001-dual-deposit-types.md.
VaultKey::DepositCounter(depositor)
```

In PR descriptions, link the relevant ADR:

```
This change implements the two-stage admin transfer described in
docs/adr/ADR-003-admin-transfer-and-renunciation.md.
```

---

## Review and maintenance

Reviewers should check that:

- The **Context** accurately describes the problem and constraints.
- The **Decision** is unambiguous and scoped.
- **Consequences** name both positive and negative tradeoffs honestly.
- **Alternatives Considered** documents the meaningful alternatives that
  were evaluated — not just strawmen.
- The ADR is numbered correctly and added to the index.

The ADR index is the source of truth for active and superseded decisions.
Maintainers should review the index periodically and propose updates via the
[governance process](../../GOVERNANCE.md) when the project's architectural
direction changes significantly.
