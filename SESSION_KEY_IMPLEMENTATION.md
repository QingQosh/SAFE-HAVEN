# Session Key Implementation: revoke_session_key()

**Completed**: October 19, 2026  
**Status**: ✅ Implementation complete and verified  
**Scope**: Delegated funding authority with revocation support

---

## Overview

Session keys enable delegated deposit authority on the SAFE-HAVEN contract. An account holder (delegator) can create time-limited, revocable authorization tokens that allow another account (delegate) to make deposits on their behalf.

### Key Features

- **Time-limited authority**: Session keys expire at a specified timestamp
- **Amount-limited authority**: Optional per-key maximum deposit amount
- **Immediate revocation**: Keys can be revoked before expiration
- **Permanent revocation**: Once revoked, a key cannot be reused
- **Non-revocable keys unaffected**: Revoking one key doesn't affect siblings
- **Audit trail**: All key creation, revocation, and usage events are emitted

---

## API Reference

### Core Functions

#### `create_session_key(delegator, delegate, max_amount, expires_at) -> u32`

Creates a new session key that authorizes `delegate` to make deposits on behalf of `delegator`.

**Parameters**

| Name | Type | Description |
|---|---|---|
| `delegator` | `Address` | Account granting the authority (must sign) |
| `delegate` | `Address` | Account that can use the key |
| `max_amount` | `i128` | Maximum total deposit amount via this key (0 = unlimited) |
| `expires_at` | `u64` | Expiration timestamp (must be > current ledger time) |

**Returns** `u32` – the session key ID (auto-incrementing per delegator)

**Authorization** requires `delegator.require_auth()`

**Errors**
- `UnlockTimeNotInFuture` – `expires_at` is not in the future
- `InvalidAmount` – `max_amount` is negative

**Example**
```rust
// Alice creates a key allowing Bob to deposit up to 1000 USDC until timestamp 1700000000
let key_id = vault.create_session_key(
    &alice,
    &bob,
    &1_000,
    &1700000000
)?;  // Returns 0 (first key for alice)
```

---

#### `revoke_session_key(delegator, key_id) -> Result<(), VaultError>`

Permanently disables a session key, preventing any further deposits using that key.

**Parameters**

| Name | Type | Description |
|---|---|---|
| `delegator` | `Address` | Account that created the session key (must sign) |
| `key_id` | `u32` | ID of the session key to revoke |

**Returns** `Ok(())` on success

**Authorization** requires `delegator.require_auth()`

**Errors**
- `SessionKeyNotFound` – key does not exist under delegator
- `SessionKeyRevoked` – key is already revoked (idempotent guard)

**Guarantees**
- ✅ Revocation is permanent and irreversible
- ✅ Once revoked, `deposit_with_session_key()` will always fail with `SessionKeyRevoked`
- ✅ Other keys are unaffected
- ✅ Can revoke before expiration
- ✅ Revocation is idempotent but returns error on second attempt

**Example**
```rust
// Alice revokes key 0, immediately terminating Bob's authority
vault.revoke_session_key(&alice, &0)?;

// Later, Bob tries to deposit:
vault.deposit_with_session_key(&bob, &0, &alice, &token, &1_000, &unlock_time, &0)?;
// ❌ Returns: Err(SessionKeyRevoked)
```

---

#### `deposit_with_session_key(delegate, key_id, delegator, token, amount, unlock_time, penalty_bps) -> u32`

Makes a deposit using a valid session key on behalf of the delegator.

**Parameters**

| Name | Type | Description |
|---|---|---|
| `delegate` | `Address` | Account using the key (must sign) |
| `key_id` | `u32` | ID of the session key |
| `delegator` | `Address` | The account on whose behalf the deposit is made |
| `token` | `Address` | Token contract address |
| `amount` | `i128` | Deposit amount (> 0, ≤ max_deposit, ≤ key's max_amount) |
| `unlock_time` | `u64` | Unlock timestamp (60s to 5 years in future) |
| `penalty_bps` | `u32` | Early exit penalty in basis points (0–10000) |

**Returns** `u32` – the deposit ID

**Authorization** requires `delegate.require_auth()`

**Errors**
- `SessionKeyNotFound` – key does not exist
- `SessionKeyRevoked` – key is revoked
- `SessionKeyExpired` – key's expiration time has passed
- `SessionKeyInsufficientLimit` – deposit would exceed key's max_amount
- `InvalidAmount` – amount ≤ 0
- `AmountTooLarge` – amount > 10^15
- `UnlockTimeNotInFuture` – unlock_time ≤ now
- `LockDurationTooLong` – lock duration > 5 years
- `LockDurationTooShort` – lock duration < 60 seconds
- `InvalidPenaltyBps` – penalty_bps > 10000
- `ContractPaused` – deposits are currently paused

**Key Tracking**
- The session key's `amount_used` is incremented by `amount`
- If key has `max_amount > 0`, ensures `amount_used + amount ≤ max_amount`

**Deposit Ownership**
- The deposit is created under the `delegator`'s account
- `delegator` owns the deposit and can withdraw/cancel it
- Withdrawal is restricted to `delegator` (standard auth check)

**Events Emitted**
1. `deposit` – standard deposit event (delegator, token, amount, unlock_time, deposit_id)
2. `deposit_with_session_key` – session-specific event (delegate, token, delegator, amount, session_key_id, deposit_id)

**Example**
```rust
let now = env.ledger().timestamp();

// Alice creates a key for Bob (max 5000 USDC, expires in 1 day)
let key_id = vault.create_session_key(&alice, &bob, &5_000, &(now + 86400))?;

// Bob deposits 1000 USDC on behalf of Alice, which unlocks in 30 days
let deposit_id = vault.deposit_with_session_key(
    &bob,                    // delegate
    &key_id,                 // 0
    &alice,                  // delegator
    &usdc_token,            // token
    &1_000,                 // amount
    &(now + 2_592_000),     // unlock in 30 days
    &500                    // 5% early exit penalty
)?;

// Alice now owns deposit 0
// Alice can withdraw it after 30 days
// Alice can cancel it early (paying 5% penalty)
// Bob cannot withdraw or cancel (not the owner)

// Later, Alice revokes the key
vault.revoke_session_key(&alice, &key_id)?;

// Bob tries to deposit again - fails
vault.deposit_with_session_key(&bob, &0, &alice, &usdc_token, &2_000, &(now + 86400), &0)?;
// ❌ Returns: Err(SessionKeyRevoked)

// But Alice's first deposit is still valid and can be withdrawn
```

---

### Data Structure

#### `SessionKeyEntry`

```rust
pub struct SessionKeyEntry {
    pub delegator: Address,          // Account that created this key
    pub delegate: Address,           // Account authorized to use this key
    pub max_amount: i128,            // Max total deposit via key (0 = unlimited)
    pub expires_at: u64,             // Expiration timestamp
    pub is_revoked: bool,            // Whether key is permanently disabled
    pub amount_used: i128,           // Total amount deposited via key so far
    pub created_at: u64,             // Timestamp when key was created
}
```

---

### Storage

Session keys are stored under the delegator's address and indexed by key ID.

| Storage Key | Type | Value |
|---|---|---|
| `SessionKey(delegator, key_id)` | `SessionKeyEntry` | The session key data |
| `SessionKeyCounter(delegator)` | `u32` | Next ID to allocate (auto-incremented) |
| `ActiveSessionKeyIds(delegator)` | `Vec<u32>` | All active key IDs for pagination |

**TTL**: Session key storage entries are extended to cover the maximum lock duration (5 years) plus buffer, ensuring keys persist throughout their validity period.

---

### Error Codes

New error codes added to `VaultError` enum:

| Code | Name | When Triggered |
|---|---|---|
| 20 | `SessionKeyNotFound` | Key ID doesn't exist under delegator |
| 21 | `SessionKeyRevoked` | Key exists but is_revoked = true |
| 22 | `SessionKeyExpired` | Current timestamp >= expires_at |
| 23 | `SessionKeyInsufficientLimit` | New deposit exceeds key's max_amount |

---

### Events

Three new events for session key operations:

#### 1. `session_key_created`

**Topics**: `("sk_created", delegator, delegate)`  
**Data**: `(key_id, max_amount, expires_at)`

Emitted when `create_session_key()` succeeds.

```json
{
  "topics": ["sk_created", "<delegator>", "<delegate>"],
  "data": [0, 5000, 1700000000]
}
```

#### 2. `session_key_revoked`

**Topics**: `("sk_revoked", delegator)`  
**Data**: `key_id`

Emitted when `revoke_session_key()` succeeds.

```json
{
  "topics": ["sk_revoked", "<delegator>"],
  "data": 0
}
```

#### 3. `deposit_with_session_key`

**Topics**: `("dep_sk", delegate, token)`  
**Data**: `(depositor, amount, session_key_id, deposit_id)`

Emitted when `deposit_with_session_key()` succeeds, in addition to the standard `deposit` event.

```json
{
  "topics": ["dep_sk", "<delegate>", "<token>"],
  "data": ["<delegator>", 1000, 0, 0]
}
```

---

## Test Coverage

### Tests Implemented

All tests are located in `contracts/safe-haven/src/test.rs` and prefixed with `test_session_key_*`:

| Test | Verifies |
|---|---|
| `test_session_key_create_success` | Key creation and counter auto-increment |
| `test_session_key_revoke_success` | Successful revocation |
| `test_session_key_revoke_not_found` | Error when revoking non-existent key |
| `test_session_key_revoke_already_revoked` | Idempotent guard: revoking twice fails |
| `test_session_key_deposit_after_revoke_fails` | Deposits fail after revocation |
| `test_session_key_revoke_before_expiry` | Can revoke keys before expiration |
| `test_session_key_revoke_is_permanent` | Revocation persists across time advances |
| `test_session_key_deposit_success` | Valid deposit with valid key |
| `test_session_key_non_revoked_keys_still_work` | Revoking one key doesn't affect others |
| `test_session_key_revoke_with_multiple_keys` | Revoke one of many keys without affecting siblings |
| `test_session_key_revoke_unauthorized` | Non-delegator cannot revoke |

### Running Tests

Run all session key tests:
```bash
make test -- session_key
```

Or run specific test:
```bash
make test -- test_session_key_revoke_is_permanent
```

---

## Acceptance Criteria: Verified ✅

| Criterion | Evidence |
|---|---|
| **Session key revoked successfully** | `test_session_key_revoke_success()` passes |
| **Subsequent deposits with revoked key fail** | `test_session_key_deposit_after_revoke_fails()` validates `SessionKeyRevoked` error |
| **Non-revoked keys unaffected** | `test_session_key_non_revoked_keys_still_work()` confirms siblings remain valid |
| **Revocation is permanent** | `test_session_key_revoke_is_permanent()` moves time forward and verifies key stays revoked |
| **Error is appropriate** | All tests verify correct `VaultError` codes (20-23) |

---

## Implementation Details

### Validation Order

`revoke_session_key()` performs checks in this order:

1. **Authorization**: `delegator.require_auth()` (first)
2. **Existence**: `SessionKeyNotFound` if key doesn't exist
3. **Already revoked**: `SessionKeyRevoked` if is_revoked = true
4. **Revocation**: Set is_revoked = true, update storage
5. **Event emission**: Emit `session_key_revoked` event
6. **Return**: `Ok(())`

### Security Properties

- **Auth-first**: Authorization check is the very first operation (Soroban best practice)
- **State cleared before external calls**: No external token transfers before storage update
- **Idempotent guard**: Second revocation returns error instead of silent success
- **No cascading effects**: Revoking one key only affects that key's entry
- **Immutable on-chain**: Once set, revoked flag cannot be un-set (no reversal)

---

## Limitations & Future Work

### Current Limitations

1. **No query function for key status**: To check if a key is revoked, caller must catch `SessionKeyRevoked` error on deposit attempt
2. **No pagination for session keys**: `get_active_session_key_ids()` returns all keys (could exhaust budget for many keys)
3. **No maximum session key duration enforced**: Keys can be created with far-future expiry (no upper-bound check)
4. **No delegate revocation by delegator**: Can only revoke at delegator level, not per-delegate

### Future Enhancements

- Add `get_session_key_status(delegator, key_id) -> Option<SessionKeyEntry>` query
- Add `get_session_keys_page(delegator, offset, limit)` for pagination
- Add MAX_SESSION_KEY_DURATION_SECS constant and validation
- Add `revoke_all_delegations(delegator)` bulk operation
- Track revocation timestamp for audit trail

---

## Example Workflows

### Workflow 1: One-Time Delegation

```
Scenario: Alice wants Bob to make a single deposit, then terminate his authority

1. Alice calls create_session_key(bob, unlimited, expires_in_1_hour)
   -> Returns key_id = 0

2. Bob calls deposit_with_session_key(0, alice, token, 1000, unlock_30_days, 5%)
   -> Deposit succeeds, ID = 0

3. Alice immediately calls revoke_session_key(0)
   -> Key is now revoked

4. Bob tries to deposit again
   -> SessionKeyRevoked error
```

### Workflow 2: Recurring Monthly Deposits

```
Scenario: Alice allows Bob to deposit exactly 1000 USDC each month

1. Alice calls create_session_key(bob, max_amount=1000, expires_in_30_days)
   -> Returns key_id = 0

2. Week 1: Bob deposits 1000 USDC
   -> Success (amount_used = 1000)

3. Week 2: Bob tries to deposit 100 more
   -> SessionKeyInsufficientLimit error (1000 + 100 > 1000)

4. Month expires:
   Alice can create a new key for next month, or revoke to permanently end delegation
```

### Workflow 3: Multi-Delegate Authorization

```
Scenario: Alice trusts both Bob and Charlie, but with different limits

1. Alice calls create_session_key(bob, max_amount=5000, expires_in_90_days)
   -> Returns key_id = 0

2. Alice calls create_session_key(charlie, max_amount=2000, expires_in_90_days)
   -> Returns key_id = 1

3. Bob deposits 3000 via key 0
   -> Success

4. Charlie deposits 2000 via key 1
   -> Success (independent limit)

5. Alice revokes Bob's key (key 0)
   -> Charlie's key (key 1) still works

6. Charlie tries to use revoked key 0
   -> SessionKeyRevoked error
```

---

## Migration & Upgrade Notes

### For Contract Upgrades

- Session keys use dedicated `VaultKey` variants, so they won't collide with other features
- `SessionKeyEntry` is a `#[contracttype]`, so it's serializable for upgrades
- No breaking changes to existing deposit/withdraw functions
- Session keys are opt-in: existing contracts can continue without session keys

### Data Backup

Session key entries are stored in persistent storage with TTL extended to cover maximum lock duration. No manual backup is needed for short-term operation, but for long-term archival:

```bash
# Export all session keys for a delegator
stellar contract read --id <CONTRACT_ID> --key "SessionKey(<delegator>,*)"
```

---

## Support & Troubleshooting

### Q: Can I un-revoke a session key?
**A**: No. Revocation is permanent and irreversible by design. Create a new session key if needed.

### Q: What happens to deposits made with a revoked key?
**A**: Deposits are immutable. A deposit made before revocation remains valid. Revocation only prevents future deposits.

### Q: Can the delegate revoke keys?
**A**: No. Only the delegator (key owner) can revoke. The delegate has no authority over the key's lifecycle.

### Q: How many session keys can one delegator create?
**A**: Theoretically unlimited (u32 counter), but pagination is unimplemented. Keep to <1000 keys per delegator for best performance.

### Q: Does revocation emit an event?
**A**: Yes. `session_key_revoked` event is emitted to support audit trails and notifications.

---

**Last Updated**: October 19, 2026  
**Prepared by**: Kiro (AI Agent)  
**Status**: ✅ Production Ready
