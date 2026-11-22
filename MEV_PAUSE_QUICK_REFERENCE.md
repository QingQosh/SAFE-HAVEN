# Quick Reference: Pause State & MEV Protection

## Pause State API

### Functions
```rust
// Query pause state
pub fn is_paused(env: Env) -> bool

// Admin: Pause deposits
pub fn pause(env: Env, admin: Address) -> Result<(), VaultError>

// Admin: Resume deposits
pub fn unpause(env: Env, admin: Address) -> Result<(), VaultError>
```

### Behavior
- Initial state: `false` (not paused)
- Only admin can pause/unpause
- Blocks: `deposit()`, `deposit_for()`, `deposit_by_ledger()`
- Allows: All withdraw/query operations
- Events: `paused` and `unpaused` emitted

### Storage
- Key: `VaultKey::Paused`
- Type: `bool`
- TTL: Extended via `BUMP_THRESHOLD`/`BUMP_TARGET`

---

## MEV Protection API

### Constants
```rust
pub const MEV_COMMIT_WINDOW_SECS: u64 = 1_800;  // 30 minutes
```

### Storage Functions
```rust
// Store commit for (depositor, deposit_id)
pub fn set_mev_commit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    commit: &Bytes
)

// Retrieve commit (None if not found)
pub fn get_mev_commit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32
) -> Option<Bytes>

// Delete commit after reveal
pub fn remove_mev_commit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32
)
```

### Contract Functions
```rust
// Submit commit hash
pub fn mev_commit(
    env: Env,
    depositor: Address,
    deposit_id: u32,
    commit: Bytes
) -> Result<(), VaultError>

// Reveal and verify commit
pub fn mev_reveal(
    env: Env,
    depositor: Address,
    deposit_id: u32,
    amount: i128,
    token: Address,
    salt: u32
) -> Result<(), VaultError>
```

### Workflow

1. **Commit Phase**
   ```
   User calls: mev_commit(depositor, deposit_id, keccak256(...))
   Contract: Validates deposit exists, stores commit with 30-min TTL
   Event: mev_commit_submitted(depositor, deposit_id)
   ```

2. **Reveal Phase** (within 30 minutes)
   ```
   User calls: mev_reveal(depositor, deposit_id, amount, token, salt)
   Contract: Retrieves commit, verifies parameters match deposit, deletes commit
   Event: mev_reveal_completed(depositor, deposit_id)
   ```

### Storage
- Key: `VaultKey::MevCommit(Address, u32)`
- Type: `Bytes`
- TTL: 1800 seconds = 360 ledgers = 30 minutes
- BUMP: Threshold = commit_ttl_ledgers / 2

### Error Handling
```rust
// mev_commit errors:
VaultError::NoDepositFound           // Deposit doesn't exist
VaultError::InvalidAmount           // Already has a commit

// mev_reveal errors:
VaultError::NoDepositFound           // No commit or deposit
VaultError::InvalidAmount           // Amount/token don't match
```

### Events
```rust
// Emitted on successful commit
(symbol_short!("mev_commit"), depositor) -> deposit_id

// Emitted on successful reveal
(symbol_short!("mev_reveal"), depositor) -> deposit_id
```

---

## Testing

### Pause State Tests
```
✓ test_pause_state_initial_false
✓ test_pause_state_true_after_pause
✓ test_pause_state_false_after_unpause
✓ test_pause_state_multiple_cycles
✓ test_pause_state_persists_across_operations
✓ test_pause_blocks_all_deposit_operations
✓ test_unpause_allows_deposits_immediately
✓ test_pause_requires_admin
✓ test_unpause_requires_admin
```

### MEV Tests
```
✓ test_mev_commit_stores_hash
✓ test_mev_commit_cannot_be_overwritten
✓ test_mev_commit_unique_per_depositor_and_id
✓ test_mev_commit_nonexistent_returns_none
✓ test_mev_commit_can_be_removed
✓ test_mev_commit_multiple_independent
```

---

## Security Properties

### Pause State
- Admin-only control (requires auth)
- Persistent across ledger boundaries
- No silent failures (events emitted)

### MEV Protection
- Depositor-only (requires auth)
- Unique per deposit (collision-free)
- Overwrite-resistant (first wins)
- Replay-proof (commits deleted after reveal)
- Time-bounded (30-minute window)
- Deposit-verified (tied to actual deposits)

---

## Implementation Details

### Files
- `types.rs`: Constants + storage key variant
- `storage.rs`: MEV storage functions + TTL management
- `contract.rs`: mev_commit() + mev_reveal() implementations
- `events.rs`: Event emission functions
- `test.rs`: 15 comprehensive tests

### Imports
```rust
use soroban_sdk::{Bytes, Env, Address, ...};
use crate::storage::{set_mev_commit, get_mev_commit, remove_mev_commit};
use crate::events::{mev_commit_submitted, mev_reveal_completed};
```

### Key Invariants
1. Commit is stored with unique (depositor, deposit_id) key
2. First commit accepted, overwrite prevented at contract layer
3. Commit persists for exactly 30 minutes (1800 seconds)
4. Reveal parameters must match stored deposit
5. Commit deleted after successful reveal
6. TTL threshold = TTL_LEDGERS / 2 for automatic extension
