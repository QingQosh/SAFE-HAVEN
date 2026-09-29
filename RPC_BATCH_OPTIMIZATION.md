# RPC Rate-Limiting Fix: Batch Vault Fetching

## Problem

For each deposit ID, the frontend was firing two concurrent RPC simulations (`getVault` and `getTimeRemaining`). For a user with 20 deposits, this resulted in **40 sequential simulations** fired in a single burst:

```
Promise.all([
  getVault(id1), getTimeRemaining(id1),
  getVault(id2), getTimeRemaining(id2),
  ...
  getVault(id20), getTimeRemaining(id20),
])
```

Most RPC endpoints have strict rate limits and will either:
- Slow down the loading dramatically
- Silently fail some requests
- Return `null` for some vaults, making the user's vault list appear incomplete

## Solution

### 1. New Contract Function: `get_deposit_batch`

Added a new read-only query that fetches multiple deposits for a single depositor in one RPC call:

```rust
pub fn get_deposit_batch(
    env: Env,
    depositor: Address,
    deposit_ids: Vec<u32>,
) -> Vec<(u32, Option<VaultEntry>)> {
    // Limit: up to 25 deposit IDs per call (MAX_BATCH_SIZE)
    // Returns Vec of (deposit_id, Option<VaultEntry>) tuples
}
```

**File:** `contracts/safe-haven/src/contract.rs`

### 2. Frontend RPC Wrapper: `getDepositBatch`

Added a new function in `stellar.ts` to call the contract's batch function:

```typescript
export async function getDepositBatch(
  depositor: string,
  depositIds: number[],
): Promise<{ id: number; entry: VaultEntry | null }[]>
```

This returns deposit entries grouped by their ID, parsed from the contract response.

**File:** `frontend/src/lib/stellar.ts`

### 3. Updated Hook: `useDeposits`

Refactored to use the batch function instead of individual calls:

**Before (40 RPC calls for 20 deposits):**
```typescript
const results = await Promise.all(
  ids.map(async (id) => {
    const [entry, remaining] = await Promise.all([
      getVault(depositor, id),
      getTimeRemaining(depositor, id),
    ])
    return { ...entry, depositId: id, timeRemaining: remaining }
  }),
)
```

**After (1-2 RPC calls for 20 deposits):**
```typescript
const now = await getLedgerTime()  // 1 call
const batchSize = 25
for (let i = 0; i < ids.length; i += batchSize) {
  const batch = ids.slice(i, i + batchSize)
  const results = await getDepositBatch(depositor, batch)  // 1 call per batch
  for (const { entry } of results) {
    const timeRemaining = Math.max(0, entry.unlockTime - now)  // computed client-side
  }
}
```

**File:** `frontend/src/hooks/useDeposits.ts`

## Impact

| Metric | Before | After |
|--------|--------|-------|
| RPC calls for 20 deposits | 40 | 2 (1 for time + 1 for batch) |
| RPC burst size | 40 concurrent | 1-2 sequential |
| Rate-limit pressure | High | Low |
| Vault list incompleteness | Possible (rate-limit failures) | Eliminated |
| Load time | Slower (waits for all 40) | Faster (2 calls) |

## How Time Remaining is Computed

Instead of fetching `time_remaining` from the contract, it's now computed client-side:

1. Call `getLedgerTime()` once to get current ledger timestamp
2. For each vault, compute: `timeRemaining = max(0, vault.unlockTime - now)`
3. On every second, decrement all `timeRemaining` by 1 (existing countdown logic)

This avoids expensive redundant RPC calls while maintaining accuracy.

## When to Use `getTimeRemaining` for Precision

The `getTimeRemaining` RPC call should be reserved for:
- **Pre-withdrawal verification** — Confirm exact time remaining immediately before withdrawal to ensure funds are truly unlocked
- **Critical accuracy scenarios** — When you need absolute guarantee that time hasn't drifted due to local clock skew

For general UI display, client-side calculation from `unlockTime` is sufficient and efficient.

## Backward Compatibility

- `getVault()` and `getTimeRemaining()` remain available for legacy use
- Existing code using individual calls will still work
- New code should prefer batch fetching for better performance
- Contract remains fully backward compatible

## Testing

To test the batch function:

1. **Contract:** `cargo test --features testutils` (existing tests verify vault storage integrity)
2. **Frontend:** The hook automatically batches calls; observe network tab for reduced RPC volume
3. **Load time:** Measure time to populate vault list with many deposits

Example flow:
```
1. User connects wallet
2. useDeposits fetches deposit IDs (1 call: getDepositIds)
3. useDeposits fetches all vaults in batch (1-2 calls: getDepositBatch)
4. useDeposits gets current time (1 call: getLedgerTime)
5. Client computes timeRemaining for each vault
6. UI displays complete vault list quickly
```

## Future Optimizations

1. **Ledger-based deposits:** Add `get_ledger_deposit_batch` for deposits with unlock by ledger sequence
2. **Pagination:** Add offset/limit to batch functions for large deposit counts (100+)
3. **Cache**: Add TTL-based caching in frontend to avoid redundant calls during rapid refreshes
