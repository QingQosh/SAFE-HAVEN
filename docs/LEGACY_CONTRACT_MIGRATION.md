# Legacy Contract Migration Guide

This guide explains how to upgrade an existing SAFE-HAVEN deployment that predates the storage-versioning work and how to safely run the migration hook added in the current contract version.

## 1. Legacy state differences

The current contract introduces a versioned storage schema and a migration hook. In pre-versioned deployments there is no persisted schema revision key, so `get_storage_version()` returns `None` and the contract treats the instance as schema version `0`.

Important legacy differences:

- `VaultKey::StorageVersion` did not exist before the migration hook was added.
- `storage::get_storage_version()` therefore returns `None` for legacy deployments.
- The current `migrate(admin)` path is intentionally idempotent and checks `current_version >= STORAGE_VERSION` before writing anything.
- For the current baseline version (`STORAGE_VERSION = 1`), the migration is effectively a no-op because the only actual schema change is the introduction of the version key itself.
- Future compatibility changes to `VaultEntry` or other `#[contracttype]` values must add a new version and an explicit backfill or rewrite step inside `migrate()`.

In other words, the actual migration risk is not a data rewrite for the current v1 baseline; the risk is managing the upgrade path for future schema changes while preserving the same deployed key layout for old entries.

## 2. Upgrade flow

1. Build the upgraded WASM.
2. Deploy or re-use the upgraded contract instance with the same admin key as the legacy deployment.
3. Confirm the admin account still owns the contract instance.
4. Call `migrate(admin)` on the upgraded contract instance.
5. Confirm the returned value is `true` for a pre-versioned deployment and `false` if the instance was already migrated.
6. Verify `get_storage_version()` returns `Some(1)`.
7. Re-run a few read/write checks: `get_admin()`, `get_deposit_ids()`, and one deposit/withdraw flow.

## 3. Step-by-step migration procedure

### Option A: direct admin call

Use the Soroban CLI against the upgraded contract instance:

```bash
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export SOROBAN_SECRET_KEY=S...
export CONTRACT_ID=...
export ADMIN_ADDRESS=...

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$SOROBAN_SECRET_KEY" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_storage_version

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$SOROBAN_SECRET_KEY" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- migrate \
  --admin "$ADMIN_ADDRESS"

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$SOROBAN_SECRET_KEY" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_storage_version
```

Expected result:

- Legacy contract before versioning: `get_storage_version()` returns `null` / `None` and the first `migrate` call returns `true`.
- Already-migrated contract: the first `migrate` call returns `false` because the version is already current.

### Option B: scripted migration

The project includes a helper script at `scripts/migrate_contract.sh` that automates the version check and the migration call when the admin key is available.

```bash
export SOROBAN_SECRET_KEY=S...
export CONTRACT_ID=...
export ADMIN_ADDRESS=...

bash scripts/migrate_contract.sh
```

The script checks the current on-chain schema version, exits early when already up to date, and invokes the migration only when needed.

## 4. What is and is not automated

This repo already automates the migration trigger itself in the contract and in the helper script:

- `migrate(admin)` is the canonical upgrade hook.
- `get_storage_version()` lets CLI automation detect whether a deployment is still on the legacy schema.
- `scripts/migrate_contract.sh` wraps the calls so ops teams can run a single command.

The contract-to-contract part is intentionally minimal because this v1 migration is local to the same contract instance. There is no cross-contract data backfill required yet. If a later contract type adds a new field, the migration code will need an explicit per-entry read/write loop in `migrate()` and a new `STORAGE_VERSION` bump.

## 5. Operational checklist

Before touching production:

- [ ] Confirm the legacy contract was deployed before the `StorageVersion` key was introduced.
- [ ] Keep the current admin key available for the migration call.
- [ ] Deploy the new WASM to the same contract instance or the intended replacement instance.
- [ ] Run `get_storage_version()` and confirm the result is `None` before migration.
- [ ] Run `migrate(admin)` only after the admin is verified.
- [ ] Confirm `get_storage_version()` is `Some(1)` after the migration.
- [ ] Run a deposit and withdraw smoke check against the upgraded instance.

## 6. Future schema bumps

When a future release adds a field to `VaultEntry`, `LedgerVaultEntry`, or another persistent `#[contracttype]`, follow this pattern:

1. Increase `STORAGE_VERSION` in `contracts/safe-haven/src/types.rs`.
2. Extend `migrate(env, admin)` with the concrete step required for the old layout.
3. Read every old-record shape, rewrite it to the new shape, and set the new version only after the rewrite succeeds.
4. Keep the migration idempotent and guard it with admin auth.
5. Document the upgrade in the changelog and in the migration runbook.

This keeps every upgrade explicit, auditable, and replay-safe.
