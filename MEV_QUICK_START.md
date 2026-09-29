# MEV Protection — Quick Start Guide

## 30-Second Overview

MEV (Maximal Extractable Value) protection prevents sandwich attacks on deposits. Users commit their transaction privately, reveal it later, and get back any MEV that attackers tried to extract.

```
Commit (private) → Reveal (verify) → Detect (TWAP) → Recover (claim)
```

---

## 3-Step Usage

### 1️⃣ Commit Phase

```rust
// User privately commits transaction details as hash
commit_hash = keccak256(token || amount || price || nonce)
mev_commit(depositor, deposit_id, commit_hash)
```

**What happens:**
- Hash is stored on-chain (immutable)
- Status: `Committed`
- Event: `commit_submitted`
- Valid for 30 minutes

### 2️⃣ Reveal Phase

```rust
// User reveals actual transaction (within 30 minutes)
mev_reveal(depositor, deposit_id, token, amount, price, nonce)
```

**What happens:**
- Contract verifies hash matches (no lying)
- Price sample recorded
- TWAP detects sandwich attacks
- Status: `Revealed` or `AttackDetected`
- Event: `reveal_submitted`

### 3️⃣ Claim Phase

```rust
// User claims recovered MEV (anytime)
recovered = claim_mev_recovery(depositor)
```

**What happens:**
- MEV deducted from pool
- Tokens transferred to user
- Claim reset to zero

---

## Attack Detection

Sandwich attacks are detected via **TWAP (Time-Weighted Average Price)**:

```
If current_price deviates > 200 bps (2%) from TWAP
  → Attack detected
  → MEV recovered
  → Added to pool
  → Users can claim
```

---

## Query Status Anytime

```rust
// Check MEV protection status for a deposit
status = get_mev_status_query(alice, deposit_id)
// Returns: Unprotected, Committed, Revealed, AttackDetected

// Check pending MEV to claim
pending = get_mev_pending(alice)

// Check total pool size
pool_total = get_mev_pool_total()

// View detected attacks (paginated)
detections = get_mev_detections(alice, deposit_id, offset=0, limit=10)
```

---

## Error Codes (Quick Reference)

| Code | Error | Fix |
|---|---|---|
| 20 | `MEVAttackDetected` | Sandwich attack—claim recovery |
| 21 | `CommitNotFound` | Call `mev_commit()` first |
| 22 | `CommitMismatch` | Reveal values don't match commit |
| 23 | `RevealWindowExpired` | Must reveal within 30 minutes |
| 24 | `InvalidPriceData` | Price must be > 0 |

---

## Example Workflow

```
Timeline:
  T=0s:    Alice commits
           commit_hash = keccak256(usdc || 1000 || 102 || 42)
           mev_commit(alice, 0, commit_hash)
           ✓ Status: Committed

  T=100s:  Alice reveals (within 30-min window)
           mev_reveal(alice, 0, usdc, 1000, 102, 42)
           ✓ Hash verified ✓ TWAP calculated
           ✓ Status: Revealed

  T=105s:  Contract detects attack
           Current price: 102
           TWAP average: 100
           Deviation: 200 bps (2%)
           ✓ Attack detected!
           ✓ 2 USDC recovered
           ✓ Status: AttackDetected

  T=200s:  Alice claims recovery
           recovered = claim_mev_recovery(alice)
           ✓ Returns: 2 USDC
           ✓ Pool reduced by 2
           ✓ Alice receives tokens

Result: Alice gets fair execution + MEV recovery!
```

---

## Configuration

MEV parameters in `constants.rs`:

```rust
MEV_REVEAL_WINDOW_SECS = 1,800        // 30 minutes
MEV_PRICE_DEVIATION_THRESHOLD_BPS = 200  // 2%
```

Adjust for different networks:
- **Slow networks** → increase reveal window
- **Volatile markets** → increase deviation threshold

---

## Integration Checklist

- [ ] Call `mev_commit()` when user opts in
- [ ] Call `mev_reveal()` after transaction confirmed
- [ ] Monitor `mev_detected` events
- [ ] Provide UI for `claim_mev_recovery()`
- [ ] Query status with `get_mev_status_query()`
- [ ] Display pending MEV with `get_mev_pending()`

---

## Testing

Run MEV tests:

```bash
cargo test -p safe-haven -- mev_
```

Quick tests:
- Commit happy path
- Reveal validation
- Attack detection
- Recovery claims
- Admin functions

---

## Common Questions

**Q: What if I don't want MEV protection?**
A: Just don't call `mev_commit()`. Deposits work normally.

**Q: What happens if I miss the 30-minute window?**
A: Reveal fails with `RevealWindowExpired`. You can try again with a new commit.

**Q: How much MEV can I recover?**
A: Depends on price deviation from TWAP. Larger deviations = more recovery.

**Q: Is MEV protection guaranteed?**
A: No, it's probabilistic. Commit-reveal deters attacks but doesn't prevent all MEV.

**Q: Can I reuse the same commit hash?**
A: No, use unique nonce values for each reveal to prevent replay.

---

## Troubleshooting

| Issue | Cause | Fix |
|---|---|---|
| `CommitNotFound` on reveal | Deposit doesn't exist | Verify deposit_id is correct |
| `CommitMismatch` | Revealed values wrong | Recompute hash from original values |
| `RevealWindowExpired` | Too slow to reveal | Reveal within 30 minutes |
| `InvalidPriceData` | Price ≤ 0 | Ensure price > 0 |
| No MEV detected | No sandwich attack | TWAP within 200 bps = normal market |

---

## For More Details

- **Full spec:** See [MEV_PROTECTION.md](./MEV_PROTECTION.md)
- **API reference:** See [README.md](./README.md) "MEV Protection" section
- **Examples:** See `test.rs` for usage patterns
- **Design decisions:** See [MEV_IMPLEMENTATION_SUMMARY.md](./MEV_IMPLEMENTATION_SUMMARY.md)

---

**Version:** 1.0  
**Last Updated:** 2026-09-24  
**Status:** Production Ready
