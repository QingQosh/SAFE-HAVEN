// ============================================================
//  Prediction Markets - Events
// ============================================================

use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

/// Emitted when a new prediction market is created
pub fn market_created(
    env: &Env,
    market_id: u32,
    market_type: &soroban_sdk::String,
    oracle: &Address,
    close_time: u64,
    outcome_count: u32,
) {
    let topics = (symbol_short!("mkt_create"), market_id);
    env.events().publish(
        topics,
        (market_type.clone(), oracle.clone(), close_time, outcome_count),
    );
}

/// Emitted when a user places a bet on a market outcome
pub fn bet_placed(
    env: &Env,
    market_id: u32,
    bettor: &Address,
    outcome_id: u32,
    amount: i128,
) {
    let topics = (symbol_short!("bet_place"), market_id, bettor.clone());
    env.events().publish(topics, (outcome_id, amount));
}

/// Emitted when a market is manually closed (stops accepting bets)
pub fn market_closed(env: &Env, market_id: u32, closed_by: &Address) {
    let topics = (symbol_short!("mkt_close"), market_id);
    env.events().publish(topics, closed_by.clone());
}

/// Emitted when an oracle submits a resolution for a market
pub fn market_resolved(
    env: &Env,
    market_id: u32,
    winning_outcome: u32,
    oracle: &Address,
    total_pool: i128,
) {
    let topics = (symbol_short!("mkt_resolve"), market_id);
    env.events().publish(topics, (winning_outcome, oracle.clone(), total_pool));
}

/// Emitted when a bettor claims their winnings
pub fn winnings_claimed(
    env: &Env,
    market_id: u32,
    bettor: &Address,
    amount: i128,
    fee: i128,
) {
    let topics = (symbol_short!("claim_win"), market_id, bettor.clone());
    env.events().publish(topics, (amount, fee));
}

/// Emitted when a market is cancelled (all bets refunded)
pub fn market_cancelled(env: &Env, market_id: u32, cancelled_by: &Address, total_refunded: i128) {
    let topics = (symbol_short!("mkt_cancel"), market_id);
    env.events().publish(topics, (cancelled_by.clone(), total_refunded));
}

/// Emitted when a bettor's bets are refunded due to market cancellation
pub fn bets_refunded(env: &Env, market_id: u32, bettor: &Address, amount: i128) {
    let topics = (symbol_short!("bet_refund"), market_id, bettor.clone());
    env.events().publish(topics, amount);
}

/// Emitted when market creation is paused/unpaused
pub fn market_creation_paused(env: &Env, admin: &Address, paused: bool) {
    let topics = (symbol_short!("mkt_pause"), admin.clone());
    env.events().publish(topics, paused);
}

/// Emitted when the oracle for a market is changed
pub fn oracle_changed(env: &Env, market_id: u32, old_oracle: &Address, new_oracle: &Address) {
    let topics = (symbol_short!("oracle_chg"), market_id);
    env.events().publish(topics, (old_oracle.clone(), new_oracle.clone()));
}
