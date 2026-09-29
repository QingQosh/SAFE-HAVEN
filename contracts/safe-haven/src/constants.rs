// ----------------------------------------------------------------
//  Protocol Constants
// ----------------------------------------------------------------

/// Maximum deposit amount (in stroops or token base units).
pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;

/// Maximum lock duration in seconds (~5 years).
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;

/// Minimum lock duration: prevent trivial, pointless vaults that waste storage.
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Maximum depositors per `batch_emergency_withdraw` call.
///
/// Soroban's per-transaction instruction budget is ~100M instructions.
/// Each iteration performs two persistent-storage removes, one token transfer,
/// and one event publish — roughly 1–2M instructions each.
/// 25 leaves comfortable headroom for the common migration use-case.
pub const MAX_BATCH_SIZE: u32 = 25;

// ----------------------------------------------------------------
//  Loyalty Program Constants
// ----------------------------------------------------------------

/// Minimum completed deposits to reach Silver tier.
pub const LOYALTY_SILVER_DEPOSITS: u32 = 3;
/// Minimum completed deposits to reach Gold tier.
pub const LOYALTY_GOLD_DEPOSITS: u32 = 10;
/// Minimum completed deposits to reach Platinum tier.
pub const LOYALTY_PLATINUM_DEPOSITS: u32 = 25;

/// Minimum cumulative volume (token base units) to reach Silver tier.
pub const LOYALTY_SILVER_VOLUME: i128 = 10_000;
/// Minimum cumulative volume to reach Gold tier.
pub const LOYALTY_GOLD_VOLUME: i128 = 100_000;
/// Minimum cumulative volume to reach Platinum tier.
pub const LOYALTY_PLATINUM_VOLUME: i128 = 1_000_000;

/// Fee discount (in basis points) applied to `penalty_bps` for Silver users.
pub const LOYALTY_SILVER_DISCOUNT_BPS: u32 = 500;   // 5%
/// Fee discount for Gold users.
pub const LOYALTY_GOLD_DISCOUNT_BPS: u32 = 1_000;   // 10%
/// Fee discount for Platinum users.
pub const LOYALTY_PLATINUM_DISCOUNT_BPS: u32 = 2_000; // 20%

/// Bonus interest (in basis points) credited as informational metadata for Silver.
pub const LOYALTY_SILVER_BONUS_BPS: u32 = 25;   // 0.25%
/// Bonus interest for Gold.
pub const LOYALTY_GOLD_BONUS_BPS: u32 = 75;     // 0.75%
/// Bonus interest for Platinum.
pub const LOYALTY_PLATINUM_BONUS_BPS: u32 = 150; // 1.50%
