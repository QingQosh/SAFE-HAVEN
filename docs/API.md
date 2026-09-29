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
