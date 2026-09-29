# Version Compatibility

This document describes how SAFE-HAVEN's frontend enforces version compatibility
with the deployed Soroban smart contract, and what to do when a mismatch is detected.

---

## Version Compatibility Matrix

Each row maps a frontend version to the contract versions and storage schema versions
it can safely interact with.

| Frontend Version | Compatible Contract Versions | Compatible Storage Versions |
|---|---|---|
| `0.1.0` | `0.1.0` | `1` |
| `0.2.0` (future) | `0.1.0`, `0.2.0` | `1`, `2` |

This matrix is defined in `frontend/src/lib/versionCompat.ts` as `COMPATIBILITY_MATRIX`.
Update it whenever a new contract release changes the on-chain ABI or storage layout.

---

## How Version Checking Works at Runtime

When the app starts, the `useVersionCheck` hook runs automatically:

1. It calls `getContractVersion()` (maps to the `version()` contract method) to fetch
   the contract's semver string (e.g. `"0.1.0"`).
2. It calls `getStorageVersion()` (maps to `get_storage_version()`) to fetch the
   storage schema version integer (e.g. `1`).
3. Both results are passed to `checkVersionCompatibility()`, a pure function in
   `versionCompat.ts` that looks up the current frontend version in `COMPATIBILITY_MATRIX`
   and checks whether the live contract/storage versions are listed as compatible.
4. The result (a `VersionCheckResult`) is passed to `<VersionWarningBanner>`, which
   renders in the app immediately below the header.

The check repeats every **5 minutes** while the app is open, so long-running sessions
will notice a contract upgrade without requiring a page reload.

Both RPC calls are made with `Promise.allSettled`-style error handling: if either fails
(e.g. the contract is temporarily unreachable), the hook still resolves with severity
`'warn'` rather than throwing. This ensures a network hiccup cannot prevent the user
from accessing the UI.

---

## Warning and Error Meanings

### `severity: 'ok'`

The live contract and storage versions both match the frontend's compatibility matrix.
No banner is shown.

### `severity: 'warn'` — Contract version mismatch

The contract semver (e.g. `0.2.0`) does not match what this frontend was built against
(`0.1.0`), but the storage schema is still compatible.

**What it means:** A new contract may have been deployed with new features that this
frontend does not yet surface. Existing deposits are still readable and withdrawable.

**How to resolve:**
- If you are a **user**: the app should still work for existing deposits. New features
  from the new contract version may not be available until the frontend is updated.
  Contact the operator for an update timeline.
- If you are an **operator**: redeploy the frontend targeting the new contract version.
  Update `VITE_EXPECTED_CONTRACT_VERSION` and optionally `VITE_CONTRACT_ID` in your
  `.env` file.

A warn banner is **dismissible** — users can close it and continue using the app.

### `severity: 'warn'` — Contract unreachable

Both `contractVersion` and `storageVersion` came back as `null`, meaning neither RPC
call succeeded.

**What it means:** The contract or RPC endpoint is temporarily unavailable. Read-only
views may be stale. Transactions cannot be submitted.

**How to resolve:** Wait a few minutes and refresh. If the problem persists, check
the RPC endpoint (`VITE_RPC_URL`) and the contract ID (`VITE_CONTRACT_ID`).

### `severity: 'error'` — Storage schema incompatibility

The on-chain storage schema version does not match what this frontend understands.

**What it means:** The contract was migrated to a new storage layout. The frontend
may misparse deposit data, display wrong amounts, or fail to render deposits entirely.
**Do not submit transactions until the frontend is updated.**

**How to resolve:**
- Stop using this frontend version immediately.
- Deploy (or obtain) a frontend build that is compatible with the new storage version.
- See the migration guide in the banner or in the section below.

An error banner is **non-dismissible** — it remains visible until the page is reloaded
with a compatible frontend version.

---

## Migration Guides

### v0.1.0 → v0.2.0

> This is a future scenario; adjust the steps once v0.2.0 is published.

**Step 1 — Read the changelog**

Review `CHANGELOG.md` for the full list of breaking changes between v0.1.0 and v0.2.0.

**Step 2 — Deploy the new contract**

```bash
export SOROBAN_SECRET_KEY=S...
make deploy-testnet   # or make deploy-mainnet
```

Note the new `CONTRACT_ID` from the output. The old contract remains on-chain and
existing deposits remain locked until users withdraw from it.

**Step 3 — Update frontend environment**

```env
VITE_CONTRACT_ID=<new contract ID>
VITE_EXPECTED_CONTRACT_VERSION=0.2.0
VITE_EXPECTED_STORAGE_VERSION=2   # if storage schema changed
```

**Step 4 — Verify compatibility**

```bash
npm run dev        # reload the dev server
# The version warning banner should now be gone
npm run test       # confirm no regressions
make smoke-test-local
```

**Step 5 — Communicate to users**

Announce the new contract address. Users with deposits in the v0.1.0 contract must
withdraw from there before re-depositing into the v0.2.0 contract. The old contract
does not disappear — it remains permanently on-chain.

---

## Overriding Expected Versions via Environment Variables

By default, the frontend ships with hardcoded expected versions in `versionCompat.ts`:

```ts
export const EXPECTED_CONTRACT_VERSION = '0.1.0'
export const EXPECTED_STORAGE_VERSION = 1
```

You can override these per deployment using environment variables in `.env`:

```env
# Pin to a specific contract semver (must match pattern: MAJOR.MINOR.PATCH)
VITE_EXPECTED_CONTRACT_VERSION=0.2.0

# Pin to a specific storage schema version (integer)
VITE_EXPECTED_STORAGE_VERSION=2
```

These values flow into `CONFIG.EXPECTED_CONTRACT_VERSION` and
`CONFIG.EXPECTED_STORAGE_VERSION` in `config.ts`. The `validateConfig()` function
called on startup will reject non-semver values for `VITE_EXPECTED_CONTRACT_VERSION`
with a clear error message.

> **Note:** The `COMPATIBILITY_MATRIX` in `versionCompat.ts` is the authoritative
> source of truth. Env var overrides shift the _displayed_ expected version in the
> banner but do not update the matrix. If you need the compatibility check to accept
> a new version, add a new row to the matrix in `versionCompat.ts`.

---

## Adding a New Contract Version

When a new contract version is released:

1. Add a new row to `COMPATIBILITY_MATRIX` in `frontend/src/lib/versionCompat.ts`.
2. If the storage schema changed, increment `EXPECTED_STORAGE_VERSION`.
3. Add a migration guide entry to `getMigrationGuide()` in `versionCompat.ts`.
4. Update this document.
5. Update `CHANGELOG.md` with the new compatibility information.

Example row addition:

```ts
{
  frontendVersion: '0.3.0',
  compatibleContractVersions: ['0.2.0', '0.3.0'],
  compatibleStorageVersions: [2],
},
```
