# Documentation Contribution Guide

This guide explains how SAFE-HAVEN documentation is organised, how to keep it
up to date when you contribute code, and how the CI pipeline enforces quality.

---

## Table of Contents

1. [How documentation is organised](#1-how-documentation-is-organised)
2. [Naming and formatting conventions](#2-naming-and-formatting-conventions)
3. [Writing Rust doc comments](#3-writing-rust-doc-comments)
4. [Updating API.md when adding contract functions](#4-updating-apimd-when-adding-contract-functions)
5. [Updating CHANGELOG.md](#5-updating-changelogmd)
6. [How the CI doc-lint job catches issues](#6-how-the-ci-doc-lint-job-catches-issues)
7. [How docs are deployed to GitHub Pages](#7-how-docs-are-deployed-to-github-pages)
8. [Doc versioning strategy](#8-doc-versioning-strategy)
9. [Spell check word list and adding project-specific terms](#9-spell-check-word-list-and-adding-project-specific-terms)

---

## 1. How documentation is organised

Documentation lives in four places:

| Location | Purpose |
|---|---|
| `README.md` | Project overview, quick-start, contract API summary, use cases |
| `CHANGELOG.md` | Human-readable history of every release, following Keep a Changelog |
| `docs/` | Reference documentation, ADRs, and operator guides |
| Rust `///` comments | Inline API documentation extracted by `cargo doc` |

### `docs/` directory layout

```
docs/
├── API.md                        Full contract function reference
├── DOC_CONTRIBUTION_GUIDE.md     ← this file
├── LEGACY_CONTRACT_MIGRATION.md  Migration guide for old contract versions
├── OPERATOR_PERFORMANCE.md       Tuning and monitoring guidance
├── ROADMAP.md                    Planned features and milestones
├── USER_ONBOARDING.md            End-user getting-started guide
└── adr/                          Architecture Decision Records
    ├── ADR-001-dual-deposit-types.md
    └── ADR-002-storage-layout.md
```

New reference documents belong in `docs/`. ADRs (decisions that are hard to
reverse or that explain *why* a design was chosen) belong in `docs/adr/` and
must follow the ADR template.

---

## 2. Naming and formatting conventions

### File names

- Use `UPPER_SNAKE_CASE.md` for top-level reference documents
  (e.g. `API.md`, `OPERATOR_PERFORMANCE.md`).
- Use `ADR-NNN-short-description.md` for Architecture Decision Records.
- Use lowercase with hyphens for any supplementary files that are not primary
  references (e.g. `getting-started.md`).

### Markdown style

- Use ATX-style headings (`#`, `##`, `###`), not underline-style.
- Use a single blank line between sections.
- Wrap long lines at 100 characters where practical, but never break a URL.
- Use fenced code blocks with a language identifier:

  ````md
  ```rust
  pub fn my_function() {}
  ```
  ````

- Use pipe tables for structured data; keep them readable with aligned columns.
- Use `**bold**` only to call out critical warnings or parameter names inline.
  Avoid bold for emphasis in prose — use plain text.
- Every new `.md` file must have a level-1 heading as its first line.

---

## 3. Writing Rust doc comments

All `pub` items in the contract crate must have a `///` doc comment.

### Minimal example

```rust
/// Returns the current ledger timestamp in seconds since the Unix epoch.
pub fn get_time(env: Env) -> u64 {
    env.ledger().timestamp()
}
```

### Full example with sections

```rust
/// Locks tokens in a vault until the given timestamp.
///
/// # Parameters
///
/// - `depositor` — Account locking the tokens. Must sign the transaction.
/// - `token`     — SAC-compatible token contract address.
/// - `amount`    — Amount to lock. Must be > 0 and ≤ `max_deposit`.
/// - `unlock_time` — Unix timestamp at or after which withdrawal is permitted.
///   Must be at least 60 seconds in the future.
/// - `penalty_bps` — Early-exit penalty in basis points (0–10 000).
///   Requires a `fee_recipient` to be configured when > 0.
///
/// # Returns
///
/// The deposit ID (`u32`), unique per depositor.
///
/// # Errors
///
/// - [`VaultError::InvalidAmount`] — `amount` is ≤ 0 or exceeds `max_deposit`.
/// - [`VaultError::LockDurationTooShort`] — Lock is less than 60 seconds.
/// - [`VaultError::LockDurationTooLong`] — Lock exceeds the configured maximum.
/// - [`VaultError::MissingFeeRecipient`] — `penalty_bps > 0` but no fee recipient set.
/// - [`VaultError::ContractPaused`] — Deposits are currently paused.
///
/// # Example
///
/// ```rust,ignore
/// let deposit_id = vault.deposit(&alice, &token, &1_000_000, unlock_ts, 500)?;
/// ```
pub fn deposit(
    env: Env,
    depositor: Address,
    token: Address,
    amount: i128,
    unlock_time: u64,
    penalty_bps: u32,
) -> Result<u32, VaultError> {
    // …
}
```

### Guidelines

- Start the first line with a verb in imperative mood: *"Returns"*, *"Locks"*, *"Registers"*.
- Document every parameter that is not self-evident.
- List every `VaultError` variant the function can return, with the condition.
- Include a short `# Example` block using `rust,ignore` so that rustdoc renders
  it without trying to compile it (Soroban requires a runtime environment).
- Do not repeat the function name in the comment; readers can see it.

---

## 4. Updating API.md when adding contract functions

`docs/API.md` is the canonical human-readable reference for the contract's
public interface. The CI `check-doc-code-sync` job fails if any `pub fn` name
in `contract.rs` is absent from `API.md`.

### Steps for a new function

1. **Add the `///` doc comment** in `contract.rs` (see §3).

2. **Choose the right section** in `API.md`:
   - Initialization → `### Initialization`
   - Core deposit / withdraw flows → `### Core Functions`
   - Admin operations → `### Admin Functions`
   - Staker registry → `### Staker Registry Functions`
   - Read-only queries → `### Read-only Queries`
   - New logical grouping → add a new `###` subsection.

3. **Use the standard entry format**:

   ```markdown
   #### `function_name(param1, param2, ...) -> ReturnType`

   Short description of what the function does.

   **Parameters**

   | Parameter | Type | Description |
   |---|---|---|
   | `param1` | `Type` | What it controls. |
   | `param2` | `Type` | What it controls. |

   **Returns** description of the return value.

   **Errors** — bullet list of `VaultError` variants that can be returned.
   ```

4. **Update the Error Codes table** in `API.md` if the function introduces a
   new `VaultError` variant.

5. **Update README.md** if the function is significant enough to appear in the
   overview table or the *How It Works* section.

---

## 5. Updating CHANGELOG.md

SAFE-HAVEN follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [Semantic Versioning](https://semver.org/).

### Format

```markdown
## [Unreleased]

### Added
- `deposit_with_compound_interest` — new deposit variant that accrues 5% p.a.

### Changed
- `cancel_deposit` now splits penalties 30/70 between fee recipient and staker pool.

### Fixed
- `withdraw_to` no longer emits a duplicate `Withdrawn` event when `recipient == depositor`.

### Deprecated
- `deposit_for` parameter order will change in v2.0. Use named parameters in the SDK.

### Removed
- Nothing.

### Security
- Hardened `require_auth()` placement in `register_staker` to be strictly first.
```

### Rules

- Every non-trivial PR must add an entry under `## [Unreleased]`.
- Pure docs or CI changes (no contract or frontend logic changed) are exempt.
- Use the subsection that best describes the change: `Added`, `Changed`,
  `Fixed`, `Deprecated`, `Removed`, or `Security`.
- Keep entries concise — one line per change is ideal. Add a brief rationale
  if the change is non-obvious.
- Do not modify released version sections after they are tagged.

### Releasing a version

When cutting a release:

1. Replace `## [Unreleased]` with `## [X.Y.Z] – YYYY-MM-DD`.
2. Add a new empty `## [Unreleased]` section at the top.
3. Add the comparison link at the bottom of the file:
   ```
   [X.Y.Z]: https://github.com/kenedybok3/SAFE-HAVEN/compare/vX.Y.(Z-1)...vX.Y.Z
   ```

---

## 6. How the CI doc-lint job catches issues

The `.github/workflows/docs.yml` workflow runs two doc-related jobs on every
pull request targeting `main` or `develop`.

### `doc-lint`

| Check | What it does | Failure behaviour |
|---|---|---|
| CHANGELOG check | Diffs `CHANGELOG.md` against the base branch; warns if non-docs files changed but `[Unreleased]` was not updated | Warning (non-fatal) |
| Spell check | Runs `npx cspell` against `docs/**/*.md`, `README.md`, and `CHANGELOG.md` | Hard failure — fix typos or add the word to `.cspell.json` |
| PR description | Counts non-whitespace characters in the `## Summary` section; fails if fewer than 50 | Hard failure — fill in the summary |

### `check-doc-code-sync`

Greps every `pub fn` name from `contracts/safe-haven/src/contract.rs` and
checks that the name appears somewhere in `docs/API.md`. If any name is
missing the job fails and lists the undocumented functions.

**When to expect this job to fail**: any PR that adds a new `pub fn` to
`contract.rs` without adding the function name to `API.md`.

**How to fix**: follow §4 above to add the entry to `API.md`.

---

## 7. How docs are deployed to GitHub Pages

On every push to `main` the `docs.yml` workflow runs two additional jobs:

1. **`rust-docs`** — runs `cargo doc --no-deps` with `RUSTDOCFLAGS="-D warnings"`,
   then copies `target/doc/` to a `public/` directory and uploads it as a
   GitHub Pages artifact using `actions/upload-pages-artifact@v3`.  A thin
   redirect at `public/index.html` forwards the root URL to
   `safe_haven/index.html`.

2. **`deploy-docs`** — depends on `rust-docs` and uses
   `actions/deploy-pages@v4` to publish the artifact to the `pages`
   environment.

### Prerequisites (one-time repo setup)

- In **Settings → Pages**, set the *Build and deployment* source to
  **GitHub Actions**.
- Ensure the `pages` environment exists under **Settings → Environments**.
- The workflow already requests `pages: write` and `id-token: write`
  permissions in the `deploy-docs` job.

### Viewing the deployed docs

After a successful deployment the URL is reported in the job summary and in the
`pages` environment. It will be of the form:

```
https://<org>.github.io/SAFE-HAVEN/safe_haven/index.html
```

---

## 8. Doc versioning strategy

Soroban contracts are immutable; each deployment creates a new contract ID.
Documentation for past contract versions is preserved via **git tags**.

### How it works

Every release is tagged `vX.Y.Z` in git. To read the documentation as it was
at that release:

```bash
git checkout vX.Y.Z
# Then open docs/ or run cargo doc locally
```

GitHub automatically archives a snapshot of the repository at each tag.  The
tag is listed under **Releases** and can be browsed directly on GitHub.

### What to do for breaking changes

1. Tag the last compatible release before the breaking change lands.
2. Document the breaking change in `CHANGELOG.md` under `### Changed` or
   `### Removed`.
3. Add a migration guide in `docs/LEGACY_CONTRACT_MIGRATION.md` (or create a
   new `docs/MIGRATION_vX_to_vY.md` for large migrations).
4. Update `README.md` to reference the migration guide.

### GitHub Pages versioning

The current GitHub Pages deployment always reflects the `main` branch.  If you
need to serve multiple doc versions simultaneously (e.g. v1 and v2 in
parallel), set up a separate branch per version (e.g. `docs/v1`) and configure
a separate Pages workflow for it.  This is not required for the current release
cadence but is supported by the infrastructure.

---

## 9. Spell check word list and adding project-specific terms

The spell checker is configured in `.cspell.json` at the repository root.

### Configuration overview

```jsonc
{
  "version": "0.2",
  "language": "en",
  "files": ["**/*.md", "**/*.ts", "**/*.tsx", "**/*.rs"],
  "ignorePaths": ["node_modules", "target", "*.lock", "frontend/package-lock.json"],
  "words": [
    "soroban", "stellar", "testnet", "mainnet", "wasm", ...
  ]
}
```

The `words` array is the project-specific allow-list.  Words in this list are
never flagged, regardless of capitalisation.

### How to add a new term

1. Open `.cspell.json`.
2. Add the term to the `words` array in **lower case** (cspell matches
   case-insensitively by default):

   ```json
   "words": [
     "soroban",
     "mynewterm"
   ]
   ```

3. If the term has a specific capitalisation that must be preserved (e.g. a
   proper noun like `Freighter`), add it in the exact case you need:

   ```json
   "words": [
     "Freighter"
   ]
   ```

4. Commit `.cspell.json` alongside the PR that introduces the new term.

### Running the spell checker locally

```bash
# Install once
npm install -g cspell@8

# Check all configured files
npx cspell "docs/**/*.md" README.md CHANGELOG.md --config .cspell.json

# Check a single file
npx cspell docs/API.md --config .cspell.json --show-suggestions
```

Fix genuine typos first.  Only add a word to the allow-list if it is a
legitimate project term, a proper noun, or an acronym that cspell does not
recognise.
