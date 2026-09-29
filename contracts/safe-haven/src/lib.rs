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
mod storage;
mod types;

pub use constants::{
    MAX_BATCH_SIZE, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS, MIN_LOCK_DURATION_SECS,
    LOYALTY_SILVER_DEPOSITS, LOYALTY_GOLD_DEPOSITS, LOYALTY_PLATINUM_DEPOSITS,
    LOYALTY_SILVER_VOLUME, LOYALTY_GOLD_VOLUME, LOYALTY_PLATINUM_VOLUME,
    LOYALTY_SILVER_DISCOUNT_BPS, LOYALTY_GOLD_DISCOUNT_BPS, LOYALTY_PLATINUM_DISCOUNT_BPS,
    LOYALTY_SILVER_BONUS_BPS, LOYALTY_GOLD_BONUS_BPS, LOYALTY_PLATINUM_BONUS_BPS,
};

pub use contract::SafeHaven;
pub use contract::SafeHavenClient;

#[cfg(test)]
mod test;
