# MEV Protection System — SAFE-HAVEN

## Overview

SAFE-HAVEN includes a **MEV (Maximal Extractable Value) protection system** designed to prevent sandwich attacks, front-running, and other extractive behaviors that reduce user value and fairness. The system uses a **commit-reveal scheme** combined with **time-weighted average pricing (TWAP)** to detect and mitigate MEV attacks.

---

## Key Features

| Feature | Description |
|---|---|
| **Commit-Reveal Scheme** | Depositors commit transaction details privately (via hash), then reveal them after the commit is immutable on-chain. Prevents front-running by hiding order details until execution. |
| **Price Monitoring** | Continuous tracking of token prices; stores up to 24 hours of hourly price samples. |
| **Sandwich Attack Detection** | Compares current price against time-weighted average price (TWAP). Deviations exceeding 200 basis points (2%) trigger attack detection. |
| **MEV Recovery & Redistribution** | Recovered value is credited to affected users' MEV pools, which they can claim proportionally. |
| **Transparent Event Logging** | All MEV detections, commits, and recoveries are emitted as on-chain events for audit and monitoring. |
| **Admin Finalization** | Admin can trigger periodic finalization of MEV redistribution across affected deposits. |

---

## How It Works

### Step 1: Commit Phase

User initiates MEV protection by submitting a **private commit**:

```
mev_commit(
  depositor: Address,
  deposit_id: u32,
  commit_hash: Keccak256(token || amount || price || nonce)
)
```

**What happens:**
- Commit hash is stored on-chain with a timestamp.
- No details are revealed; only the hash is visible.
- Commit becomes immutable and verifiable.
- MEV status is set to `Committed`.
- Event `commit_submitted` is emitted.

**Why it works:**
- Attackers cannot predict the reveal (they don't know the preimage).
- The hash commits the transaction details without exposing them.
- Nonce provides protection against replay attacks.

### Step 2: Reveal Phase

Within **30 minutes** (configurable via `MEV_REVEAL_WINDOW_SECS`), the user reveals the actual transaction:

```
mev_reveal(
  depositor: Address,
  deposit_id: u32,
  token: Address,
  amount: i128,
  price: i128,
  nonce: u32
)
```

**What happens:**
- Contract recomputes the hash from revealed values.
- Compares recomputed hash against the committed hash.
- If mismatch → `CommitMismatch` error (prevents lying about orders).
- If match → record price sample and check for MEV attacks.
- MEV status moves to `Revealed`.
- Event `reveal_submitted` is emitted.

**Reveal window safety:**
- Prevents indefinite delays; enforces timely disclosure.
- Allows legitimate network latency without penalizing users.
- Returns `RevealWindowExpired` if reveal is too late.

### Step 3: MEV Detection

After a successful reveal, the contract **detects sandwich attacks** using TWAP:

**Algorithm:**
1. Get 24-hour price history for the token (hourly buckets).
2. Compute **time-weighted average price (TWAP)** with exponential decay (recent prices weighted more).
3. Compare current price against TWAP.
4. If deviation > 200 bps (2%) → MEV attack detected.

**Detection record:**
```rust
pub struct MEVDetection {
    pub timestamp: u64,              // When detected
    pub depositor: Address,          // Affected user
    pub deposit_id: u32,             // Which deposit
    pub price_deviation_bps: u32,    // How much (200+ = attack)
    pub mev_recovered: i128,         // Estimated MEV amount
    pub resolved: bool,              // Claim status
}
```

**Event emitted:**
```
mev_detected(
  depositor,
  deposit_id,
  price_deviation_bps,
  mev_recovered
)
```

### Step 4: Recovery & Redistribution

When MEV is detected, the recovered value is **credited to the MEV pool**:

**Pool mechanism:**
- Accumulated MEV is held in a contract-level pool.
- Each depositor has a claim proportional to their detected MEV.
- Users claim their share via `claim_mev_recovery()`.

**Claiming MEV:**
```
claim_mev_recovery(depositor) -> Result<i128, VaultError>
```

**What happens:**
- Check if depositor has pending MEV claims.
- Deduct claimed amount from MEV pool.
- Return the recovered value.
- Reset the depositor's claim to zero.

**Tracking:**
- `get_mev_pending(depositor)` — pending MEV to claim.
- `get_mev_pool_total()` — total pool size.
- `get_mev_detections(depositor, deposit_id, offset, limit)` — view all detections.

---

## Data Structures

### MEVCommitment

```rust
pub struct MEVCommitment {
    pub commit_hash: BytesN<32>,  // Keccak256 hash
    pub timestamp: u64,            // When submitted
    pub depositor: Address,        // Who committed
    pub revealed: bool,            // Revealed?
}
```

### MEVDetection

```rust
pub struct MEVDetection {
    pub timestamp: u64,              // Detection time
    pub depositor: Address,          // Affected user
    pub deposit_id: u32,             // Which deposit
    pub price_deviation_bps: u32,    // Deviation in basis points
    pub mev_recovered: i128,         // MEV recovered
    pub resolved: bool,              // Claim resolved?
}
```

### MEVStatus

```rust
pub enum MEVStatus {
    Unprotected,      // No MEV protection requested
    Committed,        // Commit submitted, awaiting reveal
    Revealed,         // Reveal confirmed
    AttackDetected,   // Sandwich attack found
}
```

### PriceSample

```rust
pub struct PriceSample {
    pub token: Address,  // Token
    pub price: i128,     // Price (smallest units)
    pub timestamp: u64,  // Sample time
}
```

---

## Storage Keys

MEV data is stored in persistent Soroban storage under these keys:

| Key | Type | Description |
|---|---|---|
| `MEVCommitment(depositor, deposit_id)` | `MEVCommitment` | Private commit for a deposit |
| `MEVStatus(depositor, deposit_id)` | `MEVStatus` | Current protection status |
| `MEVDetection(depositor, deposit_id, detection_id)` | `MEVDetection` | Individual attack record |
| `MEVDetectionCounter(depositor, deposit_id)` | `u32` | Count of detections per deposit |
| `MEVPriceHistory(token, timestamp_bucket)` | `Vec<PriceSample>` | Hourly price samples (24h window) |
| `MEVPool` | `i128` | Total recovered MEV pending distribution |
| `MEVClaimed(depositor)` | `i128` | MEV pending claim for depositor |

---

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 20 | `MEVAttackDetected` | Sandwich attack confirmed; transaction reverted |
| 21 | `CommitNotFound` | No commit exists for this deposit |
| 22 | `CommitMismatch` | Revealed values don't match committed hash |
| 23 | `RevealWindowExpired` | Reveal submitted after 30-minute window |
| 24 | `InvalidPriceData` | Price data is invalid or missing |

---

## Events

All MEV operations emit events for monitoring and audit:

| Event | Data | Meaning |
|---|---|---|
| `commit_submitted` | `(depositor, deposit_id, commit_hash)` | User submitted a commit |
| `reveal_submitted` | `(depositor, deposit_id, token, amount, price)` | User revealed a commit |
| `mev_detected` | `(depositor, deposit_id, deviation_bps, mev_amount)` | Attack detected |
| `mev_recovered` | `(total_recovered, affected_users)` | MEV recovered and distributed |

---

## Configuration

### MEV Constants

| Constant | Value | Description |
|---|---|---|
| `MEV_REVEAL_WINDOW_SECS` | 1,800 (30 min) | Time allowed to reveal after commit |
| `MEV_PRICE_DEVIATION_THRESHOLD_BPS` | 200 (2%) | Deviation threshold to trigger detection |
| `RENEWABLE_ENERGY_BASELINE` | 50 (%) | Default renewable energy % (sustainability) |
| `CARBON_BASELINE_PER_UNIT_SECOND` | 1 gram | Carbon footprint baseline |

These are defined in `constants.rs` and can be adjusted for different network conditions or threat models.

---

## API Reference

### Commit-Reveal Functions

#### `mev_commit(depositor, deposit_id, commit_hash) → Result<(), VaultError>`

Submit a private commit for MEV protection.

**Parameters:**
- `depositor` — Address protecting the deposit
- `deposit_id` — ID of the deposit to protect
- `commit_hash` — Keccak256(token || amount || price || nonce)

**Returns:** `Ok(())` on success; error if deposit doesn't exist or window expired.

**Events:** `commit_submitted`

---

#### `mev_reveal(depositor, deposit_id, token, amount, price, nonce) → Result<(), VaultError>`

Reveal the commit and trigger MEV detection.

**Parameters:**
- `depositor` — Same as commit
- `deposit_id` — Same as commit
- `token` — Token address
- `amount` — Token amount
- `price` — Execution price
- `nonce` — Random value for uniqueness

**Returns:** `Ok(())` if reveal matches commit and no attack detected; error otherwise.

**Events:** `reveal_submitted`, `mev_detected` (if attack found)

**Errors:**
- `CommitNotFound` — No prior commit exists
- `CommitMismatch` — Revealed hash doesn't match
- `RevealWindowExpired` — Reveal too late
- `InvalidPriceData` — Price is invalid (≤ 0)

---

#### `claim_mev_recovery(depositor) → Result<i128, VaultError>`

Claim recovered MEV from the pool.

**Parameters:**
- `depositor` — User claiming recovery

**Returns:** Amount recovered (in smallest token units).

**Errors:**
- `NoRewardsToClaim` — No MEV detected for this user

---

### Query Functions

#### `get_mev_status_query(depositor, deposit_id) → MEVStatus`

Get current MEV protection status for a deposit.

**Returns:** `Unprotected`, `Committed`, `Revealed`, or `AttackDetected`.

---

#### `get_mev_pending(depositor) → i128`

Get pending MEV recovery for a depositor.

**Returns:** Amount pending (before claim).

---

#### `get_mev_pool_total() → i128`

Get total MEV pool (all recovered and pending).

**Returns:** Total MEV in pool.

---

#### `get_mev_detections(depositor, deposit_id, offset, limit) → Result<Vec<MEVDetection>, VaultError>`

Paginated list of detected attacks for a deposit.

**Parameters:**
- `offset` — Starting index
- `limit` — Max results (capped at 50)

**Returns:** Vector of `MEVDetection` records.

---

### Admin Functions

#### `finalize_mev_redistribution(admin) → Result<i128, VaultError>`

Finalize MEV redistribution across all affected deposits.

**Parameters:**
- `admin` — Contract admin

**Returns:** Current MEV pool total.

**Access Control:** Admin only.

---

## Usage Example

### Step-by-step example:

```
1. Alice deposits 1000 USDC with 1% penalty, locked for 1000 seconds
   → deposit_id = 0

2. Alice submits MEV commit:
   commit_hash = Keccak256(usdc_addr || 1000 || 102 || 42)
   mev_commit(alice, 0, commit_hash)
   → Status: Committed

3. 100 seconds later (within 30-min window):
   mev_reveal(alice, 0, usdc_addr, 1000, 102, 42)
   → Hash matches → Status: Revealed
   → Price sample recorded

4. Contract computes TWAP from price history:
   If current_price (102) vs TWAP (100) = 200 bps deviation
   → MEV attack detected!
   → 2 USDC recovered and added to MEV pool
   → Event: mev_detected(alice, 0, 200, 2)

5. Alice later claims her recovery:
   mev_recovered = claim_mev_recovery(alice)
   → Returns: 2 USDC
   → MEV pool reduced by 2
```

---

## Security Considerations

### Threat Model

**Sandwich attacks:** Attacker observes pending transaction, front-runs with higher gas, profits from price slippage.

**MEV protection:**
- Commit-reveal hides order details until execution is final.
- TWAP detects unusual price movements.
- Honest price history prevents manipulation (requires many samples).

### Assumptions

1. **Keccak256 preimage resistance** — Attacker cannot invert hash to find preimage.
2. **Price history accuracy** — Oracle prices are reasonably accurate (±5% error acceptable).
3. **Reveal window timing** — Network latency < 30 minutes (Soroban standard).
4. **MEV pool honesty** — No contract bugs that double-count or lose MEV.

### Mitigations

- **TTL storage management** — MEV records persist for max lock duration.
- **Immutable commits** — Once written, commit hash cannot be changed.
- **Event audit trail** — All detections are logged on-chain.
- **Admin oversight** — Admin can review MEV pool and finalize redistributions.

---

## Limitations

| Limitation | Impact | Mitigation |
|---|---|---|
| **Price oracle accuracy** | TWAP may miss attacks in low-liquidity markets | Use reputable price feeds; audit oracle data |
| **30-minute reveal window** | Very slow orders may expire | Adjust `MEV_REVEAL_WINDOW_SECS` for different networks |
| **Fixed 200 bps threshold** | May be too sensitive or too lenient | Configure `MEV_PRICE_DEVIATION_THRESHOLD_BPS` per market |
| **No proactive reversion** | Contract cannot revert in-flight attacks | Commit-reveal deters attacks; recovery redistributes profits |
| **Storage growth** | Price history grows unbounded | Pruning strategy: keep only last 24 hours (implemented) |

---

## Testing

The MEV protection system includes comprehensive tests:

```bash
# Run MEV tests specifically
cargo test -p safe-haven -- mev_

# Test categories:
cargo test -p safe-haven -- test_mev_commit       # Commit phase
cargo test -p safe-haven -- test_mev_reveal       # Reveal phase
cargo test -p safe-haven -- test_mev_detection    # Attack detection
cargo test -p safe-haven -- test_mev_recovery     # Recovery & claims
cargo test -p safe-haven -- test_mev_integration  # Full workflow
```

---

## Future Enhancements

Potential improvements (out of scope for this release):

1. **Dynamic threshold adjustment** — Adjust MEV threshold based on market volatility.
2. **MEV redistribution optimization** — Distribute via Merkle tree instead of individual claims.
3. **Cross-token MEV detection** — Detect attacks across multiple token pairs.
4. **MEV auction** — Sell MEV to validators / searchers instead of redistributing.
5. **Threshold governance** — DAO voting on MEV detection parameters.
6. **Flash loan detection** — Flag transactions with unusual liquidity patterns.

---

## References

- [Flashbots MEV Research](https://flashbots.notion.site/Flashbots-MEV-Research-1e9d3b1e9e4c4b8b8e4c4b8b8e4c4b8b)
- [Commit-Reveal Schemes](https://en.wikipedia.org/wiki/Commitment_scheme)
- [TWAP (Time-Weighted Average Price)](https://docs.uniswap.org/concepts/protocol/oracle)
- [Soroban Storage & TTL](https://developers.stellar.org/docs/learn/storing-data)

---

## Support

For questions or issues with MEV protection:

1. Check the **Error Codes** table above
2. Review the **API Reference** for function signatures
3. Run the **test suite** to verify behavior
4. Contact the maintainers via GitHub Issues

---

**Version:** 1.0  
**Last Updated:** 2026-09-24  
**Status:** Production Ready
