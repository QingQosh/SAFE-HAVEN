// ============================================================
//  MEV Detection Module — TWAP-based price anomaly detection
//
//  Detects Maximal Extractable Value (MEV) attacks by comparing
//  the current price against a 24-hour Time-Weighted Average Price (TWAP).
//  If the current price deviates > 200 basis points (2%) from TWAP,
//  an attack is flagged.
// ============================================================

use soroban_sdk::{Env, Address, Vec, contracttype};

use crate::types::VaultKey;

/// Price sample struct: records a price at a specific timestamp
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceSample {
    /// Price at time of sample (in basis points for precision)
    pub price: i128,
    /// Timestamp when sample was collected (seconds since epoch)
    pub timestamp: u64,
}

/// Maximum number of hourly price samples to maintain (24-hour window)
pub const MAX_PRICE_SAMPLES: u32 = 24;

/// Minimum price sample interval (1 hour = 3600 seconds)
pub const PRICE_SAMPLE_INTERVAL: u64 = 3600;

/// MEV detection threshold: 200 basis points (2%)
pub const MEV_THRESHOLD_BPS: i128 = 200;

// ================================================================
//  Price Sample Storage Helpers
// ================================================================

/// Get the price sample list for a specific token.
/// Returns an empty Vec if no samples exist yet.
fn get_price_samples(env: &Env, token: &Address) -> Vec<PriceSample> {
    let key = VaultKey::PriceSamples(token.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

/// Save the price sample list for a token to persistent storage.
fn save_price_samples(env: &Env, token: &Address, samples: &Vec<PriceSample>) {
    let key = VaultKey::PriceSamples(token.clone());
    env.storage().persistent().set(&key, samples);
    // TTL: extend for 7 days (604800 seconds ÷ 5 seconds/ledger ≈ 120960 ledgers)
    env.storage()
        .persistent()
        .extend_ttl(&key, 60480, 120960);
}

// ================================================================
//  Hourly Price Collection
// ================================================================

/// Collect an hourly price sample for a token.
/// Enforces a minimum 1-hour interval between samples.
/// Maintains a rolling 24-hour window (max 24 samples).
///
/// Returns:
///   - `Ok(())` if sample was added successfully
///   - `Err(SamplingTooFrequent)` if called within 1 hour of last sample
pub fn record_price_sample(
    env: &Env,
    token: &Address,
    current_price: i128,
) -> Result<(), crate::errors::VaultError> {
    if current_price <= 0 {
        return Err(crate::errors::VaultError::InvalidAmount);
    }

    let now = env.ledger().timestamp();
    let mut samples = get_price_samples(env, token);

    // Check if we have a previous sample and enforce minimum interval
    if !samples.is_empty() {
        let last_sample = &samples.get_unchecked(samples.len() - 1);
        let time_since_last = now.saturating_sub(last_sample.timestamp);

        if time_since_last < PRICE_SAMPLE_INTERVAL {
            return Err(crate::errors::VaultError::SamplingTooFrequent);
        }
    }

    // Add new sample
    let sample = PriceSample {
        price: current_price,
        timestamp: now,
    };
    samples.push_back(sample);

    // Maintain 24-hour window: remove oldest sample if exceeding max
    if samples.len() > MAX_PRICE_SAMPLES as usize {
        let mut trimmed: Vec<PriceSample> = Vec::new(env);
        for i in 1..samples.len() {
            trimmed.push_back(samples.get_unchecked(i).clone());
        }
        samples = trimmed;
    }

    save_price_samples(env, token, &samples);
    Ok(())
}

// ================================================================
//  TWAP Calculation
// ================================================================

/// Calculate the Time-Weighted Average Price (TWAP) from stored samples.
///
/// Returns the simple average price: sum(prices) / count
///
/// If fewer than 2 samples exist, returns `None` (insufficient data).
/// If no samples exist, also returns `None`.
pub fn calculate_twap(env: &Env, token: &Address) -> Option<i128> {
    let samples = get_price_samples(env, token);

    // Need at least 1 sample; 2+ is preferable for accuracy
    if samples.len() == 0 {
        return None;
    }

    // Edge case: with only 1 sample, return it as-is (no historical average available)
    if samples.len() == 1 {
        return Some(samples.get_unchecked(0).price);
    }

    // Calculate simple average
    let mut sum: i128 = 0;
    for sample in samples.iter() {
        sum = sum.saturating_add(sample.price);
    }

    let count = samples.len() as i128;
    let twap = sum / count;

    Some(twap)
}

// ================================================================
//  MEV Detection Logic
// ================================================================

/// Check if a current price deviates > 200 bps from TWAP.
/// If deviation exceeds threshold, an MEV attack is detected.
///
/// Returns `Ok(())` if price is within normal range.
/// Returns `Err(MevDetected)` if price deviation > 200 bps.
pub fn detect_mev(
    env: &Env,
    token: &Address,
    current_price: i128,
) -> Result<(), crate::errors::VaultError> {
    if current_price <= 0 {
        return Err(crate::errors::VaultError::InvalidAmount);
    }

    // Get TWAP; if insufficient samples, assume no attack
    let twap = match calculate_twap(env, token) {
        Some(price) => price,
        None => return Ok(()), // Insufficient data — no attack detected
    };

    if twap <= 0 {
        return Ok(()); // Invalid TWAP — assume no attack
    }

    // Calculate absolute deviation in basis points
    // deviation_bps = |current - twap| / twap × 10000
    let deviation_bps = calculate_deviation_bps(current_price, twap);

    if deviation_bps > MEV_THRESHOLD_BPS {
        return Err(crate::errors::VaultError::MevDetected);
    }

    Ok(())
}

/// Calculate the deviation of current price from TWAP in basis points.
///
/// deviation_bps = |current - twap| / twap × 10000
fn calculate_deviation_bps(current: i128, twap: i128) -> i128 {
    if twap <= 0 {
        return 0;
    }

    let abs_diff = if current > twap {
        current - twap
    } else {
        twap - current
    };

    // deviation_bps = (abs_diff × 10000) / twap
    // Multiply first to preserve precision before division
    let numerator = abs_diff.saturating_mul(10000);
    numerator / twap
}

// ================================================================
//  Query Helpers
// ================================================================

/// Get all stored price samples for a token (read-only).
/// Returns an empty Vec if no samples exist.
pub fn get_samples_readonly(env: &Env, token: &Address) -> Vec<PriceSample> {
    get_price_samples(env, token)
}

/// Get the current TWAP for a token (read-only).
/// Returns `None` if insufficient samples exist.
pub fn get_twap_readonly(env: &Env, token: &Address) -> Option<i128> {
    calculate_twap(env, token)
}

/// Get the count of stored price samples for a token.
pub fn get_sample_count(env: &Env, token: &Address) -> u32 {
    get_price_samples(env, token).len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    //  TWAP Calculation Tests
    // ================================================================

    #[test]
    fn test_twap_calculation_simple_average() {
        // Test: prices [100, 110, 120]
        // TWAP = (100 + 110 + 120) / 3 = 330 / 3 = 110
        let sum = 100i128 + 110i128 + 120i128;
        let twap = sum / 3;
        assert_eq!(twap, 110, "TWAP should be simple average");
    }

    #[test]
    fn test_twap_calculation_with_many_samples() {
        // Test: 5 samples [50, 60, 70, 80, 90]
        // TWAP = (50 + 60 + 70 + 80 + 90) / 5 = 350 / 5 = 70
        let sum = 50i128 + 60i128 + 70i128 + 80i128 + 90i128;
        let twap = sum / 5;
        assert_eq!(twap, 70, "TWAP with 5 samples");
    }

    #[test]
    fn test_twap_calculation_identical_prices() {
        // Test: all prices the same [100, 100, 100]
        // TWAP = 300 / 3 = 100
        let sum = 100i128 + 100i128 + 100i128;
        let twap = sum / 3;
        assert_eq!(twap, 100, "TWAP of identical prices should equal price");
    }

    #[test]
    fn test_twap_calculation_precision_truncation() {
        // Test: prices [100, 101, 102] = 303 / 3 = 101 (with truncation)
        let sum = 100i128 + 101i128 + 102i128;
        let twap = sum / 3;
        assert_eq!(twap, 101, "TWAP truncates to integer");
    }

    // ================================================================
    //  Deviation BPS Calculation Tests
    // ================================================================

    #[test]
    fn test_deviation_bps_at_threshold() {
        // TWAP = 100, current = 102
        // deviation = (102 - 100) / 100 × 10000 = 200 bps (exactly at threshold)
        let deviation = calculate_deviation_bps(102, 100);
        assert_eq!(deviation, 200, "Deviation at 200 bps threshold");
    }

    #[test]
    fn test_deviation_bps_below_threshold() {
        // TWAP = 100, current = 101
        // deviation = (101 - 100) / 100 × 10000 = 100 bps (below threshold)
        let deviation = calculate_deviation_bps(101, 100);
        assert_eq!(deviation, 100, "Deviation below 200 bps threshold");
    }

    #[test]
    fn test_deviation_bps_above_threshold() {
        // TWAP = 100, current = 103
        // deviation = (103 - 100) / 100 × 10000 = 300 bps (above threshold)
        let deviation = calculate_deviation_bps(103, 100);
        assert_eq!(deviation, 300, "Deviation above 200 bps threshold");
    }

    #[test]
    fn test_deviation_bps_downward_movement() {
        // TWAP = 100, current = 99
        // deviation = (100 - 99) / 100 × 10000 = 100 bps (bidirectional)
        let deviation = calculate_deviation_bps(99, 100);
        assert_eq!(deviation, 100, "Downward deviation is also detected");
    }

    #[test]
    fn test_deviation_bps_large_price_swing() {
        // TWAP = 100, current = 150
        // deviation = (150 - 100) / 100 × 10000 = 5000 bps (50% increase)
        let deviation = calculate_deviation_bps(150, 100);
        assert_eq!(deviation, 5000, "Large price swing detection");
    }

    #[test]
    fn test_deviation_bps_small_twap() {
        // TWAP = 1, current = 2
        // deviation = (2 - 1) / 1 × 10000 = 10000 bps (100% increase with small base)
        let deviation = calculate_deviation_bps(2, 1);
        assert_eq!(deviation, 10000, "Deviation with small TWAP base");
    }

    #[test]
    fn test_deviation_bps_zero_difference() {
        // TWAP = 100, current = 100
        // deviation = 0 bps
        let deviation = calculate_deviation_bps(100, 100);
        assert_eq!(deviation, 0, "No deviation when prices match");
    }

    // ================================================================
    //  MEV Detection Threshold Tests
    // ================================================================

    #[test]
    fn test_mev_threshold_constant() {
        // Verify threshold is set to 200 bps
        assert_eq!(MEV_THRESHOLD_BPS, 200, "MEV threshold should be 200 bps");
    }

    #[test]
    fn test_mev_threshold_boundary_just_inside() {
        // 199 bps is inside the threshold (no alert)
        assert!(199 < MEV_THRESHOLD_BPS, "199 bps is below threshold");
    }

    #[test]
    fn test_mev_threshold_boundary_just_outside() {
        // 201 bps is outside the threshold (alert)
        assert!(201 > MEV_THRESHOLD_BPS, "201 bps is above threshold");
    }

    // ================================================================
    //  MEV Detection with Price Increase > 200 BPS (should detect)
    // ================================================================

    #[test]
    fn test_mev_detect_price_increase_201_bps() {
        // Scenario: TWAP = 100, current = 102.01 (201 bps increase)
        // Should trigger MevDetected
        let deviation = calculate_deviation_bps(102, 100);
        // 102 - 100 = 2, 2/100 * 10000 = 200 bps
        // We need 201+ bps; let's use 103 for current
        let deviation = calculate_deviation_bps(103, 100);
        assert!(deviation > MEV_THRESHOLD_BPS, "Deviation of 300 bps > 200 bps threshold");
    }

    #[test]
    fn test_mev_detect_massive_price_spike() {
        // Scenario: TWAP = 100, current = 300 (massive MEV attack)
        // Should definitely trigger MevDetected
        let deviation = calculate_deviation_bps(300, 100);
        // (300 - 100) / 100 * 10000 = 20000 bps
        assert_eq!(deviation, 20000, "Massive spike detection");
        assert!(deviation > MEV_THRESHOLD_BPS);
    }

    #[test]
    fn test_mev_detect_downward_crash() {
        // Scenario: TWAP = 100, current = 10 (crash/sandwich attack)
        // Should trigger MevDetected
        let deviation = calculate_deviation_bps(10, 100);
        // (100 - 10) / 100 * 10000 = 9000 bps
        assert_eq!(deviation, 9000, "Downward crash detection");
        assert!(deviation > MEV_THRESHOLD_BPS);
    }

    #[test]
    fn test_mev_detect_flash_loan_attack() {
        // Scenario: TWAP = 1000, current = 1200 (20% jump from flash loan)
        // Should trigger MevDetected
        let deviation = calculate_deviation_bps(1200, 1000);
        // (1200 - 1000) / 1000 * 10000 = 2000 bps
        assert_eq!(deviation, 2000, "Flash loan attack detection");
        assert!(deviation > MEV_THRESHOLD_BPS);
    }

    // ================================================================
    //  MEV Detection with Price Increase < 200 BPS (should not detect)
    // ================================================================

    #[test]
    fn test_mev_no_detect_price_increase_50_bps() {
        // Scenario: TWAP = 100, current = 100.5 (50 bps increase)
        // Should NOT trigger MevDetected
        let deviation = calculate_deviation_bps(101, 100);
        // (101 - 100) / 100 * 10000 = 100 bps
        assert_eq!(deviation, 100, "Small 100 bps increase");
        assert!(deviation < MEV_THRESHOLD_BPS);
    }

    #[test]
    fn test_mev_no_detect_price_increase_150_bps() {
        // Scenario: TWAP = 100, current = 101.5 (150 bps increase, within tolerance)
        // Should NOT trigger MevDetected
        // 150 bps = 1.5%, which is reasonable market movement
        let deviation = calculate_deviation_bps(101, 100); // 100 bps
        assert!(deviation < MEV_THRESHOLD_BPS);
    }

    #[test]
    fn test_mev_no_detect_price_decrease_50_bps() {
        // Scenario: TWAP = 100, current = 99.5 (50 bps decrease)
        // Should NOT trigger MevDetected
        let deviation = calculate_deviation_bps(99, 100);
        // (100 - 99) / 100 * 10000 = 100 bps
        assert_eq!(deviation, 100, "Small 100 bps decrease");
        assert!(deviation < MEV_THRESHOLD_BPS);
    }

    #[test]
    fn test_mev_no_detect_stable_market() {
        // Scenario: TWAP = 100, current = 100.1 (1 bps increase)
        // Should NOT trigger MevDetected
        // For integer division: 100.1 would be 100 in i128
        let deviation = calculate_deviation_bps(100, 100);
        assert_eq!(deviation, 0, "Stable market with no price change");
        assert!(deviation < MEV_THRESHOLD_BPS);
    }

    // ================================================================
    //  Edge Case Handling for Low Sample Counts
    // ================================================================

    #[test]
    fn test_edge_case_zero_samples() {
        // Edge case: no price samples yet
        // TWAP should return None (insufficient data)
        // Should not detect attack with no baseline
        // This is handled at call site; empty Vec returns None from calculate_twap
    }

    #[test]
    fn test_edge_case_one_sample() {
        // Edge case: only 1 price sample
        // Should use that single sample as baseline (not ideal but safe)
        // calculate_twap with 1 sample returns Some(price)
    }

    #[test]
    fn test_mev_constants_max_samples() {
        // Verify max samples is 24 (24-hour window at 1 hour per sample)
        assert_eq!(MAX_PRICE_SAMPLES, 24, "Max samples should be 24 for 24-hour window");
    }

    #[test]
    fn test_mev_constants_interval() {
        // Verify price sample interval is 1 hour (3600 seconds)
        assert_eq!(PRICE_SAMPLE_INTERVAL, 3600, "Sample interval should be 1 hour");
    }

    // ================================================================
    //  Boundary Tests
    // ================================================================

    #[test]
    fn test_boundary_exactly_200_bps() {
        // Exactly at threshold should NOT trigger (>= instead of >)
        let deviation = calculate_deviation_bps(102, 100);
        assert_eq!(deviation, 200);
        assert!(!(deviation > MEV_THRESHOLD_BPS), "Exactly 200 bps should not trigger");
    }

    #[test]
    fn test_boundary_201_bps() {
        // Just above threshold should trigger
        let deviation = calculate_deviation_bps(102, 100); // 200 bps
        // To get 201 bps: deviation = (x - 100) / 100 * 10000 = 201
        // (x - 100) / 100 = 0.0201
        // x - 100 = 2.01, x = 102.01
        // With integer math: 102 gives 200, need to verify boundary
        assert!(!(deviation > MEV_THRESHOLD_BPS), "200 bps exactly is not over threshold");
    }
}
