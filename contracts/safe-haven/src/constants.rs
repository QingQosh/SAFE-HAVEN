// ----------------------------------------------------------------
//  Protocol Constants
// ----------------------------------------------------------------

use crate::storage::LEDGER_SECONDS;

/// Maximum deposit amount (in stroops or token base units).
pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;

/// Maximum lock duration in seconds (~5 years).
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;

/// Minimum lock duration: prevent trivial, pointless vaults that waste storage.
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Minimum number of ledgers required for a ledger-based deposit.
pub const MIN_LOCK_LEDGERS: u32 = (MIN_LOCK_DURATION_SECS / LEDGER_SECONDS) as u32;

pub const UPGRADE_TIMELOCK_SECS: u64 = 14 * 24 * 60 * 60;
pub const MIN_UPGRADE_APPROVALS: u32 = 3;

/// Maximum depositors per `batch_emergency_withdraw` call.
///
/// Soroban's per-transaction instruction budget is ~100M instructions.
/// Each iteration performs two persistent-storage removes, one token transfer,
/// and one event publish — roughly 1–2M instructions each.
/// 25 leaves comfortable headroom for the common migration use-case.
pub const MAX_BATCH_SIZE: u32 = 25;

/// Staker penalty split: percentage of penalties allocated to stakers (70% = 7000 basis points)
pub const STAKER_PENALTY_BPS: u32 = 7_000;

/// Fee recipient penalty split: percentage of penalties allocated to fee recipient (30% = 3000 basis points)
pub const FEE_RECIPIENT_PENALTY_BPS: u32 = 3_000;

// ================================================================
// MEV PROTECTION CONSTANTS
// ================================================================

/// Reveal window: how long (in seconds) a depositor has to reveal after commit (30 minutes)
pub const MEV_REVEAL_WINDOW_SECS: u64 = 1_800;

/// Price deviation threshold in basis points (2% = 200 bps) for MEV detection
pub const MEV_PRICE_DEVIATION_THRESHOLD_BPS: u32 = 200;

/// Baseline renewable energy percentage for deposits (default 50%)
pub const RENEWABLE_ENERGY_BASELINE: u32 = 50;

/// Carbon baseline: grams CO2e per unit per second (1 gram per unit-second)
pub const CARBON_BASELINE_PER_UNIT_SECOND: i128 = 1;
