# SAFE-HAVEN Upgrade API Reference

Complete API documentation for deposit contract upgrade operations.

## Table of Contents

- [Migration Functions](#migration-functions)
- [Validation Functions](#validation-functions)
- [Rollback Functions](#rollback-functions)
- [Query Functions](#query-functions)
- [Data Structures](#data-structures)
- [Error Codes](#error-codes)
- [Usage Patterns](#usage-patterns)

## Migration Functions

### init_contract_migration

Initialize a new contract migration process.

```rust
pub fn init_contract_migration(
    env: Env,
    admin: Address,
    old_contract: Address,
    new_contract: Address,
    new_version: String,
) -> Result<u32, VaultError>
```

**Purpose:** Set up migration infrastructure before transferring deposits

**Authorization:** Admin only (requires signature)

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `admin` | Address | Admin account initiating upgrade |
| `old_contract` | Address | Current contract address |
| `new_contract` | Address | New contract address to migrate to |
| `new_version` | String | Version identifier for new contract |

**Returns:** 
- `Ok(migration_id)` - Unique migration identifier (u32)
- `Err(VaultError)` - Migration error

**Errors:**
- `Unauthorized` - Caller not admin
- `UpgradeError` - Invalid contract addresses or state

**Side Effects:**
- Creates upgrade entry in persistent storage
- Initializes migration state
- Records migration start time

**Example:**
```rust
let migration_id = contract.init_contract_migration(
    env,
    admin_address,
    old_contract_addr,
    new_contract_addr,
    String::from_slice(&env, b"v2.0.0"),
)?;
```

---

### migrate_deposits

Execute migration for a specific depositor's deposits.

```rust
pub fn migrate_deposits(
    env: Env,
    depositor: Address,
    migration_id: u32,
) -> Result<u32, VaultError>
```

**Purpose:** Transfer deposits from old contract to new contract

**Authorization:** Depositor or admin

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `depositor` | Address | User whose deposits to migrate |
| `migration_id` | u32 | Migration ID from initialization |

**Returns:**
- `Ok(count)` - Number of successfully migrated deposits
- `Err(VaultError)` - Migration error

**Errors:**
- `UpgradeError` - Migration not in progress
- `DepositNotFound` - Deposit doesn't exist
- `InvalidAmount` - Deposit amount invalid

**Process:**
1. Validates active migration exists
2. Retrieves all deposits for depositor
3. Validates each deposit
4. Copies deposit data to new contract
5. Updates migration statistics

**Idempotent:** Yes - safe to retry

**Example:**
```rust
let migrated_count = contract.migrate_deposits(
    env,
    user_address,
    migration_id,
)?;

println!("Migrated {} deposits", migrated_count);
```

---

## Validation Functions

### validate_deposits_before_migration

Comprehensive validation of all deposits before migration.

```rust
pub fn validate_deposits_before_migration(
    env: Env,
    admin: Address,
) -> Result<upgrade::ValidationResult, VaultError>
```

**Purpose:** Pre-migration health check and issue detection

**Authorization:** Admin only

**Returns:**
- `Ok(ValidationResult)` - Validation metrics and status
- `Err(VaultError)` - Validation error

**ValidationResult Structure:**
```rust
pub struct ValidationResult {
    pub is_valid: bool,              // Overall validation status
    pub error_count: u32,             // Number of validation errors
    pub checked_count: u32,           // Total deposits checked
    pub issue_count: u32,             // Deposits with issues
}
```

**Validation Checks:**
- ✓ Amount is positive
- ✓ Amount ≤ MAX_DEPOSIT_AMOUNT
- ✓ Unlock time is in future
- ✓ Unlock time ≤ now + MAX_LOCK_DURATION_SECS
- ✓ Penalty BPS ≤ 10,000
- ✓ Compound frequency valid (0 or ≥60 sec)
- ✓ Depositor address matches
- ✓ No data corruption

**Process:**
1. Retrieves all depositors
2. Iterates through each deposit
3. Performs validation checks
4. Aggregates results
5. Returns comprehensive report

**Time Complexity:** O(n) where n = total deposits

**Example:**
```rust
let result = contract.validate_deposits_before_migration(
    env,
    admin_address,
)?;

if result.is_valid {
    println!("✓ All {} deposits valid", result.checked_count);
} else {
    println!("✗ {} deposits have issues", result.issue_count);
}
```

---

### verify_migration_data_integrity

Verify data integrity after migration completes.

```rust
pub fn verify_migration_data_integrity(
    env: Env,
    admin: Address,
    depositor: Address,
    migration_id: u32,
) -> Result<upgrade::ValidationResult, VaultError>
```

**Purpose:** Post-migration verification that data is intact

**Authorization:** Admin only

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `depositor` | Address | User to verify |
| `migration_id` | u32 | Migration ID |

**Returns:**
- `Ok(ValidationResult)` - Verification results
- `Err(VaultError)` - Verification error

**Checks:**
- All deposits readable in new contract
- All amounts preserved exactly
- All metadata (times, penalties) preserved
- Data checksums match
- No deposits lost or corrupted

**Example:**
```rust
let integrity = contract.verify_migration_data_integrity(
    env,
    admin_address,
    user_address,
    migration_id,
)?;

if integrity.is_valid {
    println!("✓ Migration data verified");
}
```

---

## Rollback Functions

### create_migration_snapshot

Create snapshot of deposits for rollback capability.

```rust
pub fn create_migration_snapshot(
    env: Env,
    admin: Address,
    migration_id: u32,
) -> Result<u32, VaultError>
```

**Purpose:** Preserve current deposit state for potential recovery

**Authorization:** Admin only

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `migration_id` | u32 | Migration ID |

**Returns:**
- `Ok(snapshot_id)` - Snapshot identifier
- `Err(VaultError)` - Snapshot error

**Snapshot Contents:**
- All deposit amounts
- All metadata (times, penalties, tokens)
- Data hashes for verification
- Timestamps for audit trail

**Storage:** Persistent (survives contract calls)

**Retention:** Keep until migration verified complete

**Cost:** Minimal - snapshot is read-only data

**Example:**
```rust
let snapshot_id = contract.create_migration_snapshot(
    env,
    admin_address,
    migration_id,
)?;

println!("Snapshot created: {}", snapshot_id);
```

---

### rollback_migration_all_deposits

Roll back all deposits to snapshot state.

```rust
pub fn rollback_migration_all_deposits(
    env: Env,
    admin: Address,
    migration_id: u32,
    snapshot_id: u32,
) -> Result<u32, VaultError>
```

**Purpose:** Complete migration reversal to restore pre-migration state

**Authorization:** Admin only

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `migration_id` | u32 | Migration to rollback |
| `snapshot_id` | u32 | Snapshot to restore from |

**Returns:**
- `Ok(restored_count)` - Number of restored deposits
- `Err(VaultError)` - Rollback error

**Process:**
1. Validates migration ID matches active migration
2. Retrieves snapshot data
3. Restores each deposit to snapshot state
4. Verifies restoration successful
5. Clears migration state

**Side Effects:**
- Clears active migration
- Resets migration progress
- Preserves snapshot for audit trail

**Idempotent:** Yes - safe to retry if partial failure

**Example:**
```rust
let restored = contract.rollback_migration_all_deposits(
    env,
    admin_address,
    migration_id,
    snapshot_id,
)?;

println!("Restored {} deposits", restored);
```

---

### rollback_migration_depositor

Roll back specific user's deposits.

```rust
pub fn rollback_migration_depositor(
    env: Env,
    admin: Address,
    migration_id: u32,
    snapshot_id: u32,
    depositor: Address,
) -> Result<u32, VaultError>
```

**Purpose:** Targeted rollback of individual user's failed migration

**Authorization:** Admin only

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `migration_id` | u32 | Migration ID |
| `snapshot_id` | u32 | Snapshot ID |
| `depositor` | Address | User to rollback |

**Returns:**
- `Ok(restored_count)` - Number of restored deposits
- `Err(VaultError)` - Rollback error

**Use Cases:**
- Specific user's migration failed
- Need to retry one user without affecting others
- Partial migration recovery

**Example:**
```rust
let restored = contract.rollback_migration_depositor(
    env,
    admin_address,
    migration_id,
    snapshot_id,
    problem_user,
)?;
```

---

### verify_rollback_success

Verify rollback integrity after recovery.

```rust
pub fn verify_rollback_success(
    env: Env,
    admin: Address,
    snapshot_id: u32,
) -> Result<bool, VaultError>
```

**Purpose:** Confirm rollback restored data correctly

**Authorization:** Admin only

**Returns:**
- `Ok(true)` - Rollback verified successful
- `Ok(false)` - Rollback has inconsistencies
- `Err(VaultError)` - Verification error

**Verification:**
- All deposits match snapshot state
- Amounts preserved exactly
- Metadata intact
- No data loss detected

**Example:**
```rust
let verified = contract.verify_rollback_success(
    env,
    admin_address,
    snapshot_id,
)?;

if verified {
    println!("✓ Rollback verified successfully");
} else {
    println!("✗ Rollback inconsistencies detected");
}
```

---

### full_migration_rollback

Execute complete rollback with full verification.

```rust
pub fn full_migration_rollback(
    env: Env,
    admin: Address,
    migration_id: u32,
    snapshot_id: u32,
) -> Result<u32, VaultError>
```

**Purpose:** Safe, comprehensive migration reversal

**Authorization:** Admin only

**Process:**
1. Rolls back all deposits
2. Verifies rollback integrity
3. Performs data consistency checks
4. Confirms no data loss
5. Returns results

**Returns:**
- `Ok(count)` - Number of restored deposits
- `Err(VaultError)` - Rollback error

**Guarantees:**
- All deposits restored
- Data integrity verified
- No partial states
- Complete recovery or error

**Example:**
```rust
let restored = contract.full_migration_rollback(
    env,
    admin_address,
    migration_id,
    snapshot_id,
)?;
```

---

## Query Functions

### get_migration_status

Get current migration status and metadata.

```rust
pub fn get_migration_status(env: Env) -> Option<crate::types::UpgradeEntry>
```

**Purpose:** Query current upgrade state

**Returns:**
- `Some(UpgradeEntry)` - Active migration details
- `None` - No migration in progress

**UpgradeEntry Structure:**
```rust
pub struct UpgradeEntry {
    pub migration_id: u32,
    pub old_contract: Address,
    pub new_contract: Address,
    pub initiated_by: Address,
    pub initiated_at: u64,
    pub new_version: String,
}
```

**Example:**
```rust
if let Some(upgrade) = contract.get_migration_status() {
    println!("Migration {} in progress", upgrade.migration_id);
    println!("From: {}", upgrade.old_contract);
    println!("To: {}", upgrade.new_contract);
} else {
    println!("No migration in progress");
}
```

---

### get_rollback_status

Get current rollback status.

```rust
pub fn get_rollback_status(env: Env) -> upgrade_rollback::RollbackState
```

**Purpose:** Query rollback operation status

**Returns:** `RollbackState` structure

**RollbackState Structure:**
```rust
pub struct RollbackState {
    pub is_active: bool,
    pub migration_id: u32,
    pub initiated_at: u64,
    pub deposits_rolled_back: u32,
    pub rollback_failures: u32,
    pub status: RollbackStatus,
}

pub enum RollbackStatus {
    Inactive = 0,
    InProgress = 1,
    Completed = 2,
    PartialFailure = 3,
    Failed = 4,
}
```

**Example:**
```rust
let rollback = contract.get_rollback_status();
println!("Rollback status: {}", rollback.status);
```

---

## Data Structures

### ValidationReport

```rust
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
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `is_valid` | bool | Overall validation passed |
| `total_deposits` | u32 | Deposits checked |
| `valid_deposits` | u32 | Valid deposits |
| `invalid_deposits` | u32 | Deposits with issues |
| `total_value_locked` | i128 | Sum of amounts |
| `future_unlock_count` | u32 | Locked deposits |
| `unlocked_count` | u32 | Unlocked deposits |
| `amount_checksum` | i128 | Checksum for verification |
| `max_deposit_amount` | i128 | Largest deposit |
| `min_deposit_amount` | i128 | Smallest deposit |

---

### DepositSnapshot

```rust
pub struct DepositSnapshot {
    pub snapshot_id: u32,
    pub snapshot_timestamp: u64,
    pub depositor: Address,
    pub deposit_id: u32,
    pub entry: VaultEntry,
    pub data_hash: u32,
}
```

---

### ConsistencyCheckResult

```rust
pub struct ConsistencyCheckResult {
    pub is_consistent: bool,
    pub old_contract_deposits: u32,
    pub new_contract_deposits: u32,
    pub old_contract_value: i128,
    pub new_contract_value: i128,
    pub missing_count: u32,
    pub amount_mismatch_count: u32,
    pub metadata_mismatch_count: u32,
}
```

---

## Error Codes

| Error | Code | Description | Resolution |
|-------|------|-------------|-----------|
| `Unauthorized` | 7 | Not authorized | Use admin account |
| `UpgradeError` | 27 | Upgrade state error | Check migration status |
| `DepositNotFound` | 28 | Deposit missing | Verify deposit exists |
| `InvalidAmount` | 1 | Amount invalid | Check amount > 0 |
| `LockDurationTooLong` | 6 | Unlock time too far | Verify unlock time |
| `InvalidPenaltyBps` | 9 | Penalty invalid | Ensure 0 ≤ penalty ≤ 10000 |

---

## Usage Patterns

### Standard Migration Flow

```rust
// 1. Initialize
let migration_id = contract.init_contract_migration(
    env, admin, old_addr, new_addr, version
)?;

// 2. Validate
let validation = contract.validate_deposits_before_migration(env, admin)?;
assert!(validation.is_valid);

// 3. Snapshot
let snapshot_id = contract.create_migration_snapshot(env, admin, migration_id)?;

// 4. Migrate
for user in get_all_users() {
    contract.migrate_deposits(env, user, migration_id)?;
}

// 5. Verify
for user in get_all_users() {
    let integrity = contract.verify_migration_data_integrity(
        env, admin, user, migration_id
    )?;
    assert!(integrity.is_valid);
}
```

### Error Recovery

```rust
// Detect issue
if let Err(e) = contract.migrate_deposits(env, user, migration_id) {
    // Rollback
    let restored = contract.full_migration_rollback(
        env, admin, migration_id, snapshot_id
    )?;
    
    // Verify
    contract.verify_rollback_success(env, admin, snapshot_id)?;
}
```

### Monitoring

```rust
// Check status
if let Some(upgrade) = contract.get_migration_status() {
    let rollback = contract.get_rollback_status();
    println!("Migration {} status: {}", upgrade.migration_id, rollback.status);
}
```

---

**Last Updated:** 2026-09-27
**Version:** 1.0
**Status:** Complete
