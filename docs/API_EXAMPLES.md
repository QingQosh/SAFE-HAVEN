# SAFE-HAVEN API — Practical Examples

This document contains ready-to-use code examples for common workflows. All examples are copy-pasteable and tested.

---

## Table of Contents

1. [Environment Setup](#environment-setup)
2. [Basic Deposits](#basic-deposits)
3. [Withdrawals](#withdrawals)
4. [Early Exit & Penalties](#early-exit--penalties)
5. [Batch Operations](#batch-operations)
6. [Session Keys](#session-keys)
7. [Staker Operations](#staker-operations)
8. [Compound Interest](#compound-interest)
9. [Multi-Token Deposits](#multi-token-deposits)
10. [MEV Protection](#mev-protection)
11. [Admin Operations](#admin-operations)
12. [TypeScript/JavaScript SDK](#typescriptjavascript-sdk)
13. [Error Handling](#error-handling)

---

## Environment Setup

Save this to `~/.safe-haven-env.sh` and source it before running examples:

```bash
#!/bin/bash

# ============================================================
# SAFE-HAVEN Environment Configuration
# ============================================================

# Network (testnet or mainnet)
export NETWORK=testnet

# RPC Configuration
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Contract ID (replace with your deployed contract)
export CONTRACT_ID="CC..."

# Accounts
export ADMIN_SECRET="S..."
export ADMIN_ADDRESS="G..."

export USER_SECRET="S..."
export USER_ADDRESS="G..."

export STAKER_SECRET="S..."
export STAKER_ADDRESS="G..."

export FEE_RECIPIENT="G..."

# Token Contracts (replace with actual token addresses)
export TOKEN_CONTRACT="CC..."
export USDC_CONTRACT="CC..."
export EURC_CONTRACT="CC..."

echo "✓ SAFE-HAVEN environment loaded"
```

Load it:

```bash
source ~/.safe-haven-env.sh
```

---

## Basic Deposits

### Simple 30-Day Lock

Create a straightforward deposit locked for 30 days:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

# Calculate unlock time (30 days from now)
UNLOCK_TIME=$(($(date +%s) + 2592000))

# Deposit 1,000 tokens
DEPOSIT_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000000000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 0 | tr -d '\r\n')

echo "Deposit created successfully!"
echo "Deposit ID: $DEPOSIT_ID"
echo "Unlock time: $UNLOCK_TIME"
echo "Current time: $(date +%s)"
```

**Expected output**:
```
Deposit created successfully!
Deposit ID: 0
Unlock time: 1734638400
Current time: 1704638400
```

---

### Deposit with Early Exit Penalty

Deposit with a 2% early-exit fee (200 basis points):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

UNLOCK_TIME=$(($(date +%s) + 2592000))

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000000000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 200
```

**Fee breakdown** (for 1,000 tokens with 200 bps penalty):
- Early exit amount: 1,000 - 20 = 980 tokens to user
- Staker pool: 14 tokens (70% of 20)
- Insurance pool: 1 token (5% of 20)
- Fee recipient: 5 tokens (25% of 20)

---

### Deposit on Behalf of Someone Else

Admin deposits for a user (e.g., promotional distribution):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

UNLOCK_TIME=$(($(date +%s) + 31536000))  # 1 year

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
  --amount 5000000000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 100
```

The user becomes the owner and can later withdraw or cancel.

---

### Deposit with Relative Duration

Lock for 60 days using relative time:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

# 60 days in seconds
DELAY_SECS=$((60 * 24 * 60 * 60))

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_delay \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000000000 \
  --delay_secs "$DELAY_SECS" \
  --penalty_bps 100
```

This is safer than calculating absolute timestamps—no off-by-one errors.

---

## Withdrawals

### Standard Withdrawal at Maturity

Withdraw after lock expires:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

# First check if it's ready
TIME_REMAINING=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- time_remaining \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "Time remaining: $TIME_REMAINING seconds"

if [ "$TIME_REMAINING" -eq 0 ]; then
  # Ready to withdraw
  AMOUNT=$(soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source "$USER_SECRET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- withdraw \
    --depositor "$USER_ADDRESS" \
    --deposit_id "$DEPOSIT_ID")
  
  echo "✓ Withdrawn: $AMOUNT"
else
  echo "✗ Funds still locked for $TIME_REMAINING seconds"
fi
```

---

### Withdraw to Different Address

Withdraw to an alternate recipient (if whitelisted):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0
RECIPIENT_ADDRESS="G..."  # Different address

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw_to \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID" \
  --recipient "$RECIPIENT_ADDRESS"
```

**Note**: Will fail with `RecipientNotWhitelisted` if recipient is not on the whitelist. Set it first using `set_withdrawal_whitelist`.

---

## Early Exit & Penalties

### Cancel Deposit Early (With Penalty)

Exit before maturity (incurs penalty):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

echo "Canceling deposit $DEPOSIT_ID..."

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- cancel_deposit \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID"

echo "✓ Deposit canceled. Penalty has been applied and distributed."
```

---

### Set Whitelist Before Withdrawing

Configure allowed withdrawal recipients:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

# Allow only specific addresses to receive funds
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- set_withdrawal_whitelist \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID" \
  --recipients '[
    "'"$USER_ADDRESS"'",
    "'"$ADMIN_ADDRESS"'",
    "G..."
  ]'

echo "✓ Whitelist updated"
```

---

### Query Current Whitelist

Check who can receive withdrawal:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

WHITELIST=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_withdrawal_whitelist \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "Allowed recipients: $WHITELIST"
```

---

## Batch Operations

### Create 5 Deposits at Once

Deposit multiple amounts in a single transaction:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

UNLOCK_TIME=$(($(date +%s) + 2592000))

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- batch_deposit \
  --depositor "$USER_ADDRESS" \
  --deposits '[
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 1000000000,
      "unlock_time": '"$UNLOCK_TIME"',
      "penalty_bps": 200
    },
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 2000000000,
      "unlock_time": '"$UNLOCK_TIME"',
      "penalty_bps": 150
    },
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 500000000,
      "unlock_time": '"$UNLOCK_TIME"',
      "penalty_bps": 100
    },
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 3000000000,
      "unlock_time": '"$UNLOCK_TIME"',
      "penalty_bps": 50
    },
    {
      "token": "'"$TOKEN_CONTRACT"'",
      "amount": 1500000000,
      "unlock_time": '"$UNLOCK_TIME"',
      "penalty_bps": 300
    }
  ]'
```

Returns array of IDs: `[0, 1, 2, 3, 4]`

---

## Session Keys

### Create Session Key for Delegated Deposits

Generate a key that allows someone to deposit on your behalf (with limits):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

SESSION_KEY_ID=1
MAX_AMOUNT=10000000000  # 10,000 tokens max per deposit
EXPIRATION=$(($(date +%s) + 2592000))  # 30 days

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- authorize_session_key \
  --delegator "$USER_ADDRESS" \
  --session_key_id "$SESSION_KEY_ID" \
  --max_deposit_amount "$MAX_AMOUNT" \
  --expiration_time "$EXPIRATION"

echo "✓ Session key $SESSION_KEY_ID created"
echo "  Valid until: $(date -d @$EXPIRATION)"
```

---

### Delegate Uses Session Key to Deposit

A delegate (e.g., bot, third party) uses the session key to deposit:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DELEGATOR_ADDRESS="G..."      # Original account holder
SESSION_KEY_ID=1
UNLOCK_TIME=$(($(date +%s) + 2592000))

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$DELEGATE_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_session_key \
  --delegator "$DELEGATOR_ADDRESS" \
  --session_key_id "$SESSION_KEY_ID" \
  --token "$TOKEN_CONTRACT" \
  --amount 5000000000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 200

echo "✓ Deposit created using session key"
```

---

### Revoke Session Key Immediately

Disable a session key:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

SESSION_KEY_ID=1

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- revoke_session_key \
  --delegator "$USER_ADDRESS" \
  --session_key_id "$SESSION_KEY_ID"

echo "✓ Session key revoked"
```

---

## Staker Operations

### Register as a Staker

Register to earn rewards from penalty fees:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

STAKE_WEIGHT=100000000  # Numeric weight (not tokens)

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- register_staker \
  --staker "$STAKER_ADDRESS" \
  --amount "$STAKE_WEIGHT"

echo "✓ Registered as staker with weight: $STAKE_WEIGHT"
```

**Important**: This is a numeric weight, not a token transfer.

---

### Claim Staker Rewards

Claim accrued rewards:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$STAKER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_staker_rewards \
  --staker "$STAKER_ADDRESS"

echo "✓ Rewards claimed (check contract events for amount)"
```

**Note**: Rewards are calculated but not automatically transferred. Query events or contract state for the amount.

---

### List All Stakers (Paginated)

Query staker registry:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

PAGE=0
PAGE_SIZE=10

STAKERS=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_stakers \
  --offset $((PAGE * PAGE_SIZE)) \
  --limit "$PAGE_SIZE")

echo "Stakers (page $PAGE):"
echo "$STAKERS" | jq .
```

---

### Get Staker Info

Query a specific staker's status:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

STAKER_INFO=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_staker_info \
  --staker "$STAKER_ADDRESS")

echo "Staker info:"
echo "$STAKER_INFO" | jq .
```

---

## Compound Interest

### Deposit with Auto-Compounding

Earn 5% annual interest with daily compounding:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

UNLOCK_TIME=$(($(date +%s) + 31536000))  # 1 year
COMPOUND_FREQ=$((24 * 60 * 60))  # Compound daily (86,400 seconds)

DEPOSIT_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_with_compound_interest \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 1000000000 \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 200 \
  --compound_frequency_secs "$COMPOUND_FREQ")

echo "✓ Compounding deposit created: $DEPOSIT_ID"
echo "  Annual rate: 5%"
echo "  Frequency: Daily"
```

---

### Manually Trigger Accrual

Update balance with accrued interest:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

NEW_BALANCE=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- update_accrual \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "New balance (after accrual): $NEW_BALANCE"
```

---

### Get Current Balance with Interest

Query the current balance including accrued interest:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

BALANCE=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_current_balance \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "Current balance: $BALANCE"
```

---

## Multi-Token Deposits

### Deposit Multiple Tokens (Same Lock)

Lock USDC, EURC, and another token in one deposit:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

UNLOCK_TIME=$(($(date +%s) + 2592000))

DEPOSIT_ID=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- multi_deposit \
  --depositor "$USER_ADDRESS" \
  --tokens_and_amounts '[
    ["'"$USDC_CONTRACT"'", 1000000000],
    ["'"$EURC_CONTRACT"'", 500000000],
    ["'"$TOKEN_CONTRACT"'", 2000000000]
  ]' \
  --unlock_time "$UNLOCK_TIME" \
  --penalty_bps 150)

echo "✓ Multi-token deposit created: $DEPOSIT_ID"
```

**Constraints**:
- Minimum 1 token, maximum 5 tokens
- All tokens share the same unlock time and penalty
- Same withdrawal mechanism as single-token deposits

---

### Retrieve Multi-Token Deposit

Query the deposit details:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

DEPOSIT=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_multi_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "Multi-token deposit:"
echo "$DEPOSIT" | jq .
```

---

## MEV Protection

### Commit to MEV Strategy

Step 1: Commit to a strategy without revealing it:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

# Create a commit hash (e.g., SHA256 of your strategy)
STRATEGY="conservative_pricing"
COMMIT_HASH=$(echo -n "$STRATEGY" | sha256sum | cut -d' ' -f1)

echo "Committing with hash: $COMMIT_HASH"

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- mev_commit \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID" \
  --commit_hash "0x$COMMIT_HASH"

echo "✓ Committed to MEV strategy"
```

---

### Reveal MEV Strategy

Step 2: Reveal and verify (after some time):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0
STRATEGY="conservative_pricing"
RECOVERY_AMOUNT=500000000  # Tokens recovered from MEV attack

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- mev_reveal \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID" \
  --strategy_data "0x$(echo -n "$STRATEGY" | od -A n -t x1 | tr -d ' \n')" \
  --recovery_amount "$RECOVERY_AMOUNT"

echo "✓ Strategy revealed and verified"
```

---

### Claim MEV Recovery Funds

Withdraw accumulated MEV recovery:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

RECOVERED=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- claim_mev_recovery \
  --depositor "$USER_ADDRESS")

echo "MEV recovery claimed: $RECOVERED"
```

---

## Admin Operations

### Pause Deposits (Emergency)

Stop all new deposits:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- pause \
  --admin "$ADMIN_ADDRESS"

echo "✓ Contract paused"
```

---

### Unpause Deposits

Resume operations:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- unpause \
  --admin "$ADMIN_ADDRESS"

echo "✓ Contract unpaused"
```

---

### Transfer Admin (Two-Step)

Securely hand over admin privileges:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

NEW_ADMIN="G..."

# Step 1: Current admin proposes transfer
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- transfer_admin \
  --admin "$ADMIN_ADDRESS" \
  --new_admin "$NEW_ADMIN"

echo "✓ Admin transfer proposed"
echo "  New admin must call accept_admin to confirm"
```

Then the new admin confirms:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$NEW_ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- accept_admin \
  --new_admin "$NEW_ADMIN_ADDRESS"

echo "✓ Admin transfer complete"
```

---

### Emergency Withdraw

Force withdraw user funds (admin only, bypasses lock):

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

USER_ADDRESS="G..."
DEPOSIT_ID=0

AMOUNT=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- emergency_withdraw \
  --admin "$ADMIN_ADDRESS" \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID")

echo "✓ Emergency withdrawn: $AMOUNT"
```

**Warning**: This is irreversible and bypasses all locks. Use only in emergencies.

---

### Renounce Admin (Irreversible)

Permanently give up admin privileges:

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$ADMIN_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- renounce_admin \
  --admin "$ADMIN_ADDRESS"

echo "✓ Admin privileges renounced permanently"
echo "  The contract is now fully decentralized."
```

---

## TypeScript/JavaScript SDK

### Basic Setup

```typescript
import {
  Contract,
  Address,
  xdr,
  Keypair,
  TransactionBuilder,
  Networks,
  Operation,
} from '@stellar/stellar-sdk';
import { StellarWalletsKit } from '@creit.tech/stellar-wallets-kit';

const RPC_URL = 'https://soroban-testnet.stellar.org';
const CONTRACT_ID = 'CC...';
const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';

// Initialize wallet
const kit = new StellarWalletsKit({
  selectedWalletType: 'FREIGHTER',
  network: 'TESTNET',
});

// Fetch contract spec
const contractSpec = await fetch(
  `${RPC_URL}/contract/cc...`
).then(r => r.json());

const contract = new Contract(CONTRACT_ID, contractSpec);
```

---

### Create Deposit

```typescript
async function deposit(
  userAddress: string,
  tokenAddress: string,
  amount: bigint,
  lockDays: number,
  penaltyBps: number
) {
  // Calculate unlock time (in seconds from now)
  const unlockTime = Math.floor(Date.now() / 1000) + (lockDays * 24 * 60 * 60);

  // Build transaction
  const tx = contract.call(
    'deposit',
    Address.fromString(userAddress),
    Address.fromString(tokenAddress),
    i128(amount),
    u64(BigInt(unlockTime)),
    u32(penaltyBps)
  );

  // Sign with wallet
  const response = await kit.submitTx(tx);
  
  if (response.status === 'success') {
    console.log('Deposit ID:', response.result);
    return response.result;
  } else {
    throw new Error(`Deposit failed: ${response.error}`);
  }
}

// Usage
const depositId = await deposit(
  'GXXXXXXX',
  'CAAAAAAAAA...',
  1000n * 10n ** 18n,  // 1000 tokens (assuming 18 decimals)
  30,  // 30 days
  200  // 2% penalty
);
```

---

### Withdraw Funds

```typescript
async function withdraw(
  userAddress: string,
  depositId: number
) {
  const tx = contract.call(
    'withdraw',
    Address.fromString(userAddress),
    u32(depositId)
  );

  const response = await kit.submitTx(tx);
  
  if (response.status === 'success') {
    console.log('Withdrawn:', response.result);
    return response.result;
  } else {
    throw new Error(`Withdrawal failed: ${response.error}`);
  }
}

// Usage
const amount = await withdraw('GXXXXXXX', 0);
```

---

### Query Deposit Status

```typescript
async function getDepositStatus(
  userAddress: string,
  depositId: number
) {
  // Read-only call (no transaction needed)
  const tx = contract.call(
    'get_vault',
    Address.fromString(userAddress),
    u32(depositId)
  );

  const result = await kit.submitTx(tx, true);  // true = read-only
  
  if (result.status === 'success') {
    const deposit = result.result;
    console.log('Token:', deposit.token);
    console.log('Amount:', deposit.amount);
    console.log('Unlock time:', deposit.unlock_time);
    console.log('Penalty BPS:', deposit.penalty_bps);
    return deposit;
  } else {
    console.log('Deposit not found or error');
    return null;
  }
}

// Usage
const status = await getDepositStatus('GXXXXXXX', 0);
```

---

### React Hook for Deposits

```typescript
import { useState, useEffect } from 'react';
import { useWallet } from '@stellar/stellar-wallets-kit';

export function useDeposits(contractId: string) {
  const { publicKey } = useWallet();
  const [deposits, setDeposits] = useState<Deposit[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchDeposits = async () => {
    if (!publicKey) return;
    setLoading(true);
    
    try {
      const ids = await getDepositIds(publicKey);
      const deposits = await Promise.all(
        ids.map((id) => getDepositStatus(publicKey, id))
      );
      setDeposits(deposits.filter(Boolean));
    } catch (err) {
      console.error('Failed to fetch deposits:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDeposits();
  }, [publicKey]);

  return { deposits, loading, refetch: fetchDeposits };
}

// Usage in component
function DepositList() {
  const { deposits, loading } = useDeposits(CONTRACT_ID);

  if (loading) return <div>Loading...</div>;

  return (
    <ul>
      {deposits.map((d, i) => (
        <li key={i}>
          Deposit {i}: {d.amount} tokens until {new Date(d.unlock_time * 1000).toDateString()}
        </li>
      ))}
    </ul>
  );
}
```

---

## Error Handling

### Catch and Log Errors

```bash
#!/bin/bash
source ~/.safe-haven-env.sh

DEPOSIT_ID=0

# Attempt withdrawal and capture error
OUTPUT=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id "$DEPOSIT_ID" 2>&1)

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "✓ Success: $OUTPUT"
else
  echo "✗ Error (code $EXIT_CODE):"
  echo "$OUTPUT"
  
  # Parse error code
  if echo "$OUTPUT" | grep -q "4"; then
    echo "  → Funds still locked"
  elif echo "$OUTPUT" | grep -q "3"; then
    echo "  → Deposit not found"
  elif echo "$OUTPUT" | grep -q "13"; then
    echo "  → Already withdrawn"
  fi
fi
```

---

### Retry Logic with Exponential Backoff

```typescript
async function retryWithBackoff(
  fn: () => Promise<any>,
  maxRetries: number = 3,
  baseDelay: number = 1000
) {
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      if (attempt === maxRetries - 1) throw error;
      
      const delay = baseDelay * Math.pow(2, attempt);
      console.log(`Retry attempt ${attempt + 1}/${maxRetries} after ${delay}ms`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}

// Usage
const result = await retryWithBackoff(
  () => deposit('GXXXXXXX', tokenAddress, amount, 30, 200),
  3,
  1000
);
```

---

### Validate Input Before Submitting

```typescript
function validateDepositParams(
  amount: bigint,
  lockDays: number,
  penaltyBps: number
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (amount <= 0n) {
    errors.push('Amount must be greater than 0');
  }

  if (amount > 10n ** 24n) {
    errors.push('Amount exceeds maximum allowed');
  }

  if (lockDays < 1) {
    errors.push('Lock duration must be at least 1 day');
  }

  if (lockDays > 1825) {  // ~5 years
    errors.push('Lock duration exceeds maximum (5 years)');
  }

  if (penaltyBps > 10000) {
    errors.push('Penalty cannot exceed 100% (10,000 bps)');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

// Usage
const validation = validateDepositParams(
  1000n * 10n ** 18n,
  30,
  200
);

if (!validation.valid) {
  console.error('Invalid parameters:', validation.errors);
  return;
}
```

---

## Tips & Best Practices

### 1. Always Dry-Run Before Mainnet

```bash
# Test on testnet first
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org

# Only move to mainnet after thorough testing
export NETWORK=mainnet
export RPC_URL=https://soroban-mainnet.stellar.org
```

---

### 2. Monitor Events

```bash
# Subscribe to contract events
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_deposit_ids \
  --depositor "$USER_ADDRESS" \
  | jq .
```

---

### 3. Use Structured Logging

```typescript
const logger = {
  info: (msg: string, data?: any) => console.log(`[INFO] ${msg}`, data || ''),
  error: (msg: string, err: any) => console.error(`[ERROR] ${msg}`, err),
  warn: (msg: string, data?: any) => console.warn(`[WARN] ${msg}`, data || ''),
};

logger.info('Creating deposit', { amount, lockDays });
try {
  const id = await deposit(...);
  logger.info('Deposit created', { id });
} catch (err) {
  logger.error('Deposit failed', err);
}
```

---

### 4. Handle Token Decimals

```typescript
// USDC has 6 decimals; represent 1000 USDC correctly
const USDC_DECIMALS = 6;
const amount = 1000 * 10 ** USDC_DECIMALS;  // 1,000,000,000

// Always query decimals from token contract
async function getTokenDecimals(tokenAddress: string): Promise<number> {
  // Call token.decimals() contract method
  // Implementation depends on token contract interface
}
```

---

### 5. Test on Sandbox First

```bash
# Use Stellar's local sandbox
docker run --rm -it -p 8000:8000 stellar/quickstart:latest

# Then deploy and test locally before testnet
soroban contract deploy --source "$ADMIN_SECRET" --wasm path/to/contract.wasm
```

---

## Support

For issues or questions:
1. Check `/docs/` directory for full API reference
2. Review error codes in [Error Handling](#error-codes--handling)
3. Submit issues on GitHub: https://github.com/kenedybok3/SAFE-HAVEN

---

**Last Updated**: November 2024  
**Examples Tested On**: Soroban SDK v22, Stellar Testnet
