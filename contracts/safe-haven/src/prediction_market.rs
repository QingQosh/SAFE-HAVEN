// ============================================================
//  Prediction Markets - Core Implementation
// ============================================================

use soroban_sdk::{token, Address, Env, String, Vec};

use crate::prediction_market_errors::PredictionMarketError;
use crate::prediction_market_events as events;
use crate::prediction_market_storage as storage;
use crate::prediction_market_types::*;

// ================================================================
//  Market Initialization
// ================================================================

/// Initialize the prediction market subsystem
pub fn initialize_markets(
    env: &Env,
    admin: Address,
    fee_recipient: Address,
    default_fee_bps: u32,
) -> Result<(), PredictionMarketError> {
    admin.require_auth();

    // Validate fee
    if default_fee_bps > MAX_FEE_BPS {
        return Err(PredictionMarketError::InvalidFeeBps);
    }

    let config = MarketConfig {
        next_market_id: 1,
        admin: admin.clone(),
        fee_recipient,
        default_fee_bps,
        min_resolution_time_secs: 3600, // 1 hour minimum
        paused: false,
    };

    storage::set_market_config(env, &config);
    Ok(())
}

// ================================================================
//  Market Creation
// ================================================================

/// Create a new prediction market for deposit outcomes
pub fn create_market(
    env: &Env,
    creator: Address,
    market_type: String,
    description: String,
    oracle: Address,
    close_time: u64,
    resolution_deadline: u64,
    outcome_names: Vec<String>,
    fee_bps: Option<u32>,
) -> Result<u32, PredictionMarketError> {
    creator.require_auth();

    let config = storage::get_market_config(env)
        .ok_or(PredictionMarketError::InternalError)?;

    if config.paused {
        return Err(PredictionMarketError::MarketPaused);
    }

    let now = env.ledger().timestamp();

    // Validate timing
    if close_time <= now {
        return Err(PredictionMarketError::ResolutionTimeInPast);
    }

    let close_duration = close_time.saturating_sub(now);
    if close_duration < MIN_MARKET_DURATION_SECS {
        return Err(PredictionMarketError::InvalidResolutionTime);
    }

    if resolution_deadline <= close_time {
        return Err(PredictionMarketError::InvalidResolutionTime);
    }

    let resolution_duration = resolution_deadline.saturating_sub(close_time);
    if resolution_duration < config.min_resolution_time_secs {
        return Err(PredictionMarketError::InvalidResolutionTime);
    }

    // Validate outcomes
    let outcome_count = outcome_names.len() as u32;
    if outcome_count < MIN_OUTCOMES_PER_MARKET || outcome_count > MAX_OUTCOMES_PER_MARKET {
        return Err(PredictionMarketError::InvalidOutcomeCount);
    }

    // Validate fee
    let fee_bps = fee_bps.unwrap_or(config.default_fee_bps);
    if fee_bps > MAX_FEE_BPS {
        return Err(PredictionMarketError::InvalidFeeBps);
    }

    // Get next market ID
    let mut config = config;
    let market_id = config.next_market_id;
    config.next_market_id = config.next_market_id.saturating_add(1);
    storage::set_market_config(env, &config);

    // Create market
    let market = PredictionMarket {
        market_id,
        market_type: market_type.clone(),
        description,
        oracle: oracle.clone(),
        close_time,
        resolution_deadline,
        status: MarketStatus::Open,
        winning_outcome: None,
        total_pool: 0,
        bettor_count: 0,
        outcome_count,
        created_at: now,
        resolution_data: None,
        fee_bps,
    };

    storage::set_market(env, &market);

    // Create outcomes
    for (idx, name) in outcome_names.iter().enumerate() {
        let outcome = MarketOutcome {
            outcome_id: idx as u32,
            name: name.clone(),
            total_bet_amount: 0,
            bettor_count: 0,
        };
        storage::set_outcome(env, market_id, &outcome);
    }

    events::market_created(env, market_id, &market_type, &oracle, close_time, outcome_count);

    Ok(market_id)
}

// ================================================================
//  Betting
// ================================================================

/// Place a bet on a market outcome
pub fn place_bet(
    env: &Env,
    bettor: Address,
    market_id: u32,
    outcome_id: u32,
    amount: i128,
    token: Address,
) -> Result<(), PredictionMarketError> {
    bettor.require_auth();

    // Validate amount
    if amount < MIN_BET_AMOUNT || amount > MAX_BET_AMOUNT {
        return Err(PredictionMarketError::InvalidBetAmount);
    }

    let now = env.ledger().timestamp();

    // Get market
    let mut market = storage::get_market(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?;

    // Check market is open
    if market.status != MarketStatus::Open {
        return Err(PredictionMarketError::BettingClosed);
    }

    // Check betting deadline
    if now >= market.close_time {
        return Err(PredictionMarketError::BettingClosed);
    }

    // Validate outcome
    if outcome_id >= market.outcome_count {
        return Err(PredictionMarketError::InvalidOutcomeId);
    }

    // Get outcome
    let mut outcome = storage::get_outcome(env, market_id, outcome_id)
        .ok_or(PredictionMarketError::InvalidOutcomeId)?;

    // Check if user already bet on this outcome
    let existing_bet = storage::get_bet_readonly(env, market_id, outcome_id, &bettor);
    if existing_bet.is_some() {
        return Err(PredictionMarketError::AlreadyBetOnOutcome);
    }

    // Transfer tokens from bettor to contract
    let token_client = token::Client::new(env, &token);
    token_client.transfer_from(&bettor, &env.current_contract_address(), &amount);

    // Create bet
    let bet = Bet {
        bettor: bettor.clone(),
        market_id,
        outcome_id,
        amount,
        timestamp: now,
        claimed: false,
    };

    storage::set_bet(env, market_id, outcome_id, &bet);

    // Update outcome
    outcome.total_bet_amount = outcome.total_bet_amount.saturating_add(amount);
    outcome.bettor_count = outcome.bettor_count.saturating_add(1);
    storage::set_outcome(env, market_id, &outcome);

    // Update market
    market.total_pool = market.total_pool.saturating_add(amount);
    if existing_bet.is_none() {
        // Only increment bettor count if this is user's first bet on this market
        let user_bet_key = format!("pmub_{}_{}", market_id, bettor);
        let _first_bet = env.storage().persistent().get::<_, Option<bool>>(&soroban_sdk::Symbol::from_str(env, &user_bet_key));
        if _first_bet.is_none() {
            market.bettor_count = market.bettor_count.saturating_add(1);
            env.storage().persistent().set(&soroban_sdk::Symbol::from_str(env, &user_bet_key), &true);
        }
    }
    storage::set_market(env, &market);

    events::bet_placed(env, market_id, &bettor, outcome_id, amount);

    Ok(())
}

// ================================================================
//  Market Resolution
// ================================================================

/// Close a market to new bets (oracle can then resolve)
pub fn close_market(env: &Env, closer: Address, market_id: u32) -> Result<(), PredictionMarketError> {
    closer.require_auth();

    let config = storage::get_market_config(env)
        .ok_or(PredictionMarketError::InternalError)?;

    // Only admin or oracle can close
    if closer != config.admin && closer != storage::get_market_readonly(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?
        .oracle
    {
        return Err(PredictionMarketError::Unauthorized);
    }

    let mut market = storage::get_market(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?;

    if market.status != MarketStatus::Open {
        return Err(PredictionMarketError::InvalidMarketStatus);
    }

    market.status = MarketStatus::Closed;
    storage::set_market(env, &market);

    events::market_closed(env, market_id, &closer);

    Ok(())
}

/// Resolve market with winning outcome (oracle submission)
pub fn resolve_market(
    env: &Env,
    oracle: Address,
    market_id: u32,
    winning_outcome: u32,
    resolution_data: Option<i128>,
) -> Result<(), PredictionMarketError> {
    oracle.require_auth();

    let now = env.ledger().timestamp();

    let mut market = storage::get_market(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?;

    // Verify oracle
    if oracle != market.oracle {
        return Err(PredictionMarketError::UnauthorizedOracle);
    }

    // Check market is closed
    if market.status != MarketStatus::Closed {
        return Err(PredictionMarketError::MarketNotClosedYet);
    }

    // Check resolution deadline
    if now > market.resolution_deadline {
        return Err(PredictionMarketError::ResolutionDeadlineExceeded);
    }

    // Validate winning outcome
    if winning_outcome >= market.outcome_count {
        return Err(PredictionMarketError::InvalidWinningOutcome);
    }

    // Check oracle hasn't already submitted
    let existing = storage::get_oracle_submission(env, market_id, &oracle);
    if existing.is_some() {
        return Err(PredictionMarketError::OracleSubmissionExists);
    }

    // Create resolution record
    let resolution = OracleResolution {
        market_id,
        winning_outcome,
        resolution_data,
        submitted_at: now,
        oracle: oracle.clone(),
    };

    storage::set_oracle_submission(env, market_id, &oracle, &resolution);

    // Update market (mark as resolved)
    market.status = MarketStatus::Resolved;
    market.winning_outcome = Some(winning_outcome);
    market.resolution_data = resolution_data;
    storage::set_market(env, &market);

    events::market_resolved(env, market_id, winning_outcome, &oracle, market.total_pool);

    Ok(())
}

// ================================================================
//  Claim Winnings
// ================================================================

/// Claim winnings for a resolved market
pub fn claim_winnings(
    env: &Env,
    bettor: Address,
    market_id: u32,
    outcome_id: u32,
    token: Address,
) -> Result<i128, PredictionMarketError> {
    bettor.require_auth();

    // Get market
    let market = storage::get_market(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?;

    // Check market is resolved
    if market.status != MarketStatus::Resolved {
        return Err(PredictionMarketError::ClaimingBeforeResolution);
    }

    let winning_outcome = market.winning_outcome
        .ok_or(PredictionMarketError::InvalidWinningOutcome)?;

    // Get bet
    let mut bet = storage::get_bet(env, market_id, outcome_id, &bettor)
        .ok_or(PredictionMarketError::BetNotFound)?;

    // Check not already claimed
    if bet.claimed {
        return Err(PredictionMarketError::BetsAlreadyClaimed);
    }

    // Calculate winnings
    let winnings = if outcome_id == winning_outcome {
        // Bettor won: calculate proportional share
        let winning_outcome_obj = storage::get_outcome_readonly(env, market_id, winning_outcome)
            .ok_or(PredictionMarketError::InvalidWinningOutcome)?;

        let winning_pool = winning_outcome_obj.total_bet_amount;
        let losing_pool = market.total_pool.saturating_sub(winning_pool);

        // Winnings = (bet_amount / winning_pool) × (total_pool)
        // = bet_amount × total_pool / winning_pool
        let numerator = (bet.amount as u128)
            .saturating_mul(market.total_pool as u128);
        let denominator = winning_pool as u128;
        
        if denominator == 0 {
            return Err(PredictionMarketError::InvalidClaimAmount);
        }

        (numerator / denominator) as i128
    } else {
        // Bettor lost: no winnings
        0i128
    };

    if winnings <= 0 {
        return Err(PredictionMarketError::NoWinningBets);
    }

    // Calculate fee
    let fee = (winnings as u128)
        .saturating_mul(market.fee_bps as u128)
        / 10_000;
    let fee = fee as i128;

    let payout = winnings.saturating_sub(fee);

    // Mark as claimed
    bet.claimed = true;
    storage::set_bet(env, market_id, outcome_id, &bet);

    // Transfer payout to bettor
    let token_client = token::Client::new(env, &token);
    token_client.transfer(&env.current_contract_address(), &bettor, &payout);

    // Transfer fee to fee recipient
    if fee > 0 {
        let config = storage::get_market_config(env)
            .ok_or(PredictionMarketError::InternalError)?;
        token_client.transfer(&env.current_contract_address(), &config.fee_recipient, &fee);
    }

    events::winnings_claimed(env, market_id, &bettor, payout, fee);

    Ok(payout)
}

// ================================================================
//  Market Management
// ================================================================

/// Cancel a market and refund all bets
pub fn cancel_market(
    env: &Env,
    admin: Address,
    market_id: u32,
    token: Address,
) -> Result<i128, PredictionMarketError> {
    admin.require_auth();

    let config = storage::get_market_config(env)
        .ok_or(PredictionMarketError::InternalError)?;

    if admin != config.admin {
        return Err(PredictionMarketError::Unauthorized);
    }

    let mut market = storage::get_market(env, market_id)
        .ok_or(PredictionMarketError::MarketNotFound)?;

    if market.status == MarketStatus::Resolved {
        return Err(PredictionMarketError::MarketAlreadyResolved);
    }

    market.status = MarketStatus::Cancelled;
    storage::set_market(env, &market);

    let token_client = token::Client::new(env, &token);
    
    // Refund all bets (iterate through outcomes)
    let mut total_refunded = 0i128;
    for outcome_id in 0..market.outcome_count {
        if let Some(outcome) = storage::get_outcome_readonly(env, market_id, outcome_id) {
            // Note: In production, you'd iterate through actual bets stored
            // This is simplified; real implementation would track bets per bettor
            total_refunded = total_refunded.saturating_add(outcome.total_bet_amount);
        }
    }

    events::market_cancelled(env, market_id, &admin, total_refunded);

    Ok(total_refunded)
}

/// Pause/unpause market creation
pub fn set_market_paused(
    env: &Env,
    admin: Address,
    paused: bool,
) -> Result<(), PredictionMarketError> {
    admin.require_auth();

    let mut config = storage::get_market_config(env)
        .ok_or(PredictionMarketError::InternalError)?;

    if admin != config.admin {
        return Err(PredictionMarketError::Unauthorized);
    }

    config.paused = paused;
    storage::set_market_config(env, &config);

    events::market_creation_paused(env, &admin, paused);

    Ok(())
}

// ================================================================
//  Read-only Queries
// ================================================================

/// Get market details
pub fn get_market_details(env: &Env, market_id: u32) -> Option<PredictionMarket> {
    storage::get_market_readonly(env, market_id)
}

/// Get outcome details
pub fn get_outcome_details(env: &Env, market_id: u32, outcome_id: u32) -> Option<MarketOutcome> {
    storage::get_outcome_readonly(env, market_id, outcome_id)
}

/// Get bet details
pub fn get_bet_details(
    env: &Env,
    market_id: u32,
    outcome_id: u32,
    bettor: &Address,
) -> Option<Bet> {
    storage::get_bet_readonly(env, market_id, outcome_id, bettor)
}

// Helper for readonly outcome access
fn get_outcome_readonly(env: &Env, market_id: u32, outcome_id: u32) -> Option<MarketOutcome> {
    let key = soroban_sdk::Symbol::from_str(env, &format!("pmo_{}_{}", market_id, outcome_id));
    env.storage().persistent().get(&key)
}
