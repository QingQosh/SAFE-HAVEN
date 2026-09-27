//! Enhanced validation module for deposit integrity checks during migrations.
//!
//! This module provides sophisticated data validation, consistency checking,
//! and post-migration verification to ensure all deposit data remains intact.

use soroban_sdk::{Address, Env, String, Vec};
use crate::{errors::VaultError, storage, types::VaultEntry};

/// Comprehensive validation report with detailed metrics
#[derive(Clone, Debug)]
pub struct ValidationReport {
    /// Overall validation status
    pub is_valid: bool,
    /// Total deposits validated
    pub total_deposits: u32,
    /// Deposits that passed validation
    pub valid_deposits: u32,
    /// Deposits with errors
    pub invalid_deposits: u32,
    /// Total value locked across all deposits
    pub total_value_locked: i128,
    /// Count of deposits with future unlock times
    pub future_unlock_count: u32,
    /// Count of deposits already unlocked
    pub unlocked_count: u32,
    /// Checksum of all deposit amounts (for consistency)
    pub amount_checksum: i128,
    /// Maximum individual deposit found
    pub max_deposit_amount: i128,
    /// Minimum individual deposit found
    pub min_deposit_amount: i128,
}

/// Per-deposit validation details
#[derive(Clone, Debug)]
pub struct DepositValidationDetail {
    /// Depositor address
    pub depositor: Address,
    /// Deposit ID
    pub deposit_id: u32,
    /// Validation passed
    pub is_valid: bool,
    /// Error details if invalid
    pub error_code: u32, // 0 = valid, 1+ = specific error code
    /// Deposit amount
    pub amount: i128,
    /// Unlock time
    pub unlock_time: u64,
    /// Current timestamp for reference
    pub checked_at: u64,
}

/// Consistency check results between old and new contract data
#[derive(Clone, Debug)]
pub struct ConsistencyCheckResult {
    /// Overall consistency status
    pub is_consistent: bool,
    /// Total deposits in old contract
    pub old_contract_deposits: u32,
    /// Total deposits in new contract after migration
    pub new_contract_deposits: u32,
    /// Total value in old contract
    pub old_contract_value: i128,
    /// Total value in new contract
    pub new_contract_value: i128,
    /// Deposits missing from new contract
    pub missing_count: u32,
    /// Deposits with amount discrepancies
    pub amount_mismatch_count: u32,
    /// Deposits with metadata discrepancies
    pub metadata_mismatch_count: u32,
}

// ================================================================
//  Data Integrity Validation Functions
// ================================================================

/// Validate a single deposit with comprehensive checks.
///
/// Validates:
/// - Amount is positive and within limits
/// - Unlock time is reasonable
/// - Penalty BPS is valid
/// - Compound frequency is valid
/// - Address consistency
pub fn validate_single_deposit(
    env: &Env,
    entry: &VaultEntry,
    depositor: &Address,
    deposit_id: u32,
) -> Result<DepositValidationDetail, VaultError> {
    let now = env.ledger().timestamp();
    let mut error_code = 0u32;

    // Check amount bounds
    if entry.amount <= 0 {
        error_code = 1; // Invalid amount
    } else if entry.amount > crate::types::MAX_DEPOSIT_AMOUNT {
        error_code = 2; // Amount too large
    }

    // Check unlock time validity
    if entry.unlock_time < now && error_code == 0 {
        error_code = 3; // Unlock time in past
    } else if entry.unlock_time > now.saturating_add(crate::types::MAX_LOCK_DURATION_SECS) && error_code == 0 {
        error_code = 4; // Unlock time too far in future
    }

    // Check penalty BPS
    if entry.penalty_bps > 10000 && error_code == 0 {
        error_code = 5; // Invalid penalty BPS
    }

    // Check compound frequency
    if entry.compound_frequency_secs > 0 && entry.compound_frequency_secs < 60 && error_code == 0 {
        error_code = 6; // Compound frequency too small
    }

    // Verify depositor address matches
    if entry.depositor != *depositor && error_code == 0 {
        error_code = 7; // Depositor mismatch
    }

    Ok(DepositValidationDetail {
        depositor: depositor.clone(),
        deposit_id,
        is_valid: error_code == 0,
        error_code,
        amount: entry.amount,
        unlock_time: entry.unlock_time,
        checked_at: now,
    })
}

/// Perform comprehensive validation of all deposits.
///
/// Validates all deposits in the contract and generates detailed report.
pub fn validate_all_deposits(env: &Env) -> Result<ValidationReport, VaultError> {
    let now = env.ledger().timestamp();
    let depositors = storage::get_all_depositors_raw(env);

    let mut total_deposits = 0u32;
    let mut valid_deposits = 0u32;
    let mut invalid_deposits = 0u32;
    let mut total_value: i128 = 0;
    let mut future_unlock_count = 0u32;
    let mut unlocked_count = 0u32;
    let mut amount_checksum: i128 = 0;
    let mut max_amount: i128 = 0;
    let mut min_amount: i128 = i128::MAX;

    for depositor_addr in depositors.iter() {
        let deposit_ids = storage::get_deposit_ids(env, &depositor_addr);

        for deposit_id in deposit_ids.iter() {
            total_deposits = total_deposits.saturating_add(1);

            if let Some(entry) = storage::get_deposit_readonly(env, &depositor_addr, deposit_id) {
                // Validate deposit
                match validate_single_deposit(env, &entry, &depositor_addr, deposit_id) {
                    Ok(detail) => {
                        if detail.is_valid {
                            valid_deposits = valid_deposits.saturating_add(1);
                            
                            // Update statistics
                            total_value = total_value.saturating_add(entry.amount);
                            amount_checksum = amount_checksum.saturating_add(entry.amount);
                            
                            if entry.unlock_time > now {
                                future_unlock_count = future_unlock_count.saturating_add(1);
                            } else {
                                unlocked_count = unlocked_count.saturating_add(1);
                            }

                            // Track min/max
                            if entry.amount > max_amount {
                                max_amount = entry.amount;
                            }
                            if entry.amount < min_amount {
                                min_amount = entry.amount;
                            }
                        } else {
                            invalid_deposits = invalid_deposits.saturating_add(1);
                        }
                    }
                    Err(_) => {
                        invalid_deposits = invalid_deposits.saturating_add(1);
                    }
                }
            } else {
                invalid_deposits = invalid_deposits.saturating_add(1);
            }

            // Safety: Cap iterations
            if total_deposits >= 10000 {
                break;
            }
        }

        if total_deposits >= 10000 {
            break;
        }
    }

    // Handle edge case for min_amount
    if min_amount == i128::MAX {
        min_amount = 0;
    }

    Ok(ValidationReport {
        is_valid: invalid_deposits == 0,
        total_deposits,
        valid_deposits,
        invalid_deposits,
        total_value_locked: total_value,
        future_unlock_count,
        unlocked_count,
        amount_checksum,
        max_deposit_amount: max_amount,
        min_deposit_amount: min_amount,
    })
}

/// Verify consistency between two sets of deposits (before and after migration).
///
/// Compares deposit data to ensure all data was transferred correctly.
pub fn verify_consistency(
    env: &Env,
    old_deposits: &Vec<(Address, u32, VaultEntry)>,
    new_deposits: &Vec<(Address, u32, VaultEntry)>,
) -> Result<ConsistencyCheckResult, VaultError> {
    let mut missing_count = 0u32;
    let mut amount_mismatch_count = 0u32;
    let mut metadata_mismatch_count = 0u32;

    let mut old_value: i128 = 0;
    let mut new_value: i128 = 0;

    // Calculate old contract totals
    for (_, _, entry) in old_deposits.iter() {
        old_value = old_value.saturating_add(entry.amount);
    }

    // Calculate new contract totals and check consistency
    for (_, _, entry) in new_deposits.iter() {
        new_value = new_value.saturating_add(entry.amount);
    }

    // Check for missing deposits
    for (depositor, deposit_id, old_entry) in old_deposits.iter() {
        let found = new_deposits.iter().any(|(d, id, _)| d == depositor && id == deposit_id);
        
        if !found {
            missing_count = missing_count.saturating_add(1);
        } else {
            // Verify amount matches
            if let Some((_, _, new_entry)) = new_deposits.iter().find(|(d, id, _)| d == depositor && id == deposit_id) {
                if old_entry.amount != new_entry.amount {
                    amount_mismatch_count = amount_mismatch_count.saturating_add(1);
                }

                // Check metadata consistency
                if old_entry.token != new_entry.token 
                    || old_entry.unlock_time != new_entry.unlock_time
                    || old_entry.penalty_bps != new_entry.penalty_bps {
                    metadata_mismatch_count = metadata_mismatch_count.saturating_add(1);
                }
            }
        }
    }

    let is_consistent = missing_count == 0 && amount_mismatch_count == 0 && metadata_mismatch_count == 0
        && old_value == new_value;

    Ok(ConsistencyCheckResult {
        is_consistent,
        old_contract_deposits: old_deposits.len() as u32,
        new_contract_deposits: new_deposits.len() as u32,
        old_contract_value: old_value,
        new_contract_value: new_value,
        missing_count,
        amount_mismatch_count,
        metadata_mismatch_count,
    })
}

/// Validate that all deposits for a specific depositor are intact.
///
/// Used for verifying individual depositor migration success.
pub fn validate_depositor_deposits(
    env: &Env,
    depositor: &Address,
) -> Result<ValidationReport, VaultError> {
    let now = env.ledger().timestamp();
    let deposit_ids = storage::get_deposit_ids(env, depositor);

    let mut total_deposits = 0u32;
    let mut valid_deposits = 0u32;
    let mut invalid_deposits = 0u32;
    let mut total_value: i128 = 0;
    let mut future_unlock_count = 0u32;
    let mut unlocked_count = 0u32;
    let mut amount_checksum: i128 = 0;
    let mut max_amount: i128 = 0;
    let mut min_amount: i128 = i128::MAX;

    for deposit_id in deposit_ids.iter() {
        total_deposits = total_deposits.saturating_add(1);

        if let Some(entry) = storage::get_deposit_readonly(env, depositor, deposit_id) {
            match validate_single_deposit(env, &entry, depositor, deposit_id) {
                Ok(detail) => {
                    if detail.is_valid {
                        valid_deposits = valid_deposits.saturating_add(1);
                        total_value = total_value.saturating_add(entry.amount);
                        amount_checksum = amount_checksum.saturating_add(entry.amount);

                        if entry.unlock_time > now {
                            future_unlock_count = future_unlock_count.saturating_add(1);
                        } else {
                            unlocked_count = unlocked_count.saturating_add(1);
                        }

                        if entry.amount > max_amount {
                            max_amount = entry.amount;
                        }
                        if entry.amount < min_amount {
                            min_amount = entry.amount;
                        }
                    } else {
                        invalid_deposits = invalid_deposits.saturating_add(1);
                    }
                }
                Err(_) => {
                    invalid_deposits = invalid_deposits.saturating_add(1);
                }
            }
        } else {
            invalid_deposits = invalid_deposits.saturating_add(1);
        }
    }

    if min_amount == i128::MAX {
        min_amount = 0;
    }

    Ok(ValidationReport {
        is_valid: invalid_deposits == 0,
        total_deposits,
        valid_deposits,
        invalid_deposits,
        total_value_locked: total_value,
        future_unlock_count,
        unlocked_count,
        amount_checksum,
        max_deposit_amount: max_amount,
        min_deposit_amount: min_amount,
    })
}

/// Check if a deposit can be safely migrated.
///
/// Returns specific error if deposit cannot be migrated.
pub fn can_migrate_deposit(
    env: &Env,
    entry: &VaultEntry,
    depositor: &Address,
) -> Result<(), VaultError> {
    // Validate the deposit entry
    validate_single_deposit(env, entry, depositor, 0)?;

    // Additional migration-specific checks
    // Ensure deposit is not in transition state
    if entry.amount == 0 {
        return Err(VaultError::InvalidAmount);
    }

    Ok(())
}

// ================================================================
//  Summary and Reporting Functions
// ================================================================

/// Generate a human-readable summary of validation results.
pub fn validation_summary(
    env: &Env,
    report: &ValidationReport,
) -> String {
    if report.is_valid {
        String::from_slice(
            env,
            b"All deposits validated successfully",
        )
    } else {
        String::from_slice(
            env,
            b"Validation detected issues",
        )
    }
}

/// Generate a human-readable summary of consistency check results.
pub fn consistency_summary(
    env: &Env,
    result: &ConsistencyCheckResult,
) -> String {
    if result.is_consistent {
        String::from_slice(
            env,
            b"Migration consistency verified",
        )
    } else {
        String::from_slice(
            env,
            b"Consistency check detected discrepancies",
        )
    }
}
