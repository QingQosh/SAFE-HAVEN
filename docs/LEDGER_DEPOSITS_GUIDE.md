# Ledger-Based Deposits Guide

This guide explains when and how to use ledger-based deposits in SAFE-HAVEN, providing practical examples for both CLI and SDK implementations.

## Table of Contents

1. [Overview](#overview)
2. [When to Use Ledger Deposits](#when-to-use-ledger-deposits)
3. [Ledger Deposits vs Timestamp Deposits](#ledger-deposits-vs-timestamp-deposits)
4. [Understanding Ledger Time](#understanding-ledger-time)
5. [CLI Examples](#cli-examples)
6. [TypeScript SDK Examples](#typescript-sdk-examples)
7. [Ledger Calculations](#ledger-calculations)
8. [Common Patterns and Workflows](#common-patterns-and-workflows)
9. [Troubleshooting](#troubleshooting)

---

## Overview

SAFE-HAVEN supports two types of time-locked deposits:

1. **Timestamp-based deposits** — unlock at a specific Unix timestamp (wall-clock time)
2. **Ledger-based deposits** — unlock at a specific Stellar ledger sequence number

Ledger-based deposits are useful when your lock condition is expressed in terms of ledger sequence rather than wall-clock time. This guide shows how to use them effectively.

### Key Properties

| Property | Timestamp | Ledger |
|----------|-----------|--------|
| **Unlock condition** | `current_time >= unlock_time` | `current_ledger >= unlock_ledger` |
| **Precision** | Unix seconds (approximate) | Ledger sequence (exact) |
| **Min lock** | 60 seconds | 12 ledgers (~60 seconds) |
| **Max lock** | 5 years | ~2 billion ledgers |
| **Use case** | Calendar-based events | Protocol events, block-based timings |

---

## When to Use Ledger Deposits

### Ideal Use Cases for Ledger Deposits

**1. Protocol-Coordinated Events**

When your unlock event is defined in ledger terms by a protocol or external system:

```
✓ "Release after Stellar mainnet reaches ledger 50,000,000"
✓ "Unlock tokens at the ledger of the Soroban network upgrade"
✓ "Release after the next 48 validator cycles"
```

These are naturally expressed as ledger numbers, not timestamps.

**2. Block-Based Monitoring**

When your off-chain system tracks ledger sequence numbers and you want exact correspondence:

```typescript
// Your monitoring system
const currentLedger = 48_123_456;
const unlockLedger = 48_123_456 + 1_000;

// Later, verify withdrawal eligibility
if (currentLedger >= unlockLedger) {
  // Safe to withdraw — exact match, no approximation
}
```

With timestamp-based deposits, you'd need to convert the ledger number to an estimated timestamp (5 s/ledger), introducing approximation error.

**3. Fixed-Duration Locks in Ledger Terms**

When you want to express lock duration as a number of ledgers:

```
✓ "Lock for exactly 2,000 ledgers" (~2.78 hours at 5 s/ledger)
✓ "Release after 365 * 24 * 12 ledgers" (approximately 1 year)
```

Ledger-based deposits guarantee the duration in ledger terms; timestamp-based locks can drift if validator behavior changes.

**4. Audit and Transparency**

When you need to prove on-chain exactly which ledger you specified:

```
Timestamp deposit: "I said unlock_time = 1750000000"
  → Network determines equivalently "that's about ledger 47,521,456"
  → Approximation depends on validator time-setting

Ledger deposit: "I said unlock_ledger = 47521456"
  → Network checks exactly "is current_ledger >= 47521456?"
  → No approximation, fully deterministic
```

---

## Ledger Deposits vs Timestamp Deposits

### When to Use Timestamp Deposits

**Stick with timestamp-based deposits if:**

- Your unlock event is expressed in calendar time ("Q4 2026" or "January 15, 2027")
- You integrate with time-based systems (Unix cron, ISO 8601 timestamps)
- You need a UI that shows "unlock date: December 25, 2026"
- Your contract coordinates with other Soroban contracts expecting Unix timestamps

Timestamp deposits are the default and work well for most use cases.

### Side-by-Side Comparison

```rust
// Timestamp-based deposit (time-expressed events)
deposit(
  depositor: user,
  token: USDC,
  amount: 1_000,
  unlock_time: 1750000000,  // Unix timestamp (Dec 2025)
  penalty_bps: 500
);
// Withdrawable when env.ledger().timestamp() >= 1750000000

// Ledger-based deposit (ledger-expressed events)
deposit_by_ledger(
  depositor: user,
  token: USDC,
  amount: 1_000,
  unlock_ledger: 50000000,  // Ledger sequence number
  penalty_bps: 500
);
// Withdrawable when env.ledger().sequence() >= 50000000
```

**Advantages of ledger-based:**
- No approximation in unlock condition
- Exact correspondence with ledger sequence
- Better for protocol-coordinated locks

**Advantages of timestamp-based:**
- Easier to reason about (human-readable dates)
- Better for calendar-based events
- More familiar to most developers

---

## Understanding Ledger Time

### What Is a Ledger Sequence?

A **ledger sequence** is a strictly incrementing counter that starts at 1 on Stellar genesis and increases by exactly 1 with each new ledger closed by the network.

```
Ledger 1 (genesis)
  ↓ (~5 seconds later)
Ledger 2
  ↓ (~5 seconds later)
Ledger 3
  ...
```

### The 5-Second Rule

Stellar aims to close a ledger approximately every 5 seconds:

```
1 ledger ≈ 5 seconds
1 hour ≈ 720 ledgers
1 day ≈ 17,280 ledgers
1 year ≈ 6,307,200 ledgers
```

**Important:** This is an approximation. The actual time per ledger can vary:

- Validators may close ledgers faster or slower
- Network congestion can affect timing
- Ledger close times are determined by validator consensus

**SAFE-HAVEN's approach:**
- Ledger-based deposits use **exact ledger sequence** (no approximation)
- For estimation (e.g., showing "approximately 2 hours"), multiply by 5 seconds
- Never rely on the 5-second rule for protocol-critical calculations

### Querying Current Ledger Time

```rust
let current_sequence: u32 = env.ledger().sequence();
let current_timestamp: u64 = env.ledger().timestamp();
```

In practice:
- `sequence` increments by 1 per ledger
- `timestamp` may not increment uniformly (validators set timestamps)

---

## CLI Examples

### Environment Setup

Before running any CLI commands, set your environment variables:

```bash
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export CONTRACT_ID="CC..."        # Your deployed contract
export USER_SECRET="S..."          # Your secret key
export USER_ADDRESS="G..."         # Your account address
export TOKEN_CONTRACT="CC..."      # USDC or other token
```

### Example 1: Deposit Locked to a Specific Ledger

Lock 1,000 tokens until ledger 50,000,000 with a 2% early-exit penalty:

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
  --amount 1000 \
  --unlock_ledger 50000000 \
  --penalty_bps 200
```

**Output:** A `u32` deposit ID (e.g., `0`, `1`, `2`, ...)

**Result:** The 1,000 tokens are locked in the vault and can only be withdrawn after the network reaches ledger 50,000,000. If withdrawn before that ledger, a 2% penalty is deducted.

### Example 2: Calculate Ledger Gap

You want to lock tokens for 365 days. At 5 seconds/ledger:

```bash
# Calculate ledger gap
# 365 days × 24 hours × 60 min × 60 sec = 31,536,000 seconds
# 31,536,000 seconds ÷ 5 seconds/ledger = 6,307,200 ledgers

# Current ledger (example output: 48123456)
CURRENT_LEDGER=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_time)

echo "Current ledger: $CURRENT_LEDGER"

# Calculate unlock ledger
UNLOCK_LEDGER=$((CURRENT_LEDGER + 6307200))
echo "Unlock ledger: $UNLOCK_LEDGER"

# Create the deposit
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- deposit_by_ledger \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_CONTRACT" \
  --amount 100000 \
  --unlock_ledger "$UNLOCK_LEDGER" \
  --penalty_bps 500
```

### Example 3: Retrieve Ledger Deposit Details

Query the vault to see your ledger-based deposit:

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

**Output:**
```json
{
  "token": "CC...",
  "amount": 1000,
  "unlock_ledger": 50000000,
  "depositor": "G...",
  "penalty_bps": 200
}
```

### Example 4: Withdraw After Unlock

Wait for the network to reach the unlock ledger, then withdraw:

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

The contract performs a two-stage lookup:
1. Check for a timestamp-based deposit with ID 0
2. If not found, check for a ledger-based deposit with ID 0

If the ledger-based deposit is found and `current_ledger >= unlock_ledger`, the full 1,000 tokens are returned.

### Example 5: Early Exit with Penalty

Withdraw before the unlock ledger:

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

If the deposit has a 2% penalty:
- 2% (20 tokens) goes to the fee recipient
- 98% (980 tokens) are returned to the depositor

---

## TypeScript SDK Examples

### Installation and Setup

First, ensure you have the Soroban TypeScript SDK:

```bash
npm install @stellar/stellar-sdk
```

### Example 1: Create a Ledger Deposit

```typescript
import {
  Account,
  Keypair,
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  Operation,
} from '@stellar/stellar-sdk';

const rpc = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
const network = Networks.TESTNET_NETWORK_PASSPHRASE;
const contractId = 'CC...';
const tokenId = 'CC...';

const userKeypair = Keypair.fromSecret('S...');
const userAccount = new Account(userKeypair.publicKey(), 0);

async function depositByLedger() {
  // Fetch current ledger
  const ledgerEntries = await rpc.getLatestLedger();
  const currentLedger = ledgerEntries.sequence;
  
  // Calculate unlock ledger (e.g., 100 ledgers from now)
  const unlockLedger = currentLedger + 100;

  // Build the transaction
  const account = await rpc.getAccount(userKeypair.publicKey());
  const contract = new Contract(contractId);

  const transaction = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: network,
  })
    .addOperation(
      contract.call(
        'deposit_by_ledger',
        userKeypair.publicKey(), // depositor
        tokenId,                 // token
        '1000',                  // amount (stroops or smallest unit)
        unlockLedger,            // unlock_ledger (u32)
        200                      // penalty_bps (2%)
      )
    )
    .setTimeout(300)
    .build();

  // Sign and submit
  transaction.sign(userKeypair);
  const result = await rpc.sendTransaction(transaction);

  console.log(`Deposit ID: ${result.id}`);
  return result;
}

await depositByLedger();
```

### Example 2: Query a Ledger Deposit

```typescript
import { Contract } from '@stellar/stellar-sdk';

async function getLedgerVault(depositorAddress: string, depositId: number) {
  const rpc = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
  const contractId = 'CC...';

  const contract = new Contract(contractId);

  // Call get_ledger_vault to retrieve the deposit
  const result = await rpc.simulateTransaction(
    new TransactionBuilder(
      new Account('GBRPYHIL2CI3WHZDTOOQFC6EB4CWXLJC3CSTW3KM27KQNVUT5QDRKVZ', 0),
      { fee: '100', networkPassphrase: Networks.TESTNET_NETWORK_PASSPHRASE }
    )
      .addOperation(
        contract.call(
          'get_ledger_vault',
          depositorAddress,
          depositId
        )
      )
      .setTimeout(300)
      .build()
  );

  // Parse the result
  if (result.results?.[0].result?.retval) {
    const deposit = result.results[0].result.retval;
    console.log('Ledger Deposit:', {
      token: deposit.token,
      amount: deposit.amount,
      unlockLedger: deposit.unlock_ledger,
      depositor: deposit.depositor,
      penaltyBps: deposit.penalty_bps,
    });
    return deposit;
  }

  console.log('Deposit not found');
  return null;
}

await getLedgerVault('GBVD...', 0);
```

### Example 3: Withdraw After Unlock

```typescript
async function withdraw(depositorAddress: string, depositId: number) {
  const rpc = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
  const contractId = 'CC...';
  const userKeypair = Keypair.fromSecret('S...');

  const contract = new Contract(contractId);
  const account = await rpc.getAccount(userKeypair.publicKey());

  const transaction = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: Networks.TESTNET_NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        'withdraw',
        depositorAddress,
        depositId
      )
    )
    .setTimeout(300)
    .build();

  transaction.sign(userKeypair);
  const result = await rpc.sendTransaction(transaction);

  console.log('Withdraw successful:', result);
  return result;
}

await withdraw('GBVD...', 0);
```

### Example 4: Check Current Ledger

```typescript
async function getCurrentLedger() {
  const rpc = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
  const contractId = 'CC...';

  const contract = new Contract(contractId);

  const result = await rpc.simulateTransaction(
    new TransactionBuilder(
      new Account('GBRPYHIL2CI3WHZDTOOQFC6EB4CWXLJC3CSTW3KM27KQNVUT5QDRKVZ', 0),
      { fee: '100', networkPassphrase: Networks.TESTNET_NETWORK_PASSPHRASE }
    )
      .addOperation(
        contract.call('get_time')
      )
      .setTimeout(300)
      .build()
  );

  if (result.results?.[0].result?.retval) {
    const currentLedger = result.results[0].result.retval;
    console.log('Current Ledger:', currentLedger);
    return currentLedger;
  }

  return null;
}

const ledger = await getCurrentLedger();
```

### Example 5: Calculate Lock Duration

```typescript
function calculateLedgerGap(durationSeconds: number): number {
  const LEDGER_SECONDS = 5;
  return Math.ceil(durationSeconds / LEDGER_SECONDS);
}

function calculateEstimatedTime(ledgers: number): number {
  const LEDGER_SECONDS = 5;
  return ledgers * LEDGER_SECONDS;
}

// Lock for 1 hour
const oneHourSeconds = 60 * 60;
const ledgersFor1Hour = calculateLedgerGap(oneHourSeconds);
console.log(`1 hour ≈ ${ledgersFor1Hour} ledgers`);

// Lock for 7 days
const sevenDaysSeconds = 7 * 24 * 60 * 60;
const ledgersFor7Days = calculateLedgerGap(sevenDaysSeconds);
console.log(`7 days ≈ ${ledgersFor7Days} ledgers`);

// Verify round-trip
const estimatedSeconds = calculateEstimatedTime(ledgersFor1Hour);
console.log(`${ledgersFor1Hour} ledgers ≈ ${estimatedSeconds} seconds`);
```

---

## Ledger Calculations

### Key Formulas

**From seconds to ledgers:**
```
ledger_gap = ceil(duration_seconds / 5)
```

**From ledgers to seconds (estimate):**
```
estimated_seconds = ledger_gap × 5
```

**Absolute unlock ledger:**
```
unlock_ledger = current_ledger + ledger_gap
```

### Common Durations

| Duration | Seconds | Ledgers | Formula |
|----------|---------|---------|---------|
| 1 minute | 60 | 12 | `60 ÷ 5` |
| 1 hour | 3,600 | 720 | `3,600 ÷ 5` |
| 1 day | 86,400 | 17,280 | `86,400 ÷ 5` |
| 1 week | 604,800 | 120,960 | `604,800 ÷ 5` |
| 1 month (30 days) | 2,592,000 | 518,400 | `2,592,000 ÷ 5` |
| 1 year | 31,536,000 | 6,307,200 | `31,536,000 ÷ 5` |

### Minimum and Maximum Locks

SAFE-HAVEN enforces bounds on ledger-based deposits:

```rust
const MIN_LOCK_LEDGERS: u32 = 12;      // ~60 seconds
const MAX_LOCK_LEDGERS: u32 = 10_000_000_000; // ~158 years
```

If you attempt to create a deposit with `ledger_gap < 12`, the contract returns `LockDurationTooShort`.

### Practical Calculation Example

You want to lock tokens for approximately 90 days. Here's the calculation:

```
Duration: 90 days
Seconds: 90 × 24 × 60 × 60 = 7,776,000 seconds
Ledger gap: 7,776,000 ÷ 5 = 1,555,200 ledgers

Current ledger (example): 48,000,000
Unlock ledger: 48,000,000 + 1,555,200 = 49,555,200

Verification: 1,555,200 ledgers × 5 seconds/ledger = 7,776,000 seconds ✓
```

---

## Common Patterns and Workflows

### Pattern 1: Lock-and-Forget with Ledger Precision

**Scenario:** You want to guarantee funds are locked for exactly N ledgers, regardless of network time drift.

```typescript
async function createFixedLedgerLock(
  depositAmount: number,
  lockLedgers: number
) {
  const rpc = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
  const ledger = await rpc.getLatestLedger();
  const unlockLedger = ledger.sequence + lockLedgers;

  // Create deposit
  console.log(`Locking ${depositAmount} tokens for ${lockLedgers} ledgers`);
  console.log(`Current ledger: ${ledger.sequence}`);
  console.log(`Unlock ledger: ${unlockLedger}`);

  // ... call deposit_by_ledger with unlockLedger
}

// Lock 1,000 tokens for exactly 500 ledgers
await createFixedLedgerLock(1000, 500);
```

**Advantage:** The lock duration is exact, not approximate.

### Pattern 2: Protocol-Coordinated Release

**Scenario:** A protocol specifies that funds should release at ledger 50,000,000. You want to deposit with exact correspondence.

```typescript
async function depositUntilProtocolEvent(
  amount: number,
  protocolEventLedger: number
) {
  // Validate: event must be in the future
  const currentLedger = (await rpc.getLatestLedger()).sequence;
  if (protocolEventLedger <= currentLedger) {
    throw new Error('Event ledger must be in the future');
  }

  // Deposit directly to the event ledger
  console.log(`Depositing until protocol event at ledger ${protocolEventLedger}`);
  
  // ... call deposit_by_ledger with protocolEventLedger
  // Result: funds release exactly when the protocol event occurs
}

await depositUntilProtocolEvent(5000, 50000000);
```

### Pattern 3: Ladder Deposits for Progressive Release

**Scenario:** You want to release tokens in stages. Create multiple ledger deposits at different unlock ledgers.

```typescript
async function createLadderDeposits(
  amount: number,
  stageCount: number,
  ledgersPerStage: number
) {
  const currentLedger = (await rpc.getLatestLedger()).sequence;
  const depositIds = [];

  for (let i = 0; i < stageCount; i++) {
    const unlockLedger = currentLedger + (i + 1) * ledgersPerStage;
    
    console.log(`Stage ${i + 1}: Unlock at ledger ${unlockLedger}`);
    
    // ... call deposit_by_ledger
    // depositIds.push(resultId);
  }

  return depositIds;
}

// Create 4 releases over 1000 ledgers each
// First release: ~5000 seconds (1.4 hours)
// Last release: ~20000 seconds (5.6 hours)
await createLadderDeposits(10000, 4, 1000);
```

### Pattern 4: Monitoring with Exact Correspondence

**Scenario:** Your off-chain system monitors deposits. Use ledger-based deposits for exact auditability.

```typescript
// Off-chain monitoring
async function checkDepositStatus(depositor: string, depositId: number) {
  const deposit = await getLedgerVault(depositor, depositId);
  
  if (!deposit) {
    console.log('Deposit not found');
    return;
  }

  const currentLedger = (await rpc.getLatestLedger()).sequence;
  const isUnlocked = currentLedger >= deposit.unlock_ledger;

  console.log(`Deposit ${depositId}:`);
  console.log(`  Amount: ${deposit.amount}`);
  console.log(`  Current ledger: ${currentLedger}`);
  console.log(`  Unlock ledger: ${deposit.unlock_ledger}`);
  console.log(`  Status: ${isUnlocked ? 'UNLOCKED' : 'LOCKED'}`);
  console.log(`  Ledgers remaining: ${Math.max(0, deposit.unlock_ledger - currentLedger)}`);
}

await checkDepositStatus('GBVD...', 0);
```

### Pattern 5: Batch Deposits with Consistent Locks

**Scenario:** Deposit funds from multiple token sources, all unlocking at the same ledger.

```typescript
async function batchDepositToLedger(
  deposits: Array<{ token: string; amount: number }>,
  unlockLedger: number
) {
  const depositor = userKeypair.publicKey();

  for (const { token, amount } of deposits) {
    console.log(`Depositing ${amount} of ${token} until ledger ${unlockLedger}`);
    
    // ... call deposit_by_ledger for each token
  }

  console.log(`All deposits synchronized to unlock at ledger ${unlockLedger}`);
}

await batchDepositToLedger([
  { token: 'USDC_ID', amount: 1000 },
  { token: 'USDT_ID', amount: 2000 },
  { token: 'EUR_ID', amount: 500 },
], 50000000);
```

---

## Troubleshooting

### "LockDurationTooShort" Error

**Problem:** You get `VaultError::LockDurationTooShort` when creating a deposit.

**Cause:** `unlock_ledger - current_ledger < 12`

**Solution:** Increase the ledger gap to at least 12 ledgers (~60 seconds):

```typescript
const currentLedger = (await rpc.getLatestLedger()).sequence;
const minimumUnlock = currentLedger + 12;  // At least 12 ledgers

// ✓ Correct
const unlockLedger = currentLedger + 100;  // 100 ledgers from now

// ✗ Incorrect
const unlockLedger = currentLedger + 5;    // Too short! Only 5 ledgers
```

### "LockDurationTooLong" Error

**Problem:** You get `VaultError::LockDurationTooLong` when creating a deposit.

**Cause:** `unlock_ledger - current_ledger > MAX_LOCK_LEDGERS`

**Solution:** Reduce the lock duration or use a timestamp-based deposit for very long locks:

```typescript
// Current max: ~10 billion ledgers (~158 years)
const maxLockLedgers = 10_000_000_000;
const currentLedger = (await rpc.getLatestLedger()).sequence;

// ✗ Incorrect: exceeds maximum
const unlockLedger = currentLedger + 100_000_000_000;

// ✓ Correct: within maximum
const unlockLedger = currentLedger + 5_000_000_000;
```

### Deposit Not Found

**Problem:** You call `get_ledger_vault` and get `None`.

**Cause:** Either the deposit was not created, or you're querying the wrong depositor/ID.

**Solution:** Check your deposit creation:

```typescript
// Verify the deposit ID after creation
const result = await depositByLedger(userAddress, tokenId, amount, unlockLedger, penalty);
console.log('Created deposit ID:', result.depositId);

// Later, query with the correct ID
const deposit = await getLedgerVault(userAddress, result.depositId);
```

### Withdrawal Fails with "FundsStillLocked"

**Problem:** You call `withdraw` before the unlock ledger and get `FundsStillLocked`.

**Cause:** The current ledger is less than `unlock_ledger`.

**Solution:** Either wait for the ledger to reach `unlock_ledger`, or use `cancel_deposit` for early exit:

```typescript
const currentLedger = (await rpc.getLatestLedger()).sequence;
const deposit = await getLedgerVault(depositor, depositId);

if (currentLedger < deposit.unlock_ledger) {
  console.log(`Funds still locked. ${deposit.unlock_ledger - currentLedger} ledgers remaining.`);
  
  // Option 1: Wait for ledger
  console.log('Waiting for ledger...');
  
  // Option 2: Exit early with penalty
  console.log('Canceling deposit with penalty...');
  // await cancelDeposit(depositor, depositId);
}
```

### Ledger Timestamp Not Advancing

**Problem:** You notice `env.ledger().sequence()` is not incrementing as expected.

**Cause:** Network may be congested or validators may have unusual time settings.

**Reason for using ledger-based deposits:** This is exactly why ledger-based deposits exist! The sequence is guaranteed to increment by exactly 1 per ledger, while timestamps can vary.

**Solution:** Ledger-based deposits guarantee exact sequence behavior. Use them instead of timestamps for protocol-critical timings.

### Penalty Calculation Appears Wrong

**Problem:** You cancel a deposit early and the penalty amount doesn't match your calculation.

**Explanation:** Penalties are calculated as `amount × penalty_bps / 10_000`.

**Example:**
```
Amount: 1,000 tokens
Penalty: 500 bps (5%)
Penalty deducted: 1,000 × 500 / 10,000 = 50 tokens
Returned: 1,000 - 50 = 950 tokens
```

**Verify your calculation:**

```typescript
function calculatePenalty(amount: bigint, penaltyBps: number): bigint {
  return (amount * BigInt(penaltyBps)) / BigInt(10_000);
}

const amount = BigInt(1_000);
const penalty = calculatePenalty(amount, 500);
console.log(`Amount: ${amount}, Penalty: ${penalty}, Returned: ${amount - penalty}`);
// Amount: 1000, Penalty: 50, Returned: 950
```

---

## Summary

Ledger-based deposits are powerful for:

- ✓ Protocol-coordinated events
- ✓ Exact ledger sequence correspondence
- ✓ Block-based monitoring systems
- ✓ Audit and transparency

They're not necessary for:

- ✗ Calendar-based time locks ("December 25")
- ✗ Simple "lock for N days" scenarios
- ✗ Integration with non-Soroban time-based systems

**Next steps:**

1. Decide if your use case is ledger-based or timestamp-based
2. Use the CLI examples to prototype
3. Integrate into your TypeScript SDK workflow
4. Test thoroughly on testnet before mainnet

For more details on deposit mechanics, see the [ADR-001 Dual Deposit Types](./adr/ADR-001-dual-deposit-types.md).

For full API reference, see the [Contract API](./API.md).
