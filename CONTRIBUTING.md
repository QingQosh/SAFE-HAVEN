# Contributing to SAFE-HAVEN

Thank you for your interest in contributing. This document describes how to
set up your environment, the conventions this project follows, and what a PR
needs to look like before it can be merged.

---

## Table of Contents

1. [Code of Conduct](#1-code-of-conduct)
2. [Project Roles](#2-project-roles)
3. [Before You Start](#3-before-you-start)
4. [Development Environment](#4-development-environment)
5. [Git Workflow](#5-git-workflow)
6. [Coding Standards](#6-coding-standards)
7. [Testing Requirements](#7-testing-requirements)
8. [Commit Message Format](#8-commit-message-format)
9. [Opening a Pull Request](#9-opening-a-pull-request)
10. [PR Checklist](#10-pr-checklist)
11. [Review Process](#11-review-process)
12. [Reporting Issues](#12-reporting-issues)
13. [Security Vulnerabilities](#13-security-vulnerabilities)
14. [Utility Scripts](#14-utility-scripts)
15. [Pre-Commit Hooks](#15-pre-commit-hooks)
16. [Licensing](#16-licensing)

---

## 1. Code of Conduct

All contributors — whether opening a PR, reviewing code, filing an issue, or
participating in discussions — are expected to:

- Be respectful and constructive. Critique code and ideas, not people.
- Be inclusive. This project does not discriminate based on background,
  identity, or experience level.
- Keep disagreements technical. Back claims with evidence or code.
- Give clear, actionable feedback. "This is wrong" is not useful; explain
  why and suggest a better path.

Maintainers may remove comments, reject contributions, or restrict
participation when conduct violates these expectations. Serious violations
or private concerns should be reported directly to the maintainers. Security
vulnerabilities must follow the process in [SECURITY.md](SECURITY.md), not
be filed as public issues.

---

## 2. Project Roles

| Role | Responsibilities |
|---|---|
| **Contributor** | Proposes changes, implements features or fixes, responds to review feedback, follows this guide |
| **Maintainer** | Reviews and merges changes, triages issues, enforces quality and security standards, maintains the project |
| **Admin** | Manages repository settings, branch protection, CI secrets, and permissions. Admin access does not bypass review or security requirements |

Anyone may contribute. New contributors become maintainers over time based on
consistent, high-quality contributions and demonstrated understanding of the
codebase.

---

## 3. Before You Start

For any change that takes more than an hour to implement:

1. **Search existing issues** to avoid duplicating work. If an issue already
   exists, comment to claim it before starting.
2. **Open an issue** describing what you want to do if one doesn't exist.
   This avoids wasted effort if the maintainers have context that changes
   the approach.
3. **For architectural changes**, propose an ADR (see
   [docs/adr/README.md](docs/adr/README.md)) before writing code. The ADR
   describes the decision and tradeoffs; the PR implements it.
4. **For large features or breaking changes**, start a discussion in
   GitHub Discussions or open a draft PR early so maintainers can give
   direction before you invest significant time.

Small, well-scoped contributions (bug fixes, documentation, test additions)
can go straight to a PR without prior discussion.

---

## 4. Development Environment

### Prerequisites

See [GETTING_STARTED.md](GETTING_STARTED.md) for the full, OS-specific setup
guide. In summary, you need:

- Rust stable (MSRV 1.81) with the `wasm32-unknown-unknown` target
- Stellar CLI (`stellar`) 26.x
- Node.js 20 LTS + npm
- Freighter browser extension (for frontend testing)

### Install dev tools

```bash
make install-tools
```

This installs `cargo-watch`, `cargo-audit`, `cargo-deny`, and the Soroban
CLI. Run this once after cloning.

### Local builds and tests

```bash
# Contract
make build     # compile to WASM
make test      # run all unit tests
make watch     # re-run tests on file save
make check     # full CI-equivalent check

# Frontend
cd frontend
npm install
npm run dev        # start Vite dev server
npm run test       # Vitest unit tests
npm run typecheck  # TypeScript check
npm run build      # production build
```

---

## 5. Git Workflow

### Fork and clone

Fork the repository on GitHub, then clone your fork:

```bash
git clone https://github.com/YOUR_USERNAME/SAFE-HAVEN.git
cd SAFE-HAVEN
git remote add upstream https://github.com/kenedybok3/SAFE-HAVEN.git
```

### Branching

Always create a branch from `main`. Never commit directly to `main` or
`develop`.

| Type | Pattern | Example |
|---|---|---|
| New feature | `feat/<short-description>` | `feat/multi-token-support` |
| Bug fix | `fix/<short-description>` | `fix/unlock-time-overflow` |
| Documentation | `docs/<short-description>` | `docs/adr-storage-layout` |
| Chore / tooling | `chore/<short-description>` | `chore/update-dependencies` |
| CI / workflow | `ci/<short-description>` | `ci/pin-stellar-cli-version` |
| Test additions | `test/<short-description>` | `test/cancel-deposit-edge-cases` |

```bash
git checkout main
git pull upstream main
git checkout -b feat/your-feature
```

Keep branches short-lived and focused on a single concern. If you notice an
unrelated issue while working, open a separate issue rather than expanding
your PR.

### Staying up to date

```bash
git fetch upstream
git rebase upstream/main
```

Prefer `rebase` over `merge` when updating your branch from `main` to keep
a clean history. Do not force-push branches that have already been reviewed
unless explicitly agreed with the reviewer.

---

## 6. Coding Standards

### Rust (contract)

**Formatting**: run `cargo fmt --all` before every commit. CI fails on any
formatting divergence.

**Linting**: run `cargo clippy --all-targets --features testutils -- -D warnings`.
CI fails on any Clippy warning. Do not suppress warnings with
`#[allow(...)]` without a code comment explaining why.

**`require_auth()` must be first**: in every mutating contract function,
`caller.require_auth()` must be the very first meaningful statement (before
any reads, storage access, or external calls). This is a security invariant,
not a style preference.

```rust
// Correct
pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
    depositor.require_auth();   // ← first
    let entry = storage::get_deposit(...)?;
    ...
}

// Wrong — auth check after a read
pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
    let entry = storage::get_deposit(...)?;  // ← state read before auth
    depositor.require_auth();
    ...
}
```

**Cache host function results**: every call to `env.ledger().timestamp()`,
`env.ledger().sequence()`, or `env.current_contract_address()` is a
host-function invocation with a non-trivial instruction cost. Cache the
result in a `let` binding and reuse it:

```rust
// Good
let now = env.ledger().timestamp();
if now < entry.unlock_time { return Err(VaultError::FundsStillLocked); }

// Bad — calls the host twice
if env.ledger().timestamp() < entry.unlock_time { ... }
let elapsed = env.ledger().timestamp() - entry.unlock_time;
```

**Error codes**: use existing `VaultError` variants. If a new error is
needed, add it to the `VaultError` enum with the next available code and
document it in the README error-code table. Do not reuse or remove existing
codes — code numbers are part of the ABI and removing them breaks clients.

**No unsafe code**: the project is 100% safe Rust. `#[allow(unsafe_code)]`
is banned. CI runs `cargo-geiger` to enforce this.

**Integer arithmetic**: use `i128` for token amounts to match the Soroban
token interface. Use `checked_*` arithmetic or rely on Soroban's
`overflow-checks = true` release profile. Never cast to a smaller integer
type without bounds checking.

### TypeScript (frontend)

**Formatting**: the project uses ESLint with TypeScript rules. Run
`npm run lint` before committing frontend changes.

**Type safety**: do not use `any`. If the Stellar SDK returns an unknown type
that must be narrowed, write a type guard or assertion and comment why.

**React conventions**: use functional components and hooks. Do not add
class components. State that needs to be shared should go in context
(`WalletContext`) or be lifted to the nearest common ancestor.

**Environment variables**: access configuration through `src/config.ts`, not
directly via `import.meta.env`. Config validation runs at startup and
reports misconfiguration clearly.

**No console.log in production code**: use the project's existing error
handling patterns. `console.error` is acceptable for unexpected catch paths
with a comment explaining what is being logged.

### Documentation

- Update `CHANGELOG.md` under `[Unreleased]` with a concise summary of
  every user-visible change (added, changed, fixed, removed, security).
- Update the README if the public contract API changed (functions, error
  codes, known limitations).
- For architectural changes, write or update the relevant ADR in
  `docs/adr/`. See [docs/adr/README.md](docs/adr/README.md).

---

## 7. Testing Requirements

### Contract tests

Every new contract function or changed behavior **must** have a test. Tests
live in `contracts/safe-haven/src/test.rs`. The project currently has 48+
unit tests; each PR should leave this number the same or higher.

Test requirements:

- **Happy path**: the function works correctly with valid inputs.
- **Error paths**: every `VaultError` variant the function can return is
  covered by at least one test.
- **Boundary values**: test at the exact boundary (e.g.,
  `unlock_time == now + MIN_LOCK_DURATION_SECS`, `amount == MAX_DEPOSIT_AMOUNT`).
- **Auth**: every auth-gated function has a test that verifies an
  unauthorized caller is rejected.

For performance-sensitive changes, add a comment in the test explaining the
instruction-budget implications.

Run tests with:

```bash
make test
# or
cargo test --features testutils
```

> **Test snapshots**: `cargo test` may generate
> `contracts/safe-haven/test_snapshots/` containing XDR snapshots of
> contract state. These are transient artifacts and are gitignored. Do not
> commit them.

### Frontend tests

The frontend uses Vitest for unit tests and Playwright for end-to-end smoke
tests. When modifying a hook, utility, or component:

- Add or update the corresponding unit test in `frontend/src/__tests__/`.
- For significant user flows, add or update a Playwright test in
  `frontend/tests/`.

Run frontend tests:

```bash
cd frontend
npm run test      # Vitest
npx playwright test   # Playwright (requires a running frontend)
```

---

## 8. Commit Message Format

This project follows [Conventional Commits](https://www.conventionalcommits.org/).
CI enforces this format on PR titles; keep individual commit messages
consistent as well.

```
<type>(<scope>): <short summary in imperative mood>

[optional body — explain the why, not the what]

[optional footer: Closes #issue, BREAKING CHANGE: description]
```

**Types**:

| Type | When to use |
|---|---|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only |
| `test` | Test additions or corrections |
| `refactor` | Code restructuring without behavior change |
| `chore` | Tooling, dependencies, build scripts |
| `ci` | CI workflow changes |
| `security` | Security-related fix or hardening |
| `perf` | Performance improvement |

**Scope**: the part of the codebase affected (e.g., `contract`, `storage`,
`frontend`, `ci`, `adr`). Omit if the change is project-wide.

**Examples**:

```
feat(contract): add deposit_by_ledger entry point
fix(storage): correct TTL bump on emergency_withdraw
docs: update README error-code table with error 15
test(contract): add boundary tests for penalty_bps = 10000
chore(deps): update stellar-sdk to 12.3.0
ci: pin stellar-cli to 26.0.0 in build job
```

**Breaking changes** go in the footer:

```
feat(contract): rename get_vault to get_timestamp_vault

BREAKING CHANGE: get_vault has been renamed to get_timestamp_vault.
Callers must update their invocations.

Closes #99
```

---

## 9. Opening a Pull Request

1. Push your branch and open a PR against `main`:
   ```bash
   git push -u origin feat/your-feature
   ```

2. Fill in the PR description using the template. Include:
   - **What changed** and why
   - **How it was tested** (which test commands you ran, any manual steps)
   - **Screenshots** for frontend visual changes
   - `Closes #<issue-number>` if the PR resolves an open issue

3. Ensure the PR title follows [Conventional Commits](#8-commit-message-format).
   CI checks PR titles and will fail if the format is wrong.

4. Complete the [PR Checklist](#10-pr-checklist) before requesting review.

5. Do not mark the PR as ready for review until CI passes. Reviewers should
   not need to debug CI failures on your behalf.

---

## 10. PR Checklist

Before requesting review, confirm every item below:

**Code quality**
- [ ] `make check` passes locally (fmt + lint + test + audit + deny)
- [ ] `cd frontend && npm run build && npm run typecheck` passes

**Tests**
- [ ] New behavior is covered by tests
- [ ] No existing tests were removed or weakened to make CI pass

**Documentation**
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] README updated if the public contract API or known limitations changed
- [ ] ADR written or updated if this is an architectural decision
- [ ] Code comments added for non-obvious logic

**Hygiene**
- [ ] No `.env` files, secret keys, or credentials committed
- [ ] No test snapshots or build artifacts committed (`test_snapshots/`,
  `target/`, `dist/`, `node_modules/`)
- [ ] No unrelated changes bundled into this PR
- [ ] Branch is up to date with `main` (rebase if needed)

**Security** (required for contract changes)
- [ ] `require_auth()` is the first call in every new mutating function
- [ ] No `unsafe` blocks
- [ ] Token transfers happen after state changes (checks-effects-interactions)
- [ ] New storage keys do not overlap with existing ones

---

## 11. Review Process

### What reviewers check

Reviewers evaluate:

- **Correctness**: does the code do what the PR claims?
- **Security**: does the change introduce auth gaps, re-entrancy, or
  incorrect state transitions? (Checked by CODEOWNERS for contract changes.)
- **Test coverage**: are happy paths, error paths, and boundary conditions
  covered?
- **Documentation**: is the change understandable from the PR description
  and code comments alone?
- **Backward compatibility**: does this break existing callers, storage
  entries, or on-chain state?

### Smart contract changes require CODEOWNERS approval

Any PR that modifies files under `contracts/` must be reviewed and approved
by a CODEOWNERS member before merging. This is enforced via
`.github/CODEOWNERS`. If you are modifying a contract file and no CODEOWNERS
member has reviewed after a few days, ping them in the PR.

### Responding to review

- Respond to every comment — either with a code change or an explanation of
  why the comment doesn't apply.
- Request re-review when all comments are addressed.
- Do not resolve reviewer comments yourself unless the reviewer explicitly
  says "feel free to resolve."
- If a reviewer's suggestion changes the approach significantly, start a
  discussion in the PR thread before implementing it.

### Merging

A maintainer merges the PR when:

1. All required CI jobs pass.
2. All blocking review comments are resolved.
3. The PR has at least one approving review (from a CODEOWNERS member for
   contract changes).

Squash merge is preferred for PRs with multiple work-in-progress commits.
If the commit history tells a coherent story, a regular merge commit is fine.

### Major decisions

For changes that affect architecture, security, or governance — open a
discussion, write an ADR, and follow the process in
[GOVERNANCE.md](GOVERNANCE.md) before implementation begins.

---

## 12. Reporting Issues

Before opening a new issue, search existing open and closed issues to avoid
duplicates. If an issue exists, add a comment rather than opening a new one.

Use the appropriate issue template:

- **[Bug Report](https://github.com/kenedybok3/SAFE-HAVEN/issues/new?template=bug_report.md)**
  — unexpected behavior, crashes, or incorrect output. Include steps to
  reproduce, the actual result, and the expected result.

- **[Feature Request](https://github.com/kenedybok3/SAFE-HAVEN/issues/new?template=feature_request.md)**
  — new functionality or improvements. Describe the problem you are trying
  to solve and why the existing behavior is insufficient.

Filling in the template fully helps maintainers triage and reproduce issues
faster. An incomplete bug report without reproduction steps will be marked
`needs-info` and may be closed if it doesn't receive a response.

---

## 13. Security Vulnerabilities

**Do not file security vulnerabilities as public GitHub issues.**

Public disclosure before a fix is in place puts every user of the contract
at risk.

If you discover a security vulnerability — including smart contract logic
bugs that could result in fund loss, auth bypasses, or storage manipulation —
follow the responsible disclosure process in [SECURITY.md](SECURITY.md):

- Email **security@example.com** with a full description, reproduction steps,
  and impact assessment.
- You will receive acknowledgment within 72 hours.
- Critical issues receive a patch target within 14 days.

We follow coordinated disclosure: please allow reasonable time to patch
before any public disclosure.

---

## 14. Utility Scripts

The `scripts/` directory contains helpers for common workflows:

| Script | Purpose |
|---|---|
| `scripts/deploy_testnet.sh` | Deploy the optimized WASM to Stellar testnet. Requires `SOROBAN_SECRET_KEY`. |
| `scripts/smoke_test_local.sh` | End-to-end smoke tests against a local Stellar node. |
| `scripts/final_merge.sh` | Batch-merges a predefined list of PRs into `main`, auto-resolving conflicts by favouring the PR branch. |
| `scripts/smart_merge.sh` | Merges simple and complex PRs in two passes. |

Review a script before running it in an environment you care about. The
merge scripts in particular are designed for batch operations and should not
be run casually against the upstream repository.

---

## 15. Pre-Commit Hooks

A pre-commit hook ships in `.githooks/`. Activate it once after cloning:

```bash
git config core.hooksPath .githooks
```

The hook runs automatically before every `git commit` and checks:

1. `cargo fmt --all -- --check` — formatting
2. `cargo clippy --all-targets -- -D warnings` — linting

If either check fails, the commit is aborted. Fix the reported issues and
re-run `git commit`.

```bash
# Fix formatting
make fmt

# Fix lint warnings
make lint    # shows the specific warnings; address them in the code
```

The hook is optional (it is not enforced server-side) but strongly
recommended. CI will catch the same errors, but catching them locally is
faster.

---

## 16. Licensing

SAFE-HAVEN is released under the [MIT License](LICENSE).

By submitting a contribution, you confirm that:

1. You have the right to submit the work (it is your original work, or you
   have appropriate rights from the original author).
2. You agree that the contribution may be distributed under the MIT license.

This project does not require a separate formal Contributor License Agreement.
