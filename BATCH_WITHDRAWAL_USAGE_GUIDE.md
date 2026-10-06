# Batch Withdrawal Usage Guide

## Overview

The batch withdrawal feature enables depositors to withdraw from multiple deposits in a single transaction, reducing transaction fees and complexity.

## API Reference

### Function Signature

```rust
pub fn withdraw_batch(
    env: Env,
    depositor: Address,
    deposit_ids: Vec<u32>,
) -> Result<BatchWithdrawalResult, VaultError>
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `env` | `Env` | Soroban environment (provided by SDK) |
| `depositor` | `Address` | Account owner (must sign transaction) |
| `deposit_ids` | `Vec<u32>` | Vector of deposit IDs to withdraw from (max 25) |

### Return Value

```rust
pub struct BatchWithdrawalResult {
    pub results: Vec<WithdrawalResult>,  // Per-deposit results
    pub total_attempted: u32,            // Number of deposits attempted
    pub successful_count: u32,           // Number successful
    pub failed_count: u32,               // Number failed
    pub total_amount: i128,              // Total amount withdrawn
}

pub struct WithdrawalResult {
    pub deposit_id: u32,                 // Deposit ID
    pub success: bool,                   // Success flag
    pub error_code: u32,                 // Error code (0 = success)
    pub amount: i128,                    // Amount withdrawn
}
```

## Usage Examples

### Rust/Soroban Example

```rust
use soroban_sdk::{Address, Vec, Env};
use safe_haven::SafeHavenClient;

fn example_batch_withdraw(
    env: &Env,
    vault: &SafeHavenClient,
    depositor: Address,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create vector of deposit IDs to withdraw from
    let mut deposit_ids = Vec::new(env);
    deposit_ids.push_back(0);
    deposit_ids.push_back(1);
    deposit_ids.push_back(2);

    // Call batch_withdraw
    let result = vault.withdraw_batch(&depositor, &deposit_ids)?;

    // Check results
    println!("Total attempted: {}", result.total_attempted);
    println!("Successful: {}", result.successful_count);
    println!("Failed: {}", result.failed_count);
    println!("Total withdrawn: {}", result.total_amount);

    // Iterate through per-deposit results
    for withdrawal_result in result.results.iter() {
        if withdrawal_result.success {
            println!(
                "Deposit {} withdrawn: {}",
                withdrawal_result.deposit_id, withdrawal_result.amount
            );
        } else {
            println!(
                "Deposit {} failed with error: {}",
                withdrawal_result.deposit_id, withdrawal_result.error_code
            );
        }
    }

    Ok(())
}
```

### TypeScript/Frontend Example

```typescript
import { Address } from "@stellar/js-stellar-base";
import { SafeHavenClient } from "./safe-haven-client";

interface WithdrawalResult {
  deposit_id: number;
  success: boolean;
  error_code: number;
  amount: string;
}

interface BatchWithdrawalResult {
  results: WithdrawalResult[];
  total_attempted: number;
  successful_count: number;
  failed_count: number;
  total_amount: string;
}

async function batchWithdrawExamples(
  client: SafeHavenClient,
  depositorAddress: string
): Promise<void> {
  // Example 1: Withdraw from all deposits
  const allDepositIds = [0, 1, 2, 3, 4];
  
  try {
    const result = await client.withdrawBatch(
      depositorAddress,
      allDepositIds
    );
    
    console.log(`Batch Result:`);
    console.log(`  Total: ${result.total_attempted}`);
    console.log(`  Success: ${result.successful_count}`);
    console.log(`  Failed: ${result.failed_count}`);
    console.log(`  Amount: ${result.total_amount}`);

    // Process per-deposit results
    for (const res of result.results) {
      if (res.success) {
        console.log(
          `✓ Deposit #${res.deposit_id}: ${res.amount} withdrawn`
        );
      } else {
        console.log(
          `✗ Deposit #${res.deposit_id}: Error ${res.error_code}`
        );
      }
    }
  } catch (error) {
    if (error.code === 27) {
      console.error("Batch size exceeds maximum (25)");
    } else {
      console.error("Batch withdrawal failed:", error);
    }
  }

  // Example 2: Selective withdrawal
  const selectiveIds = [0, 2, 4]; // Skip deposits 1 and 3
  const result2 = await client.withdrawBatch(
    depositorAddress,
    selectiveIds
  );
  console.log(`Selective withdrawal: ${result2.successful_count} succeeded`);

  // Example 3: Handle partial success
  const result3 = await client.withdrawBatch(
    depositorAddress,
    [0, 1, 2]
  );
  
  const unlocked = result3.results.filter((r) => r.success);
  const locked = result3.results.filter((r) => !r.success);
  
  console.log(`Unlocked deposits: ${unlocked.length}`);
  console.log(`Locked deposits: ${locked.length}`);
  console.log(`Total withdrawn: ${result3.total_amount}`);
}
```

### CLI Example (Stellar CLI)

```bash
# Set up environment
export CONTRACT_ID="CXXXXXX..."
export DEPOSITOR_ADDRESS="GXXXXXX..."
export SOROBAN_SECRET_KEY="S..."

# Batch withdraw from deposits 0, 1, 2
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  --source $SOROBAN_SECRET_KEY \
  -- withdraw_batch \
  --depositor $DEPOSITOR_ADDRESS \
  --deposit_ids '[0, 1, 2]'

# Response example:
# {
#   "results": [
#     {"deposit_id": 0, "success": true, "error_code": 0, "amount": 100},
#     {"deposit_id": 1, "success": true, "error_code": 0, "amount": 200},
#     {"deposit_id": 2, "success": false, "error_code": 4, "amount": 0}
#   ],
#   "total_attempted": 3,
#   "successful_count": 2,
#   "failed_count": 1,
#   "total_amount": 300
# }
```

## Error Codes

| Code | Name | Meaning | Handling |
|------|------|---------|----------|
| 0 | Success | Deposit withdrawn | Use `amount` field |
| 3 | NoDepositFound | Deposit doesn't exist | Check deposit ID validity |
| 4 | FundsStillLocked | Lock time not reached | Retry after unlock time |
| 27 | BatchSizeExceeded | More than 25 deposits | Split into multiple batches |
| other | Various | Check `VaultError` enum | Refer to error docs |

## Common Patterns

### 1. Batch Withdraw Everything

```typescript
async function withdrawAll(
  client: SafeHavenClient,
  depositor: string
): Promise<void> {
  // Get all deposit IDs
  const deposits = await client.getDepositIds(depositor);
  
  // Convert to array and batch withdraw
  const depositArray = deposits.map((id) => parseInt(id));
  const result = await client.withdrawBatch(depositor, depositArray);
  
  console.log(`Withdrawn ${result.successful_count} deposits`);
  return result;
}
```

### 2. Batch Withdraw Unlocked Only

```typescript
async function withdrawUnlocked(
  client: SafeHavenClient,
  depositor: string
): Promise<void> {
  const deposits = await client.getDepositIds(depositor);
  
  // Check which are unlocked
  const unlockedIds: number[] = [];
  for (const id of deposits) {
    const timeRemaining = await client.timeRemaining(depositor, id);
    if (timeRemaining === 0) {
      unlockedIds.push(parseInt(id));
    }
  }
  
  // Batch withdraw unlocked
  if (unlockedIds.length > 0) {
    const result = await client.withdrawBatch(depositor, unlockedIds);
    console.log(`Withdrawn ${result.total_amount} from ${result.successful_count} deposits`);
  }
}
```

### 3. Paginated Batch Withdrawals (for > 25 deposits)

```typescript
async function withdrawAllPaginated(
  client: SafeHavenClient,
  depositor: string
): Promise<void> {
  const deposits = await client.getDepositIds(depositor);
  const depositArray = deposits.map((id) => parseInt(id));
  
  let totalWithdrawn = 0;
  
  // Process in batches of 25
  for (let i = 0; i < depositArray.length; i += 25) {
    const batch = depositArray.slice(i, i + 25);
    const result = await client.withdrawBatch(depositor, batch);
    
    totalWithdrawn += parseInt(result.total_amount);
    console.log(
      `Batch ${Math.floor(i / 25) + 1}: ` +
      `${result.successful_count} succeeded, ` +
      `${result.failed_count} failed`
    );
  }
  
  console.log(`Total withdrawn: ${totalWithdrawn}`);
}
```

### 4. Error Recovery

```typescript
async function smartBatchWithdraw(
  client: SafeHavenClient,
  depositor: string,
  depositIds: number[]
): Promise<void> {
  // Check batch size first
  if (depositIds.length > 25) {
    throw new Error(`Too many deposits (${depositIds.length}), max is 25`);
  }
  
  try {
    const result = await client.withdrawBatch(depositor, depositIds);
    
    // Log failures
    const failed = result.results.filter((r) => !r.success);
    if (failed.length > 0) {
      console.warn(`${failed.length} deposits failed to withdraw:`);
      for (const f of failed) {
        const errorName = getErrorName(f.error_code);
        console.warn(`  Deposit ${f.deposit_id}: ${errorName}`);
      }
    }
    
    // Log success
    console.log(`Successfully withdrawn: ${result.total_amount}`);
    
  } catch (error: any) {
    if (error.code === 25) {
      console.error("Auth failed - transaction not signed");
    } else {
      console.error("Batch withdrawal failed:", error.message);
    }
    throw error;
  }
}

function getErrorName(code: number): string {
  const errors: Record<number, string> = {
    0: "Success",
    1: "InvalidAmount",
    2: "UnlockTimeNotInFuture",
    3: "NoDepositFound",
    4: "FundsStillLocked",
    // ... add other error codes as needed
    27: "BatchSizeExceeded",
  };
  return errors[code] || `UnknownError(${code})`;
}
```

## Performance Comparison

### Individual Withdrawals vs Batch

```typescript
// ❌ Individual withdrawals (3 deposits)
for (const id of [0, 1, 2]) {
  await contract.withdraw(depositor, id); // 3 transactions
}
// Gas cost: ~3x baseline
// Time: ~3 blocks (15 seconds on Stellar)

// ✅ Batch withdrawal (3 deposits)
const result = await contract.withdraw_batch(depositor, [0, 1, 2]); // 1 transaction
// Gas cost: ~1x baseline (60-70% savings)
// Time: ~1 block (5 seconds on Stellar)
```

## Best Practices

### ✅ DO

- **Use batch withdrawals** for multiple deposits (saves gas)
- **Check error codes** for each deposit in results
- **Handle partial success** - locked deposits don't prevent others from succeeding
- **Split large withdrawals** into multiple batches if > 25 deposits
- **Emit individual events** for audit trails
- **Verify signature** before sending transaction

### ❌ DON'T

- **Assume all-or-nothing** - batch can have mixed success/failure
- **Exceed batch size** - max 25 deposits per batch
- **Ignore error codes** - each deposit has independent status
- **Retry without checking** - locked deposits will always fail until unlock time
- **Send unsigned transactions** - `require_auth()` is enforced
- **Assume order preservation** - though results are in input order

## Troubleshooting

### "BatchSizeExceeded" Error

**Problem:** Trying to withdraw from more than 25 deposits
**Solution:** Split into multiple batches of ≤25

```typescript
// Split large batch
for (let i = 0; i < depositIds.length; i += 25) {
  const batch = depositIds.slice(i, i + 25);
  await contract.withdraw_batch(depositor, batch);
}
```

### "FundsStillLocked" on All Deposits

**Problem:** All deposits are returning error code 4 (FundsStillLocked)
**Solution:** Check unlock times and retry later

```typescript
const now = Math.floor(Date.now() / 1000);
for (const depositId of depositIds) {
  const remaining = await contract.timeRemaining(depositor, depositId);
  if (remaining > 0) {
    console.log(`Deposit ${depositId} unlocks in ${remaining}s`);
  }
}
```

### "NoDepositFound" on Some Deposits

**Problem:** Some deposit IDs don't exist
**Solution:** Verify deposit IDs before batch withdrawal

```typescript
const validIds = [];
for (const id of depositIds) {
  const deposit = await contract.getVault(depositor, id);
  if (deposit) {
    validIds.push(id);
  }
}
const result = await contract.withdraw_batch(depositor, validIds);
```

### Transaction Out of Funds

**Problem:** Not enough native tokens for fees
**Solution:** Check balance before withdrawal

```typescript
const balance = await contract.getBalance(depositor, "native");
if (balance < estimatedFees) {
  throw new Error("Insufficient balance for transaction fees");
}
```

## Integration Checklist

- [ ] Understand `BatchWithdrawalResult` structure
- [ ] Handle per-deposit errors individually
- [ ] Support batch size limit (max 25)
- [ ] Implement pagination for > 25 deposits
- [ ] Add batch withdrawal UI component
- [ ] Display cost savings estimate
- [ ] Show per-deposit status
- [ ] Handle partial success gracefully
- [ ] Log events for audit
- [ ] Test with locked and unlocked deposits
- [ ] Test error scenarios
- [ ] Document for end users

## References

- [Main Implementation Docs](./BATCH_WITHDRAWAL_IMPLEMENTATION.md)
- [Quick Reference](./BATCH_WITHDRAWAL_SUMMARY.md)
- [Soroban SDK Docs](https://github.com/stellar/rs-soroban-sdk)
- [Contract Error Codes](./contracts/safe-haven/src/errors.rs)

