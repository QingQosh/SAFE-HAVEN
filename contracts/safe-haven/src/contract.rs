// ============================================================
//  SAFE-HAVEN — Soroban Smart Contract
//  Stellar Blockchain | Soroban SDK v22
// ============================================================

use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

use crate::{
    constants::{
        MAX_BATCH_SIZE, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS, MIN_LOCK_DURATION_SECS,
        LOYALTY_SILVER_DEPOSITS, LOYALTY_GOLD_DEPOSITS, LOYALTY_PLATINUM_DEPOSITS,
        LOYALTY_SILVER_VOLUME, LOYALTY_GOLD_VOLUME, LOYALTY_PLATINUM_VOLUME,
        LOYALTY_SILVER_DISCOUNT_BPS, LOYALTY_GOLD_DISCOUNT_BPS, LOYALTY_PLATINUM_DISCOUNT_BPS,
        LOYALTY_SILVER_BONUS_BPS, LOYALTY_GOLD_BONUS_BPS, LOYALTY_PLATINUM_BONUS_BPS,
    },
    errors::VaultError,
    events, storage,
    types::{LedgerVaultEntry, LoyaltyInfo, LoyaltyTier, TierBenefits, VaultEntry},
};

#[contract]
pub struct SafeHaven;

#[contractimpl]
impl SafeHaven {
    // ----------------------------------------------------------------
    //  Initialization
    // ----------------------------------------------------------------

    pub fn initialize(
        env: Env,
        admin: Address,
        fee_recipient: Address,
        max_deposit: Option<i128>,
        max_lock_secs: Option<u64>,
    ) -> Result<(), VaultError> {
        admin.require_auth();

        if storage::is_initialized(&env) {
            return Err(VaultError::Unauthorized);
        }

        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        storage::set_fee_recipient(&env, &fee_recipient);

        if let Some(v) = max_deposit {
            if v <= 0 {
                return Err(VaultError::InvalidAmount);
            }
            storage::set_max_deposit(&env, v);
        }

        if let Some(v) = max_lock_secs {
            if v == 0 {
                return Err(VaultError::LockDurationTooLong);
            }
            storage::set_max_lock_secs(&env, v);
        }

        let effective_max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        let effective_max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        events::contract_initialized(&env, &admin, &fee_recipient, effective_max_deposit, effective_max_lock);

        Ok(())
    }

    // ----------------------------------------------------------------
    //  Core: Deposit
    // ----------------------------------------------------------------

    pub fn deposit(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    pub fn deposit_for(
        env: Env,
        payer: Address,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        payer.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&payer, &env.current_contract_address(), &amount);

        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Core: Deposit by Ledger Sequence (Issue #88)
    // ----------------------------------------------------------------

    pub fn deposit_by_ledger(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_ledger: u32,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let current_ledger = env.ledger().sequence();
        if unlock_ledger <= current_ledger {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let entry = LedgerVaultEntry {
            token: token.clone(),
            amount,
            unlock_ledger,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit_by_ledger(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit_by_ledger(&env, &depositor, &token, amount, unlock_ledger, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Core: Cancel Deposit (early exit with penalty)
    // ----------------------------------------------------------------

    pub fn cancel_deposit(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now >= entry.unlock_time {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            // Apply loyalty discount to the early-exit penalty
            let effective_penalty_bps =
                SafeHaven::apply_loyalty_discount(&env, &depositor, entry.penalty_bps);
            let penalty: i128 = (entry.amount * effective_penalty_bps as i128) / 10_000;
            let refund = entry.amount - penalty;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                token_client.transfer(&contract, &fee_recipient, &penalty);
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger >= entry.unlock_ledger {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            // Apply loyalty discount to the early-exit penalty
            let effective_penalty_bps =
                SafeHaven::apply_loyalty_discount(&env, &depositor, entry.penalty_bps);
            let penalty: i128 = (entry.amount * effective_penalty_bps as i128) / 10_000;
            let refund = entry.amount - penalty;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                token_client.transfer(&contract, &fee_recipient, &penalty);
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Core: Withdraw
    // ----------------------------------------------------------------

    pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            SafeHaven::record_loyalty_completion(&env, &depositor, entry.amount);
            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            SafeHaven::record_loyalty_completion(&env, &depositor, entry.amount);
            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    pub fn withdraw_to(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        recipient: Address,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            SafeHaven::record_loyalty_completion(&env, &depositor, entry.amount);
            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            SafeHaven::record_loyalty_completion(&env, &depositor, entry.amount);
            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Admin: Emergency Withdrawal
    // ----------------------------------------------------------------

    pub fn emergency_withdraw(
        env: Env,
        admin: Address,
        depositor: Address,
        deposit_id: u32,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::emergency_withdraw(&env, &admin, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::emergency_withdraw(&env, &admin, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Admin: Pause / Unpause
    // ----------------------------------------------------------------

    pub fn pause(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }
        storage::set_paused(&env, true);
        events::paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }
        storage::set_paused(&env, false);
        events::unpaused(&env, &admin);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    // ----------------------------------------------------------------
    //  Admin: Two-Step Admin Transfer
    // ----------------------------------------------------------------

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        if new_admin == stored_admin {
            return Err(VaultError::InvalidAdmin);
        }

        storage::set_pending_admin(&env, &new_admin);
        events::admin_transfer_initiated(&env, &admin, &new_admin);
        Ok(())
    }

    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), VaultError> {
        new_admin.require_auth();

        let pending_admin = storage::get_pending_admin(&env).ok_or(VaultError::Unauthorized)?;
        if new_admin != pending_admin {
            return Err(VaultError::Unauthorized);
        }

        storage::set_admin(&env, &new_admin);
        storage::remove_pending_admin(&env);
        events::admin_transfer_accepted(&env, &new_admin);
        Ok(())
    }

    pub fn cancel_transfer_admin(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();

        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        // Emit an event when a pending admin is cancelled so off-chain indexers
        // and UIs observing admin state transitions won't show a stale pending admin.
        if let Some(pending) = storage::get_pending_admin(&env) {
            storage::remove_pending_admin(&env);
            events::admin_transfer_cancelled(&env, &admin, &pending);
        }
        Ok(())
    }

    pub fn renounce_admin(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();

        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        storage::remove_admin(&env);
        storage::remove_pending_admin(&env);
        events::admin_renounced(&env, &admin);
        Ok(())
    }

    // ----------------------------------------------------------------
    //  Read-only Queries
    // ----------------------------------------------------------------

    /// No auth required — this is a public read-only query (closes #81)
    pub fn get_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<VaultEntry> {
        storage::get_deposit_readonly(&env, &depositor, deposit_id)
    }

    /// Returns the `LedgerVaultEntry` for a ledger-sequence-based deposit, or `None` if not found.
    /// No auth required — public read-only query (closes #44).
    pub fn get_ledger_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<LedgerVaultEntry> {
        storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id)
    }

    pub fn get_vault_batch(env: Env, depositors: Vec<Address>, deposit_id: u32) -> Vec<Option<VaultEntry>> {
        let limit = if depositors.len() > MAX_BATCH_SIZE { MAX_BATCH_SIZE } else { depositors.len() as u32 };
        let mut results = Vec::new(&env);
        for i in 0..limit {
            if let Some(depositor) = depositors.get(i) {
                let entry = storage::get_deposit_readonly(&env, &depositor, deposit_id);
                results.push_back(entry);
            }
        }
        results
    }

    pub fn get_deposit_ids(env: Env, depositor: Address) -> Vec<u32> {
        storage::get_deposit_ids(&env, &depositor)
    }

    /// Returns the current ledger timestamp.
    /// Read-only — does not bump storage TTL.
    pub fn get_time(env: Env) -> u64 {
        env.ledger().timestamp()
    }

    /// No auth required — this is a public read-only query (closes #81)
    ///
    /// For timestamp-based deposits: returns exact seconds remaining.
    /// For ledger-based deposits: returns an estimate in seconds using
    /// `LEDGER_SECONDS` (fixes #21). Returns 0 when unlocked or not found.
    pub fn time_remaining(env: Env, depositor: Address, deposit_id: u32) -> u64 {
        // Timestamp-based path
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            return entry.unlock_time.saturating_sub(now);
        }

        // Ledger-based path: convert remaining ledgers → estimated seconds (fixes #21)
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current = env.ledger().sequence();
            if current >= entry.unlock_ledger {
                return 0;
            }
            let remaining_ledgers = (entry.unlock_ledger - current) as u64;
            return remaining_ledgers.saturating_mul(storage::LEDGER_SECONDS);
        }

        0
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        storage::get_admin(&env)
    }

    pub fn get_pending_admin(env: Env) -> Option<Address> {
        storage::get_pending_admin(&env)
    }

    pub fn get_constants(env: Env) -> (i128, u64) {
        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        (max_deposit, max_lock)
    }

    pub fn get_fee_recipient(env: Env) -> Option<Address> {
        storage::get_fee_recipient(&env)
    }

    pub fn get_depositor_count(env: Env) -> u32 {
        storage::get_depositor_count(&env)
    }

    pub fn get_depositors(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        storage::get_depositors_page(&env, offset, limit)
    }

    pub fn is_initialized(env: Env) -> bool {
        storage::is_initialized(&env)
    }

    // ----------------------------------------------------------------
    //  Loyalty Program — Queries
    // ----------------------------------------------------------------

    /// Returns the current `LoyaltyTier` and progress details for `depositor`.
    /// No auth required — public read-only query.
    ///
    /// Tier rules (either condition qualifies for a tier):
    ///   Bronze   — new user (default)
    ///   Silver   — 3+ completed deposits OR 10,000+ total volume
    ///   Gold     — 10+ completed deposits OR 100,000+ total volume
    ///   Platinum — 25+ completed deposits OR 1,000,000+ total volume
    pub fn get_loyalty_tier(env: Env, depositor: Address) -> LoyaltyInfo {
        let completed = storage::get_loyalty_deposit_count(&env, &depositor);
        let volume = storage::get_loyalty_volume(&env, &depositor);
        let tier = SafeHaven::compute_tier(completed, volume);

        // Progress to next tier
        let (deposits_to_next, volume_to_next) = match tier {
            LoyaltyTier::Bronze => (
                LOYALTY_SILVER_DEPOSITS.saturating_sub(completed),
                LOYALTY_SILVER_VOLUME.saturating_sub(volume),
            ),
            LoyaltyTier::Silver => (
                LOYALTY_GOLD_DEPOSITS.saturating_sub(completed),
                LOYALTY_GOLD_VOLUME.saturating_sub(volume),
            ),
            LoyaltyTier::Gold => (
                LOYALTY_PLATINUM_DEPOSITS.saturating_sub(completed),
                LOYALTY_PLATINUM_VOLUME.saturating_sub(volume),
            ),
            LoyaltyTier::Platinum => (0, 0),
        };

        LoyaltyInfo {
            tier,
            completed_deposits: completed,
            total_volume: volume,
            deposits_to_next_tier: deposits_to_next,
            volume_to_next_tier: volume_to_next,
        }
    }

    /// Returns the benefits associated with each tier.
    /// Pass the desired `LoyaltyTier` variant to get its perks.
    /// No auth required — public read-only query.
    pub fn tier_benefits(env: Env, tier: LoyaltyTier) -> TierBenefits {
        match tier {
            LoyaltyTier::Bronze => TierBenefits {
                tier: LoyaltyTier::Bronze,
                fee_discount_bps: 0,
                bonus_interest_bps: 0,
                description: String::from_str(
                    &env,
                    "Bronze: No discount. Deposit to unlock Silver (3 deposits or 10,000 volume).",
                ),
            },
            LoyaltyTier::Silver => TierBenefits {
                tier: LoyaltyTier::Silver,
                fee_discount_bps: LOYALTY_SILVER_DISCOUNT_BPS,
                bonus_interest_bps: LOYALTY_SILVER_BONUS_BPS,
                description: String::from_str(
                    &env,
                    "Silver: 5% early-exit fee discount + 0.25% bonus interest on matured withdrawals.",
                ),
            },
            LoyaltyTier::Gold => TierBenefits {
                tier: LoyaltyTier::Gold,
                fee_discount_bps: LOYALTY_GOLD_DISCOUNT_BPS,
                bonus_interest_bps: LOYALTY_GOLD_BONUS_BPS,
                description: String::from_str(
                    &env,
                    "Gold: 10% early-exit fee discount + 0.75% bonus interest on matured withdrawals.",
                ),
            },
            LoyaltyTier::Platinum => TierBenefits {
                tier: LoyaltyTier::Platinum,
                fee_discount_bps: LOYALTY_PLATINUM_DISCOUNT_BPS,
                bonus_interest_bps: LOYALTY_PLATINUM_BONUS_BPS,
                description: String::from_str(
                    &env,
                    "Platinum: 20% early-exit fee discount + 1.50% bonus interest on matured withdrawals.",
                ),
            },
        }
    }
}

// ----------------------------------------------------------------
//  Loyalty Program — Private Helpers
// ----------------------------------------------------------------

impl SafeHaven {
    /// Computes the tier purely from `completed_deposits` and `total_volume`.
    /// Either condition (deposits OR volume) is sufficient to reach a tier.
    fn compute_tier(completed_deposits: u32, total_volume: i128) -> LoyaltyTier {
        if completed_deposits >= LOYALTY_PLATINUM_DEPOSITS || total_volume >= LOYALTY_PLATINUM_VOLUME {
            return LoyaltyTier::Platinum;
        }
        if completed_deposits >= LOYALTY_GOLD_DEPOSITS || total_volume >= LOYALTY_GOLD_VOLUME {
            return LoyaltyTier::Gold;
        }
        if completed_deposits >= LOYALTY_SILVER_DEPOSITS || total_volume >= LOYALTY_SILVER_VOLUME {
            return LoyaltyTier::Silver;
        }
        LoyaltyTier::Bronze
    }

    /// Returns the fee discount in bps for the given tier.
    fn tier_fee_discount(tier: LoyaltyTier) -> u32 {
        match tier {
            LoyaltyTier::Bronze => 0,
            LoyaltyTier::Silver => LOYALTY_SILVER_DISCOUNT_BPS,
            LoyaltyTier::Gold => LOYALTY_GOLD_DISCOUNT_BPS,
            LoyaltyTier::Platinum => LOYALTY_PLATINUM_DISCOUNT_BPS,
        }
    }

    /// Records a completed deposit for loyalty tracking, upgrades the stored
    /// tier if warranted, and emits a `TierUpgraded` event on promotion.
    fn record_loyalty_completion(env: &Env, depositor: &Address, amount: i128) {
        let new_count = storage::increment_loyalty_deposit_count(env, depositor);
        let new_volume = storage::add_loyalty_volume(env, depositor, amount);

        let new_tier = SafeHaven::compute_tier(new_count, new_volume);
        let old_tier = storage::get_stored_loyalty_tier(env, depositor);

        if new_tier > old_tier {
            storage::set_loyalty_tier(env, depositor, new_tier);
            events::tier_upgraded(env, depositor, old_tier, new_tier, new_count, new_volume);
        }
    }

    /// Applies the depositor's loyalty fee discount to `penalty_bps`, returning
    /// the effective penalty. The discount reduces the penalty but never below 0.
    fn apply_loyalty_discount(env: &Env, depositor: &Address, penalty_bps: u32) -> u32 {
        let completed = storage::get_loyalty_deposit_count(env, depositor);
        let volume = storage::get_loyalty_volume(env, depositor);
        let tier = SafeHaven::compute_tier(completed, volume);
        let discount = SafeHaven::tier_fee_discount(tier);
        penalty_bps.saturating_sub(discount)
    }
}
