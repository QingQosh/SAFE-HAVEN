// ============================================================
//  Prediction Markets - Storage Helpers
// ============================================================

use soroban_sdk::{Address, Env, String, Vec};

use crate::constants::MAX_LOCK_DURATION_SECS;
use crate::prediction_market_types::{
    Bet, MarketConfig, MarketOutcome, OracleResolution, PredictionMarket,
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

/// Returns `true` if the prediction market subsystem has been initialized.
pub fn is_pm_initialized(env: &Env) -> bool {
    let key = soroban_sdk::Symbol::new(env, "pm_init");
    env.storage()
        .persistent()
        .get::<_, bool>(&key)
        .unwrap_or(false)
}

/// Marks the prediction market subsystem as initialized. Called exactly once
/// from `init_prediction_markets`.
pub fn set_pm_initialized(env: &Env) {
    let key = soroban_sdk::Symbol::new(env, "pm_init");
    env.storage().persistent().set(&key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

/// Load the global market configuration.
///
/// The returned `MarketConfig` carries the full configuration state,
/// including the `paused` flag that gates market creation.
pub fn get_market_config(env: &Env) -> Option<MarketConfig> {
    let key = soroban_sdk::Symbol::new(env, "pm_cfg");
    let config: Option<MarketConfig> = env.storage().persistent().get(&key);
    if config.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    config
}

/// Persist the global market configuration, including the `paused` flag.
pub fn set_market_config(env: &Env, config: &MarketConfig) {
    let key = soroban_sdk::Symbol::new(env, "pm_cfg");
    env.storage().persistent().set(&key, config);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

/// Returns `true` if market creation is currently paused.
///
/// Defaults to `false` when no configuration has been stored yet.
pub fn is_market_paused(env: &Env) -> bool {
    get_market_config(env).map(|c| c.paused).unwrap_or(false)
}

/// Set the market pause flag, preserving all other configuration fields.
///
/// Returns `false` if the subsystem has not been initialized (no config
/// stored), otherwise persists the updated config and returns `true`.
pub fn set_market_paused(env: &Env, paused: bool) -> bool {
    match get_market_config(env) {
        Some(mut config) => {
            config.paused = paused;
            set_market_config(env, &config);
            true
        }
        None => false,
    }
}

// ================================================================
//  Market CRUD Helpers
// ================================================================

/// Load the full `PredictionMarket` struct for a market.
///
/// The returned struct contains all market fields: `market_id`, `status`,
/// `close_time`, `oracle`, `outcome_count`, `total_pool`, `bettor_count`,
/// `winning_outcome`, `resolution_data`, `fee_bps`, and the rest of the
/// persisted market state.
pub fn get_market(env: &Env, market_id: u32) -> Option<PredictionMarket> {
    let key = soroban_sdk::Symbol::new(env, &format!("pm_{}", market_id));
    let market: Option<PredictionMarket> = env.storage().persistent().get(&key);
    if market.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    market
}

pub fn set_market(env: &Env, market: &PredictionMarket) {
    let key = soroban_sdk::Symbol::new(env, &format!("pm_{}", market.market_id));
    env.storage().persistent().set(&key, market);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn get_market_readonly(env: &Env, market_id: u32) -> Option<PredictionMarket> {
    let key = soroban_sdk::Symbol::new(env, &format!("pm_{}", market_id));
    env.storage().persistent().get(&key)
}

// ================================================================
//  Market Outcome Helpers
// ================================================================

pub fn get_outcome(env: &Env, market_id: u32, outcome_id: u32) -> Option<MarketOutcome> {
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_{}_{}", market_id, outcome_id));
    let outcome: Option<MarketOutcome> = env.storage().persistent().get(&key);
    if outcome.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    outcome
}

pub fn set_outcome(env: &Env, market_id: u32, outcome: &MarketOutcome) {
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_{}_{}", market_id, outcome.outcome_id));
    env.storage().persistent().set(&key, outcome);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

// ================================================================
//  Bet Helpers
// ================================================================

pub fn get_bet(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) -> Option<Bet> {
    let key = soroban_sdk::Symbol::new(
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
    let key = soroban_sdk::Symbol::new(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bet.bettor),
    );
    env.storage().persistent().set(&key, bet);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn remove_bet(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) {
    let key = soroban_sdk::Symbol::new(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bettor),
    );
    env.storage().persistent().remove(&key);
}

/// Pure read of a bet keyed by (market_id, outcome_id, bettor).
///
/// Performs no TTL extension and no writes, so it is safe to use from
/// read-only query entry points such as `get_user_bet`. The stored `Bet`
/// carries the amount, outcome_id, timestamp and claimed status.
pub fn get_bet_readonly(env: &Env, market_id: u32, outcome_id: u32, bettor: &Address) -> Option<Bet> {
    let key = soroban_sdk::Symbol::new(
        env,
        &format!("pmb_{}_{}_{}", market_id, outcome_id, bettor),
    );
    env.storage().persistent().get(&key)
}

pub fn get_user_market_outcomes(env: &Env, bettor: &Address, market_id: u32) -> Option<Vec<u32>> {
    let key = soroban_sdk::Symbol::new(env, &format!("pmuo_{}_{}", market_id, bettor));
    let outcomes: Option<Vec<u32>> = env.storage().persistent().get(&key);
    if outcomes.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    outcomes
}

pub fn set_user_market_outcomes(env: &Env, bettor: &Address, market_id: u32, outcomes: &Vec<u32>) {
    let key = soroban_sdk::Symbol::new(env, &format!("pmuo_{}_{}", market_id, bettor));
    env.storage().persistent().set(&key, outcomes);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn has_user_bet_on_outcome(env: &Env, bettor: &Address, market_id: u32, outcome_id: u32) -> bool {
    let outcomes = get_user_market_outcomes(env, bettor, market_id).unwrap_or_else(|| Vec::new(env));
    for existing_outcome in outcomes.iter() {
        if existing_outcome == outcome_id {
            return true;
        }
    }
    false
}

pub fn add_user_market_outcome(env: &Env, bettor: &Address, market_id: u32, outcome_id: u32) {
    let mut outcomes = get_user_market_outcomes(env, bettor, market_id).unwrap_or_else(|| Vec::new(env));
    let mut already_present = false;
    for existing_outcome in outcomes.iter() {
        if existing_outcome == outcome_id {
            already_present = true;
            break;
        }
    }

    if !already_present {
        outcomes.push_back(outcome_id);
        set_user_market_outcomes(env, bettor, market_id, &outcomes);
    }
}

// ================================================================
//  Oracle Resolution Helpers
// ================================================================

pub fn get_oracle_submission(
    env: &Env,
    market_id: u32,
    oracle: &Address,
) -> Option<OracleResolution> {
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_sub_{}_{}", market_id, oracle));
    let submission: Option<OracleResolution> = env.storage().persistent().get(&key);
    if submission.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    submission
}

pub fn set_oracle_submission(env: &Env, market_id: u32, oracle: &Address, submission: &OracleResolution) {
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_sub_{}_{}", market_id, oracle));
    env.storage().persistent().set(&key, submission);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

pub fn remove_oracle_submission(env: &Env, market_id: u32, oracle: &Address) {
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_sub_{}_{}", market_id, oracle));
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
    let key = soroban_sdk::Symbol::new(env, &format!("pmo_{}_{}", market_id, outcome_id));
    env.storage().persistent().get(&key)
}

// ================================================================
//  Outcome Names List Storage (Issue #100)
//
//  Stores the ordered Vec<String> of outcome names for a market
//  under a dedicated key "pmon_{market_id}". This ensures names
//  are stored in order, retrievable by index (outcome_id), and
//  the count is maintained as the length of the Vec.
// ================================================================

/// Persist the ordered list of outcome names for a market.
/// Key format: `pmon_{market_id}`
pub fn set_outcome_names_list(env: &Env, market_id: u32, names: &Vec<String>) {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmon_{}", market_id));
    env.storage().persistent().set(&key, names);
    env.storage()
        .persistent()
        .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
}

/// Retrieve the ordered list of outcome names for a market.
/// Returns `None` if no names list has been stored for this market.
pub fn get_outcome_names_list(env: &Env, market_id: u32) -> Option<Vec<String>> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmon_{}", market_id));
    let result: Option<Vec<String>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, MARKET_BUMP_THRESHOLD, MARKET_BUMP_TARGET);
    }
    result
}

/// Retrieve all outcomes for a market as an ordered `Vec<MarketOutcome>`.
///
/// Outcomes are returned in `outcome_id` order (0, 1, 2, 3) by iterating
/// up to `market.outcome_count`. Each outcome is fetched by its individual
/// storage key `pmo_{market_id}_{outcome_id}`.
///
/// Returns an empty Vec if the market has no outcomes stored yet.
pub fn get_market_outcomes(env: &Env, market_id: u32, outcome_count: u32) -> Vec<MarketOutcome> {
    let mut outcomes = Vec::new(env);
    for i in 0..outcome_count {
        if let Some(outcome) = get_outcome_readonly(env, market_id, i) {
            outcomes.push_back(outcome);
        }
    }
    outcomes
}

/// Load all outcomes for a market in ascending `outcome_id` order.
///
/// Convenience wrapper around [`get_market_outcomes`] that derives the
/// outcome count from the stored market struct, so callers (e.g. the
/// `get_prediction_market` query) can include the full outcome list without
/// knowing the count up front. Returns an empty Vec if the market does not
/// exist or has no outcomes stored.
pub fn get_all_market_outcomes(env: &Env, market_id: u32) -> Vec<MarketOutcome> {
    match get_market_readonly(env, market_id) {
        Some(market) => get_market_outcomes(env, market_id, market.outcome_count),
        None => Vec::new(env),
    }
}
