use soroban_sdk::{contracttype, Address, String};

pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultKey {
    Deposit(Address, u32),
    DepositByLedger(Address, u32),
    DepositCounter(Address),
    /// Stores a `Vec<u32>` of active deposit IDs for a depositor (both timestamp- and
    /// ledger-based). Maintained alongside the counter so `get_deposit_ids` is O(1).
    ActiveDepositIds(Address),
    Admin,
    PendingAdmin,
    Initialized,
    DepositorList,
    /// Boolean existence flag per depositor — O(1) duplicate check in `add_depositor`.
    DepositorFlag(Address),
    FeeRecipient,
    MaxDeposit,
    MaxLockSecs,
    Paused,
    /// Tracks cumulative deposit count for a depositor (used for tier calculation).
    LoyaltyDepositCount(Address),
    /// Tracks cumulative deposit volume (in token base units) for a depositor.
    LoyaltyTotalVolume(Address),
    /// Caches the current tier so upgrades can be detected and events emitted.
    LoyaltyCurrentTier(Address),
}

// ----------------------------------------------------------------
//  Loyalty Program Types
// ----------------------------------------------------------------

/// Four-tier loyalty system. Tiers are determined by total completed deposits
/// and cumulative deposit volume. Once a tier is reached it is never revoked
/// (no demotion on inactivity, per spec).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Copy)]
#[repr(u32)]
pub enum LoyaltyTier {
    /// New user — no history required.
    Bronze = 0,
    /// 3+ completed deposits OR 10,000+ total volume.
    Silver = 1,
    /// 10+ completed deposits OR 100,000+ total volume.
    Gold = 2,
    /// 25+ completed deposits OR 1,000,000+ total volume.
    Platinum = 3,
}

/// Returned by `get_loyalty_tier()` — current tier plus progress counters
/// so UIs can show how close a user is to the next tier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoyaltyInfo {
    pub tier: LoyaltyTier,
    pub completed_deposits: u32,
    pub total_volume: i128,
    /// Deposits still needed to reach the next tier (0 when already Platinum).
    pub deposits_to_next_tier: u32,
    /// Volume still needed to reach the next tier (0 when already Platinum).
    pub volume_to_next_tier: i128,
}

/// Returned by `tier_benefits()` — human-readable description of what each
/// tier provides. `fee_discount_bps` is the reduction applied to `penalty_bps`
/// at deposit time (capped so it never goes negative).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierBenefits {
    pub tier: LoyaltyTier,
    /// Reduction to the early-exit penalty, expressed in basis points.
    /// E.g. 500 means the user's penalty_bps is reduced by 5%.
    pub fee_discount_bps: u32,
    /// Bonus interest rate in basis points credited on matured withdrawals.
    /// Applied to the withdrawn amount as an on-chain annotation (informational
    /// in this version; the contract emits it in the Withdraw event data).
    pub bonus_interest_bps: u32,
    pub description: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub depositor: Address,
    pub penalty_bps: u32,
}
