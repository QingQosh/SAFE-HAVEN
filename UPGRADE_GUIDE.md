# SAFE-HAVEN Deposit Contract Upgrade Guide

## Overview

This guide documents the deposit contract upgrade path for SAFE-HAVEN, enabling seamless protocol evolution while preserving all user deposit data. The upgrade system provides optional migrations with comprehensive data validation and rollback capabilities.

## Table of Contents

1. [Key Features](#key-features)
2. [Architecture](#architecture)
3. [Migration Workflow](#migration-workflow)
4. [API Reference](#api-reference)
5. [Validation Procedures](#validation-procedures)
6. [Rollback Procedures](#rollback-procedures)
7. [Safety Mechanisms](#safety-mechanisms)
8. [Examples](#examples)
9. [Troubleshooting](#troubleshooting)

## Key Features

### Data Preservation
- All deposit amounts preserved exactly during migration
- Metadata (unlock times, penalties, token addresses) maintained
- User addresses and permissions unchanged
- Historical data integrity verified

### Safe Migration
- Pre-migration validation of all deposits
- Step-by-step migration with progress tracking
- Batch processing for large deposit sets
- Real-time verification of data consistency

### Optional Upgrades
- Users choose when to migrate (not forced)
- Can migrate partial or complete deposit portfolios
- Retry capability if migration fails
- Selective user rollback if needed

### Rollback Capability
- Full migration rollback available
- Snapshot-based recovery
- Multiple rollback strategies (conservative, full, targeted)
- Integrity verification post-rollback

## Architecture

### Core Modules

#### `upgrade.rs` - Migration Orchestration
- `init_migration()` - Initialize upgrade process
- `migrate_depositor_deposits()` - Execute migration for user
- `validate_deposits_for_migration()` - Pre-migration validation
- `verify_migration_integrity()` - Post-migration verification
- `rollback_migration()` - Revert failed migration

#### `upgrade_validation.rs` - Data Integrity Checks
- `validate_all_deposits()` - Comprehensive validation
- `validate_depositor_deposits()` - User-specific validation
- `validate_single_deposit()` - Individual deposit validation
- `verify_consistency()` - Before/after consistency check
- `can_migrate_deposit()` - Migration feasibility check

#### `upgrade_rollback.rs` - Recovery Operations
- `create_deposit_snapshot()` - Snapshot all deposits
- `create_depositor_snapshot()` - Snapshot user deposits
- `rollback_all_deposits()` - Restore all deposits
- `rollback_depositor()` - Restore specific user
- `verify_rollback_integrity()` - Verify recovery success

### Data Structures

```rust
/// Migration state tracking
pub struct MigrationState {
    pub migration_id: u32,
    pub old_contract: Address,
    pub new_contract: Address,
    pub progress: u32,
    pub total_deposits: u32,
    pub migrated_count: u32,
    pub failed_count: u32,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub can_rollback: bool,
    pub failure_reason: u32,
}

/// Validation report with metrics
pub struct ValidationReport {
    pub is_valid: bool,
    pub total_deposits: u32,
    pub valid_deposits: u32,
    pub invalid_deposits: u32,
    pub total_value_locked: i128,
    pub future_unlock_count: u32,
    pub unlocked_count: u32,
    pub amount_checksum: i128,
    pub max_deposit_amount: i128,
    pub min_deposit_amount: i128,
}

/// Rollback state
pub struct RollbackState {
    pub is_active: bool,
    pub migration_id: u32,
    pub initiated_at: u64,
    pub deposits_rolled_back: u32,
    pub rollback_failures: u32,
    pub status: RollbackStatus,
}
```

## Migration Workflow

### Step 1: Preparation

Before initiating migration, prepare the new contract version:

```
1. Deploy new contract version to network
2. Verify new contract is operational
3. Obtain new contract address
4. Prepare migration communication to users
```

### Step 2: Initialization

Admin initiates migration with the new contract address:

```
init_contract_migration(
    admin_address,
    old_contract_address,
    new_contract_address,
    "v2.0.0"  // new version
)
```

Returns: `migration_id` for reference

### Step 3: Pre-Migration Validation

Validate all deposits are migration-ready:

```
validate_deposits_before_migration(admin_address)
```

Checks:
- All amounts are positive and within limits
- Unlock times are valid (not in past)
- Penalty BPS values are valid (0-10000)
- Compound frequencies are valid (0 or ≥60 seconds)
- Depositor addresses are consistent

### Step 4: Create Snapshot (Optional but Recommended)

Create snapshot before migration for safety:

```
create_migration_snapshot(
    admin_address,
    migration_id
)
```

Returns: `snapshot_id` for potential rollback

### Step 5: User Migration

Users (or admin on their behalf) migrate deposits:

```
migrate_deposits(
    depositor_address,
    migration_id
)
```

Returns: Number of successfully migrated deposits

### Step 6: Verification

Verify data integrity after migration:

```
verify_migration_data_integrity(
    admin_address,
    depositor_address,
    migration_id
)
```

Confirms:
- All deposits transferred
- Amounts match exactly
- Metadata preserved
- Data checksums valid

### Step 7: Completion

Once all users migrated and verified, migration is complete.

## API Reference

### Contract Functions

#### `init_contract_migration`

Initializes upgrade process for all deposits.

**Parameters:**
- `admin: Address` - Admin account (requires auth)
- `old_contract: Address` - Current contract
- `new_contract: Address` - Target contract
- `new_version: String` - Version string (e.g., "v2.0.0")

**Returns:** `Result<u32, VaultError>` - Migration ID

**Errors:**
- `Unauthorized` - Caller not admin
- `UpgradeError` - Contracts are same or invalid state

**Example:**
```rust
let migration_id = contract_client
    .init_contract_migration(
        &admin,
        &old_contract,
        &new_contract,
        &String::from_slice(env, b"v2.0.0"),
    )?;
```

#### `validate_deposits_before_migration`

Validates all deposits for migration readiness.

**Parameters:**
- `admin: Address` - Admin account (requires auth)

**Returns:** `Result<ValidationReport, VaultError>` - Detailed validation results

**ValidationReport fields:**
- `is_valid: bool` - Overall validation status
- `total_deposits: u32` - Total deposits checked
- `valid_deposits: u32` - Deposits that passed validation
- `invalid_deposits: u32` - Deposits with issues
- `total_value_locked: i128` - Total deposit value
- `future_unlock_count: u32` - Deposits still locked
- `unlocked_count: u32` - Deposits that can be withdrawn
- `amount_checksum: i128` - Sum of all amounts
- `max_deposit_amount: i128` - Largest deposit
- `min_deposit_amount: i128` - Smallest deposit

**Example:**
```rust
let report = contract_client
    .validate_deposits_before_migration(&admin)?;

if report.is_valid {
    println!("All {} deposits valid", report.total_deposits);
} else {
    println!("Found {} invalid deposits", report.invalid_deposits);
}
```

#### `migrate_deposits`

Migrates specific user's deposits to new contract.

**Parameters:**
- `depositor: Address` - User whose deposits to migrate
- `migration_id: u32` - ID from initialization

**Returns:** `Result<u32, VaultError>` - Number of migrated deposits

**Errors:**
- `UpgradeError` - Migration not in progress or ID mismatch
- `DepositNotFound` - Deposit doesn't exist
- `InvalidAmount` - Deposit amount invalid

**Example:**
```rust
let migrated = contract_client
    .migrate_deposits(&user_address, migration_id)?;

println!("Successfully migrated {} deposits", migrated);
```

#### `verify_migration_data_integrity`

Verifies deposits are intact after migration.

**Parameters:**
- `admin: Address` - Admin account
- `depositor: Address` - User to verify
- `migration_id: u32` - Migration ID

**Returns:** `Result<ValidationReport, VaultError>` - Validation results

**Example:**
```rust
let integrity = contract_client
    .verify_migration_data_integrity(
        &admin,
        &user_address,
        migration_id,
    )?;

if integrity.is_valid {
    println!("Migration verified successfully");
}
```

#### `create_migration_snapshot`

Creates snapshot for rollback capability.

**Parameters:**
- `admin: Address` - Admin account
- `migration_id: u32` - Migration ID

**Returns:** `Result<u32, VaultError>` - Snapshot ID

**Example:**
```rust
let snapshot_id = contract_client
    .create_migration_snapshot(&admin, migration_id)?;

println!("Snapshot created: {}", snapshot_id);
```

#### `rollback_migration_all_deposits`

Rolls back all deposits to pre-migration state.

**Parameters:**
- `admin: Address` - Admin account
- `migration_id: u32` - Migration that failed
- `snapshot_id: u32` - Snapshot to restore from

**Returns:** `Result<u32, VaultError>` - Number of restored deposits

**Example:**
```rust
let restored = contract_client
    .rollback_migration_all_deposits(
        &admin,
        migration_id,
        snapshot_id,
    )?;

println!("Rolled back {} deposits", restored);
```

#### `rollback_migration_depositor`

Rolls back specific user's deposits.

**Parameters:**
- `admin: Address` - Admin account
- `migration_id: u32` - Migration ID
- `snapshot_id: u32` - Snapshot ID
- `depositor: Address` - User to rollback

**Returns:** `Result<u32, VaultError>` - Number of restored deposits

**Example:**
```rust
let restored = contract_client
    .rollback_migration_depositor(
        &admin,
        migration_id,
        snapshot_id,
        &problem_user,
    )?;
```

#### `verify_rollback_success`

Confirms rollback was successful.

**Parameters:**
- `admin: Address` - Admin account
- `snapshot_id: u32` - Snapshot used for rollback

**Returns:** `Result<bool, VaultError>` - Success status

**Example:**
```rust
let verified = contract_client
    .verify_rollback_success(&admin, snapshot_id)?;

if verified {
    println!("Rollback verified successfully");
}
```

#### `full_migration_rollback`

Performs complete rollback with safety checks.

**Parameters:**
- `admin: Address` - Admin account
- `migration_id: u32` - Migration ID
- `snapshot_id: u32` - Snapshot ID

**Returns:** `Result<u32, VaultError>` - Number of restored deposits

**Example:**
```rust
let restored = contract_client
    .full_migration_rollback(
        &admin,
        migration_id,
        snapshot_id,
    )?;
```

#### `get_migration_status`

Gets current migration status.

**Returns:** `Option<UpgradeEntry>` - Current upgrade info or None

**Example:**
```rust
if let Some(status) = contract_client.get_migration_status() {
    println!("Migration {} in progress", status.migration_id);
}
```

## Validation Procedures

### Pre-Migration Validation

All deposits are validated for:

1. **Amount Validity**
   - Amount > 0
   - Amount ≤ MAX_DEPOSIT_AMOUNT (1_000_000_000_000_000)

2. **Time Validity**
   - Unlock time in future (not in past)
   - Unlock time ≤ now + MAX_LOCK_DURATION_SECS

3. **Penalty Validity**
   - Penalty BPS between 0 and 10,000

4. **Compound Frequency Validity**
   - If non-zero, must be ≥ 60 seconds

5. **Consistency Checks**
   - Depositor address matches entry
   - Token address is valid
   - No data corruption

### Validation Report Metrics

The validation report provides:

- **Coverage Metrics**
  - Total deposits checked
  - Valid vs. invalid count
  - Coverage percentage

- **Value Metrics**
  - Total value locked
  - Amount checksum for verification
  - Min/max deposit ranges

- **Status Metrics**
  - Deposits still locked (future unlock)
  - Deposits unlocked (can be withdrawn)
  - Deposits with issues

### Interpretation

```
if report.is_valid && report.invalid_deposits == 0:
    "Safe to proceed with migration"
else:
    "Address invalid deposits before migration:"
    for deposit in invalid_deposits:
        "Deposit {deposit_id} from {depositor}: {error}"
```

## Rollback Procedures

### Conservative Rollback

Restores deposits only, leaves other state untouched:

```
rollback_migration_all_deposits(
    admin_address,
    migration_id,
    snapshot_id
)
```

**Use when:**
- Need minimal contract disruption
- Other state changes don't require rollback
- Only deposits failed

### Full Rollback

Complete recovery with verification:

```
full_migration_rollback(
    admin_address,
    migration_id,
    snapshot_id
)
```

**Use when:**
- Complete migration failure
- Need maximum safety assurance
- Includes integrity verification

### Targeted Rollback

Rollback specific user only:

```
rollback_migration_depositor(
    admin_address,
    migration_id,
    snapshot_id,
    problem_user_address
)
```

**Use when:**
- Specific user's migration failed
- Other users' migrations successful
- Need surgical precision

### Verification

After any rollback:

```
verify_rollback_success(
    admin_address,
    snapshot_id
)
```

Confirms:
- All deposits restored to snapshot state
- Amounts match exactly
- Metadata preserved
- No data loss

## Safety Mechanisms

### Authorization

- Only admin can initialize migration
- Only admin can create snapshots
- Only admin can execute rollback
- Deposits owned by users, admin cannot modify directly

### Validation Layers

1. **Input Validation** - Check parameters before processing
2. **State Validation** - Verify consistent contract state
3. **Data Validation** - Validate every deposit
4. **Consistency Validation** - Compare before/after state

### Error Handling

- All operations return `Result<T, VaultError>`
- Errors prevent partial states
- Failed operations are atomic
- No data loss on error

### State Management

- Upgrades tracked with unique IDs
- Migration progress tracked
- Snapshots preserved for rollback
- Historical records maintained

## Examples

### Complete Migration Scenario

```rust
// Step 1: Initialize
let migration_id = contract_client
    .init_contract_migration(
        &admin,
        &old_contract,
        &new_contract,
        &version,
    )?;

// Step 2: Validate
let validation = contract_client
    .validate_deposits_before_migration(&admin)?;

if !validation.is_valid {
    println!("Issues found: {}", validation.invalid_deposits);
    return;
}

println!("Total value locked: {}", validation.total_value_locked);

// Step 3: Snapshot
let snapshot_id = contract_client
    .create_migration_snapshot(&admin, migration_id)?;

// Step 4: Migrate user
let migrated = contract_client
    .migrate_deposits(&user, migration_id)?;

println!("Migrated {} deposits", migrated);

// Step 5: Verify
let integrity = contract_client
    .verify_migration_data_integrity(
        &admin,
        &user,
        migration_id,
    )?;

if integrity.is_valid {
    println!("User migration verified!");
}
```

### Rollback Scenario

```rust
// Detect migration issue
let status = contract_client.get_migration_status();

if status.is_some() {
    // Get the migration ID and snapshot ID from status
    let migration_id = status.migration_id;
    let snapshot_id = /* from earlier snapshot call */;

    // Perform rollback
    let restored = contract_client
        .full_migration_rollback(
            &admin,
            migration_id,
            snapshot_id,
        )?;

    println!("Rolled back {} deposits", restored);

    // Verify success
    let verified = contract_client
        .verify_rollback_success(&admin, snapshot_id)?;

    if verified {
        println!("Rollback completed successfully");
    }
}
```

## Troubleshooting

### Issue: Validation Fails with Invalid Deposits

**Symptoms:** `validate_deposits_before_migration()` returns invalid deposits

**Solution:**
1. Review validation report for specific errors
2. Investigate problematic deposits:
   - Check if amounts are reasonable
   - Verify unlock times haven't passed
   - Confirm penalty BPS values
3. Fix issues in old contract before migration
4. Re-run validation

### Issue: Migration Incomplete

**Symptoms:** Some deposits not migrated after `migrate_deposits()`

**Solution:**
1. Call `migrate_deposits()` again for same user
2. It's safe to retry (idempotent)
3. Check for specific error codes
4. Verify user has valid deposits to migrate

### Issue: Rollback Needed

**Symptoms:** Migration failed and needs recovery

**Solution:**
1. Ensure snapshot was created with `create_migration_snapshot()`
2. Call `rollback_migration_all_deposits()` with snapshot ID
3. Verify rollback with `verify_rollback_success()`
4. Investigate root cause
5. Re-attempt migration when ready

### Issue: Data Inconsistency After Migration

**Symptoms:** `verify_migration_data_integrity()` fails

**Solution:**
1. Check validation report for specific discrepancies
2. Do NOT proceed with more migrations
3. Initiate rollback immediately
4. Investigate inconsistency root cause
5. Fix root cause before re-attempting

### Common Error Codes

| Error | Cause | Solution |
|-------|-------|----------|
| `Unauthorized` | Caller not admin | Ensure caller is contract admin |
| `UpgradeError` | Invalid upgrade state | Check migration is initialized |
| `DepositNotFound` | Deposit doesn't exist | Verify deposit ID is correct |
| `InvalidAmount` | Amount invalid | Check deposit amounts are positive |
| `LockDurationTooLong` | Unlock time too far | Verify unlock times are reasonable |

## Best Practices

1. **Always create snapshot before migration**
   - Enables rollback if needed
   - Minimal overhead
   - Essential safety net

2. **Validate before migrating**
   - Catch issues early
   - Prevent failed migrations
   - Informs users of any problems

3. **Migrate in batches**
   - Reduces risk if failure occurs
   - Easier to debug issues
   - Better user communication

4. **Verify after migration**
   - Confirm data integrity
   - Detect any corruption
   - Build user confidence

5. **Keep snapshot until migration complete**
   - Required for rollback
   - Can delete after successful verification
   - Minimal storage impact

6. **Communicate with users**
   - Explain migration benefits
   - Set timeline expectations
   - Provide support contact
   - Explain rollback process

## Support

For issues or questions regarding the upgrade process, contact the development team with:

- Migration ID
- Affected user addresses (if applicable)
- Error codes or messages
- Recent actions taken
- Validation report snapshots

---

**Last Updated:** 2026-09-27
**Version:** 1.0
**Document Status:** Complete
