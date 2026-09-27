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
mod storage;
mod types;
mod upgrade;
mod upgrade_validation;
mod upgrade_rollback;

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
    MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER, MAX_TOKENS_PER_DEPOSIT,
};

pub use upgrade::{
    init_migration, migrate_depositor_deposits, rollback_migration,
    validate_deposits_for_migration, verify_migration_integrity,
    MigrationState, ValidationResult, UpgradeMetadata,
};

pub use upgrade_validation::{
    validate_all_deposits, validate_depositor_deposits, validate_single_deposit,
    verify_consistency, can_migrate_deposit,
    ValidationReport, DepositValidationDetail, ConsistencyCheckResult,
};

pub use upgrade_rollback::{
    create_deposit_snapshot, create_depositor_snapshot,
    rollback_all_deposits, rollback_depositor,
    verify_rollback_integrity, can_safely_rollback,
    conservative_rollback, full_rollback, targeted_rollback,
    DepositSnapshot, RollbackState, RollbackStatus,
};

pub use contract::SafeHaven;
pub use contract::SafeHavenClient;

// Prediction Market exports
pub use prediction_market_types::{
    PredictionMarket, MarketOutcome, Bet, MarketStatus, MarketConfig,
};
pub use prediction_market_errors::PredictionMarketError;

#[cfg(test)]
mod test;

#[cfg(test)]
mod upgrade_test;
