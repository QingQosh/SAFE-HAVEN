# SAFE-HAVEN Contract API

This document describes the public Soroban contract entry points and shows practical CLI examples for common flows.

## Environment setup

Use a standard Soroban CLI setup before calling any function:

```bash
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export CONTRACT_ID="CC..."
export ADMIN_SECRET="S..."
export USER_SECRET="S..."
export ADMIN_ADDRESS="G..."
export USER_ADDRESS="G..."
export FEE_RECIPIENT="G..."
export TOKEN_CONTRACT="CC..."
```

## Initialization

### initialize(admin, fee_recipient, max_deposit?, max_lock_secs?)

Initializes the vault once. This writes the admin, fee recipient, and optional runtime limits.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --fee_recipient "$FEE_RECIPIENT" \
  --max_deposit 1000000000000 \
  --max_lock_secs 157788000
```

## Core deposit flows

### deposit(depositor, token, amount, unlock_time, penalty_bps)

Locks tokens until a wall-clock timestamp is reached.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 5000 \
  --unlock_time 1750000000 \
  --penalty_bps 200
```

Returns a `u32` deposit ID.

### deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps)

A third party pays for the deposit while the beneficiary is a different address.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_for \
  --payer "$ADMIN_ADDRESS" \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 5000 \
  --unlock_time 1750000000 \
  --penalty_bps 200
```

### deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps)

Locks tokens until a specific ledger sequence number is reached instead of a timestamp.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_by_ledger \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 2500 \
  --unlock_ledger 130000000 \
  --penalty_bps 100
```

## Withdraw and early exit

### withdraw(depositor, deposit_id)

Withdraws the funds once the time or ledger threshold has been reached.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### withdraw_to(depositor, deposit_id, recipient)

Withdraws to a specific target address instead of the original depositor.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw_to \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0 \
  --recipient "$ADMIN_ADDRESS"
```

### cancel_deposit(depositor, deposit_id)

Allows an early exit before unlock. The penalty is deducted and sent to `fee_recipient`.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- cancel_deposit \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

## Staker registry

### register_staker(staker, amount)

Register a positive stake amount or update the staker's existing amount. The staker must authorize the call. `total_staked` tracks the sum of all active registered stakes.

### withdraw_stake(staker, amount)

Reduce or remove the staker's registered amount. The staker must authorize the call, and `amount` must be positive and no greater than the current stake. A full withdrawal removes the active registry entry. This accounting-only operation does not transfer tokens.

### claim_staker_rewards(staker)

Claim the staker's proportional share of the rewards pool, calculated from their current registered stake and `total_staked`. The staker must authorize the call.
## Staker rewards

### register_staker(staker, amount)

Registers a staker or updates their stake amount.

### claim_staker_rewards(staker)

Claims the staker's proportional share of the current rewards pool. The staker must authorize
the call. A staker can claim only once for each pool replenishment; a duplicate claim fails
with `NoRewardsToClaim`. A successful claim reduces the pool and increments the staker's
cumulative claimed-rewards total.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_staker_rewards \
  --staker "$USER_ADDRESS"
```

### get_staker_rewards_claimed(staker) / get_rewards_pool()

Read the staker's cumulative claimed rewards and the current unclaimed pool balance.

## Admin operations

### emergency_withdraw(admin, depositor, deposit_id)

Admin-only emergency return of funds regardless of lock status.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- emergency_withdraw \
  --admin "$ADMIN_ADDRESS" \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### pause(admin) / unpause(admin)

Pauses or resumes new deposits.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- pause \
  --admin "$ADMIN_ADDRESS"
```

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- unpause \
  --admin "$ADMIN_ADDRESS"
```

### transfer_admin(admin, new_admin)

Starts a two-step admin transfer.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- transfer_admin \
  --admin "$ADMIN_ADDRESS" \
  --new_admin "$USER_ADDRESS"
```

Then the pending admin finalizes it:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- accept_admin \
  --new_admin "$USER_ADDRESS"
```

### cancel_transfer_admin(admin)

Cancels a pending admin transfer.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- cancel_transfer_admin \
  --admin "$ADMIN_ADDRESS"
```

### renounce_admin(admin)

Permanently removes admin privileges.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- renounce_admin \
  --admin "$ADMIN_ADDRESS"
```

## Read-only queries

### get_vault(depositor, deposit_id)

Returns the timestamp-based vault entry for a deposit.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### get_ledger_vault(depositor, deposit_id)

Returns the ledger-based vault entry for a ledger deposit.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_ledger_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### get_deposit_ids(depositor)

Returns all active deposit IDs for a given depositor.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposit_ids \
  --depositor "$USER_ADDRESS"
```

### get_deposits_page(offset, limit)

Returns a flat page of `(depositor, deposit_id, VaultEntry)` tuples for timestamp-based, single-token deposits. This query reads the timestamp `Deposit(depositor, deposit_id)` storage key only, so it does **not** include ledger-based deposits (stored under `DepositByLedger`) or multi-token deposits (stored under `MultiDeposit`). These storage keys hold different entry types; use the matching query for each type: `get_vault`, `get_ledger_vault`, or `get_multi_vault`.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposits_page \
  --offset 0 \
  --limit 10
```

To enumerate active ledger deposits, page through `get_depositors`, call `get_deposit_ids` for each depositor, and query `get_ledger_vault` for each ID. The active ID list is shared across deposit types; an ID is a ledger deposit when `get_ledger_vault` returns an entry. The following SDK-neutral JavaScript pseudocode assumes `invokeRead(method, args)` calls the contract and returns decoded native values (`Option::None` as `null`):

```javascript
async function listLedgerDeposits(invokeRead) {
  const pageSize = 50;
  const ledgerDeposits = [];

  for (let offset = 0; ; offset += pageSize) {
    const page = await invokeRead("get_depositors", {
      offset,
      limit: pageSize,
    });

    for (const depositor of page.items) {
      const depositIds = await invokeRead("get_deposit_ids", { depositor });
      for (const depositId of depositIds) {
        const entry = await invokeRead("get_ledger_vault", {
          depositor,
          deposit_id: depositId,
        });
        if (entry !== null) {
          ledgerDeposits.push({ depositor, depositId, entry });
        }
      }
    }

    if (offset + pageSize >= page.total_count) break;
  }

  return ledgerDeposits;
}
```

`get_depositors.total_count` counts active depositors, not deposits. This workaround lists currently active ledger entries only; it does not provide historical deposits. A future unified enumeration API could expose all deposit types under a common paginated interface.

### get_time()

Returns the current ledger timestamp.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_time
```

### time_remaining(depositor, deposit_id)

Returns seconds remaining until unlock for timestamp-based deposits, or an estimated second value for ledger-based deposits.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- time_remaining \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### get_admin(), get_pending_admin(), get_fee_recipient(), get_constants(), get_depositor_count(), get_depositors()

Example:

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_admin
```

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_constants
```

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_depositors \
  --offset 0 \
  --limit 10
```

## Carbon Footprint & Sustainability Metrics

The contract tracks an estimated carbon footprint for locked deposits and exposes sustainability metrics for dashboards and reports. All carbon values are expressed in **grams of CO2 equivalent (g CO2e)**.

### Methodology and formula

The carbon estimate uses a simple, deterministic model:

```text
carbon_grams = locked_units × locked_seconds × CARBON_BASELINE_G_PER_UNIT_SEC
```

- `locked_units` — the deposit amount (contract units, `i128`).
- `locked_seconds` — the number of seconds the funds remain locked.
- `CARBON_BASELINE_G_PER_UNIT_SEC` — the carbon baseline: **1 g CO2e per unit per second**.

The baseline represents the estimated emissions of holding one unit of value for one second under a non-renewable energy assumption. It is a fixed constant; the renewable energy percentage is applied on top of it to discount the estimate.

### Renewable energy percentage

The renewable energy percentage is an integer in the range **0–100** (default **50**). It represents the share of energy assumed to come from renewable sources:

- `0` — no renewable energy; the full baseline estimate is reported.
- `50` — half renewable; the reported footprint is halved (default).
- `100` — fully renewable; the reported footprint is zero.

The effective footprint is:

```text
effective_carbon_grams = carbon_grams × (100 − renewable_percentage) / 100
```

### Worked examples

**Example 1 — 1,000 units locked for 1 day (default 50% renewable):**

```text
locked_units   = 1,000
locked_seconds = 86,400
carbon_grams   = 1,000 × 86,400 × 1 = 86,400,000 g CO2e
effective      = 86,400,000 × (100 − 50) / 100 = 43,200,000 g CO2e
```

**Example 2 — 1,000 units locked for 1 day at 0% renewable:**

```text
effective = 86,400,000 × (100 − 0) / 100 = 86,400,000 g CO2e
```

**Example 3 — 1,000 units locked for 1 day at 100% renewable:**

```text
effective = 86,400,000 × (100 − 100) / 100 = 0 g CO2e
```

**Example 4 — 500 units locked for 1 hour at 50% renewable:**

```text
carbon_grams = 500 × 3,600 × 1 = 1,800,000 g CO2e
effective    = 1,800,000 × 50 / 100 = 900,000 g CO2e
```

### get_sustainability_metrics()

Returns the current sustainability snapshot for the contract. The return type is a struct with the following fields:

| Field | Type | Description |
| --- | --- | --- |
| `total_carbon_grams` | `i128` | Cumulative estimated footprint in g CO2e |
| `renewable_percentage` | `u32` | Renewable energy share, 0–100 |
| `effective_carbon_grams` | `i128` | Footprint after applying the renewable percentage |
| `total_locked_units` | `i128` | Sum of currently locked units |
| `milestone_count` | `u32` | Number of sustainability milestones reached |

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_sustainability_metrics
```

### generate_sustainability_report()

Generates a report summarizing the current sustainability state. The return type is a struct with the following fields:

| Field | Type | Description |
| --- | --- | --- |
| `total_carbon_grams` | `i128` | Cumulative estimated footprint in g CO2e |
| `effective_carbon_grams` | `i128` | Footprint after applying the renewable percentage |
| `renewable_percentage` | `u32` | Renewable energy share, 0–100 |
| `deposit_count` | `u32` | Number of deposits included in the report |
| `milestones` | `Vec<u32>` | Milestone thresholds that have been reached |

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- generate_sustainability_report
```

Sample output (decoded):

```json
{
  "total_carbon_grams": 86400000,
  "effective_carbon_grams": 43200000,
  "renewable_percentage": 50,
  "deposit_count": 3,
  "milestones": [1000000, 10000000]
}
```

### Frontend integration

Both queries are read-only and safe to call from a dashboard. Decode the returned structs into native values; all carbon amounts are `i128` and may exceed JavaScript's safe integer range, so decode them as `BigInt` (or strings) rather than `Number`.

Dashboard example that calls both queries:

```javascript
async function loadSustainabilityDashboard(invokeRead) {
  const [metrics, report] = await Promise.all([
    invokeRead("get_sustainability_metrics", {}),
    invokeRead("generate_sustainability_report", {}),
  ]);

  return {
    // i128 values: decode as BigInt to avoid precision loss
    totalCarbonGrams: BigInt(metrics.total_carbon_grams),
    effectiveCarbonGrams: BigInt(metrics.effective_carbon_grams),
    renewablePercentage: Number(metrics.renewable_percentage),
    totalLockedUnits: BigInt(metrics.total_locked_units),
    milestoneCount: Number(metrics.milestone_count),
    depositCount: Number(report.deposit_count),
    milestones: report.milestones.map((m) => Number(m)),
  };
}
```

Display guidance:

- Format `effectiveCarbonGrams` for users (for example, convert to kilograms by dividing by `1000`).
- Show `renewablePercentage` as a progress indicator (0–100).
- Render `milestones` as badges; each entry is a threshold in g CO2e that has been reached.

### FAQ

**What units are used?** All carbon values are grams of CO2 equivalent (g CO2e). Divide by 1,000 for kilograms.

**What is the carbon baseline?** 1 g CO2e per unit per second. It is a fixed constant used to estimate emissions from locked value.

**How does the renewable percentage affect results?** It discounts the baseline estimate: `effective = total × (100 − renewable) / 100`. The default is 50, meaning half the baseline estimate is reported.

**Can the renewable percentage be changed?** It is configured by the contract admin; valid values are 0–100.

**Why is my footprint zero?** A renewable percentage of 100 yields zero effective emissions, and deposits with zero locked seconds or zero units contribute nothing.

**Are these values scientifically precise?** No. They are deterministic estimates intended for relative tracking and reporting, not regulatory accounting.

**Do the queries cost anything?** They are read-only simulations and do not modify state.

## Migration helpers

### get_storage_version() and migrate(admin)

Check whether the contract is already at the current schema version.

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_storage_version
```

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- migrate \
  --admin "$ADMIN_ADDRESS"
```

For production upgrades, use the helper script in `scripts/migrate_contract.sh` to auto-detect legacy deployments and run the migration only when needed.

## Common usage patterns

### One full deposit and withdraw cycle

```bash
DEPOSIT_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000 \
  --unlock_time 1750000000 \
  --penalty_bps 0 \
  | tr -d '\r\n')

echo "Deposit ID: $DEPOSIT_ID"

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID"
```

### Check whether a deposit is active

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposit_type \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

The response is either `TimeBased`, `LedgerBased`, or `null`.

---

## Related Guides

### Ledger-Based Deposits

For comprehensive documentation on using `deposit_by_ledger`, lock calculations, and ledger-vs-timestamp decisions, see:

- **[Ledger Deposits Guide](./LEDGER_DEPOSITS_GUIDE.md)** — Full guide with use cases, examples, and patterns
- **[Ledger Deposits Quick Reference](./LEDGER_DEPOSITS_QUICK_REFERENCE.md)** — Quick lookup for common tasks
- **[ADR-001: Dual Deposit Types](./adr/ADR-001-dual-deposit-types.md)** — Technical design and rationale

### Other Guides

- **[Staker Registry Guide](./STAKER_REGISTRY_GUIDE.md)** — Staker registration and rewards
- **[Contract Upgrade and Migration Playbook](./CONTRACT_UPGRADE_AND_MIGRATION_PLAYBOOK.md)** — Production upgrades
- **[Circuit Breakers](./CIRCUIT_BREAKERS.md)** — Emergency pause and rate limiting
