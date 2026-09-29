# Staker Registry - Quick Reference

## 🎯 What's New

A complete staker registry system with penalty splitting rewards mechanism for SAFE-HAVEN contract.

## 💡 Core Concepts

### Staker Registration
```rust
vault.register_staker(&alice, &1000)?;  // Alice stakes 1000 tokens
vault.register_staker(&alice, &2000)?;  // Alice updates to 2000 tokens
```

### Penalty Splitting
When someone exits early with a penalty:
- 30% → Fee Recipient (direct transfer)
- 70% → Staker Rewards Pool (accumulated)

### Reward Distribution
Stakers claim proportional rewards based on their stake share:
```
staker_reward = (staker_stake / total_staked) * rewards_pool
```

## 📋 New API Functions

### register_staker(staker, amount) → Result<(), VaultError>
- Register or update staker's stake amount
- Requires: `amount > 0`
- Auth: Staker must sign
- Error: `InvalidStakeAmount`, `Unauthorized`

### claim_staker_rewards(staker) → Result<(), VaultError>
- Claim accumulated rewards from pool
- Requires: Staker registered, reward > 0
- Auth: Staker must sign
- Error: `StakerNotFound`, `NoRewardsToClaim`, `Unauthorized`

## 📊 Storage Schema

| Key | Type | Purpose |
|-----|------|---------|
| Staker(Address) | i128 | Staker's current stake |
| StakerList | Vec<Address> | All registered stakers |
| TotalStaked | i128 | Sum of all stakes |
| RewardsPool | i128 | Accumulated penalty rewards |
| StakerRewardsClaimed(Address) | i128 | Cumulative rewards claimed |

## 📈 Example Flow

```
1. Alice registers with 1000 tokens
   register_staker(&alice, &1000)

2. Bob registers with 3000 tokens
   register_staker(&bob, &3000)
   
3. Carol deposits 100 tokens with 10% penalty (1000 bps)
   deposit(&carol, &token, &100, &unlock_time, &1000)
   
4. Carol cancels early → 10 token penalty
   cancel_deposit(&carol, &deposit_id)
   
5. Penalty split: 3 to fee_recipient, 7 to rewards pool
   RewardsPool = 7

6. Alice claims (1000/4000 * 7 = 1.75 ≈ 1 token)
   claim_staker_rewards(&alice)
   
7. Bob claims (3000/4000 * 6 = 4.5 ≈ 4 tokens)
   claim_staker_rewards(&bob)
```

## 🔍 Error Codes

| Code | Error | When |
|------|-------|------|
| 16 | InvalidStakeAmount | Stake amount ≤ 0 |
| 17 | StakerNotFound | Staker not registered |
| 18 | NoRewardsToClaim | Pool empty or reward rounds to 0 |
| 19 | InsufficientStakeAmount | (Reserved for future use) |

## 📡 Events

### StakerRegistered
- Emitted when staker registers or updates stake
- Topics: staker address
- Data: stake amount

### PenaltySplit
- Emitted when penalty is split on cancel
- Topics: depositor address
- Data: total_penalty, fee_recipient_share, stakers_share, deposit_id

### RewardsClaimed
- Emitted when staker claims rewards
- Topics: staker address
- Data: claimed amount

## 🧪 Test Coverage

15 comprehensive tests covering:
- ✅ Registration (valid/invalid)
- ✅ Stake updates
- ✅ Multiple stakers
- ✅ Reward claiming
- ✅ Penalty splitting
- ✅ Proportional distribution
- ✅ Event emission
- ✅ Auth enforcement
- ✅ Edge cases (empty pool, rounding)

## 🔒 Security

- ✅ Auth-first: All mutations require staker signature
- ✅ Input validation: All amounts checked > 0
- ✅ TTL management: All storage has proper TTL extension
- ✅ Integer safety: No arithmetic side effects
- ✅ Backwards compatible: Old deposits unaffected

## 📚 Documentation

- Full API docs in README.md
- Implementation details in STAKER_REGISTRY_IMPLEMENTATION.md
- Verification checklist in IMPLEMENTATION_CHECKLIST.md

## 🚀 Getting Started

1. **Register as staker:**
   ```
   register_staker(my_address, 1000)
   ```

2. **Wait for penalties to accumulate** (from early withdrawals)

3. **Claim your rewards:**
   ```
   claim_staker_rewards(my_address)
   ```

4. **Check your claim history** via rewards_claimed tracking

## ⚙️ Constants

- STAKER_PENALTY_BPS = 7_000 (70%)
- FEE_RECIPIENT_PENALTY_BPS = 3_000 (30%)

## 🔗 Related Functions

- `cancel_deposit()` - Now splits penalties (modified)
- `deposit()` - Unchanged (works as before)
- `withdraw()` - Unchanged (works as before)
- `initialize()` - Unchanged (works as before)

---

**More Details:** See README.md, STAKER_REGISTRY_IMPLEMENTATION.md, IMPLEMENTATION_CHECKLIST.md
