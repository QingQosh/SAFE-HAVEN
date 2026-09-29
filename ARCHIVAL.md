# Archived Deposits

## Overview

The SAFE-HAVEN vault contract provides a **manual archival system** for completed (withdrawn or cancelled) deposits. This feature enables clean record-keeping and permanent deletion of old deposit records after a 1-year retention period.

## Key Concepts

### What is an Archived Deposit?

An archived deposit is a record of a deposit that has already been **completed** (either withdrawn or cancelled). Once a deposit is archived:

- It cannot be re-withdrawn (the original deposit is already complete)
- It cannot be re-cancelled
- It can only be deleted after 1 year has passed
- It serves as a historical record for auditing and record-keeping

### Archival Timeline

```
┌─────────────────────────────────────────────┐
│ Active Deposit                              │
│ (funds locked, unlocking at future time)    │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
   [Withdraw]          [Cancel]
        │                     │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────┐
        │ Can be Archived     │
        │ (manually via API)  │
        └──────────┬──────────┘
                   │
          ┌────────▼────────┐
          │ Archived Deposit│
          │ Age: 0-365 days │
          └────────┬────────┘
                   │
          ┌────────▼────────┐
          │   After 1 year  │
          │  Can be deleted │
          └────────┬────────┘
                   │
                [Delete]
                   │
                ✓ Complete removal
```

## API Functions

### Archive Functions

#### `archive_deposit(depositor, deposit_id, token, amount, unlock_time, penalty_bps)`

Archive a completed timestamp-based deposit.

**Parameters:**
- `depositor` (Address): The original deposit owner
- `deposit_id` (u32): The ID of the deposit to archive
- `token` (Address): Token address (must match original deposit)
- `amount` (i128): Amount (must match original deposit)
- `unlock_time` (u64): Unlock time (must match original deposit)
- `penalty_bps` (u32): Penalty basis points (must match original deposit)

**Preconditions:**
- Deposit must NOT exist in active storage (must have been withdrawn or cancelled)
- No archived deposit with this ID should already exist for this depositor
- Caller must be the depositor (auth required)

**Returns:**
- `Ok(())` on success
- `Err(NoDepositFound)` if active deposit still exists
- `Err(NoArchivedDepositFound)` if archived deposit already exists

**Example:**
```rust
vault.archive_deposit(
    &depositor,
    &0,                    // deposit_id
    &token_address,        // original token
    &1000,                 // original amount
    &unlock_time,          // original unlock_time
    &0,                    // original penalty_bps
);
```

#### `archive_deposit_by_ledger(depositor, deposit_id, token, amount, unlock_ledger, penalty_bps)`

Archive a completed ledger-sequence-based deposit.

**Parameters:**
- `depositor` (Address): The original deposit owner
- `deposit_id` (u32): The ID of the deposit to archive
- `token` (Address): Token address (must match original deposit)
- `amount` (i128): Amount (must match original deposit)
- `unlock_ledger` (u32): Unlock ledger (must match original deposit)
- `penalty_bps` (u32): Penalty basis points (must match original deposit)

**Preconditions:**
- Same as `archive_deposit`
- But for ledger-based deposits

**Returns:**
- `Ok(())` on success
- `Err(NoDepositFound)` if active deposit still exists
- `Err(NoArchivedDepositFound)` if archived deposit already exists

### Delete Functions

#### `delete_archived_deposit(depositor, deposit_id)`

Delete an archived deposit that is at least 1 year old.

**Parameters:**
- `depositor` (Address): The original deposit owner
- `deposit_id` (u32): The ID of the archived deposit to delete

**Preconditions:**
- Archived deposit must exist
- Deposit must be at least 1 year old (365 days = 31,536,000 seconds)
- Caller must be the depositor (auth required)

**Returns:**
- `Ok(())` on success (deposit permanently deleted)
- `Err(NoArchivedDepositFound)` if no archived deposit exists
- `Err(ArchivedDepositTooYoung)` if deposit is less than 1 year old

**Example:**
```rust
// After 1 year has passed since archival
vault.delete_archived_deposit(&depositor, &0);  // deposit_id = 0
```

## Usage Flow

### Scenario: Archive and Delete a Withdrawn Deposit

1. **Deposit is active:**
   ```rust
   let deposit_id = vault.deposit(
       &alice,
       &token,
       &1_000,
       &unlock_time,
       &0,  // no penalty
   );
   ```

2. **After unlock time, withdraw the deposit:**
   ```rust
   vault.withdraw(&alice, &deposit_id);
   // Funds returned to alice, deposit removed from active storage
   ```

3. **Archive the completed deposit (for record-keeping):**
   ```rust
   vault.archive_deposit(
       &alice,
       &deposit_id,
       &token,
       &1_000,
       &unlock_time,
       &0,
   );
   // Deposit now stored in archived storage with current timestamp
   ```

4. **Wait 1 year, then delete:**
   ```rust
   advance_time(&env, 31_536_000);  // advance by 1 year
   vault.delete_archived_deposit(&alice, &deposit_id);
   // Archived deposit permanently removed from storage
   ```

### Scenario: Archive and Delete a Cancelled Deposit

1. **Create deposit with penalty:**
   ```rust
   let deposit_id = vault.deposit(
       &alice,
       &token,
       &1_000,
       &unlock_time,
       &5_000,  // 50% penalty
   );
   ```

2. **Cancel early (early exit):**
   ```rust
   vault.cancel_deposit(&alice, &deposit_id);
   // alice receives 500 (50% after penalty)
   // fee_recipient receives 500 (50% penalty)
   ```

3. **Archive the cancelled deposit:**
   ```rust
   vault.archive_deposit(
       &alice,
       &deposit_id,
       &token,
       &1_000,
       &unlock_time,
       &5_000,
   );
   ```

4. **After 1 year, delete:**
   ```rust
   vault.delete_archived_deposit(&alice, &deposit_id);
   ```

## Error Codes

| Error | Code | Meaning |
|---|---|---|
| `NoDepositFound` | 3 | Attempted to archive but active deposit still exists |
| `NoArchivedDepositFound` | 17 | Archived deposit not found, or tried to re-archive |
| `ArchivedDepositTooYoung` | 16 | Attempted to delete but deposit is less than 1 year old |

## Data Structures

### ArchivedVaultEntry

```rust
pub struct ArchivedVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    pub archive_timestamp: u64,  // timestamp when archived
}
```

### ArchivedLedgerVaultEntry

```rust
pub struct ArchivedLedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub depositor: Address,
    pub penalty_bps: u32,
    pub archive_timestamp: u64,  // timestamp when archived
}
```

## Storage

### VaultKey Variants

- `ArchivedDeposit(Address, u32)` – stores `ArchivedVaultEntry` for timestamp-based deposits
- `ArchivedDepositByLedger(Address, u32)` – stores `ArchivedLedgerVaultEntry` for ledger-based deposits

### TTL Management

Archived deposits respect the same TTL (Time-To-Live) as active deposits:
- `BUMP_TARGET` covers the maximum lock duration (~5 years) plus buffer
- TTL is bumped on every read/write to prevent expiry

## Constants

- `ARCHIVED_DEPOSIT_MIN_AGE_SECS` = 31,536,000 seconds = 365 days = 1 year
  - Deposits must be at least this old before deletion is allowed
  - Prevents accidental or malicious quick deletion

## Events

### Event: `arch_deposit`
Emitted when a timestamp-based deposit is archived.
```rust
env.events().publish(
    (Symbol::new(env, "arch_deposit"), depositor, token),
    (amount, deposit_id),
);
```

### Event: `arch_dep_ledger`
Emitted when a ledger-based deposit is archived.
```rust
env.events().publish(
    (Symbol::new(env, "arch_dep_ledger"), depositor, token),
    (amount, deposit_id),
);
```

### Event: `del_archive`
Emitted when an archived deposit is deleted.
```rust
env.events().publish(
    (Symbol::new(env, "del_archive"), depositor),
    deposit_id,
);
```

## Security Considerations

### Auth Enforcement

- `archive_deposit` and `delete_archived_deposit` require the depositor's auth
- Non-depositors cannot archive or delete other users' deposits
- The contract calls `depositor.require_auth()` as the first operation

### Data Integrity

- Original deposit data must be provided by the caller when archiving
- Archive stores the complete deposit record for auditing
- Archive timestamp is set by the contract (not caller) to prevent tampering

### Age Validation

- Minimum 1-year age prevents accidental deletion
- Age is checked at deletion time (exact boundary: `age >= 31,536,000 seconds`)
- Archive timestamp is immutable (set at archival, never updated)

### No Storage Leaks

- Deposits are completely removed from storage on deletion
- No "soft delete" flags (permanent removal only)
- Storage is reclaimed immediately

## Audit Trail

The archival feature preserves an audit trail through events:

1. `deposit` – Original deposit created
2. `withdraw` or `dep_cancel` – Deposit completed (funds returned)
3. `arch_deposit` or `arch_dep_ledger` – Archived with timestamp
4. `del_archive` – Permanently deleted (after 1 year)

Off-chain indexers can reconstruct the full lifecycle of any deposit by following the event log.

## Design Decisions

### Why Manual Archival?

- **Flexibility**: Users choose when to archive, not the contract
- **Predictable cost**: Archival is an explicit operation with known gas cost
- **Auditability**: Clear event trail for each archival action

### Why 1-Year Minimum?

- **Legal compliance**: Many jurisdictions require 1-year record retention
- **Accident prevention**: Reduces risk of accidental deletion
- **Stakeholder protection**: Gives all parties time to archive disputes or claims

### Why Caller Provides Original Data?

- **Reduces contract storage**: No need to store data twice during active phase
- **Event log recovery**: Callers can reconstruct data from emitted events
- **Caller verification**: Forces caller to be aware of original terms

### Why No Automatic Archival?

- **Explicit choice**: Users control when their history is "archived"
- **Simplicity**: No background jobs or cron-like mechanisms needed
- **Cost control**: Users decide when to pay the archival gas cost

## Testing

The archival feature includes 15+ comprehensive unit tests:

- ✅ Archive success (timestamp-based and ledger-based)
- ✅ Archive fails if active deposit still exists
- ✅ Archive fails if already archived
- ✅ Delete success after 1 year
- ✅ Delete fails if too young (age < 1 year)
- ✅ Delete fails at boundary (age < exactly 1 year)
- ✅ Delete succeeds at exact boundary
- ✅ Delete fails if not found
- ✅ Archive after deposit cancellation
- ✅ Multiple deposits archival
- ✅ Archive preserves original data
- ✅ Archive works for different depositors
- ✅ Delete requires depositor auth

Run tests with:
```bash
cargo test --features testutils -- archive
cargo test --features testutils -- delete
```

## Related Functions

- [`deposit()`](./README.md#deposit) – Create active deposit
- [`withdraw()`](./README.md#withdraw) – Complete and return funds
- [`cancel_deposit()`](./README.md#cancel-deposit) – Early exit with penalty
- [`get_vault()`](./README.md#read-only-queries) – Query active deposit
