// ============================================================
//  Prediction Markets for SAFE-HAVEN Deposits
//  Market Types and Data Structures
// ============================================================

use soroban_sdk::{contracttype, Address, Vec};

/// Represents a possible outcome in a prediction market.
/// Markets can have 2-4 outcomes (e.g., "High Deposits", "Medium Deposits", "Low Deposits").
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketOutcome {
    /// Unique identifier for this outcome (0-3)
    pub outcome_id: u32,
    /// Human-readable name (e.g., "High Deposits > 100K")
    pub name: soroban_sdk::String,
    /// Total amount bet on this outcome (in contract token units)
    pub total_bet_amount: i128,
    /// Count of bettors for this outcome
    pub bettor_count: u32,
}

/// Status of a prediction market
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketStatus {
    /// Market is open for new bets
    Open = 0,
    /// Market is closed to new bets, awaiting resolution
    Closed = 1,
    /// Oracle has resolved the market with a winner
    Resolved = 2,
    /// Market was cancelled (all bets refunded)
    Cancelled = 3,
}

/// A single bet placed by a user on a market outcome
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bet {
    /// Who placed this bet
    pub bettor: Address,
    /// Which market this bet is for
    pub market_id: u32,
    /// Which outcome was bet on
    pub outcome_id: u32,
    /// Amount wagered (in contract tokens)
    pub amount: i128,
    /// Timestamp when the bet was placed
    pub timestamp: u64,
    /// Whether the bettor has already claimed winnings
    pub claimed: bool,
}

/// Unique identifier for a bet within a market
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BetKey {
    pub market_id: u32,
    pub outcome_id: u32,
    pub bettor: Address,
}

/// A prediction market for SAFE-HAVEN deposit outcomes
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictionMarket {
    /// Unique identifier for this market
    pub market_id: u32,
    /// Type of outcome being predicted (e.g., "aggregate_deposits", "avg_lock_time")
    pub market_type: soroban_sdk::String,
    /// Human-readable description
    pub description: soroban_sdk::String,
    /// Address of the oracle that will resolve this market
    pub oracle: Address,
    /// Unix timestamp when betting closes
    pub close_time: u64,
    /// Unix timestamp when market resolution must complete
    pub resolution_deadline: u64,
    /// Current status of the market
    pub status: MarketStatus,
    /// The winning outcome (set only after resolution)
    pub winning_outcome: Option<u32>,
    /// Total amount bet across all outcomes
    pub total_pool: i128,
    /// Number of distinct bettors
    pub bettor_count: u32,
    /// Number of outcomes in this market
    pub outcome_count: u32,
    /// Timestamp when the market was created
    pub created_at: u64,
    /// Optional resolution data (e.g., aggregate deposit amount)
    pub resolution_data: Option<i128>,
    /// Fee percentage (in basis points) charged when claiming winnings (0-1000)
    pub fee_bps: u32,
}

/// Global market counter and configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketConfig {
    /// Next market ID to be created
    pub next_market_id: u32,
    /// Admin address that can create markets and override resolutions
    pub admin: Address,
    /// Fee recipient (receives fees from claimed winnings)
    pub fee_recipient: Address,
    /// Default fee in basis points (0-1000)
    pub default_fee_bps: u32,
    /// Minimum lock duration for market creation (prevents spam)
    pub min_resolution_time_secs: u64,
    /// Whether new market creation is paused
    pub paused: bool,
}

/// Oracle resolution data submitted by an oracle
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleResolution {
    /// Which market is being resolved
    pub market_id: u32,
    /// The winning outcome ID
    pub winning_outcome: u32,
    /// Additional resolution data
    pub resolution_data: Option<i128>,
    /// Timestamp of submission
    pub submitted_at: u64,
    /// Oracle that submitted this
    pub oracle: Address,
}

/// Stores user's total winnings across all markets (for batch claiming)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRewards {
    pub user: Address,
    pub total_claimable: i128,
    pub markets_won: Vec<u32>,
}

/// Constants for prediction market operations
pub const MIN_MARKET_DURATION_SECS: u64 = 3600; // 1 hour minimum
pub const MAX_MARKET_DURATION_SECS: u64 = 157_788_000; // 5 years max (same as deposit max)
pub const MIN_BET_AMOUNT: i128 = 1; // Minimum 1 unit
pub const MAX_BET_AMOUNT: i128 = 1_000_000_000_000_000; // 10^15 (same as max deposit)
pub const MAX_OUTCOMES_PER_MARKET: u32 = 4;
pub const MIN_OUTCOMES_PER_MARKET: u32 = 2;
pub const MAX_ORACLE_SUBMISSIONS: u32 = 5; // Prevent spam
pub const DEFAULT_FEE_BPS: u32 = 100; // 1% default fee
pub const MAX_FEE_BPS: u32 = 1000; // 10% max fee
