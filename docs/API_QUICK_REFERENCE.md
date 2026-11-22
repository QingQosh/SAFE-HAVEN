# SAFE-HAVEN API — Quick Reference Card

Print this page and keep it handy while developing!

---

## Function Signatures at a Glance

### Core Deposits
| Function | Signature | Returns |
|----------|-----------|---------|
| `deposit` | `(depositor, token, amount, unlock_time, penalty_bps)` | `u32` (ID) |
| `deposit_for` | `(payer, depositor, token, amount, unlock_time, penalty_bps)` | `u32` (ID) |
| `deposit_with_delay` | `(depositor, token, amount, delay_secs, penalty_bps)` | `u32` (ID) |
| `deposit_by_ledger` | `(depositor, token, amount, unlock_ledger, penalty_bps)` | `u32` (ID) |
| `batch_deposit` | `(depositor, [deposits])` | `Vec<u32>` (IDs) |
| `multi_deposit` | `(depositor, [(token, amount), ...], unlock_time, penalty_bps)` | `u32` (ID) |
| `deposit_with_compound_interest` | `(depositor, token, amount, unlock_time, penalty_bps, freq_secs)` | `u32` (ID) |

### Withdrawals
| Function | Signature | Returns |
|----------|-----------|---------|
| `withdraw` | `(depositor, deposit_id)` | `i128` (amount) |
| `withdraw_to` | `(depositor, deposit_id, recipient)` | `i128` (amount) |
| `cancel_deposit` | `(depositor, deposit_id)` | `()` |

### Staker Operations
| Function | Signature | Returns |
|----------|-----------|---------|
| `register_staker` | `(staker, amount)` | `()` |
| `claim_staker_rewards` | `(staker)` | `()` |
| `get_stakers` | `(offset, limit)` | `Page<Address>` |
| `get_staker_info` | `(staker)` | `Option<StakerInfo>` |

### Read-Only Queries
| Function | Signature | Returns |
|----------|-----------|---------|
| `get_vault` | `(depositor, deposit_id)` | `Option<VaultEntry>` |
| `get_ledger_vault` | `(depositor, deposit_id)` | `Option<LedgerVaultEntry>` |
| `get_multi_vault` | `(depositor, deposit_id)` | `Option<MultiTokenVaultEntry>` |
| `get_deposit_ids` | `(depositor)` | `Vec<u32>` |
| `get_deposit_type` | `(depositor, deposit_id)` | `Option<DepositType>` |
| `get_current_balance` | `(depositor, deposit_id)` | `i128` |
| `time_remaining` | `(depositor, deposit_id)` | `u64` |
| `get_time` | `()` | `u64` |
| `get_admin` | `()` | `Option<Address>` |
| `get_fee_recipient` | `()` | `Option<Address>` |
| `get_constants` | `()` | `ContractConstants` |

### Admin Operations
| Function | Signature | Returns |
|----------|-----------|---------|
| `initialize` | `(admin, fee_recipient, max_deposit?, max_lock_secs?)` | `()` |
| `pause` | `(admin)` | `()` |
| `unpause` | `(admin)` | `()` |
| `transfer_admin` | `(admin, new_admin)` | `()` |
| `accept_admin` | `(new_admin)` | `()` |
| `cancel_transfer_admin` | `(admin)` | `()` |
| `renounce_admin` | `(admin)` | `()` |
| `emergency_withdraw` | `(admin, depositor, deposit_id)` | `i128` |

### Session Keys
| Function | Signature | Returns |
|----------|-----------|---------|
| `authorize_session_key` | `(delegator, session_key_id, max_amount, expiration)` | `()` |
| `revoke_session_key` | `(delegator, session_key_id)` | `()` |
| `is_session_key_authorized` | `(delegator, session_key_id)` | `bool` |
| `deposit_with_session_key` | `(delegator, session_key_id, token, amount, unlock_time, penalty_bps)` | `u32` (ID) |

### Quantum-Safe Keys
| Function | Signature | Returns |
|----------|-----------|---------|
| `register_quantum_safe_key` | `(account, public_key)` | `()` |
| `get_quantum_safe_key` | `(account)` | `Option<Bytes>` |
| `quantum_safe_message` | `(account, message)` | `Result<Bytes>` |
| `deposit_quantum_safe` | `(account, token, amount, unlock_time, penalty_bps, signature)` | `u32` (ID) |

### MEV Protection
| Function | Signature | Returns |
|----------|-----------|---------|
| `mev_commit` | `(depositor, deposit_id, commit_hash)` | `()` |
| `mev_reveal` | `(depositor, deposit_id, strategy_data, recovery_amount)` | `()` |
| `claim_mev_recovery` | `(depositor)` | `i128` |
| `get_mev_detections` | `(depositor, limit)` | `Vec<MEVDetection>` |
| `get_mev_status_query` | `(depositor, deposit_id)` | `MEVStatus` |

### Prediction Markets
| Function | Signature | Returns |
|----------|-----------|---------|
| `init_prediction_markets` | `(admin)` | `()` |
| `create_prediction_market` | `(creator, description, outcomes, resolution_time, config)` | `u32` (ID) |
| `place_prediction_bet` | `(bettor, market_id, outcome_id, amount)` | `()` |
| `close_prediction_market` | `(admin, market_id)` | `()` |
| `resolve_prediction_market` | `(admin, market_id, winning_outcome_id)` | `()` |
| `claim_prediction_winnings` | `(bettor, market_id, outcome_id)` | `i128` |
| `get_prediction_market` | `(market_id)` | `Option<PredictionMarket>` |
| `get_market_outcome` | `(market_id, outcome_id)` | `Option<MarketOutcome>` |
| `get_user_bet` | `(market_id, outcome_id, bettor)` | `Option<Bet>` |

### Insurance Claims
| Function | Signature | Returns |
|----------|-----------|---------|
| `claim_insurance` | `(claimant, token, amount, evidence_uri)` | `u32` (ID) |
| `approve_claim` | `(admin, claim_id)` | `i128` |
| `deny_claim` | `(admin, claim_id)` | `()` |
| `get_claim` | `(claim_id)` | `Option<InsuranceClaim>` |
| `get_insurance_pool_balance` | `(token)` | `i128` |

### Migration
| Function | Signature | Returns |
|----------|-----------|---------|
| `get_storage_version` | `()` | `u32` |
| `migrate` | `(admin)` | `bool` |

---

## Key Constants

```
MIN_LOCK_DURATION_SECS        60
MAX_LOCK_DURATION_SECS        157,788,000  (≈ 5 years)
MAX_DEPOSIT_AMOUNT            1,000,000,000,000,000
MAX_TOKENS_PER_DEPOSIT        5
MAX_BATCH_SIZE                20
MAX_PENALTY_BPS               10,000  (100%)
MAX_SESSION_KEY_DURATION      2,592,000  (30 days)
MAX_EMERGENCY_WITHDRAW_LEDGER 100,000,000
PENALTY_BPS_STAKER_SHARE      7,000  (70%)
PENALTY_BPS_INSURANCE         500  (5%)
PENALTY_BPS_FEE_RECIPIENT     2,500  (25%)
ANNUAL_INTEREST_RATE          500 bps (5%)
```

---

## Error Codes (Most Common)

| Code | Error | Fix |
|------|-------|-----|
| 1 | `InvalidAmount` | Amount must be > 0 |
| 2 | `UnlockTimeNotInFuture` | Use future timestamp |
| 3 | `NoDepositFound` | Use valid deposit ID |
| 4 | `FundsStillLocked` | Wait until unlock |
| 6 | `LockDurationTooLong` | Max 5 years |
| 8 | `AmountTooLarge` | Exceeds max_deposit |
| 9 | `InvalidPenaltyBps` | Max 10,000 bps |
| 11 | `LockDurationTooShort` | Min 60 seconds |
| 12 | `ContractPaused` | Wait for unpause |
| 13 | `VaultAlreadyUnlocked` | Already withdrawn |
| 14 | `MissingFeeRecipient` | Need fee recipient |
| 16 | `TooManyTokens` | Max 5 per deposit |
| 17 | `EmptyTokenList` | Min 1 token |
| 18 | `RecipientNotWhitelisted` | Add to whitelist |
| 19 | `InvalidCompoundFrequency` | 0 or ≥ 60 secs |
| 27 | `BatchSizeExceeded` | Max 20 deposits |

---

## Common CLI Command Template

```bash
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source "$USER_SECRET" \
  --network testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- FUNCTION_NAME \
  --param1 value1 \
  --param2 value2
```

---

## Environment Variables to Set

```bash
export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export CONTRACT_ID="CC..."
export ADMIN_SECRET="S..."
export ADMIN_ADDRESS="G..."
export USER_SECRET="S..."
export USER_ADDRESS="G..."
export TOKEN_CONTRACT="CC..."
```

---

## Penalty Calculation

For early withdrawal with `penalty_bps`:

```
penalty_amount = (amount × penalty_bps) / 10,000

distribution:
  staker_pool    = penalty_amount × 7,000 / 10,000  (70%)
  insurance      = penalty_amount × 500 / 10,000    (5%)
  fee_recipient  = penalty_amount × 2,500 / 10,000  (25%)

user_receives = amount - penalty_amount
```

**Example**: 1,000 tokens, 200 bps penalty
- Penalty: 1,000 × 200 / 10,000 = 20 tokens
- Staker: 14 tokens
- Insurance: 1 token
- Fee: 5 tokens
- **User: 980 tokens**

---

## Compound Interest Formula

Applied each period:

```
new_balance = balance × (1 + (annual_rate × period_secs) / secs_per_year)
annual_rate  = 5%  (0.05 or 500 bps)
secs_per_year = 31,536,000
```

For daily compounding (86,400 secs):
```
daily_interest = balance × 0.05 × 86,400 / 31,536,000
               ≈ balance × 0.000137  (0.0137% per day)
```

---

## Type Reference

### VaultEntry
```rust
{
  token: Address,           // Token contract
  amount: i128,             // Locked amount
  unlock_time: u64,         // Unix timestamp
  penalty_bps: u32,         // Early-exit penalty
}
```

### LedgerVaultEntry
```rust
{
  token: Address,           // Token contract
  amount: i128,             // Locked amount
  unlock_ledger: u32,       // Ledger sequence
  penalty_bps: u32,         // Early-exit penalty
}
```

### MultiTokenVaultEntry
```rust
{
  tokens_and_amounts: Vec<(Address, i128)>,  // Token-amount pairs
  unlock_time: u64,                           // Unix timestamp
  penalty_bps: u32,                           // Shared penalty
}
```

### Page<T>
```rust
{
  items: Vec<T>,            // Current page items
  total_count: u32,         // Total items across all pages
  offset: u32,              // Current offset
  limit: u32,               // Page size
}
```

---

## Workflow Checklists

### First-Time Setup
- [ ] Deploy contract with `initialize(admin, fee_recipient, max_deposit, max_lock_secs)`
- [ ] Verify with `get_admin()` and `get_constants()`
- [ ] Test deposit on testnet
- [ ] Fund multiple test accounts
- [ ] Test withdrawal flow

### Production Deployment
- [ ] Thoroughly test on testnet (minimum 2 weeks)
- [ ] Get code audit (recommended)
- [ ] Set appropriate limits in `initialize()`
- [ ] Deploy to mainnet
- [ ] Verify contract ID
- [ ] Announce to users
- [ ] Monitor contract events daily

### Admin Handoff
- [ ] Prepare new admin account
- [ ] Call `transfer_admin(current_admin, new_admin)`
- [ ] Have new admin call `accept_admin(new_admin)`
- [ ] Verify with `get_admin()`
- [ ] Confirm new admin has backup private key

### Emergency Response
- [ ] Call `pause(admin)` to stop new deposits
- [ ] Investigate issue
- [ ] Use `emergency_withdraw()` if needed for affected users
- [ ] Document incident
- [ ] Call `unpause(admin)` once resolved

---

## Performance Tips

1. **Batch Operations** (up to 20 deposits in 1 tx):
   - Use `batch_deposit()` instead of separate calls
   - Reduces fees, faster confirmation

2. **Multi-Token Deposits** (up to 5 tokens in 1 tx):
   - Use `multi_deposit()` instead of separate deposits
   - Shared unlock time and penalty

3. **Pagination for Large Queries**:
   - Use `limit=50` for `get_depositors()` and `get_stakers()`
   - Higher limit = longer response time

4. **Cache Lookups**:
   - Store deposit IDs locally after creation
   - Reuse `get_deposit_ids()` results

5. **Monitor Events**:
   - Subscribe to contract events instead of polling
   - Use indexer for historical queries

---

## Security Reminders

⚠️ **Never**:
- Share private keys (even with developers)
- Store secrets in code or logs
- Use same key for multiple accounts
- Call `renounce_admin()` without being absolutely sure

✅ **Always**:
- Use environment variables for secrets
- Test on testnet first
- Set reasonable limits in `initialize()`
- Keep admin key in secure storage
- Use session keys for delegated access
- Monitor contract state changes
- Have incident response plan

---

## Useful Links

- **Full API Docs**: `/docs/API_COMPREHENSIVE.md`
- **Code Examples**: `/docs/API_EXAMPLES.md`
- **Contract Source**: `/contracts/safe-haven/src/contract.rs`
- **Test Suite**: `/contracts/safe-haven/src/test.rs`
- **GitHub Issues**: https://github.com/kenedybok3/SAFE-HAVEN/issues
- **Soroban Docs**: https://developers.stellar.org/docs/build/smart-contracts

---

**Last Updated**: November 2024  
**Quick Ref Version**: 1.0
