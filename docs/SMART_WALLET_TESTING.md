# Smart Wallet Integration Test Suite Documentation

## Overview

The SAFE-HAVEN smart wallet integration test suite (`smart_wallet_tests.rs`) provides comprehensive coverage of contract addresses as depositors, authorization validation, session key workflows, and multisig/account abstraction patterns. This document describes the test suite, its organization, and how to use it.

## Quick Start

### Running All Smart Wallet Tests

```bash
# Run all smart wallet tests
cargo test --lib smart_wallet --features testutils

# Run with output
cargo test --lib smart_wallet --features testutils -- --nocapture

# Run a specific test
cargo test --lib smart_wallet test_contract_address_deposit_success --features testutils
```

### Running by Category

```bash
# Contract address tests
cargo test --lib smart_wallet test_contract_address --features testutils

# Session key tests
cargo test --lib smart_wallet test_session_key --features testutils

# Deposit for (third-party funding) tests
cargo test --lib smart_wallet test_deposit_for --features testutils

# Multisig tests
cargo test --lib smart_wallet test_multisig --features testutils

# Account abstraction tests
cargo test --lib smart_wallet test_account_abstraction --features testutils

# Authorization tests
cargo test --lib smart_wallet test_require_auth --features testutils
```

## Test Suite Structure

### 1. Contract Address as Depositor (5 tests)

Tests verify that Soroban contract addresses can own vault positions and perform all deposit lifecycle operations.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_contract_address_deposit_success` | Contract creates a deposit | Deposit ID = 0, amount verified |
| `test_contract_address_withdraw_after_unlock` | Contract withdraws own deposit | Deposit removed after unlock time |
| `test_contract_address_cannot_withdraw_before_unlock` | Time lock enforced | Returns `FundsStillLocked` error |
| `test_contract_address_cancel_deposit_with_penalty` | Early exit with penalty | Deposit removed on cancel |
| `test_contract_address_multiple_separate_deposits` | Multiple deposits per address | IDs increment correctly |

**Key Concepts**:
- Smart wallet addresses are Soroban contract addresses, not EOAs
- `require_auth()` mechanism validates the contract address is authorized
- Deposits are owned by the contract address, not any individual signer

### 2. require_auth() Validation (3 tests)

Tests verify that `require_auth()` is properly enforced at the contract boundary.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_require_auth_contract_enforced_on_deposit` | Auth check enforced | State mutation prevented without auth |
| `test_require_auth_enforced_before_state_mutation` | Auth-first pattern | Verified state changes only after auth |
| `test_unauthorized_address_cannot_withdraw` | Only depositor can withdraw | Attacker fails with authorization error |
| `test_unauthorized_address_cannot_cancel` | Only depositor can cancel | Attacker fails with authorization error |

**Key Concepts**:
- `require_auth()` must be the first call in state-mutating functions
- Authorization is scoped to the caller's address
- Depositor-only operations reject unauthorized callers

### 3. Session Keys and Time-Bounded Authorization (6 tests)

Tests validate delegated payer workflows with time-bounded expiry.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_authorize_session_key_success` | Session key registration | Wallet authorizes payer |
| `test_session_key_deposit_with_valid_expiry` | Valid delegation workflow | Deposit created under wallet, funded by session key |
| `test_session_key_expiry_rejection` | Time boundary enforcement | Expired key rejected with `SessionKeyExpired` |
| `test_unauthorized_session_key_rejected` | Unregistered key rejection | Returns `SessionKeyNotAuthorized` |
| `test_revoke_session_key` | Key revocation | After revoke, key can no longer be used |
| `test_session_key_scoped_to_wallet` | Authorization scope | Session key valid for only one wallet |

**Key Concepts**:
- Session keys enable delegated funding for a bounded time period
- Expiry is checked against ledger timestamp
- Keys are wallet-specific and cannot be reused across wallets
- Revocation immediately disables the key

#### Session Key Security Properties

**Scope**: Session keys are scoped to a single wallet address. A session key authorized for wallet A cannot be used with wallet B.

**Expiry**: Session keys expire based on ledger timestamp. The check is `expires_at > env.ledger().timestamp()`.

**Revocation**: Wallets can revoke session keys at any time via `revoke_session_key()`. Revocation is immediate.

**Funding**: Session keys fund the deposit (tokens transferred from session key), but the deposit is owned by the wallet (not the session key).

### 4. Deposit For (Third-Party Funding) (5 tests)

Tests verify relayer-funded deposits where payer ≠ depositor/owner.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_deposit_for_payer_funds_beneficiary_wallet` | Third-party funding | Deposit created under wallet owner |
| `test_deposit_for_beneficiary_can_withdraw` | Ownership is correct | Only beneficiary can withdraw |
| `test_deposit_for_payer_cannot_withdraw` | Payer lacks access | Payer cannot withdraw beneficiary's deposit |
| `test_deposit_for_with_penalty` | Penalty configuration | Penalty terms applied correctly |
| `test_multiple_payers_for_same_beneficiary` | Multiple funding sources | Each payer creates independent deposit |

**Key Concepts**:
- `deposit_for(payer, depositor, ...)` creates a deposit owned by `depositor`
- Payer funds the deposit; beneficiary owns it
- Payer never gains control over the deposit
- Multiple payers can fund the same beneficiary

### 5. Multisig Patterns (4 tests)

Tests simulate multisig wallet scenarios and authorization requirements.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_multisig_wallet_deposit` | Multisig holds positions | Multisig address owns deposit |
| `test_multisig_wallet_withdraw` | Multisig withdrawal | Multisig contract can withdraw |
| `test_multisig_prevents_single_signer_withdrawal` | Access control | Single signer cannot withdraw |
| `test_multisig_delegated_funding` | Relayer funding | Payer funds multisig wallet |

**Key Concepts**:
- Multisig wallets are represented as Soroban contract addresses
- The vault does not inspect individual signers; multisig policy is enforced by the wallet contract
- Once multisig approval is met, the wallet calls the vault contract
- `require_auth()` validates the multisig contract address, not individual signers

### 6. Account Abstraction Patterns (4 tests)

Tests cover AA wallet scenarios and delegated operations.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_account_abstraction_wallet_deposit` | AA wallet deposits | AA address owns position |
| `test_account_abstraction_entry_point_call` | Entry point invocation | Entry point contract calls vault |
| `test_account_abstraction_session_key_delegation` | Delegated payer | Session key funds for AA wallet |
| `test_account_abstraction_multi_operation_sequence` | Multi-deposit lifecycle | AA executes multiple operations |

**Key Concepts**:
- AA wallets are Soroban contract addresses with custom authorization logic
- Entry points can call the vault on behalf of AA accounts
- Session keys enable gasless or relayed operations
- AA wallets maintain full control even with delegation

### 7. Authorization Edge Cases and Security (3 tests)

Tests verify security properties and boundary conditions.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_session_key_correct_wallet_scope` | Wallet boundary enforcement | Key valid for one wallet only |
| `test_session_key_time_boundary` | Timestamp boundary check | Exact expiry behavior tested |
| `test_multiple_deposits_independent_ownership` | Deposit independence | Each deposit has independent lifecycle |

**Key Concepts**:
- Session key scope is enforced per-wallet
- Time boundaries use `>` (expires_at > current_time)
- Each deposit has independent ownership and can be managed separately

### 8. Error Handling (3 tests)

Tests verify error cases and rejection patterns.

| Test | Purpose | Key Assertion |
|---|---|---|
| `test_invalid_session_key_auth_check` | Unregistered key rejection | Returns `SessionKeyNotAuthorized` |
| `test_session_key_not_found_after_revoke` | Post-revoke rejection | Revoked key cannot be used |
| `test_deposit_to_contract_address_with_zero_amount` | Input validation | Invalid amounts rejected |
| `test_session_key_expired_before_use` | Expiry enforcement | Expired key rejected immediately |

**Key Concepts**:
- Unregistered keys fail early with clear error code
- Revoked keys are immediately invalidated
- Invalid inputs (zero amounts) are rejected
- Expiry is enforced at operation time

## Test Helpers

### `setup()`

Creates a test environment with standard configuration.

```rust
let (env, vault, token, admin, alice, fee_recipient) = setup();
```

**Returns**:
- `env`: Soroban test environment
- `vault`: Deployed vault contract client
- `token`: Stellar asset contract address
- `admin`: Admin address
- `alice`: Funded test account
- `fee_recipient`: Fee recipient address

### `create_wallet(env)`

Creates a mock smart contract address for testing.

```rust
let wallet = create_wallet(&env);
```

### `advance_time(env, seconds)`

Advances the ledger timestamp by the specified number of seconds.

```rust
advance_time(&env, 3600);  // Move time forward 1 hour
```

## Test Data and Constants

| Constant | Value | Purpose |
|---|---|---|
| Default unlock time | `now + 3600` | 1 hour lock |
| Standard deposit amount | `5_000` | Test token amount |
| Session key expiry | `now + 3600` | 1 hour session validity |
| Penalty basis points | `1000` or `2000` | 10% or 20% early-exit fee |

## Security Assumptions

### 1. Smart Wallet Responsibility

The vault assumes smart wallets implement their own approval policies:
- Signature aggregation and threshold checking
- Key recovery procedures
- Transaction replay protection
- Token approval logic

The vault does NOT:
- Inspect individual signers
- Implement custom wallet logic
- Recover lost keys
- Bypass wallet policies

### 2. Authorization Model

- `require_auth()` is enforced as the first call in state-mutating functions
- Authorization is checked by Soroban, not by the contract
- Deposits are owned by the `depositor` address, regardless of who submitted the transaction

### 3. Session Key Trust Model

Session keys are semi-trusted:
- Trusted to spend tokens from the session key's account (not the wallet)
- Scoped to a single wallet (cannot cross-wallet attacks)
- Time-bounded (automatic expiry)
- Can be revoked by the wallet at any time

### 4. Ledger Timestamp Assumptions

- Ledger timestamps increment monotonically
- Session key expiry is checked: `expires_at > env.ledger().timestamp()`
- Wall-clock time depends on Stellar network consensus (typically 5-second ledger closure)

## Running Tests in CI

The test suite can be integrated into CI/CD pipelines:

```bash
# In a Docker container or CI environment with Rust installed
cargo test --lib smart_wallet --features testutils -- --test-threads=1 --nocapture
```

### Exit Codes

- `0` = All tests passed
- `1` = One or more tests failed
- Tests use standard `assert!` macros; failures are reported with line numbers

## Common Test Patterns

### Pattern 1: Setup + Action + Assert

```rust
#[test]
fn test_example() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    
    // Setup: Prepare test data
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);
    let unlock_time = env.ledger().timestamp() + 3600;
    
    // Action: Perform operation
    vault.deposit(&alice, &token, &5_000, &unlock_time, &0);
    
    // Assert: Verify result
    let entry = vault.get_vault(&alice, &0).unwrap();
    assert_eq!(entry.amount, 5_000);
}
```

### Pattern 2: Error Case Testing

```rust
#[test]
fn test_error_example() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    
    let result = vault.try_deposit(&alice, &token, &0, &100, &0);
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}
```

### Pattern 3: Time-Dependent Tests

```rust
#[test]
fn test_time_example() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    
    vault.deposit(&alice, &token, &5_000, &unlock_time, &0);
    
    // Before unlock
    let result1 = vault.try_withdraw(&alice, &0);
    assert_eq!(result1, Err(Ok(VaultError::FundsStillLocked)));
    
    // After unlock
    advance_time(&env, 3601);
    let result2 = vault.try_withdraw(&alice, &0);
    assert!(result2.is_ok());
}
```

## Integration Checklist

When integrating smart wallets with SAFE-HAVEN:

- [ ] Wallet authorizes both vault invocation and token transfer
- [ ] Depositor address is the wallet contract address (not relayer)
- [ ] Use `deposit_for` for third-party funding scenarios
- [ ] Set explicit expiry for all session keys
- [ ] Revoke session keys when sessions end
- [ ] Test wallet's multisig threshold and rejection scenarios
- [ ] Verify replay protection in wallet implementation
- [ ] Monitor deposit events by `depositor` address (not transaction signer)
- [ ] Test wallet address derivation and consistency

## Troubleshooting

### Test Fails: "NotAuthorized"

**Cause**: `require_auth()` check failed

**Solution**: Ensure the caller is the expected address and test uses `env.mock_all_auths()`

### Test Fails: "SessionKeyNotAuthorized"

**Cause**: Session key not registered or scoped to wrong wallet

**Solution**: Verify `authorize_session_key` was called with correct wallet and session_key addresses

### Test Fails: "SessionKeyExpired"

**Cause**: Current ledger timestamp >= session key expiry

**Solution**: Use `advance_time()` before expiry time, or increase expiry duration

### Test Fails: "FundsStillLocked"

**Cause**: Withdraw attempted before unlock time

**Solution**: Use `advance_time()` to move past the unlock timestamp

## Future Enhancements

Potential additions to the test suite:

1. **Stress Tests**: Many deposits/withdrawals per address
2. **Gas Profiling**: Measure instruction budget per operation
3. **State Corruption**: Simulate missing/corrupted ledger entries
4. **Network Conditions**: Test with high latency/network delays
5. **Multi-Token Wallets**: Test wallets managing multiple token types
6. **Rotation**: Test session key rotation (revoke + reauth)
7. **Recovery**: Test wallet recovery workflows with session keys

## References

- [Smart Wallet Integration Guide](../docs/SMART_WALLETS.md)
- [SAFE-HAVEN README](../README.md)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Contract Security Guide](../SECURITY.md)
