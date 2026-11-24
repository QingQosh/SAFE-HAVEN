// ============================================================
//  MEV Detection Integration Examples
//
//  Shows how MEV detection can be integrated into the SAFE-HAVEN
//  contract for price validation during deposit/withdrawal flows.
//
//  Note: This is a reference implementation demonstrating the API.
//  Actual integration depends on availability of price oracle.
// ============================================================

use soroban_sdk::{Address, Env};

use crate::{errors::VaultError, mev_detection};

/// Example: Validate a deposit token's current price against MEV detection.
/// This would be called by a price oracle callback or before processing
/// a large token transfer.
///
/// In a production system, this function would:
/// 1. Record the hourly price sample for the token
/// 2. Calculate TWAP from the last 24 hours
/// 3. Check if current price deviates > 200 bps from TWAP
/// 4. Return an error if MEV is detected
///
/// # Arguments
/// * `env` - Soroban environment
/// * `token` - Token address being validated
/// * `current_price` - Current price (in contract's precision)
///
/// # Returns
/// * `Ok(())` if price is within normal range
/// * `Err(MevDetected)` if price deviation > 200 bps
/// * `Err(SamplingTooFrequent)` if hourly sample interval not met
/// * `Err(InvalidAmount)` if price is invalid
pub fn validate_price_for_deposit(
    env: &Env,
    token: &Address,
    current_price: i128,
) -> Result<(), VaultError> {
    // Step 1: Record the hourly price sample
    // This call will fail with SamplingTooFrequent if called twice within 1 hour
    mev_detection::record_price_sample(env, token, current_price)?;

    // Step 2: Detect MEV attack
    // Returns Err(MevDetected) if price deviates > 200 bps from TWAP
    mev_detection::detect_mev(env, token, current_price)?;

    Ok(())
}

/// Query helper: Get the current TWAP for a token without validation.
/// Used by monitoring/analytics systems.
///
/// # Returns
/// * `Some(twap)` if 1+ samples exist
/// * `None` if no samples have been recorded
pub fn get_token_twap(env: &Env, token: &Address) -> Option<i128> {
    mev_detection::get_twap_readonly(env, token)
}

/// Query helper: Get the current price sample count for a token.
///
/// # Returns
/// The number of hourly price samples recorded (0-24)
pub fn get_token_sample_count(env: &Env, token: &Address) -> u32 {
    mev_detection::get_sample_count(env, token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_integration_constants_available() {
        // Ensure constants are available for integration
        assert_eq!(mev_detection::MAX_PRICE_SAMPLES, 24);
        assert_eq!(mev_detection::PRICE_SAMPLE_INTERVAL, 3600);
        assert_eq!(mev_detection::MEV_THRESHOLD_BPS, 200);
    }
}
