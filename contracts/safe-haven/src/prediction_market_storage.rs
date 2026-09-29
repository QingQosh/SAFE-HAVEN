// ============================================================
//  Prediction Markets - Storage Helpers
// ============================================================

use soroban_sdk::{Address, Env, Vec};

use crate::prediction_market_types::{
    PredictionMarket, MarketOutcome, Bet, BetKey, MarketConfig, OracleResolution,
    MAX_LOCK_DURATION_SECS,
};

// Storage key types for prediction markets
#[derive(Clone)]
pub enum PredictionMarketKey {
    /// Global market configuration
    MarketConfig,
    /// Individual market by ID
    Market(u32),
    /// Outcome data for a market
    MarketOutcome(u32, u32), // (market_id, outcome_id)
    /// Individual bet: (market_id, outcome_id, bettor)
    Bet(u32, u32, Address),
    /// All bets for a user (used for enumeration)
    UserBets(Address),
    /// All markets (global list)
    AllMarkets,
    /// Bets by outcome (for resolution calculation)
    OutcomeBets(u32, u32), // (market_id, outcome_id)
    /// Oracle resolution submissions for a market
    OracleSubmission(u32, Address), // (market_id, oracle)
    /// Track which outcomes a user has bet on in a market
    UserMarketOutcomes(Address, u32), // (bettor, market_id)
}

// TTL settings for prediction market storage
pub const MARKET_BUMP_TARGET: u32 = ((MAX_LOCK_DURATION_SECS + 5 - 1) / 5) as u32;
pub const MARKET_BUMP_THRESHOLD: u32 = MARKET_BUMP_TARGET / 2;

// ================================================================
//  Market Config Helpers
// ================================================================

pub fn get_market_config(env: &Env) -> Option<MarketConfig> {
    let key = soroban_sdk::Symbol::new(env, "pm_cfg");
    env.storage().persistent().get(&key)
}

pub fn set_market_config(env: &Env, config: &MarketConfig) {
    let key = soroban_sdk::Symbol::new(env, "pm_cfg");
    env.storage().persistent().set(&key, config);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

// ================================================================
//  Market CRUD Helpers
// ================================================================

pub fn get_market(env: &Env, market_id: u32) -> Option<PredictionMarket> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pm_{}", market_id));
    let market: Option<PredictionMarket> = env.storage().persistent().get(&key);
    if market.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    market
}

pub fn set_market(env: &Env, market: &PredictionMarket) {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pm_{}", market.market_id));
    env.storage().persistent().set(&key, market);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn get_market_readonly(env: &Env, market_id: u32) -> Option<PredictionMarket> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pm_{}", market_id));
    env.storage().persistent().get(&key)
}

// ================================================================
//  Market Outcome Helpers
// ================================================================

pub fn get_outcome(env: &Env, market_id: u32, outcome_id: u32) -> Option<MarketOutcome> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_{}_{}", market_id, outcome_id));
    let outcome: Option<MarketOutcome> = env.storage().persistent().get(&key);
    if outcome.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    outcome
}

pub fn set_outcome(env: &Env, market_id: u32, outcome: &MarketOutcome) {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_{}_{}", market_id, outcome.outcome_id));
    env.storage().persistent().set(&key, outcome);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

// ================================================================
//  Bet Helpers
// ================================================================

pub fn get_bet(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) -> Option<Bet> {
    let key = soroban_sdk::Symbol::from_str(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bettor),
    );
    let bet: Option<Bet> = env.storage().persistent().get(&key);
    if bet.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    bet
}

pub fn set_bet(env: &Env, market_id: u32, outcome_id: u32, bet: &Bet) {
    let key = soroban_sdk::Symbol::from_str(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bet.bettor),
    );
    env.storage().persistent().set(&key, bet);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn remove_bet(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) {
    let key = soroban_sdk::Symbol::from_str(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bettor),
    );
    env.storage().persistent().remove(&key);
}

pub fn get_bet_readonly(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) -> Option<Bet> {
    let key = soroban_sdk::Symbol::from_str(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bettor),
    );
    env.storage().persistent().get(&key)
}

// ================================================================
//  Oracle Resolution Helpers
// ================================================================

pub fn get_oracle_submission(
    env: &Env,
    market_id: u32,
    oracle: &Address,
) -> Option<OracleResolution> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_sub_{}_{}", market_id, oracle));
    let submission: Option<OracleResolution> = env.storage().persistent().get(&key);
    if submission.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    submission
}

pub fn set_oracle_submission(env: &Env, market_id: u32, oracle: &Address, submission: &OracleResolution) {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_sub_{}_{}", market_id, oracle));
    env.storage().persistent().set(&key, submission);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn remove_oracle_submission(env: &Env, market_id: u32, oracle: &Address) {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_sub_{}_{}", market_id, oracle));
    env.storage().persistent().remove(&key);
}

// ================================================================
//  Market State Transition Helpers
// ================================================================

/// Get the total amount that would be won by bettors on a given outcome
pub fn calculate_outcome_winnings(
    env: &Env,
    market_id: u32,
    outcome_id: u32,
    winning_pool: i128,
    total_pool: i128,
) -> i128 {
    // Proportional distribution: (bet_amount / winning_bets) × (total_pool - fee)
    // This is calculated per-bettor in claim_winnings
    winning_pool
}

/// Get outcome details in readonly mode (for queries)
pub fn get_outcome_readonly(env: &Env, market_id: u32, outcome_id: u32) -> Option<MarketOutcome> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_{}_{}", market_id, outcome_id));
    env.storage().persistent().get(&key)
}
