# SAFE-HAVEN — Technical Debt Register

## Purpose

This file is the single authoritative register of known technical debt in the SAFE-HAVEN project. It exists to make debt visible, prioritised, and actionable so it doesn't accumulate silently.

**How to use it**

- Before starting a sprint, review the Open items and pull the highest-priority ones that fit the sprint budget.
- When you discover new debt during a review or implementation, add a row immediately rather than filing a mental note.
- When debt is resolved, update the Status column to `resolved` and record the closing PR/commit in the Linked Issue cell.
- Do not delete resolved rows — they serve as an audit trail.

**Update cadence**

- Ad hoc: any contributor may add a new row at any time.
- Sprint boundary: the tech lead reviews the table at every sprint planning session.
- Quarterly: full review as described in the [Quarterly Review Process](#quarterly-review-process) section below.

---

## Debt Items

| ID | Title | Severity | Effort (h) | Impact | Location | Linked Issue | Status |
|----|-------|----------|------------|--------|----------|--------------|--------|
| TD-001 | Duplicate `DepositType` enum defined four times in `types.rs` | 🟡 Medium | 1 | Confuses the compiler (only the last definition wins silently), causes reader confusion, makes `#[contracttype]` codegen unpredictable | `contracts/safe-haven/src/types.rs:20, 36, 144, 175` | — | open |
| TD-002 | No frontend UI for ledger-based deposits (`deposit_by_ledger`) | 🟡 Medium | 8 | Users who want block-number-gated locks must fall back to the Stellar CLI or write custom SDK code; documented in README "Known Limitations" | `frontend/src/pages/DepositPage.tsx` | — | open |
| TD-003 | `time_remaining()` returns an estimate for ledger-based deposits, not a guaranteed wall-clock time | 🟢 Low | 2 | UI may display slightly inaccurate countdowns (±1–2 s per ledger); already documented in README but no code-level warning emitted | `contracts/safe-haven/src/contract.rs` (time_remaining), `contracts/safe-haven/src/storage.rs:20` (LEDGER_SECONDS) | — | open |
| TD-004 | `get_vault()` returns `None` for ledger-based deposits | 🟡 Medium | 3 | Frontend callers that call `get_vault` for any deposit ID will silently miss all ledger-based deposits; must instead call `get_ledger_vault`; documented in README "Known Limitations" but no guard or deprecation hint | `contracts/safe-haven/src/contract.rs` (get_vault impl) | — | open |
| TD-005 | `get_deposits_page()` excludes ledger-based and multi-token deposits | 🟡 Medium | 4 | Admin and analytics views that rely on the paginated flat list (`get_deposits_page`) are blind to all ledger-based and multi-token vaults; `contracts/safe-haven/src/contract.rs:2100` only iterates `get_deposit_readonly` (timestamp-based) | `contracts/safe-haven/src/contract.rs:2100–2136` | — | open |
| TD-006 | No contract version pinning in frontend — `useContractInfo` loads `storageVersion` as `number` but `getContractVersion()` returns `string` | 🔴 High | 3 | Frontend has no mechanism to detect that the connected contract ID belongs to an incompatible binary version; `version: number \| null` in the hook interface conflates the storage schema version (u32) with the semantic contract version (string); silent type mismatch can hide version drift | `frontend/src/hooks/useContractInfo.ts:14, 27`, `frontend/src/lib/stellar.ts:220` | #399 (in-progress) | in-progress |
| TD-007 | No maximum lock-duration enforcement in `deposit_by_ledger` | 🟡 Medium | 2 | `deposit()` and `deposit_for()` reject lock durations longer than `max_lock_secs` (default 5 years). `deposit_by_ledger` only enforces a minimum gap of 12 ledgers (`MIN_LOCK_LEDGERS`); arbitrarily large `unlock_ledger` values are accepted, creating vaults that will never be reachable within TTL lifespan | `contracts/safe-haven/src/contract.rs:592–655` | — | open |
| TD-008 | `ANNUAL_INTEREST_BPS` is a hardcoded module-level constant; not admin-configurable | 🟢 Low | 4 | The compound interest rate (currently 5 % p.a.) cannot be updated without a contract redeploy; a governance vote or admin setter would allow rate adjustments without migration | `contracts/safe-haven/src/contract.rs:26` | — | open |
| TD-009 | No end-to-end test covering the full `cancel_deposit` → `claim_staker_rewards` accounting flow | 🟡 Medium | 4 | Unit tests for `cancel_deposit` and `claim_staker_rewards` exist in isolation (`test.rs:3596–3810`) but no single test verifies the complete penalty-split invariant: `fee_recipient_balance + stakers_pool + depositor_refund == original_amount` after a cancel | `contracts/safe-haven/src/test.rs` | — | open |
| TD-010 | README error-code table lists staker error codes (16–19) that do not exist in `errors.rs` | 🔴 High | 1 | `errors.rs` defines codes 16–19 as `TooManyTokens`, `EmptyTokenList`, `RecipientNotWhitelisted`, `InvalidCompoundFrequency`. The README table claims 16=`InvalidStakeAmount`, 17=`StakerNotFound`, 18=`NoRewardsToClaim`, 19=`InsufficientStakeAmount`. Any developer relying on the README for error handling will build against the wrong codes | `README.md` (Error Codes table), `contracts/safe-haven/src/errors.rs:16–19` | — | open |
| TD-011 | `STORAGE_VERSION=1` has no documented upgrade path in code; `migrate()` body is a no-op placeholder | 🟡 Medium | 3 | `types.rs:13` sets `STORAGE_VERSION=1`. The `migrate()` function (`contract.rs:2141`) only sets the version key — it performs no actual data transformation. There is no inline comment or ADR reference explaining what v1→v2 migration would entail, raising risk when a schema change requires a real migration | `contracts/safe-haven/src/types.rs:13`, `contracts/safe-haven/src/contract.rs:2141–2153` | — | open |
| TD-012 | No automated doc generation from contract source in CI | 🟢 Low | 2 | `make doc` runs `cargo doc --no-deps --open` locally but there is no CI step that publishes or validates rustdoc output; public API documentation can silently go stale | `Makefile:112–113`, `.github/workflows/ci.yml` | — | open |
| TD-013 | Several `devDependencies` in `frontend/package.json` use caret (`^`) ranges instead of pinned versions | 🟢 Low | 1 | `@playwright/test`, `@testing-library/react`, `@testing-library/user-event`, `happy-dom`, `vitest` all use `^` ranges; patch or minor updates could silently break CI without a version bump in the lockfile | `frontend/package.json:26–46` | — | open |
| TD-014 | `PublicWifiWarning` component makes unauthenticated requests to `ip-api.com` free tier with no retry budget or fallback on rate-limit (45 req/min) | 🟢 Low | 2 | If the free tier limit is hit the component silently swallows the error; there is no fallback UI state nor a user-facing warning that geolocation is unavailable | `frontend/src/components/PublicWifiWarning.tsx:21` | — | open |

---

## Prioritization Matrix

The matrix maps debt items by **Impact** (vertical axis) and **Effort** (horizontal axis). Items in the top-left quadrant (high impact, low effort) should be addressed first.

```
             LOW EFFORT (< 4 h)          HIGH EFFORT (≥ 4 h)
           ┌───────────────────────────┬───────────────────────────┐
HIGH       │  DO FIRST                 │  PLAN & SCHEDULE          │
IMPACT     │  TD-006  (version pinning)│  TD-002  (ledger UI)      │
           │  TD-010  (error code docs)│  TD-009  (e2e staker test)│
           │  TD-007  (max lock ledger)│  TD-005  (page excludes)  │
           │  TD-011  (migrate no-op)  │                           │
           ├───────────────────────────┼───────────────────────────┤
LOW        │  DO WHEN CONVENIENT       │  DEFER / BACKLOG          │
IMPACT     │  TD-001  (dup enum)       │  TD-008  (configurable    │
           │  TD-003  (estimate warn)  │          interest rate)   │
           │  TD-004  (get_vault hint) │                           │
           │  TD-012  (rustdoc CI)     │                           │
           │  TD-013  (dep pinning)    │                           │
           │  TD-014  (ip-api fallback)│                           │
           └───────────────────────────┴───────────────────────────┘
```

---

## Sprint Allocation

**Policy:** 10 % of every sprint's story-point budget is reserved for debt reduction.

- In a 20-point sprint this means 2 points (≈ 2–4 hours) are pre-allocated to debt items before feature work is scheduled.
- The sprint tech lead picks debt items from the top-left quadrant of the prioritization matrix unless the team agrees on a different priority.
- Debt work is tracked in the same issue tracker as features; every resolved debt item must reference this file's ID in the PR description (e.g. `Closes TD-001`).

---

## Quarterly Review Process

**When:** The first sprint planning session of each calendar quarter (Q1 = January, Q2 = April, Q3 = July, Q4 = October).

**Who:** Tech lead + at least one domain expert (contract dev for Rust items, frontend dev for TypeScript items).

**Agenda:**
1. Review all `open` items — re-assess severity and effort if circumstances have changed.
2. Retire `resolved` items older than two quarters by archiving them to `DEBT_ARCHIVE.md`.
3. Scan the codebase for `// TODO`, `// FIXME`, `// HACK`, and open GitHub issues tagged `tech-debt`; add any new items found.
4. Update the Prioritization Matrix based on the current roadmap.
5. Confirm the 10 % sprint budget policy is being followed; adjust if velocity has changed.

**Outcome:** A short (≤ 1 page) summary committed to `CHANGELOG.md` under the heading `## Debt Review — YYYY-QN`.

---

## Definition of Done for Debt Items

A debt item is `resolved` when **all** of the following are true:

1. **Root cause removed** — the underlying code or configuration that caused the debt has been changed, not merely commented or suppressed.
2. **Tests updated** — any test that validated the old (broken) behaviour is updated or removed; new tests cover the corrected behaviour where applicable.
3. **Documentation updated** — README, ADRs, or inline comments that referenced the old behaviour are updated to reflect the new state.
4. **CI passes** — `make check` (fmt + lint + test + audit + deny) passes on the PR branch.
5. **PR merged** — the change is merged to the main branch, not just pushed to a feature branch.
6. **This file updated** — the Status column in this table is changed to `resolved` and the Linked Issue cell references the closing PR number.

Partial fixes (e.g. documenting a workaround without removing the root cause) do not count as resolved. They may reduce severity; update the Severity column accordingly and add a note to the Title.

---

## How to Add New Items

When you discover technical debt, add a new row to the [Debt Items](#debt-items) table immediately. Use the next available `TD-NNN` ID.

**Required fields:**

| Field | Rules |
|---|---|
| **ID** | Sequential `TD-NNN`. Check existing rows to find the next number. |
| **Title** | One sentence. State what the debt is, not a fix. Bad: "Fix duplicate enum." Good: "Duplicate `DepositType` enum defined four times in `types.rs`." |
| **Severity** | 🔴 High (security/correctness risk or likely to cause a production incident), 🟡 Medium (degrades developer experience or correctness under edge cases), 🟢 Low (cosmetic, ergonomic, or future-risk only). |
| **Effort (h)** | Integer hours for a single developer to resolve the item end-to-end, including tests and documentation. |
| **Impact** | One sentence describing what goes wrong if the debt is not addressed. |
| **Location** | File path relative to repo root, with line number(s) where possible. If the debt spans multiple files, list the most important one and note "and others." |
| **Linked Issue** | GitHub issue number (`#NNN`) if one exists, or `—` if not yet filed. Create the issue using the [technical debt issue template](.github/ISSUE_TEMPLATE/technical_debt.md). |
| **Status** | `open`, `in-progress`, or `resolved`. |

If you are uncertain whether something qualifies as debt, bias toward adding it. It is easier to remove a row than to rediscover a problem.
