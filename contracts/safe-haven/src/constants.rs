// ----------------------------------------------------------------
//  Protocol Constants
// ----------------------------------------------------------------

use crate::storage::LEDGER_SECONDS;

/// Maximum deposit amount (in stroops or token base units).
pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;

/// Maximum lock duration in seconds: exactly 5 years (5 × 365.25 × 24 × 3600).
///
/// This cap applies to all deposit types — `deposit`, `deposit_for`,
/// `deposit_with_compound_interest`, and `deposit_by_ledger`.  It prevents
/// lock durations that would overflow ledger-TTL arithmetic and keeps the
/// maximum storage-bump window manageable.
///
/// Computed as: 5 * 365.25 * 24 * 3600 = 157,788,000 seconds.
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;

/// Minimum lock duration: prevent trivial, pointless vaults that waste storage.
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Minimum number of ledgers required for a ledger-based deposit (~60 seconds at 5 s/ledger).
pub const MIN_LOCK_LEDGERS: u32 = (MIN_LOCK_DURATION_SECS / LEDGER_SECONDS) as u32;

pub const EPOCH_SIZE_LEDGERS: u32 = 1_440;
pub const WITHDRAWAL_LIMIT_PER_EPOCH: i128 = 50_000_000;
pub const RENEWABLE_ENERGY_BASELINE: u32 = 50;
/// Carbon baseline: grams CO2e per token base unit per second.
///
/// A deposit of `amount` base units locked for `duration` seconds emits
/// `amount * duration * CARBON_BASELINE_PER_UNIT_SECOND` grams of CO2e
/// (before any renewable-energy offset is applied).
pub const CARBON_BASELINE_PER_UNIT_SECOND: i128 = 1;

/// Theoretical maximum carbon footprint in grams CO2e.
///
/// Computed as `MAX_DEPOSIT_AMOUNT * MAX_LOCK_DURATION_SECS * CARBON_BASELINE_PER_UNIT_SECOND`
/// = 10^15 * 157_788_000 * 1 ≈ 1.57788 × 10^23 grams CO2e.
///
/// This is far below `i128::MAX` (~1.7 × 10^38), so carbon arithmetic cannot
/// overflow for any valid deposit/lock combination.
pub const MAX_CARBON_FOOTPRINT: i128 =
    MAX_DEPOSIT_AMOUNT * (MAX_LOCK_DURATION_SECS as i128) * CARBON_BASELINE_PER_UNIT_SECOND;

/// Maximum number of ledgers allowed for a ledger-based deposit, equivalent to
/// `MAX_LOCK_DURATION_SECS` converted at the nominal 5-second ledger rate.
///
/// This is the ledger-domain counterpart to `MAX_LOCK_DURATION_SECS` and is
/// used by `deposit_by_ledger` to enforce a 5-year upper bound on the lock
/// duration (`ledger_gap <= MAX_LOCK_LEDGERS`).
///
/// Computed as: 157_788_000 / 5 = 31_557_600 ledgers.
pub const MAX_LOCK_LEDGERS: u32 = (MAX_LOCK_DURATION_SECS / LEDGER_SECONDS) as u32;

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

/// Carbon baseline: grams CO2e per unit per second (1 gram per unit-second).
///
/// This is the baseline coefficient used by the deposit carbon footprint
/// formula:
///
/// ```text
/// carbon_footprint = amount × duration_seconds × CARBON_BASELINE_PER_UNIT_SECOND
/// ```
///
/// With the default value of `1`, a deposit of 1000 units locked for 86400
/// seconds (1 day) yields 86,400,000 grams CO2e.  The maximum deposit
/// (`MAX_DEPOSIT_AMOUNT` = 10^15) locked for the maximum duration
/// (`MAX_LOCK_DURATION_SECS` = 157,788,000 s) yields
/// 1.57788 × 10^23 grams CO2e, which fits comfortably within `i128`.
pub const CARBON_BASELINE_PER_UNIT_SECOND: i128 = 1;
