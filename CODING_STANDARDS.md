# Coding Standards and Style Guide

This document defines the standards that all new code in SAFE-HAVEN must follow.
The goal is a codebase that is readable, consistent, and safe by construction.

Existing code is not required to be retroactively updated (see scope in issue #410),
but any file you touch in a PR should be left cleaner than you found it.

---

## Table of Contents

1. [Enforcement](#1-enforcement)
2. [Rust — Smart Contract](#2-rust--smart-contract)
   - [Formatting](#21-formatting)
   - [Naming](#22-naming)
   - [Structure and Organisation](#23-structure-and-organisation)
   - [Documentation Comments](#24-documentation-comments)
   - [Error Handling](#25-error-handling)
   - [Security-Critical Patterns](#26-security-critical-patterns)
   - [Performance Conventions](#27-performance-conventions)
   - [Testing](#28-testing)
3. [React / TypeScript — Frontend](#3-react--typescript--frontend)
   - [Formatting](#31-formatting)
   - [Naming](#32-naming)
   - [Components](#33-components)
   - [Hooks](#34-hooks)
   - [State Management](#35-state-management)
   - [Types](#36-types)
   - [Documentation Comments](#37-documentation-comments)
   - [Error Handling](#38-error-handling)
   - [Testing](#39-testing)
4. [General Conventions](#4-general-conventions)
   - [Commit Messages](#41-commit-messages)
   - [PR Reviews](#42-pr-reviews)
5. [Tooling Reference](#5-tooling-reference)

---

## 1. Enforcement

Standards are enforced at three layers so problems are caught as early as possible:

| Layer | Tool | When it runs |
|---|---|---|
| **Local (pre-commit)** | `.githooks/pre-commit` | On every `git commit` |
| **CI (on every PR)** | `lint` and `frontend` CI jobs | On push to `main`/`develop` and all PRs |
| **PR review** | Human reviewer | Required for all contract changes (CODEOWNERS) |

Activate the pre-commit hook once after cloning:

```bash
git config core.hooksPath .githooks
```

The CI jobs are authoritative. A PR that passes the pre-commit hook but fails CI must be fixed before merging.

---

## 2. Rust — Smart Contract

### 2.1 Formatting

Formatting is fully automated by `cargo fmt`. Do not argue about it — let the tool decide.

```bash
# Apply formatting
cargo fmt --all

# Check without modifying (used in CI and pre-commit hook)
cargo fmt --all -- --check
```

Key settings in `rustfmt.toml`:

- Max line width: **100 characters**
- Import grouping: `std` → external crates → `crate::`
- Trailing commas: always in multi-line constructs
- Opening brace: same line as the declaration

### 2.2 Naming

Follow standard Rust conventions. The table below adds project-specific guidance.

| Item | Convention | Example |
|---|---|---|
| Types, traits, enums, enum variants | `UpperCamelCase` | `VaultEntry`, `VaultError`, `VaultKey` |
| Functions, methods, local variables | `snake_case` | `get_vault`, `unlock_time`, `deposit_id` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_DEPOSIT_AMOUNT`, `MIN_LOCK_LEDGERS` |
| Module names | `snake_case` | `contract`, `storage`, `events` |
| Storage key variants | `UpperCamelCase` with argument types in parentheses | `VaultKey::Deposit(Address, u32)` |
| Error variants | `UpperCamelCase`, named after the *condition*, not the *action* | `FundsStillLocked`, not `WithdrawFailed` |

**Avoid abbreviations** unless they are universally understood in the Soroban/Stellar context (`bps`, `xdr`, `ttl`). Write `depositor` not `dep`, `unlock_time` not `ulk`.

### 2.3 Structure and Organisation

**Module layout** — each file has one responsibility:

```
contracts/safe-haven/src/
├── lib.rs          — crate root; re-exports only
├── contract.rs     — all #[contractimpl] entry points
├── types.rs        — VaultKey, VaultEntry, LedgerVaultEntry, constants
├── errors.rs       — VaultError enum
├── events.rs       — event emission helpers
├── storage.rs      — all storage reads/writes; TTL helpers
└── test.rs         — all unit tests
```

Rules:

- **No business logic in `storage.rs`** — storage helpers read and write, nothing more.
- **No storage calls in `events.rs`** — event helpers emit events, nothing more.
- **No direct `env.storage()` calls in `contract.rs`** — always go through `storage::*`.
- New contract features that add significant complexity may introduce a new module (e.g., `migration.rs`). Discuss in the PR before doing so.

**Function length** — if a function exceeds ~50 lines, split it into named private helpers. The 80-line Clippy threshold in `.clippy.toml` is a hard warning; aim well below it.

**Module-level doc comment** — every source file must begin with a `//!` doc comment that describes the module's purpose in one or two sentences.

```rust
// good
//! Storage read/write helpers for SAFE-HAVEN vault entries.
//! All TTL bumps are centralised here.

use soroban_sdk::Env;
```

### 2.4 Documentation Comments

- All `pub` functions in `contract.rs` **must** have a `///` doc comment.
- The comment must describe: what the function does, its preconditions (auth requirements), and its error cases.
- Internal helpers need a comment only if the logic is non-obvious.

```rust
// good
/// Locks `amount` tokens until `unlock_time`.
///
/// # Auth
/// Requires `depositor.require_auth()`. The depositor must sign the transaction.
///
/// # Errors
/// - [`VaultError::ContractPaused`] if deposits are currently paused.
/// - [`VaultError::InvalidAmount`] if `amount` ≤ 0.
/// - [`VaultError::LockDurationTooShort`] if `unlock_time - now < MIN_LOCK_DURATION_SECS`.
pub fn deposit(
    env: Env,
    depositor: Address,
    token: Address,
    amount: i128,
    unlock_time: u64,
    penalty_bps: u32,
) -> Result<u32, VaultError> {

// bad — no doc comment on a public entry point
pub fn deposit(env: Env, depositor: Address, ...) -> Result<u32, VaultError> {
```

### 2.5 Error Handling

- **Never use `unwrap()` or `expect()` in production contract code.** Both panic the contract and consume the full instruction budget.
- Return `Result<T, VaultError>` from every fallible function.
- Use `?` for error propagation; do not match and re-wrap unless you need to transform the error.
- Add new error variants to `VaultError` rather than reusing an existing variant for a different condition. Error codes are part of the public ABI — never renumber or remove them.

```rust
// good
let entry = storage::get_deposit(&env, &depositor, deposit_id)
    .ok_or(VaultError::NoDepositFound)?;

// bad — panics in production
let entry = storage::get_deposit(&env, &depositor, deposit_id).unwrap();
```

### 2.6 Security-Critical Patterns

These are non-negotiable. A PR that violates any of them will not be merged.

#### `require_auth()` must be first

`depositor.require_auth()` (or `admin.require_auth()`) must be the **very first meaningful statement** in every mutating function — before any storage reads, token transfers, or other logic.

```rust
// correct
pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
    depositor.require_auth();                        // ← first
    let entry = storage::get_deposit(...)?;
    ...
}

// WRONG — auth after storage read
pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
    let entry = storage::get_deposit(...)?;          // ← reading state before auth
    depositor.require_auth();
    ...
}
```

#### Checks-Effects-Interactions

Every withdrawal path must:

1. **Check** — validate all conditions (lock time, ownership, amounts)
2. **Effect** — clear storage (`storage::remove_deposit`)
3. **Interact** — call the token contract (`token::Client::transfer`)

Never call an external contract (the token) before clearing your own state.

```rust
// correct order
let amount = entry.amount;
storage::remove_deposit(&env, &depositor, deposit_id);   // effect first
token_client.transfer(&env.current_contract_address(), &depositor, &amount); // then interact

// WRONG
token_client.transfer(...);          // interact before clear — re-entrancy window
storage::remove_deposit(...);
```

#### No `unsafe` blocks

SAFE-HAVEN is a no-`unsafe` codebase. `cargo-geiger` runs in CI and will fail if any new unsafe code is introduced. If `soroban-sdk` itself requires unsafe internally, that is the SDK's responsibility — do not call it via `unsafe` blocks in contract code.

### 2.7 Performance Conventions

Soroban instructions are metered. Every host function call costs budget.

**Cache host accessors**

Read a host value once, store it in a `let` binding, reuse the binding.

```rust
// good
let now = env.ledger().timestamp();
if now < entry.unlock_time { ... }
let remaining = entry.unlock_time - now;

// bad — two host calls for the same value
if env.ledger().timestamp() < entry.unlock_time { ... }
let remaining = entry.unlock_time - env.ledger().timestamp();
```

This applies to: `env.ledger().timestamp()`, `env.ledger().sequence()`, `env.current_contract_address()`.

**Paginate unbounded collections**

Any function that iterates over `DepositorList` or per-depositor deposit IDs must accept `offset` and `limit` parameters and must not iterate more than `MAX_BATCH_SIZE` items per call.

**Prefer `get_deposit_readonly` for queries**

Read-only queries (functions that don't modify state) must use `storage::get_deposit_readonly` instead of `storage::get_deposit`, to avoid an unnecessary TTL-bump write.

### 2.8 Testing

- Every new public contract function must have at least one positive test (happy path) and one negative test (error case) in `test.rs`.
- Error-path tests must assert the exact `VaultError` variant using `assert_eq!` or `assert!(matches!(...))`.
- Use the Soroban test environment (`soroban_sdk::testutils`) — never write tests that require network access.
- Test function names follow the pattern `test_<function>_<scenario>`:

```rust
#[test]
fn test_deposit_success() { ... }

#[test]
fn test_deposit_rejects_zero_amount() { ... }

#[test]
fn test_deposit_rejects_expired_unlock_time() { ... }
```

---

## 3. React / TypeScript — Frontend

### 3.1 Formatting

There is no Prettier config committed to this repository. Use the settings in `.editorconfig`:

- Indent: 2 spaces (for `.ts`, `.tsx`, `.json`)
- End of line: LF
- Trailing newline: yes

We recommend configuring your editor to format on save. A Prettier config will be added in a future issue if the team agrees on it.

### 3.2 Naming

| Item | Convention | Example |
|---|---|---|
| React components | `PascalCase` | `DepositCard`, `WalletInfoModal` |
| Hooks | `camelCase`, prefixed with `use` | `useDeposits`, `useContractInfo` |
| Regular functions, variables | `camelCase` | `formatAmount`, `depositId` |
| Constants (module-level) | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES`, `DEFAULT_TIMEOUT_MS` |
| Types and interfaces | `PascalCase` | `VaultEntry`, `WalletContextValue` |
| Enum-like constant objects | `PascalCase` for the object, `SCREAMING_SNAKE_CASE` for values | `Network.TESTNET` |
| Files — components | `PascalCase.tsx` | `DepositCard.tsx` |
| Files — hooks | `camelCase.ts` | `useDeposits.ts` |
| Files — utilities | `camelCase.ts` | `format.ts`, `stellar.ts` |
| Files — tests | match the file under test + `.test.ts(x)` | `format.test.ts` |

### 3.3 Components

- One component per file.
- Export the component as a named export, not a default export.

```tsx
// good
export function DepositCard({ entry }: DepositCardProps) { ... }

// avoid default exports — they make refactoring harder
export default function DepositCard(...) { ... }
```

- Define prop types as a `type` (not `interface`) immediately above the component, named `<ComponentName>Props`.

```tsx
type DepositCardProps = {
  entry: VaultEntry
  onWithdraw: (depositId: number) => void
}

export function DepositCard({ entry, onWithdraw }: DepositCardProps) {
  ...
}
```

- Avoid inline object/array literals in JSX props when the reference would change on every render and the value is passed to a child that uses `React.memo` or a hook dependency array.

```tsx
// bad — new array reference on every render
<TokenSelector tokens={['XLM', 'USDC']} />

// good — stable reference
const SUPPORTED_TOKENS = ['XLM', 'USDC'] as const
<TokenSelector tokens={SUPPORTED_TOKENS} />
```

- Keep components focused. If a component's JSX exceeds ~100 lines, extract sub-components.

### 3.4 Hooks

- Custom hooks must return a plain object `{ data, isLoading, error }` or a tuple `[value, setter]`, not a mix.
- Always handle the loading and error states — never assume data is available.
- Provide explicit return types on all hooks.

```ts
// good
export function useDeposits(depositor: string | null): UseDepositsResult {
  const [deposits, setDeposits] = useState<VaultEntry[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  ...
  return { deposits, isLoading, error }
}
```

- Document what the hook fetches, what triggers a re-fetch, and what `error` contains.

### 3.5 State Management

This project uses React Context + hooks — no Redux or Zustand.

- Keep state as local as possible. Lift to context only when multiple unrelated subtrees need the same value.
- `WalletContext` is the single source of truth for wallet connection state.
- Do not duplicate wallet state into component-local state.
- Async state transitions must always handle the error case with `try/catch` and surface the error to the user via `react-hot-toast` or the component's `error` state.

### 3.6 Types

- `strict: true` is enforced in `tsconfig.json`. Do not disable it.
- Never use `any` in production code (`@typescript-eslint/no-explicit-any` is set to `error`). Use `unknown` if the type is genuinely unknown, then narrow it explicitly.
- Prefer `type` aliases over `interface` for props, hook return values, and data shapes. Use `interface` only when you need declaration merging.
- Use `as const` for immutable literal objects and arrays that represent a fixed set of values.

```ts
// good
const NETWORKS = ['testnet', 'mainnet'] as const
type Network = typeof NETWORKS[number]  // 'testnet' | 'mainnet'

// bad
const NETWORKS = ['testnet', 'mainnet']  // type is string[]
```

- Avoid type assertions (`as SomeType`) except when dealing with external SDK types that cannot be typed precisely. Add a comment explaining why the assertion is safe.

### 3.7 Documentation Comments

Public utilities in `src/lib/` must have a JSDoc comment that describes what the function does and what its parameters mean.

```ts
/**
 * Formats a raw token amount (in stroops) as a human-readable string.
 *
 * @param amount - The raw amount as a bigint (stroops, 7 decimal places for XLM).
 * @param decimals - The token's decimal precision. Defaults to 7 for XLM.
 * @returns A formatted string, e.g. "1,234.5670000".
 */
export function formatTokenAmount(amount: bigint, decimals = 7): string {
```

Component-level comments are optional but encouraged for non-obvious UI behaviour.

### 3.8 Error Handling

- Every `async` function that calls the Stellar SDK must be wrapped in `try/catch`.
- Contract invocation errors from `stellar.ts` should surface as user-readable messages via `react-hot-toast`, not raw SDK error strings.
- Never silence a caught error with an empty `catch {}` block.
- Log unexpected errors with `console.error` (allowed by the ESLint config), not `console.log`.

```ts
// good
try {
  const result = await stellarClient.withdraw(depositId)
  toast.success('Withdrawal successful')
} catch (err) {
  const message = err instanceof Error ? err.message : 'Unknown error'
  console.error('[withdraw]', err)
  toast.error(`Withdrawal failed: ${message}`)
}

// bad — swallowed error
try {
  await stellarClient.withdraw(depositId)
} catch {}
```

### 3.9 Testing

- Unit tests use **Vitest** and live in `src/__tests__/`, named `<subject>.test.ts(x)`.
- E2E tests use **Playwright** and live in `frontend/tests/`.
- Tests for a hook should mock the Stellar SDK client — do not make real network calls.
- Test file naming: `<subject>.test.ts` for utilities, `<ComponentName>.test.tsx` for components.
- Minimum coverage expectation: every exported utility in `src/lib/` should have tests for the common case and at least one edge case.

---

## 4. General Conventions

### 4.1 Commit Messages

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short summary under 72 chars>

[optional body]

[optional footer: Closes #NNN]
```

Allowed types: `feat`, `fix`, `docs`, `chore`, `test`, `refactor`, `ci`, `security`, `perf`.

Scopes for this project:

| Scope | When to use |
|---|---|
| `contract` | Changes to `contracts/safe-haven/src/` |
| `storage` | Changes to storage layout or TTL helpers |
| `frontend` | Changes to `frontend/src/` |
| `ci` | Changes to `.github/workflows/` |
| `docs` | Changes to markdown documentation |
| `deps` | Dependency bumps |

The PR title is linted by the `pr-title-lint` CI job. If your PR title does not match the
Conventional Commits pattern, CI will fail.

### 4.2 PR Reviews

Contract changes require review by at least one CODEOWNER (see `.github/CODEOWNERS`).

**What reviewers check:**

- [ ] `require_auth()` is the first call in every new mutating function
- [ ] Checks-Effects-Interactions order is correct on every new withdrawal/transfer path
- [ ] No `unwrap()` or `expect()` in production code
- [ ] New public functions have doc comments
- [ ] New error variants are added at the end of `VaultError` (never renumber existing ones)
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] Tests added for new behaviour and error cases
- [ ] `make check` passes (fmt + clippy + tests + audit + deny)

---

## 5. Tooling Reference

### Rust

| Command | Purpose |
|---|---|
| `cargo fmt --all` | Format all Rust source files |
| `cargo fmt --all -- --check` | Verify formatting without modifying (CI) |
| `cargo clippy --all-targets --features testutils -- -D warnings` | Run linter; fail on any warning |
| `cargo test --features testutils` | Run all unit and doc tests |
| `make check` | fmt-check + clippy + test + audit + deny in sequence |
| `make lint` | Clippy only |
| `make fmt` | Format only |

Config files:

- `rustfmt.toml` — formatting rules
- `.clippy.toml` — Clippy lint thresholds
- `rust-toolchain.toml` — Rust channel pin (`stable`)
- `.cargo/config.toml` — workspace cargo settings

### Frontend

| Command | Purpose |
|---|---|
| `npm run lint` | ESLint — zero warnings allowed |
| `npm run lint:fix` | ESLint with auto-fix |
| `npm run typecheck` | `tsc --noEmit` — type errors only, no emit |
| `npm run test:unit` | Vitest unit tests |
| `npm run test:e2e` | Playwright end-to-end tests |
| `npm run build` | Production build (fails on type errors) |

Config files:

- `frontend/eslint.config.js` — ESLint flat config
- `frontend/tsconfig.json` — TypeScript strict config
- `frontend/vitest.config.ts` — Vitest config
- `frontend/playwright.config.ts` — Playwright config
- `.editorconfig` — indentation and line endings (applies to all files)

### Pre-commit hook

```bash
# Activate (once per clone)
git config core.hooksPath .githooks

# What it runs
#   [Rust]     cargo fmt --all -- --check
#   [Rust]     cargo clippy --all-targets --features testutils -- -D warnings
#   [Frontend] npm run lint          (only if frontend/ files are staged)
#   [Frontend] npm run typecheck     (only if frontend/ files are staged)
```
