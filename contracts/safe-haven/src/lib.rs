// ============================================================
//  SAFE-HAVEN -- Soroban Smart Contract
//  Stellar Blockchain | Soroban SDK v22
// ============================================================

#![no_std]
// Deny silent integer overflow in all arithmetic operations.
// All arithmetic must use checked, saturating, or wrapping variants.
// This catches potential overflow bugs at compile time rather than silently
// wrapping at runtime in the deterministic Soroban WASM environment.
#![deny(clippy::arithmetic_side_effects)]

mod constants;
mod contract;
mod errors;
mod events;
mod nft;
mod pq;
mod storage;
mod types;

// Prediction Market modules
mod prediction_market;
mod prediction_market_errors;
mod prediction_market_events;
mod prediction_market_storage;
mod prediction_market_types;

pub use constants::{
    EPOCH_SIZE_LEDGERS, MAX_BATCH_SIZE, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS,
    MIN_LOCK_DURATION_SECS, WITHDRAWAL_LIMIT_PER_EPOCH,
};

pub use types::{
    CircuitBreakerActivation, DepositSubscription, DepositType, MultiTokenVaultEntry, Page,
    SubscriptionExecution, SubscriptionStats, TaxLossHarvest, TokenDeposit, STORAGE_VERSION,
    MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER, MAX_TOKENS_PER_DEPOSIT, BatchWithdrawalResult,
    WithdrawalResult,
};
pub use pq::{ML_DSA_PUBLIC_KEY_BYTES, ML_DSA_SIGNATURE_BYTES};

pub use contract::SafeHaven;
pub use contract::SafeHavenClient;

// Prediction Market exports
pub use prediction_market_types::{
    PredictionMarket, MarketOutcome, Bet, MarketStatus, MarketConfig,
};
pub use prediction_market_errors::PredictionMarketError;

#[cfg(test)]
mod test;
