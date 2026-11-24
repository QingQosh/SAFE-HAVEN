# Ledger Deposits Quick Reference

Quick lookup guide for ledger-based deposit operations in SAFE-HAVEN.

## When to Use

| Scenario | Use Ledger | Use Timestamp |
|----------|-----------|---------------|
| "Release at ledger 50,000,000" | ✓ | - |
| "Release on Dec 25, 2026" | - | ✓ |
| "Release in 7 days" | - | ✓ |
| "Release in exactly 1,000 ledgers" | ✓ | - |
| "Release after Soroban upgrade" | ✓ | - |
| Protocol-coordinated events | ✓ | - |
| Calendar/date events | - | ✓ |

## Ledger Time Conversion

```
1 ledger ≈ 5 seconds
1 hour ≈ 720 ledgers
1 day ≈ 17,280 ledgers
1 week ≈ 120,960 ledgers
1 year ≈ 6,307,200 ledgers
```

Formula: `ledgers = ceil(seconds / 5)`

## Constraints

- **Minimum lock:** 12 ledgers (~60 seconds)
- **Maximum lock:** ~10 billion ledgers (~158 years)
- **Penalty:** 0–10,000 bps (0–100%)
- **Amount:** > 0, ≤ max_deposit (default 1 trillion)

## CLI Quick Start

### Create a Ledger Deposit

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network testnet \
  -- deposit_by_ledger \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_ID" \
  --amount 1000 \
  --unlock_ledger 50000000 \
  --penalty_bps 500
```

### Get Current Ledger

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  -- get_time
```

### Query a Ledger Deposit

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  -- get_ledger_vault \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### Withdraw After Unlock

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  -- withdraw \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

### Cancel Early (With Penalty)

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  -- cancel_deposit \
  --depositor "$USER_ADDRESS" \
  --deposit_id 0
```

## TypeScript Quick Start

### Create a Ledger Deposit

```typescript
import { Contract, TransactionBuilder, Account, Keypair } from '@stellar/stellar-sdk';

const contract = new Contract(contractId);
const keypair = Keypair.fromSecret(userSecret);
const account = await rpc.getAccount(keypair.publicKey());

// Get current ledger
const ledger = await rpc.getLatestLedger();
const unlockLedger = ledger.sequence + 500;  // 500 ledgers from now

// Build transaction
const tx = new TransactionBuilder(account, { fee: '100', networkPassphrase })
  .addOperation(contract.call(
    'deposit_by_ledger',
    keypair.publicKey(),  // depositor
    tokenId,              // token
    '1000',               // amount
    unlockLedger,         // unlock_ledger
    500                   // penalty_bps (5%)
  ))
  .setTimeout(300)
  .build();

tx.sign(keypair);
await rpc.sendTransaction(tx);
```

### Query a Deposit

```typescript
const result = await rpc.simulateTransaction(
  new TransactionBuilder(account, { fee: '100', networkPassphrase })
    .addOperation(contract.call('get_ledger_vault', depositorAddress, depositId))
    .setTimeout(300)
    .build()
);

const deposit = result.results[0].result.retval;
console.log(`Unlock ledger: ${deposit.unlock_ledger}`);
console.log(`Amount: ${deposit.amount}`);
console.log(`Penalty: ${deposit.penalty_bps} bps`);
```

### Calculate Lock Duration

```typescript
// 7 days = 604,800 seconds
const sevenDaysLedgers = Math.ceil(604_800 / 5);  // 120,960 ledgers

const currentLedger = (await rpc.getLatestLedger()).sequence;
const unlockLedger = currentLedger + sevenDaysLedgers;
```

## Contract Entry Points

### Deposit Operations

| Function | Purpose | Parameters |
|----------|---------|-----------|
| `deposit_by_ledger` | Lock tokens until ledger | depositor, token, amount, unlock_ledger, penalty_bps |
| `withdraw` | Unlock after reaching ledger | depositor, deposit_id |
| `withdraw_to` | Unlock to recipient | depositor, deposit_id, recipient |
| `cancel_deposit` | Exit early with penalty | depositor, deposit_id |
| `get_ledger_vault` | Query deposit | depositor, deposit_id |
| `get_time` | Current ledger | _(none)_ |

### Two-Stage Lookup

`withdraw`, `withdraw_to`, `cancel_deposit` work on both types:

1. Check timestamp-based deposit with ID
2. If not found, check ledger-based deposit with ID

Callers don't need to specify which type.

## Common Patterns

### Lock Until Protocol Event

```typescript
const eventLedger = 50_000_000;
await depositByLedger(amount, eventLedger, penalty);
// Funds unlock exactly when event occurs
```

### Ladder Releases

```typescript
for (let i = 1; i <= 4; i++) {
  const unlockLedger = currentLedger + i * 1_000;  // Staggered
  await depositByLedger(amount, unlockLedger, penalty);
}
```

### Monitor Deposit Status

```typescript
const deposit = await getLedgerVault(depositor, id);
const ledgersRemaining = Math.max(0, deposit.unlock_ledger - currentLedger);
const isUnlocked = ledgersRemaining === 0;
```

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| `LockDurationTooShort` | Gap < 12 ledgers | Increase to ≥ 12 |
| `LockDurationTooLong` | Gap > 10B ledgers | Decrease or use timestamp |
| `UnlockTimeNotInFuture` | `unlock_ledger <= current` | Use future ledger |
| `FundsStillLocked` | `current < unlock_ledger` | Wait or cancel |
| `NoDepositFound` | ID doesn't exist | Verify deposit ID |
| `FeeRecipientNotSet` | Penalty > 0 but no recipient | Set fee recipient |
| `TokenNotAllowed` | Token in strict allowlist | Use allowed token |

## Penalty Calculation

```
Penalty Amount = Amount × Penalty_BPS / 10,000

Example:
  Amount: 1,000
  Penalty: 500 bps (5%)
  Deducted: 1,000 × 500 / 10,000 = 50
  Returned: 1,000 - 50 = 950
```

## Advantages vs Disadvantages

### Advantages of Ledger Deposits

✓ No approximation error in unlock condition
✓ Exact correspondence with ledger sequence
✓ Better for protocol-coordinated events
✓ Deterministic behavior

### Disadvantages

✗ Not suitable for calendar-based events
✗ Less familiar to most developers
✗ Requires ledger knowledge

### Advantages of Timestamp Deposits

✓ Human-readable dates
✓ Familiar concept (Unix timestamps)
✓ Easier to integrate with time-based systems

### Disadvantages

✗ Approximate (5-second rule)
✗ Affected by validator time drift
✗ Not ideal for protocol events

## Links

- [Full Guide](./LEDGER_DEPOSITS_GUIDE.md)
- [ADR-001: Dual Deposit Types](./adr/ADR-001-dual-deposit-types.md)
- [Contract API](./API.md)

## Example: 90-Day Lock

```bash
# Calculate gap
# 90 days = 7,776,000 seconds
# 7,776,000 / 5 = 1,555,200 ledgers

CURRENT=$(soroban contract invoke --id "$CONTRACT_ID" -- get_time)
UNLOCK=$((CURRENT + 1555200))

soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  -- deposit_by_ledger \
  --depositor "$USER_ADDRESS" \
  --token "$TOKEN_ID" \
  --amount 100000 \
  --unlock_ledger "$UNLOCK" \
  --penalty_bps 500
```

---

**Last Updated:** 2026  
**Status:** Latest  
**See Also:** Timestamp-based deposits, ADR-001, STAKER_REGISTRY_GUIDE
