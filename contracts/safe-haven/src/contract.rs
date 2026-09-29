// ============================================================
//  SAFE-HAVEN — Soroban Smart Contract
//  Stellar Blockchain | Soroban SDK v22
// ============================================================

use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String, Symbol, Vec};

use crate::{
    constants::{
        MAX_BATCH_SIZE, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS, MIN_LOCK_DURATION_SECS,
        MIN_LOCK_LEDGERS, STAKER_PENALTY_BPS, FEE_RECIPIENT_PENALTY_BPS,
    },
    errors::VaultError,
    events, storage, constants,
    types::{
        DepositType, MultiTokenVaultEntry, TokenDeposit, VaultEntry, LedgerVaultEntry, Page,
        STORAGE_VERSION, MAX_TOKENS_PER_DEPOSIT, MEVCommitment, MEVDetection, MEVStatus,
        PriceSample, SustainabilityMetrics, BatchWithdrawalResult, WithdrawalResult,
    },
};

/// Minimum compound frequency: must be at least 60 seconds if non-zero.
const MIN_COMPOUND_FREQUENCY_SECS: u64 = 60;

/// Annual interest rate used for compound accrual: 5% expressed as basis points (500).
/// In production this could be made configurable, but per the issue scope it is fixed.
const ANNUAL_INTEREST_BPS: u128 = 500;

/// Flash loan fee charged on each borrowed amount. 10 bps = 0.10%.
const FLASH_LOAN_FEE_BPS: u32 = 10;

/// Seconds in a year (non-leap) used for pro-rata interest calculations.
const SECS_PER_YEAR: u128 = 31_536_000;

fn auto_pause(env: &Env, amount: i128) {
    let ledger = env.ledger().sequence();
    let activation = CircuitBreakerActivation {
        ledger,
        amount,
        threshold: MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER,
    };
    storage::set_paused(env, true);
    storage::record_circuit_breaker_activation(env, &activation);
    let admin = storage::get_admin(env).expect("initialized contract must have admin");
    events::circuit_breaker_tripped(
        env,
        &admin,
        ledger,
        amount,
        MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER,
    );
}

fn check_emergency_withdrawal_limit(env: &Env, amount: i128) -> Result<(), VaultError> {
    let total = storage::get_current_ledger_emergency_withdrawal(env);
    if total.saturating_add(amount) > MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER {
        Err(VaultError::EmergencyWithdrawalLimitExceeded)
    } else {
        Ok(())
    }
}

fn record_emergency_withdrawal(env: &Env, amount: i128) {
    let total = storage::add_emergency_withdrawal(env, amount);
    if total >= MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER && !storage::is_paused(env) {
        auto_pause(env, total);
    }
}

#[contract]
pub struct SafeHaven;

// ----------------------------------------------------------------
//  Internal helpers
// ----------------------------------------------------------------

/// Compute the accrued balance for `entry` as of `now`.
///
/// Uses compound interest applied period-by-period:
///   balance_after_one_period = balance + balance × ANNUAL_INTEREST_BPS × freq_secs
///                                                    ─────────────────────────────
///                                                       SECS_PER_YEAR × 10_000
///
/// Crucially, the multiplication is done **before** the division so that integer
/// truncation does not zero-out the per-period rate for sub-year frequencies.
/// This guarantees that any `amount ≥ (SECS_PER_YEAR × 10_000) / ANNUAL_INTEREST_BPS`
/// (≈ 6.3 M for 5% annual rate) will accrue at least 1 unit per period.
///
/// Returns the new balance (>= original amount).
fn compute_accrued_amount(amount: i128, entry_freq: u64, last_accrual: u64, now: u64) -> i128 {
    if entry_freq == 0 || now <= last_accrual {
        return amount;
    }

    let elapsed = now.saturating_sub(last_accrual);
    let periods = elapsed / entry_freq; // integer division — only complete periods

    if periods == 0 {
        return amount;
    }

    // Denominator: SECS_PER_YEAR × 10_000 (fixed for 5% p.a. expressed in bps)
    let denominator: u128 = SECS_PER_YEAR.saturating_mul(10_000);
    let freq_u128 = entry_freq as u128;

    // Apply compounding iteratively.  Capped to guard against enormous period counts.
    let mut balance = amount;
    let periods_capped = periods.min(2_628_000); // 5 years ÷ 60 s/period
    for _ in 0..periods_capped {
        // interest = balance × ANNUAL_INTEREST_BPS × freq / (SECS_PER_YEAR × 10_000)
        // Multiplying balance × numerator first avoids truncating the small rate.
        let interest = (balance as u128)
            .saturating_mul(ANNUAL_INTEREST_BPS)
            .saturating_mul(freq_u128)
            / denominator;
        balance = balance.saturating_add(interest as i128);
    }
    balance
}

/// Check the withdrawal whitelist for `(depositor, deposit_id)`.
/// Returns `Ok(())` if:
///   - no whitelist is configured (None), OR
///   - the whitelist is empty, OR
///   - `recipient` is in the whitelist.
/// Returns `Err(RecipientNotWhitelisted)` otherwise.
fn check_whitelist(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    recipient: &Address,
) -> Result<(), VaultError> {
    if let Some(wl) = storage::get_withdrawal_whitelist(env, depositor, deposit_id) {
        if wl.is_empty() {
            return Ok(());
        }
        for addr in wl.iter() {
            if addr == *recipient {
                return Ok(());
            }
        }
        return Err(VaultError::RecipientNotWhitelisted);
    }
    Ok(())
}

fn check_tax_wash_sale(
    env: &Env,
    depositor: &Address,
    token: &Address,
) -> Result<(), VaultError> {
    if let Some(until) = storage::get_tax_wash_sale_until(env, depositor, token) {
        if env.ledger().timestamp() < until {
            return Err(VaultError::TaxWashSalePeriodActive);
        }
    }
    Ok(())
}

#[contractimpl]
impl SafeHaven {
    pub fn propose_upgrade(
        env: Env, proposer: Address, old_version: soroban_sdk::String,
        new_version: soroban_sdk::String, diff_url: soroban_sdk::String,
        audit_url: soroban_sdk::String, wasm_hash: soroban_sdk::BytesN<32>,
    ) -> Result<u32, VaultError> {
        proposer.require_auth();
        if diff_url.is_empty() || audit_url.is_empty() { return Err(VaultError::UpgradeEvidenceRequired); }
        let id_key = crate::types::VaultKey::NextUpgradeId;
        let id: u32 = env.storage().persistent().get(&id_key).unwrap_or(0);
        env.storage().persistent().set(&id_key, &id.saturating_add(1));
        let proposal = UpgradeProposal {
            id, proposer, old_version, new_version, diff_url, audit_url,
            review_url: soroban_sdk::String::from_slice(&env, ""), wasm_hash,
            status: UpgradeStatus::Review, approval_votes: 0, rejection_votes: 0,
            veto_votes: 0, approved_at: None,
        };
        env.storage().persistent().set(&crate::types::VaultKey::UpgradeProposal(id), &proposal);
        Ok(id)
    }

    pub fn review_upgrade(
        env: Env, reviewer: Address, proposal_id: u32, review_url: soroban_sdk::String,
        security_audit_complete: bool,
    ) -> Result<(), VaultError> {
        reviewer.require_auth();
        let key = crate::types::VaultKey::UpgradeProposal(proposal_id);
        let mut proposal: UpgradeProposal = env.storage().persistent().get(&key).ok_or(VaultError::UpgradeNotFound)?;
        if proposal.status != UpgradeStatus::Review || reviewer == proposal.proposer || review_url.is_empty() || !security_audit_complete {
            return Err(VaultError::UpgradeReviewRequired);
        }
        proposal.review_url = review_url;
        proposal.status = UpgradeStatus::Voting;
        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    pub fn vote_upgrade(env: Env, voter: Address, proposal_id: u32, approve: bool) -> Result<(), VaultError> {
        voter.require_auth();
        let key = crate::types::VaultKey::UpgradeProposal(proposal_id);
        let mut proposal: UpgradeProposal = env.storage().persistent().get(&key).ok_or(VaultError::UpgradeNotFound)?;
        if proposal.status != UpgradeStatus::Voting { return Err(VaultError::UpgradeNotVoting); }
        let vote_key = crate::types::VaultKey::UpgradeVote(proposal_id, voter);
        if env.storage().persistent().has(&vote_key) { return Err(VaultError::UpgradeAlreadyVoted); }
        env.storage().persistent().set(&vote_key, &approve);
        if approve { proposal.approval_votes = proposal.approval_votes.saturating_add(1); }
        else { proposal.rejection_votes = proposal.rejection_votes.saturating_add(1); }
        if proposal.approval_votes >= MIN_UPGRADE_APPROVALS {
            proposal.status = UpgradeStatus::Approved;
            proposal.approved_at = Some(env.ledger().timestamp());
        }
        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    pub fn veto_upgrade(env: Env, voter: Address, proposal_id: u32) -> Result<(), VaultError> {
        voter.require_auth();
        let key = crate::types::VaultKey::UpgradeProposal(proposal_id);
        let mut proposal: UpgradeProposal = env.storage().persistent().get(&key).ok_or(VaultError::UpgradeNotFound)?;
        if proposal.status != UpgradeStatus::Approved { return Err(VaultError::UpgradeNotApproved); }
        let veto_key = crate::types::VaultKey::UpgradeVeto(proposal_id, voter);
        if env.storage().persistent().has(&veto_key) { return Err(VaultError::UpgradeAlreadyVoted); }
        env.storage().persistent().set(&veto_key, &true);
        proposal.veto_votes = proposal.veto_votes.saturating_add(1);
        proposal.status = UpgradeStatus::Vetoed;
        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    pub fn execute_upgrade(env: Env, proposal_id: u32) -> Result<(), VaultError> {
        let key = crate::types::VaultKey::UpgradeProposal(proposal_id);
        let mut proposal: UpgradeProposal = env.storage().persistent().get(&key).ok_or(VaultError::UpgradeNotFound)?;
        if proposal.status != UpgradeStatus::Approved { return Err(VaultError::UpgradeNotApproved); }
        let approved_at = proposal.approved_at.ok_or(VaultError::UpgradeNotApproved)?;
        if env.ledger().timestamp() < approved_at.saturating_add(UPGRADE_TIMELOCK_SECS) { return Err(VaultError::UpgradeTimelocked); }
        env.deployer().update_current_contract_wasm(proposal.wasm_hash);
        proposal.status = UpgradeStatus::Executed;
        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    pub fn get_upgrade_proposal(env: Env, proposal_id: u32) -> Option<UpgradeProposal> {
        env.storage().persistent().get(&crate::types::VaultKey::UpgradeProposal(proposal_id))
    }

    // ----------------------------------------------------------------
    //  Faucet
    // ----------------------------------------------------------------

    pub fn configure_faucet_asset(
        env: Env,
        admin: Address,
        asset: FaucetAsset,
        token: Address,
        max_amount: i128,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        if max_amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        storage::set_faucet_asset(&env, &asset, &token, max_amount);
        Ok(())
    }

    pub fn fund_faucet(env: Env, admin: Address, token: Address, amount: i128) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        token::Client::new(&env, &token).transfer(&admin, &env.current_contract_address(), &amount);
        Ok(())
    }

    pub fn request_faucet(env: Env, account: Address, asset: FaucetAsset, amount: i128) -> Result<(), VaultError> {
        account.require_auth();
        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }
        let status = storage::get_faucet_status(&env, &asset);
        let token = status.token.ok_or(VaultError::FaucetNotConfigured)?;
        if amount <= 0 || amount > status.max_amount {
            return Err(VaultError::FaucetAmountTooLarge);
        }
        let now = env.ledger().timestamp();
        if let Some(last_request) = storage::get_faucet_last_request(&env, &account) {
            if now < last_request.saturating_add(3600) {
                return Err(VaultError::FaucetRateLimited);
            }
        }
        if status.balance < amount {
            return Err(VaultError::FaucetInsufficientFunds);
        }
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &account, &amount);
        storage::record_faucet_request(&env, &account, &asset, amount, now);
        events::faucet_claim(&env, &account, asset, amount);
        Ok(())
    }

    pub fn get_faucet_status(env: Env, asset: FaucetAsset) -> FaucetStatus {
        storage::get_faucet_status(&env, &asset)
    }

    pub fn get_faucet_last_request(env: Env, account: Address) -> Option<u64> {
        storage::get_faucet_last_request(&env, &account)
    }

    // ----------------------------------------------------------------
    //  ACL — Access Control List
    // ----------------------------------------------------------------

    /// Grant `permission` to `grantee`. Only the admin may call this.
    ///
    /// Multiple calls accumulate permissions (bitmask OR). To grant all
    /// permissions at once, call `grant_permission` for each one.
    ///
    /// # Events
    /// Emits `perm_granted(admin, grantee, permission_mask)`.
    pub fn grant_permission(
        env: Env,
        admin: Address,
        grantee: Address,
        permission: PermissionType,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::grant_permission(&env, &grantee, permission);
        events::permission_granted(&env, &admin, &grantee, permission.mask());
        Ok(())
    }

    /// Revoke `permission` from `grantee`. Only the admin may call this.
    ///
    /// Revoking a permission the address does not hold is a no-op (idempotent).
    ///
    /// # Events
    /// Emits `perm_revoked(admin, grantee, permission_mask)`.
    pub fn revoke_permission(
        env: Env,
        admin: Address,
        grantee: Address,
        permission: PermissionType,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::revoke_permission(&env, &grantee, permission);
        events::permission_revoked(&env, &admin, &grantee, permission.mask());
        Ok(())
    }

    /// Return the raw permission bitmask for `address`.
    ///
    /// The bitmask encodes which `PermissionType` bits are set:
    /// - bit 0 (`0x01`): `ViewVault`
    /// - bit 1 (`0x02`): `Deposit`
    /// - bit 2 (`0x04`): `Withdraw`
    /// - bit 3 (`0x08`): `CancelDeposit`
    /// - bit 4 (`0x10`): `Manage`
    ///
    /// A value of `0` means no permissions have been granted. The admin
    /// address implicitly has all permissions regardless of this value.
    pub fn get_permissions(env: Env, address: Address) -> u32 {
        storage::get_acl_bitmask(&env, &address)
    }

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
            return Err(VaultError::AlreadyInitialized);
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
        events::contract_initialized(
            &env,
            &admin,
            &fee_recipient,
            effective_max_deposit,
            effective_max_lock,
        );

        Ok(())
    }

    // ----------------------------------------------------------------
    //  Smart-wallet and session-key authorization
    // ----------------------------------------------------------------

    /// Authorize a delegated key for deposits attributed to `wallet`.
    ///
    /// The wallet signs this one-time authorization. The delegated entrypoint
    /// still transfers tokens from the session key, never from the wallet.
    pub fn authorize_session_key(
        env: Env,
        wallet: Address,
        session_key: Address,
        expires_at: u64,
    ) -> Result<(), VaultError> {
        wallet.require_auth();
        if expires_at <= env.ledger().timestamp() {
            return Err(VaultError::InvalidSessionKeyExpiry);
        }
        storage::set_session_key(&env, &wallet, &session_key, expires_at);
        Ok(())
    }

    pub fn revoke_session_key(
        env: Env,
        wallet: Address,
        session_key: Address,
    ) -> Result<(), VaultError> {
        wallet.require_auth();
        storage::remove_session_key(&env, &wallet, &session_key);
        Ok(())
    }

    pub fn is_session_key_authorized(
        env: Env,
        wallet: Address,
        session_key: Address,
    ) -> bool {
        storage::get_session_key_expiry(&env, &wallet, &session_key)
            .map(|expires_at| expires_at > env.ledger().timestamp())
            .unwrap_or(false)
    }

    /// Deposit using a wallet-authorized delegated key.
    ///
    /// The session key is the payer and the wallet is the depositor. This
    /// preserves the wallet's multisig/account-abstraction boundary: a session
    /// key can submit an approved operation, but cannot spend wallet funds.
    pub fn deposit_with_session_key(
        env: Env,
        session_key: Address,
        wallet: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        session_key.require_auth();
        match storage::get_session_key_expiry(&env, &wallet, &session_key) {
            Some(expires_at) if expires_at > env.ledger().timestamp() => {}
            Some(_) => return Err(VaultError::SessionKeyExpired),
            None => return Err(VaultError::SessionKeyNotAuthorized),
        }

        Self::deposit_for(
            env,
            session_key,
            wallet,
            token,
            amount,
            unlock_time,
            penalty_bps,
        )
    }

    // ----------------------------------------------------------------
    //  Core: Single-token Deposit
    // ----------------------------------------------------------------

    pub fn register_quantum_safe_key(
        env: Env,
        account: Address,
        public_key: Bytes,
    ) -> Result<(), VaultError> {
        account.require_auth();
        if !pq::is_valid_public_key(&public_key) {
            return Err(VaultError::InvalidQuantumSafeKey);
        }
        storage::set_quantum_safe_public_key(&env, &account, &public_key);
        Ok(())
    }

    pub fn get_quantum_safe_key(env: Env, account: Address) -> Option<Bytes> {
        storage::get_quantum_safe_public_key(&env, &account)
    }

    /// Returns the exact bytes that must be signed for the next deposit.
    /// The payload includes the current deposit nonce and encrypted metadata.
    pub fn quantum_safe_message(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
        encrypted_metadata: Bytes,
    ) -> Bytes {
        quantum_safe_payload(
            &env,
            depositor.clone(),
            token,
            amount,
            unlock_time,
            penalty_bps,
            encrypted_metadata,
            storage::peek_next_deposit_id(&env, &depositor),
        )
    }

    /// Creates a deposit authorized by FIPS-204 ML-DSA-44 in addition to the
    /// normal Stellar account authorization required by the token transfer.
    pub fn deposit_quantum_safe(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
        encrypted_metadata: Bytes,
        signature: Bytes,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();
        let public_key = storage::get_quantum_safe_public_key(&env, &depositor)
            .ok_or(VaultError::InvalidQuantumSafeKey)?;
        if !pq::has_valid_lengths(&public_key, &signature) {
            return Err(VaultError::InvalidQuantumSafeKey);
        }

        let message = quantum_safe_payload(
            &env,
            depositor.clone(),
            token.clone(),
            amount,
            unlock_time,
            penalty_bps,
            encrypted_metadata.clone(),
            storage::peek_next_deposit_id(&env, &depositor),
        );
        if !pq::verify(&public_key, &signature, &message) {
            return Err(VaultError::InvalidQuantumSafeSignature);
        }

        let deposit_id = Self::deposit(
            env.clone(),
            depositor.clone(),
            token,
            amount,
            unlock_time,
            penalty_bps,
        )?;
        storage::set_quantum_safe_metadata(&env, &depositor, deposit_id, &encrypted_metadata);
        Ok(deposit_id)
    }

    pub fn get_quantum_safe_metadata(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<Bytes> {
        storage::get_quantum_safe_metadata(&env, &depositor, deposit_id)
    }

    pub fn deposit(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();
        storage::require_permission(&env, &depositor, PermissionType::Deposit)?;

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if storage::is_emergency_lockdown(&env) {
            return Err(VaultError::EmergencyLockdown);
        }

        if storage::is_strict_token_allowlist(&env) && !storage::is_token_allowed(&env, &token) {
            return Err(VaultError::TokenNotAllowed);
        }
        check_tax_wash_sale(&env, &depositor, &token)?;

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
            compound_frequency_secs: 0,
            last_accrual_timestamp: now,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        // Initialize NFT evolution record for the deposit
        let nft_record = crate::nft::create_evolution_record(
            &env,
            deposit_id,
            amount,
            unlock_time,
            now,
        );
        storage::set_nft_evolution(&env, &depositor, deposit_id, &nft_record);

        // Track sustainability metrics
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        let carbon_footprint = calculate_carbon_footprint(amount, lock_duration);
        let carbon_offset = calculate_carbon_offset(carbon_footprint, penalty_bps);
        
        let metrics = SustainabilityMetrics {
            deposit_id,
            depositor: depositor.clone(),
            carbon_footprint,
            renewable_energy_percent: crate::constants::RENEWABLE_ENERGY_BASELINE,
            carbon_offset_grams: carbon_offset,
            timestamp: now,
        };
        
        storage::set_sustainability_metrics(&env, &depositor, deposit_id, &metrics);
        
        // Update aggregated totals
        let total_carbon = storage::get_total_carbon_footprint(&env, &depositor).saturating_add(carbon_footprint);
        let total_offset = storage::get_total_carbon_offset(&env, &depositor).saturating_add(carbon_offset);
        storage::set_total_carbon_footprint(&env, &depositor, total_carbon);
        storage::set_total_carbon_offset(&env, &depositor, total_offset);
        
        // Check for sustainability milestones
        let deposit_ids = storage::get_deposit_ids(&env, &depositor);
        let mut total_renewable: u64 = 0;
        for id in deposit_ids.iter() {
            if let Some(m) = storage::get_sustainability_metrics_readonly(&env, &depositor, id) {
                total_renewable = total_renewable.saturating_add(m.renewable_energy_percent as u64);
            }
        }
        let average_renewable = if deposit_ids.len() > 0 {
            ((total_renewable / deposit_ids.len() as u64) as u32).min(100)
        } else {
            0
        };
        check_sustainability_milestones(&env, &depositor, total_carbon, total_offset, average_renewable);

        Ok(deposit_id)
    }

    pub fn batch_deposit(
        env: Env,
        depositor: Address,
        deposits: Vec<DepositRequest>,
    ) -> Result<Vec<u32>, VaultError> {
        depositor.require_auth();
        storage::require_permission(&env, &depositor, PermissionType::Deposit)?;

        if deposits.len() > MAX_BATCH_SIZE {
            return Err(VaultError::BatchTooLarge);
        }

        let mut deposit_ids = Vec::new(&env);
        for request in deposits.iter() {
            if storage::is_paused(&env) || request.amount <= 0 {
                return Err(if storage::is_paused(&env) {
                    VaultError::ContractPaused
                } else {
                    VaultError::InvalidAmount
                });
            }
            let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
            if request.amount > max_deposit {
                return Err(VaultError::AmountTooLarge);
            }
            if request.penalty_bps > 10_000 {
                return Err(VaultError::InvalidPenaltyBps);
            }
            if request.penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
                return Err(VaultError::MissingFeeRecipient);
            }
            let now = env.ledger().timestamp();
            if request.unlock_time <= now {
                return Err(VaultError::UnlockTimeNotInFuture);
            }
            let lock_duration = request.unlock_time.saturating_sub(now);
            let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
            if lock_duration > max_lock {
                return Err(VaultError::LockDurationTooLong);
            }
            if lock_duration < MIN_LOCK_DURATION_SECS {
                return Err(VaultError::LockDurationTooShort);
            }
        }

        for request in deposits.iter() {
            let deposit_id = storage::next_deposit_id(&env, &depositor);
            token::Client::new(&env, &request.token).transfer(
                &depositor,
                &env.current_contract_address(),
                &request.amount,
            );
            let entry = VaultEntry {
                token: request.token.clone(),
                amount: request.amount,
                unlock_time: request.unlock_time,
                depositor: depositor.clone(),
                penalty_bps: request.penalty_bps,
            };
            storage::set_deposit(&env, &depositor, deposit_id, &entry);
            storage::add_depositor(&env, &depositor);
            events::deposit(
                &env,
                &depositor,
                &request.token,
                request.amount,
                request.unlock_time,
                deposit_id,
            );
            deposit_ids.push_back(deposit_id);
        }
        Ok(deposit_ids)
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
        storage::require_permission(&env, &payer, PermissionType::Deposit)?;

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if storage::is_strict_token_allowlist(&env) && !storage::is_token_allowed(&env, &token) {
            return Err(VaultError::TokenNotAllowed);
        }
        check_tax_wash_sale(&env, &depositor, &token)?;

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
            compound_frequency_secs: 0,
            last_accrual_timestamp: now,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);

        // Initialize NFT evolution record for the deposit
        let nft_record = crate::nft::create_evolution_record(
            &env,
            deposit_id,
            amount,
            unlock_time,
            now,
        );
        storage::set_nft_evolution(&env, &depositor, deposit_id, &nft_record);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        // Track sustainability metrics
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        let carbon_footprint = calculate_carbon_footprint(amount, lock_duration);
        let carbon_offset = calculate_carbon_offset(carbon_footprint, penalty_bps);
        
        let metrics = SustainabilityMetrics {
            deposit_id,
            depositor: depositor.clone(),
            carbon_footprint,
            renewable_energy_percent: crate::constants::RENEWABLE_ENERGY_BASELINE,
            carbon_offset_grams: carbon_offset,
            timestamp: now,
        };
        
        storage::set_sustainability_metrics(&env, &depositor, deposit_id, &metrics);
        
        // Update aggregated totals
        let total_carbon = storage::get_total_carbon_footprint(&env, &depositor).saturating_add(carbon_footprint);
        let total_offset = storage::get_total_carbon_offset(&env, &depositor).saturating_add(carbon_offset);
        storage::set_total_carbon_footprint(&env, &depositor, total_carbon);
        storage::set_total_carbon_offset(&env, &depositor, total_offset);
        
        // Check for sustainability milestones
        let deposit_ids = storage::get_deposit_ids(&env, &depositor);
        let mut total_renewable: u64 = 0;
        for id in deposit_ids.iter() {
            if let Some(m) = storage::get_sustainability_metrics_readonly(&env, &depositor, id) {
                total_renewable = total_renewable.saturating_add(m.renewable_energy_percent as u64);
            }
        }
        let average_renewable = if deposit_ids.len() > 0 {
            ((total_renewable / deposit_ids.len() as u64) as u32).min(100)
        } else {
            0
        };
        check_sustainability_milestones(&env, &depositor, total_carbon, total_offset, average_renewable);

        Ok(deposit_id)
    }

    pub fn deposit_with_delay(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
        withdrawal_delay_secs: u64,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();
        storage::require_permission(&env, &depositor, PermissionType::Deposit)?;

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }
        check_tax_wash_sale(&env, &depositor, &token)?;
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
        let lock_duration = unlock_time.saturating_sub(now);
        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);
        token::Client::new(&env, &token).transfer(
            &depositor,
            &env.current_contract_address(),
            &amount,
        );

        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
            withdrawal_delay_secs,
        };
        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Core: Deposit by Ledger Sequence
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
        storage::require_permission(&env, &depositor, PermissionType::Deposit)?;

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if storage::is_strict_token_allowlist(&env) && !storage::is_token_allowed(&env, &token) {
            return Err(VaultError::TokenNotAllowed);
        }
        check_tax_wash_sale(&env, &depositor, &token)?;

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

        let ledger_gap = unlock_ledger.saturating_sub(current_ledger);
        if ledger_gap < MIN_LOCK_LEDGERS {
            return Err(VaultError::LockDurationTooShort);
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

        // Track sustainability metrics (convert ledger gap to estimated seconds)
        let estimated_duration_secs = (ledger_gap as u64).saturating_mul(storage::LEDGER_SECONDS);
        let carbon_footprint = calculate_carbon_footprint(amount, estimated_duration_secs);
        let carbon_offset = calculate_carbon_offset(carbon_footprint, penalty_bps);
        
        let metrics = SustainabilityMetrics {
            deposit_id,
            depositor: depositor.clone(),
            carbon_footprint,
            renewable_energy_percent: crate::constants::RENEWABLE_ENERGY_BASELINE,
            carbon_offset_grams: carbon_offset,
            timestamp: env.ledger().timestamp(),
        };
        
        storage::set_sustainability_metrics(&env, &depositor, deposit_id, &metrics);
        
        // Update aggregated totals
        let total_carbon = storage::get_total_carbon_footprint(&env, &depositor).saturating_add(carbon_footprint);
        let total_offset = storage::get_total_carbon_offset(&env, &depositor).saturating_add(carbon_offset);
        storage::set_total_carbon_footprint(&env, &depositor, total_carbon);
        storage::set_total_carbon_offset(&env, &depositor, total_offset);
        
        // Check for sustainability milestones
        let deposit_ids = storage::get_deposit_ids(&env, &depositor);
        let mut total_renewable: u64 = 0;
        for id in deposit_ids.iter() {
            if let Some(m) = storage::get_sustainability_metrics_readonly(&env, &depositor, id) {
                total_renewable = total_renewable.saturating_add(m.renewable_energy_percent as u64);
            }
        }
        let average_renewable = if deposit_ids.len() > 0 {
            ((total_renewable / deposit_ids.len() as u64) as u32).min(100)
        } else {
            0
        };
        check_sustainability_milestones(&env, &depositor, total_carbon, total_offset, average_renewable);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  #330 — Multi-token Deposit
    // ----------------------------------------------------------------

    /// Deposit multiple tokens in a single vault entry.
    ///
    /// - `tokens_and_amounts`: list of `(token, amount)` pairs, length 1–MAX_TOKENS_PER_DEPOSIT.
    /// - Each individual `amount` is validated (> 0, ≤ max_deposit).
    /// - All tokens are transferred from `depositor` to the contract atomically.
    /// - Returns the new deposit ID (shared counter with single-token deposits).
    pub fn multi_deposit(
        env: Env,
        depositor: Address,
        tokens_and_amounts: Vec<TokenDeposit>,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();
        storage::require_permission(&env, &depositor, PermissionType::Deposit)?;

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if storage::is_emergency_lockdown(&env) {
            return Err(VaultError::EmergencyLockdown);
        }

        let count = tokens_and_amounts.len();
        if count == 0 {
            return Err(VaultError::EmptyTokenList);
        }
        if count > MAX_TOKENS_PER_DEPOSIT {
            return Err(VaultError::TooManyTokens);
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

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);

        // Validate all amounts before transferring anything (checks before effects).
        for td in tokens_and_amounts.iter() {
            if td.amount <= 0 {
                return Err(VaultError::InvalidAmount);
            }
            if td.amount > max_deposit {
                return Err(VaultError::AmountTooLarge);
            }
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);
        let contract = env.current_contract_address();

        // Transfer all tokens into the contract.
        for td in tokens_and_amounts.iter() {
            let token_client = token::Client::new(&env, &td.token);
            token_client.transfer(&depositor, &contract, &td.amount);
        }

        let entry = MultiTokenVaultEntry {
            tokens: tokens_and_amounts.clone(),
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
            compound_frequency_secs: 0,
            last_accrual_timestamp: now,
        };

        storage::set_multi_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::multi_deposit(&env, &depositor, count, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    /// Query a multi-token vault entry.  Returns `None` if no multi-token deposit
    /// exists at the given `(depositor, deposit_id)`.
    pub fn get_multi_vault(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<MultiTokenVaultEntry> {
        storage::get_multi_deposit_readonly(&env, &depositor, deposit_id)
    }

    // ----------------------------------------------------------------
    //  #331 — Withdrawal Whitelist
    // ----------------------------------------------------------------

    /// Set (or replace) the withdrawal whitelist for an existing deposit.
    /// Only the depositor themselves may call this.
    ///
    /// - An empty `addresses` vec means "no restriction" (anyone may receive).
    /// - Calling this again replaces the previous whitelist entirely.
    pub fn set_withdrawal_whitelist(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        addresses: Vec<Address>,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Verify the deposit exists (timestamp-based, ledger-based, or multi-token).
        let deposit_exists =
            storage::get_deposit_readonly(&env, &depositor, deposit_id).is_some()
                || storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some()
                || storage::get_multi_deposit_readonly(&env, &depositor, deposit_id).is_some();

        if !deposit_exists {
            return Err(VaultError::NoDepositFound);
        }

        storage::set_withdrawal_whitelist(&env, &depositor, deposit_id, &addresses);
        events::whitelist_set(&env, &depositor, deposit_id, &addresses);

        Ok(())
    }

    /// Return the current withdrawal whitelist for `(depositor, deposit_id)`.
    /// Returns `None` if no whitelist has been configured (= no restriction).
    pub fn get_withdrawal_whitelist(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<Vec<Address>> {
        storage::get_withdrawal_whitelist(&env, &depositor, deposit_id)
    }

    /// Link a verified W3C DID credential to a deposit.
    ///
    /// The verifier must authorize this call, proving that it attested the
    /// credential. Only the SHA-256 DID commitment and credential commitment
    /// are stored; raw identity claims never enter contract storage or events.
    /// `did:key` and `did:ethr` are supported DID method identifiers.
    pub fn link_identity(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        did: String,
        credential_commitment: soroban_sdk::BytesN<32>,
        verifier: Address,
    ) -> Result<(), VaultError> {
        depositor.require_auth();
        verifier.require_auth();

        let did_bytes = did.to_bytes();
        let did_method = if did_bytes.len() > 7
            && did_bytes.get(0) == Some(b'd')
            && did_bytes.get(1) == Some(b'i')
            && did_bytes.get(2) == Some(b'd')
            && did_bytes.get(3) == Some(b':')
            && did_bytes.get(4) == Some(b'k')
            && did_bytes.get(5) == Some(b'e')
            && did_bytes.get(6) == Some(b'y')
        {
            String::from_slice(&env, "did:key")
        } else if did_bytes.len() > 8
            && did_bytes.get(0) == Some(b'd')
            && did_bytes.get(1) == Some(b'i')
            && did_bytes.get(2) == Some(b'd')
            && did_bytes.get(3) == Some(b':')
            && did_bytes.get(4) == Some(b'e')
            && did_bytes.get(5) == Some(b't')
            && did_bytes.get(6) == Some(b'h')
            && did_bytes.get(7) == Some(b'r')
        {
            String::from_slice(&env, "did:ethr")
        } else {
            return Err(VaultError::UnsupportedDidMethod);
        };

        let deposit_exists = storage::get_deposit_readonly(&env, &depositor, deposit_id).is_some()
            || storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some()
            || storage::get_multi_deposit_readonly(&env, &depositor, deposit_id).is_some();
        if !deposit_exists {
            return Err(VaultError::NoDepositFound);
        }

        let identity = IdentityLink {
            did_method: did_method.clone(),
            did_commitment: env.crypto().sha256(&did_bytes),
            credential_commitment,
            verifier: verifier.clone(),
            verified_at: env.ledger().timestamp(),
        };
        storage::set_identity(&env, &depositor, deposit_id, &identity);
        let event_method = if did_method == String::from_slice(&env, "did:key") {
            Symbol::new(&env, "did_key")
        } else {
            Symbol::new(&env, "did_ethr")
        };
        events::identity_linked(&env, &depositor, deposit_id, &event_method, &verifier);
        Ok(())
    }

    pub fn get_identity(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<IdentityLink> {
        storage::get_identity(&env, &depositor, deposit_id)
    }

    // ----------------------------------------------------------------
    //  #332 — Compound Interest Deposit
    // ----------------------------------------------------------------

    /// Like `deposit` but with an explicit compound-interest frequency.
    ///
    /// - `compound_frequency_secs`: interval between compounding events in seconds.
    ///   Must be ≥ 60 if non-zero; 0 disables compounding (equivalent to plain deposit).
    /// - Interest rate is fixed at `ANNUAL_INTEREST_BPS` (500 bps = 5% p.a.).
    pub fn deposit_with_compound_interest(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
        compound_frequency_secs: u64,
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

        // Validate compound frequency: 0 = disabled; otherwise ≥ 60s.
        if compound_frequency_secs != 0 && compound_frequency_secs < MIN_COMPOUND_FREQUENCY_SECS {
            return Err(VaultError::InvalidCompoundFrequency);
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
            compound_frequency_secs,
            last_accrual_timestamp: now,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    /// Trigger compound-interest accrual for a deposit.
    ///
    /// Accrues all complete compound periods elapsed since `last_accrual_timestamp`
    /// and updates the stored `amount` and `last_accrual_timestamp` in-place.
    /// If fewer than one period has elapsed, this is a no-op (returns `Ok(false)`).
    ///
    /// Returns `Ok(true)` if accrual happened, `Ok(false)` if nothing changed,
    /// `Err(NoDepositFound)` if no deposit exists.
    pub fn update_accrual(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Result<bool, VaultError> {
        // No auth required — anyone can trigger accrual (it only benefits the depositor).

        let mut entry = storage::get_deposit(&env, &depositor, deposit_id)
            .ok_or(VaultError::NoDepositFound)?;

        if entry.compound_frequency_secs == 0 {
            // Compounding not enabled for this deposit.
            return Ok(false);
        }

        let now = env.ledger().timestamp();
        let new_amount =
            compute_accrued_amount(entry.amount, entry.compound_frequency_secs, entry.last_accrual_timestamp, now);

        if new_amount == entry.amount {
            return Ok(false);
        }

        let old_amount = entry.amount;

        // Advance last_accrual_timestamp by the number of complete periods that were applied.
        let elapsed = now.saturating_sub(entry.last_accrual_timestamp);
        let periods = elapsed / entry.compound_frequency_secs;
        entry.last_accrual_timestamp = entry
            .last_accrual_timestamp
            .saturating_add(periods.saturating_mul(entry.compound_frequency_secs));
        entry.amount = new_amount;

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        events::interest_accrued(&env, &depositor, deposit_id, old_amount, new_amount);

        Ok(true)
    }

    /// Returns the current balance of a deposit including any unaccrued compound
    /// interest (i.e. the value `withdraw` would release if called right now).
    ///
    /// This is a read-only query — it does NOT update storage.
    pub fn get_current_balance(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<i128> {
        let entry = storage::get_deposit_readonly(&env, &depositor, deposit_id)?;
        let now = env.ledger().timestamp();
        let balance = compute_accrued_amount(
            entry.amount,
            entry.compound_frequency_secs,
            entry.last_accrual_timestamp,
            now,
        );
        Some(balance)
    }

    // ----------------------------------------------------------------
    //  Core: Cancel Deposit (early exit with penalty)
    // ----------------------------------------------------------------

    pub fn cancel_deposit(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first (accrue interest before calculating refund).
        if let Some(mut entry) = storage::get_deposit(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now >= entry.unlock_time {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            // Accrue interest before computing refund (so penalty is taken from current balance).
            if entry.compound_frequency_secs > 0 {
                let new_amount = compute_accrued_amount(
                    entry.amount,
                    entry.compound_frequency_secs,
                    entry.last_accrual_timestamp,
                    now,
                );
                entry.amount = new_amount;
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
            let refund = entry.amount.saturating_sub(penalty);

            // Split penalty: fee_recipient gets FEE_RECIPIENT_PENALTY_BPS, stakers get STAKER_PENALTY_BPS
            let fee_recipient_share: i128 = (penalty * FEE_RECIPIENT_PENALTY_BPS as i128) / 10_000;
            let stakers_share: i128 = penalty - fee_recipient_share;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                if fee_recipient_share > 0 {
                    token_client.transfer(&contract, &fee_recipient, &fee_recipient_share);
                }
                // Add stakers_share to rewards pool
                if stakers_share > 0 {
                    let current_pool = storage::get_rewards_pool(&env);
                    storage::set_rewards_pool(&env, current_pool + stakers_share);
                }
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            
            // Clean up NFT evolution record on cancellation
            storage::remove_nft_evolution(&env, &depositor, deposit_id);
            storage::remove_sustainability_metrics(&env, &depositor, deposit_id);
            events::penalty_split(&env, &depositor, penalty, fee_recipient_share, stakers_share, deposit_id);
            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit.
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger >= entry.unlock_ledger {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
            let refund = entry.amount.saturating_sub(penalty);

            // Split penalty: fee_recipient gets FEE_RECIPIENT_PENALTY_BPS, stakers get STAKER_PENALTY_BPS
            let fee_recipient_share: i128 = (penalty * FEE_RECIPIENT_PENALTY_BPS as i128) / 10_000;
            let stakers_share: i128 = penalty - fee_recipient_share;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                if fee_recipient_share > 0 {
                    token_client.transfer(&contract, &fee_recipient, &fee_recipient_share);
                }
                // Add stakers_share to rewards pool
                if stakers_share > 0 {
                    let current_pool = storage::get_rewards_pool(&env);
                    storage::set_rewards_pool(&env, current_pool + stakers_share);
                }
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            events::penalty_split(&env, &depositor, penalty, fee_recipient_share, stakers_share, deposit_id);
            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        // Try multi-token deposit.
        if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now >= entry.unlock_time {
                return Err(VaultError::VaultAlreadyUnlocked);
            }
            let withdrawal_time = entry.unlock_time.saturating_add(entry.withdrawal_delay_secs);
            if now < withdrawal_time {
                return Err(VaultError::WithdrawalDelayActive);
            }

            storage::remove_multi_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let contract = env.current_contract_address();
            for td in entry.tokens.iter() {
                let penalty: i128 = (td.amount * entry.penalty_bps as i128) / 10_000;
                let refund = td.amount.saturating_sub(penalty);

                let token_client = token::Client::new(&env, &td.token);
                if penalty > 0 {
                    let fee_recipient =
                        storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                    token_client.transfer(&contract, &fee_recipient, &penalty);
                }
                if refund > 0 {
                    token_client.transfer(&contract, &depositor, &refund);
                }
                events::deposit_cancelled(&env, &depositor, &td.token, td.amount, penalty, deposit_id);
            }
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Staker Registry Functions
    // ----------------------------------------------------------------

    /// Register a staker with a stake amount. Updates their stake if already registered.
    pub fn register_staker(env: Env, staker: Address, amount: i128) -> Result<(), VaultError> {
        staker.require_auth();

        if amount <= 0 {
            return Err(VaultError::InvalidStakeAmount);
        }

        // Get the current total staked
        let current_total = storage::get_total_staked(&env);
        let old_stake = storage::get_staker(&env, &staker).unwrap_or(0);

        // Update staker's stake amount
        storage::set_staker(&env, &staker, amount);

        // Update total staked
        let new_total = current_total - old_stake + amount;
        storage::set_total_staked(&env, new_total);

        // Add to staker list if first-time registration
        if old_stake == 0 {
            storage::add_staker_to_list(&env, &staker);
        }

        events::staker_registered(&env, &staker, amount);
        Ok(())
    }

    /// Claim rewards for the caller. Calculates their share based on stake proportion.
    pub fn claim_staker_rewards(env: Env, staker: Address) -> Result<(), VaultError> {
        staker.require_auth();

        // Get staker's stake amount
        let stake_amount = storage::get_staker(&env, &staker)
            .ok_or(VaultError::StakerNotFound)?;

        if stake_amount <= 0 {
            return Err(VaultError::InvalidStakeAmount);
        }

        // Get current rewards pool and total staked
        let rewards_pool = storage::get_rewards_pool(&env);
        let total_staked = storage::get_total_staked(&env);

        if rewards_pool <= 0 {
            return Err(VaultError::NoRewardsToClaim);
        }

        if total_staked <= 0 {
            return Err(VaultError::NoRewardsToClaim);
        }

        // Calculate staker's proportional share
        // reward = (stake_amount / total_staked) * rewards_pool
        let reward = (stake_amount * rewards_pool) / total_staked;

        if reward <= 0 {
            return Err(VaultError::NoRewardsToClaim);
        }

        // Track total claimed
        let already_claimed = storage::get_staker_rewards_claimed(&env, &staker);
        let new_claimed = already_claimed + reward;
        storage::set_staker_rewards_claimed(&env, &staker, new_claimed);

        // Reduce rewards pool
        let new_pool = rewards_pool - reward;
        storage::set_rewards_pool(&env, new_pool);

        events::rewards_claimed(&env, &staker, reward);
        Ok(())
    }

    // ================================================================
    //  MEV PROTECTION: COMMIT-REVEAL SCHEME
    // ================================================================

    /// Submit a private commit for a deposit to enable MEV protection.
    /// The commit hash is Keccak256(token || amount || price || nonce).
    /// Depositor must reveal within MEV_REVEAL_WINDOW_SECS.
    pub fn mev_commit(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        commit_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Verify deposit exists
        if storage::get_deposit_readonly(&env, &depositor, deposit_id).is_none() {
            return Err(VaultError::NoDepositFound);
        }

        let now = env.ledger().timestamp();
        let commitment = MEVCommitment {
            commit_hash: commit_hash.clone(),
            timestamp: now,
            depositor: depositor.clone(),
            revealed: false,
        };

        storage::set_mev_commitment(&env, &depositor, deposit_id, &commitment);
        storage::set_mev_status(&env, &depositor, deposit_id, &MEVStatus::Committed);

        events::commit_submitted(&env, &depositor, deposit_id, &commit_hash);

        Ok(())
    }

    /// Reveal a MEV commitment with actual transaction details.
    /// Verifies the reveal matches the commit hash and checks for MEV attacks.
    pub fn mev_reveal(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        token: Address,
        amount: i128,
        price: i128,
        nonce: u32,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Get the commitment
        let commitment = storage::get_mev_commitment(&env, &depositor, deposit_id)
            .ok_or(VaultError::CommitNotFound)?;

        // Check reveal window (30 minutes)
        let now = env.ledger().timestamp();
        if now > commitment.timestamp + constants::MEV_REVEAL_WINDOW_SECS {
            return Err(VaultError::RevealWindowExpired);
        }

        // Verify the reveal hash matches the commitment
        let reveal_hash = compute_commit_hash(&env, &token, amount, price, nonce);
        if reveal_hash != commitment.commit_hash {
            return Err(VaultError::CommitMismatch);
        }

        // Record the revealed price
        let price_sample = PriceSample {
            token: token.clone(),
            price,
            timestamp: now,
        };
        storage::add_price_sample(&env, &token, &price_sample);

        // Detect MEV attacks via price deviation
        detect_mev_attack(&env, &depositor, deposit_id, &token, price, now)?;

        // Mark commitment as revealed
        let mut commitment = commitment;
        commitment.revealed = true;
        storage::set_mev_commitment(&env, &depositor, deposit_id, &commitment);
        storage::set_mev_status(&env, &depositor, deposit_id, &MEVStatus::Revealed);

        events::reveal_submitted(&env, &depositor, deposit_id, &token, amount, price);

        Ok(())
    }

    /// Claim MEV recovered for a depositor from the MEV pool.
    pub fn claim_mev_recovery(env: Env, depositor: Address) -> Result<i128, VaultError> {
        depositor.require_auth();

        let mev_recovered = storage::get_mev_claimed(&env, &depositor);
        if mev_recovered <= 0 {
            return Err(VaultError::NoRewardsToClaim);
        }

        // Deduct from MEV pool
        let current_pool = storage::get_mev_pool(&env);
        let new_pool = current_pool.saturating_sub(mev_recovered);
        storage::set_mev_pool(&env, new_pool);

        // Reset claimed amount
        storage::set_mev_claimed(&env, &depositor, 0);

        Ok(mev_recovered)
    }

    /// Query detected MEV attacks for a deposit
    pub fn get_mev_detections(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<MEVDetection>, VaultError> {
        let limit = limit.min(50); // Cap at 50 results
        let max_id = storage::get_mev_detection_counter(&env, &depositor, deposit_id);

        let mut detections = Vec::new(&env);
        for i in offset..offset.saturating_add(limit) {
            if i >= max_id {
                break;
            }
            if let Some(detection) = storage::get_mev_detection(&env, &depositor, deposit_id, i) {
                detections.push_back(detection);
            }
        }

        Ok(detections)
    }

    /// Query MEV status for a deposit
    pub fn get_mev_status_query(env: Env, depositor: Address, deposit_id: u32) -> MEVStatus {
        storage::get_mev_status(&env, &depositor, deposit_id)
    }

    /// Query MEV recovered (pending claim) for a depositor
    pub fn get_mev_pending(env: Env, depositor: Address) -> i128 {
        storage::get_mev_claimed(&env, &depositor)
    }

    /// Query total MEV pool (recovered and pending redistribution)
    pub fn get_mev_pool_total(env: Env) -> i128 {
        storage::get_mev_pool(&env)
    }

    /// Admin function: finalize MEV redistribution across all affected deposits.
    /// This is called periodically to process accumulated MEV and redistribute
    /// it to depositors who had detected attacks.
    pub fn finalize_mev_redistribution(env: Env, admin: Address) -> Result<i128, VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        let current_pool = storage::get_mev_pool(&env);
        if current_pool <= 0 {
            return Ok(0);
        }

        // For simplicity, the pool is not automatically distributed here.
        // Depositors claim their rewards individually via claim_mev_recovery().
        // This function simply returns the current pool amount for monitoring.
        Ok(current_pool)
    }

    // ================================================================
    //  MEV DETECTION HELPERS (internal)
    // ================================================================

    // ----------------------------------------------------------------
    //  Core: Withdraw
    // ----------------------------------------------------------------

    pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        if storage::is_emergency_lockdown(&env) {
            return Err(VaultError::EmergencyLockdown);
        }

        // Try timestamp-based deposit first.
        if let Some(mut entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            // Accrue any outstanding interest before computing the final payout.
            if entry.compound_frequency_secs > 0 {
                entry.amount = compute_accrued_amount(
                    entry.amount,
                    entry.compound_frequency_secs,
                    entry.last_accrual_timestamp,
                    now,
                );
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            // Increment withdrawal count and clean up old epochs
            let current_epoch = storage::get_current_epoch(&env);
            storage::increment_withdrawal_count(&env, &depositor, current_epoch);
            storage::cleanup_old_epochs(&env, &depositor, current_epoch);

            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);

            // Clean up NFT evolution record on withdrawal
            storage::remove_nft_evolution(&env, &depositor, deposit_id);
            storage::remove_sustainability_metrics(&env, &depositor, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit.
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            // Increment withdrawal count and clean up old epochs
            let current_epoch = storage::get_current_epoch(&env);
            storage::increment_withdrawal_count(&env, &depositor, current_epoch);
            storage::cleanup_old_epochs(&env, &depositor, current_epoch);


            // Clean up NFT evolution record on withdrawal
            storage::remove_nft_evolution(&env, &depositor, deposit_id);
            storage::remove_sustainability_metrics(&env, &depositor, deposit_id);
            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        // Try multi-token deposit.
        if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_multi_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let contract = env.current_contract_address();
            let token_count = entry.tokens.len();
            for td in entry.tokens.iter() {
                let token_client = token::Client::new(&env, &td.token);
                token_client.transfer(&contract, &depositor, &td.amount);
                events::withdraw(&env, &depositor, &td.token, td.amount, deposit_id);
            }
            events::multi_withdraw(&env, &depositor, &depositor, deposit_id, token_count);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    /// Realize an explicit loss and replace the position with a non-identical token.
    /// `current_value` and `tax_rate_bps` are supplied by the caller because this
    /// contract has no price oracle and does not provide tax advice.
    pub fn harvest_tax_losses(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        current_value: i128,
        replacement_token: Address,
        replacement_amount: i128,
        tax_rate_bps: u32,
        wash_sale_period_secs: u64,
    ) -> Result<TaxLossHarvest, VaultError> {
        depositor.require_auth();

        if current_value <= 0 || replacement_amount <= 0 || tax_rate_bps > 10_000 {
            return Err(VaultError::InvalidTaxLossHarvest);
        }
        if storage::is_strict_token_allowlist(&env)
            && !storage::is_token_allowed(&env, &replacement_token)
        {
            return Err(VaultError::ReplacementTokenNotAllowed);
        }

        let mut entry = storage::get_deposit(&env, &depositor, deposit_id)
            .ok_or(VaultError::NoDepositFound)?;
        if entry.token == replacement_token {
            return Err(VaultError::InvalidTaxLossHarvest);
        }

        let now = env.ledger().timestamp();
        entry.amount = compute_accrued_amount(
            entry.amount,
            entry.compound_frequency_secs,
            entry.last_accrual_timestamp,
            now,
        );
        if current_value >= entry.amount {
            return Err(VaultError::InvalidTaxLossHarvest);
        }

        let original_token = entry.token.clone();
        let cost_basis = entry.amount;
        let realized_loss = cost_basis.saturating_sub(current_value);
        let tax_benefit = realized_loss.saturating_mul(tax_rate_bps as i128) / 10_000;
        let wash_sale_until = now.saturating_add(wash_sale_period_secs);
        let replacement_unlock = entry.unlock_time.max(wash_sale_until);
        let replacement_deposit_id = storage::next_deposit_id(&env, &depositor);

        let contract = env.current_contract_address();
        token::Client::new(&env, &original_token).transfer(&contract, &depositor, &cost_basis);
        token::Client::new(&env, &replacement_token).transfer(
            &depositor,
            &contract,
            &replacement_amount,
        );

        storage::remove_deposit(&env, &depositor, deposit_id);
        storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
        storage::set_deposit(
            &env,
            &depositor,
            replacement_deposit_id,
            &VaultEntry {
                token: replacement_token.clone(),
                amount: replacement_amount,
                unlock_time: replacement_unlock,
                depositor: depositor.clone(),
                penalty_bps: entry.penalty_bps,
                compound_frequency_secs: entry.compound_frequency_secs,
                last_accrual_timestamp: now,
            },
        );
        storage::add_depositor(&env, &depositor);

        let harvest = TaxLossHarvest {
            depositor: depositor.clone(),
            original_token: original_token.clone(),
            replacement_token: replacement_token.clone(),
            original_deposit_id: deposit_id,
            replacement_deposit_id,
            cost_basis,
            current_value,
            realized_loss,
            tax_benefit,
            harvested_at: now,
            wash_sale_until,
        };
        storage::add_tax_loss_harvest(&env, &depositor, &harvest);
        storage::set_tax_wash_sale_until(&env, &depositor, &original_token, wash_sale_until);
        events::tax_loss_harvested(
            &env,
            &depositor,
            &original_token,
            &replacement_token,
            realized_loss,
            tax_benefit,
            deposit_id,
            replacement_deposit_id,
            wash_sale_until,
        );
        Ok(harvest)
    }

    /// Withdraw to a specific recipient address.
    ///
    /// If a whitelist has been set for this deposit, `recipient` must be in it.
    pub fn withdraw_to(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        recipient: Address,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        if storage::is_emergency_lockdown(&env) {
            return Err(VaultError::EmergencyLockdown);
        }

        // Try timestamp-based deposit first.
        if let Some(mut entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }
            let withdrawal_time = entry.unlock_time.saturating_add(entry.withdrawal_delay_secs);
            if now < withdrawal_time {
                return Err(VaultError::WithdrawalDelayActive);
            }

            // #331: enforce whitelist.
            check_whitelist(&env, &depositor, deposit_id, &recipient)?;

            // Accrue interest before payout.
            if entry.compound_frequency_secs > 0 {
                entry.amount = compute_accrued_amount(
                    entry.amount,
                    entry.compound_frequency_secs,
                    entry.last_accrual_timestamp,
                    now,
                );
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            // Increment withdrawal count and clean up old epochs
            let current_epoch = storage::get_current_epoch(&env);
            storage::increment_withdrawal_count(&env, &depositor, current_epoch);
            storage::cleanup_old_epochs(&env, &depositor, current_epoch);

            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        // Try ledger-based deposit.
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            // #331: enforce whitelist.
            check_whitelist(&env, &depositor, deposit_id, &recipient)?;

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            // Increment withdrawal count and clean up old epochs
            let current_epoch = storage::get_current_epoch(&env);
            storage::increment_withdrawal_count(&env, &depositor, current_epoch);
            storage::cleanup_old_epochs(&env, &depositor, current_epoch);

            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        // Try multi-token deposit.
        if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            // #331: enforce whitelist.
            check_whitelist(&env, &depositor, deposit_id, &recipient)?;

            storage::remove_multi_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let contract = env.current_contract_address();
            let token_count = entry.tokens.len();
            for td in entry.tokens.iter() {
                let token_client = token::Client::new(&env, &td.token);
                token_client.transfer(&contract, &recipient, &td.amount);
                events::withdraw_to(&env, &depositor, &recipient, &td.token, td.amount);
            }
            events::multi_withdraw(&env, &depositor, &recipient, deposit_id, token_count);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Batch Withdrawal
    // ----------------------------------------------------------------

    /// Withdraw from multiple deposits in a single transaction.
    ///
    /// Attempts to withdraw from each deposit ID in the provided vector.
    /// For each deposit, performs the same checks as a single `withdraw()` call:
    /// - Verifies lock time/ledger has passed
    /// - Accrues any outstanding compound interest
    /// - Removes deposit from storage
    /// - Transfers tokens to depositor
    ///
    /// Deposits that cannot be withdrawn (locked, not found, etc.) are recorded
    /// as failed in the result. This allows partial success: some deposits may
    /// withdraw while others fail.
    ///
    /// # Parameters
    /// - `depositor`: Account owner of the deposits (must sign the transaction)
    /// - `deposit_ids`: Vector of deposit IDs to attempt withdrawal from
    ///
    /// # Returns
    /// A `BatchWithdrawalResult` containing:
    /// - Individual `WithdrawalResult` for each deposit (success/failure, error code, amount)
    /// - Aggregate counts and total amount withdrawn
    ///
    /// # Constraints
    /// - `deposit_ids.len()` must be ≤ `MAX_BATCH_SIZE` (25)
    /// - Deposits exceeding `MAX_BATCH_SIZE` are rejected
    ///
    /// # Events
    /// - Emits individual `withdraw` events for each successful withdrawal
    /// - Emits a single `batch_withdraw` event with aggregate results
    pub fn withdraw_batch(
        env: Env,
        depositor: Address,
        deposit_ids: Vec<u32>,
    ) -> Result<BatchWithdrawalResult, VaultError> {
        depositor.require_auth();

        if storage::is_emergency_lockdown(&env) {
            return Err(VaultError::EmergencyLockdown);
        }

        // Check batch size limit
        if deposit_ids.len() as u32 > MAX_BATCH_SIZE {
            return Err(VaultError::BatchSizeExceeded);
        }

        let mut results: Vec<WithdrawalResult> = Vec::new(&env);
        let mut successful_count: u32 = 0;
        let mut failed_count: u32 = 0;
        let mut total_amount: i128 = 0;

        let total_attempted = deposit_ids.len() as u32;
        let now = env.ledger().timestamp();
        let current_ledger = env.ledger().sequence();
        let contract_address = env.current_contract_address();

        // Iterate through each deposit ID
        for deposit_id in deposit_ids.iter() {
            let mut success = false;
            let mut error_code: u32 = 0;
            let mut amount: i128 = 0;

            // Try timestamp-based deposit first
            if let Some(mut entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
                if now < entry.unlock_time {
                    error_code = VaultError::FundsStillLocked as u32;
                } else {
                    // Accrue any outstanding interest before computing the final payout
                    if entry.compound_frequency_secs > 0 {
                        entry.amount = compute_accrued_amount(
                            entry.amount,
                            entry.compound_frequency_secs,
                            entry.last_accrual_timestamp,
                            now,
                        );
                    }

                    amount = entry.amount;
                    total_amount = total_amount.saturating_add(amount);

                    // Remove deposit from storage
                    storage::remove_deposit(&env, &depositor, deposit_id);
                    storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
                    if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                        storage::remove_depositor(&env, &depositor);
                    }

                    // Transfer tokens
                    let token_client = token::Client::new(&env, &entry.token);
                    token_client.transfer(&contract_address, &depositor, &amount);

                    // Increment withdrawal count and clean up old epochs
                    let current_epoch = storage::get_current_epoch(&env);
                    storage::increment_withdrawal_count(&env, &depositor, current_epoch);
                    storage::cleanup_old_epochs(&env, &depositor, current_epoch);

                    // Emit individual withdrawal event
                    events::withdraw(&env, &depositor, &entry.token, amount, deposit_id);

                    // Clean up NFT evolution record and sustainability metrics
                    storage::remove_nft_evolution(&env, &depositor, deposit_id);
                    storage::remove_sustainability_metrics(&env, &depositor, deposit_id);

                    success = true;
                }
            } else if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
                // Try ledger-based deposit
                if current_ledger < entry.unlock_ledger {
                    error_code = VaultError::FundsStillLocked as u32;
                } else {
                    amount = entry.amount;
                    total_amount = total_amount.saturating_add(amount);

                    // Remove deposit from storage
                    storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
                    storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
                    if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                        storage::remove_depositor(&env, &depositor);
                    }

                    // Transfer tokens
                    let token_client = token::Client::new(&env, &entry.token);
                    token_client.transfer(&contract_address, &depositor, &amount);

                    // Increment withdrawal count and clean up old epochs
                    let current_epoch = storage::get_current_epoch(&env);
                    storage::increment_withdrawal_count(&env, &depositor, current_epoch);
                    storage::cleanup_old_epochs(&env, &depositor, current_epoch);

                    // Emit individual withdrawal event
                    events::withdraw(&env, &depositor, &entry.token, amount, deposit_id);

                    // Clean up NFT evolution record and sustainability metrics
                    storage::remove_nft_evolution(&env, &depositor, deposit_id);
                    storage::remove_sustainability_metrics(&env, &depositor, deposit_id);

                    success = true;
                }
            } else if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
                // Try multi-token deposit
                if now < entry.unlock_time {
                    error_code = VaultError::FundsStillLocked as u32;
                } else {
                    let token_count = entry.tokens.len();
                    
                    // Remove deposit from storage
                    storage::remove_multi_deposit(&env, &depositor, deposit_id);
                    storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
                    if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                        storage::remove_depositor(&env, &depositor);
                    }

                    // Transfer all tokens
                    for td in entry.tokens.iter() {
                        let token_client = token::Client::new(&env, &td.token);
                        token_client.transfer(&contract_address, &depositor, &td.amount);
                        amount = amount.saturating_add(td.amount);
                        events::withdraw(&env, &depositor, &td.token, td.amount, deposit_id);
                    }

                    total_amount = total_amount.saturating_add(amount);

                    // Increment withdrawal count and clean up old epochs
                    let current_epoch = storage::get_current_epoch(&env);
                    storage::increment_withdrawal_count(&env, &depositor, current_epoch);
                    storage::cleanup_old_epochs(&env, &depositor, current_epoch);

                    events::multi_withdraw(&env, &depositor, &depositor, deposit_id, token_count);

                    success = true;
                }
            } else {
                // Deposit not found
                error_code = VaultError::NoDepositFound as u32;
            }

            // Record result
            let result = WithdrawalResult {
                deposit_id,
                success,
                error_code,
                amount,
            };
            results.push(result);

            if success {
                successful_count += 1;
            } else {
                failed_count += 1;
            }
        }

        // Emit batch withdrawal event
        events::batch_withdraw(&env, &depositor, successful_count, failed_count, total_amount);

        Ok(BatchWithdrawalResult {
            results,
            total_attempted,
            successful_count,
            failed_count,
            total_amount,
        })
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

        // Try timestamp-based deposit first.
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            check_emergency_withdrawal_limit(&env, entry.amount)?;
            storage::remove_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);
            record_emergency_withdrawal(&env, entry.amount);

            events::emergency_withdraw(
                &env,
                &admin,
                &depositor,
                &entry.token,
                entry.amount,
                deposit_id,
            );
            return Ok(());
        }

        // Try ledger-based deposit.
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            check_emergency_withdrawal_limit(&env, entry.amount)?;
            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);
            record_emergency_withdrawal(&env, entry.amount);

            events::emergency_withdraw(
                &env,
                &admin,
                &depositor,
                &entry.token,
                entry.amount,
                deposit_id,
            );
            return Ok(());
        }

        // Try multi-token deposit.
        if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
            let total = entry
                .tokens
                .iter()
                .fold(0i128, |total, token| total.saturating_add(token.amount));
            check_emergency_withdrawal_limit(&env, total)?;
            storage::remove_multi_deposit(&env, &depositor, deposit_id);
            storage::remove_withdrawal_whitelist(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let contract = env.current_contract_address();
            for td in entry.tokens.iter() {
                let token_client = token::Client::new(&env, &td.token);
                token_client.transfer(&contract, &depositor, &td.amount);
                events::emergency_withdraw(&env, &admin, &depositor, &td.token, td.amount, deposit_id);
            }
            record_emergency_withdrawal(&env, total);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Archival: Archive and Delete Deposits
    // ----------------------------------------------------------------

    /// Archive a completed (withdrawn or cancelled) timestamp-based deposit for record-keeping.
    /// The depositor manually archives a deposit entry that has been withdrawn or cancelled,
    /// storing it with the current archive timestamp.
    ///
    /// # Preconditions
    /// - Deposit must NOT exist in active storage (must have been withdrawn or cancelled already)
    /// - Caller must provide the original deposit entry data
    ///
    /// # Parameters
    /// - `depositor`: The original deposit owner
    /// - `deposit_id`: The ID of the deposit to archive
    /// - `token`: Token address (must match the original deposit)
    /// - `amount`: Amount (must match the original deposit)
    /// - `unlock_time`: Unlock time (must match the original deposit)
    /// - `penalty_bps`: Penalty basis points (must match the original deposit)
    ///
    /// # Returns
    /// - `Ok(())` on success
    /// - `Err(VaultError::NoDepositFound)` if active deposit still exists
    /// - `Err(VaultError::NoArchivedDepositFound)` if archived deposit already exists
    pub fn archive_deposit(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Verify the deposit is NOT in active storage
        if storage::get_deposit_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoDepositFound);
        }

        if storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoDepositFound);
        }

        // Verify not already archived
        if storage::get_archived_deposit_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoArchivedDepositFound);
        }

        let now = env.ledger().timestamp();
        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_archived_deposit(&env, &depositor, deposit_id, &entry, now);
        events::deposit_archived(&env, &depositor, &token, amount, deposit_id);
        Ok(())
    }

    /// Archive a completed (withdrawn or cancelled) ledger-based deposit for record-keeping.
    /// The depositor manually archives a ledger-based deposit entry that has been withdrawn or cancelled,
    /// storing it with the current archive timestamp.
    ///
    /// # Preconditions
    /// - Deposit must NOT exist in active storage (must have been withdrawn or cancelled already)
    /// - Caller must provide the original deposit entry data
    ///
    /// # Parameters
    /// - `depositor`: The original deposit owner
    /// - `deposit_id`: The ID of the deposit to archive
    /// - `token`: Token address (must match the original deposit)
    /// - `amount`: Amount (must match the original deposit)
    /// - `unlock_ledger`: Unlock ledger (must match the original deposit)
    /// - `penalty_bps`: Penalty basis points (must match the original deposit)
    ///
    /// # Returns
    /// - `Ok(())` on success
    /// - `Err(VaultError::NoDepositFound)` if active deposit still exists
    /// - `Err(VaultError::NoArchivedDepositFound)` if archived deposit already exists
    pub fn archive_deposit_by_ledger(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        token: Address,
        amount: i128,
        unlock_ledger: u32,
        penalty_bps: u32,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Verify the deposit is NOT in active storage
        if storage::get_deposit_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoDepositFound);
        }

        if storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoDepositFound);
        }

        // Verify not already archived
        if storage::get_archived_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some() {
            return Err(VaultError::NoArchivedDepositFound);
        }

        let now = env.ledger().timestamp();
        let entry = LedgerVaultEntry {
            token: token.clone(),
            amount,
            unlock_ledger,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_archived_deposit_by_ledger(&env, &depositor, deposit_id, &entry, now);
        events::deposit_archived_by_ledger(&env, &depositor, &token, amount, deposit_id);
        Ok(())
    }

    /// Delete an archived deposit that is old enough (>= 1 year).
    /// Only deposits archived more than 1 year ago can be deleted.
    ///
    /// # Preconditions
    /// - Archived deposit must exist
    /// - Deposit must be at least 1 year old
    ///
    /// # Parameters
    /// - `depositor`: The original deposit owner
    /// - `deposit_id`: The ID of the archived deposit to delete
    ///
    /// # Returns
    /// - `Ok(())` on success (deposit deleted)
    /// - `Err(VaultError::NoArchivedDepositFound)` if no archived deposit exists
    /// - `Err(VaultError::ArchivedDepositTooYoung)` if deposit is less than 1 year old
    pub fn delete_archived_deposit(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        let now = env.ledger().timestamp();

        // Check timestamp-based archived deposit
        if let Some(archived) = storage::get_archived_deposit_readonly(&env, &depositor, deposit_id) {
            let age = now.saturating_sub(archived.archive_timestamp);
            if age < ARCHIVED_DEPOSIT_MIN_AGE_SECS {
                return Err(VaultError::ArchivedDepositTooYoung);
            }
            storage::remove_archived_deposit(&env, &depositor, deposit_id);
            return Ok(());
        }

        // Check ledger-based archived deposit
        if let Some(archived) = storage::get_archived_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let age = now.saturating_sub(archived.archive_timestamp);
            if age < ARCHIVED_DEPOSIT_MIN_AGE_SECS {
                return Err(VaultError::ArchivedDepositTooYoung);
            }
            storage::remove_archived_deposit_by_ledger(&env, &depositor, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoArchivedDepositFound)
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
    //  Admin: Emergency Lockdown
    // ----------------------------------------------------------------

    /// Activate emergency lockdown. Only admin can call this.
    /// Blocks all user operations while preserving deposit integrity.
    pub fn activate_lockdown(
        env: Env,
        admin: Address,
        reason: soroban_sdk::String,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        let now = env.ledger().timestamp();

        // Set lockdown flag and track metadata
        storage::set_emergency_lockdown(&env, true);
        storage::set_lockdown_activated_at(&env, now);
        storage::set_lockdown_reason(&env, &reason);

        // Emit event
        events::emergency_lockdown_activated(&env, &admin, &reason, now);

        Ok(())
    }

    /// Deactivate emergency lockdown. Only admin can call this.
    /// Requires verification by checking the lockdown was active.
    pub fn deactivate_lockdown(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        // Verify lockdown is active
        if !storage::is_emergency_lockdown(&env) {
            return Err(VaultError::Unauthorized); // Not in lockdown
        }

        let now = env.ledger().timestamp();
        let activated_at = storage::get_lockdown_activated_at(&env).ok_or(VaultError::Unauthorized)?;
        let duration_secs = now.saturating_sub(activated_at);

        // Get the reason for history tracking
        let reason = storage
            .get_lockdown_reason(&env)
            .unwrap_or_else(|| soroban_sdk::String::from_slice(&env, ""));

        // Create lockdown history entry
        let entry = crate::types::LockdownEntry {
            activated_at,
            deactivated_at: Some(now),
            admin: admin.clone(),
            reason,
            duration_secs: Some(duration_secs),
        };
        storage::add_lockdown_history(&env, &entry);

        // Clear lockdown state
        storage::set_emergency_lockdown(&env, false);

        // Emit event
        events::emergency_lockdown_deactivated(&env, &admin, now, duration_secs);

        Ok(())
    }

    /// Check if contract is in emergency lockdown mode.
    pub fn is_emergency_lockdown(env: Env) -> bool {
        storage::is_emergency_lockdown(&env)
    }

    /// Get lockdown history for audit purposes.
    pub fn get_lockdown_history(env: Env) -> Vec<crate::types::LockdownEntry> {
        storage::get_lockdown_history(&env)
    }

    // ----------------------------------------------------------------
    //  Governance: proposal, weighted voting, and timelocked execution
    // ----------------------------------------------------------------

    pub fn propose_change(
        env: Env,
        proposer: Address,
        mode: GovernanceMode,
        proposal_type: ProposalType,
        value: i128,
    ) -> Result<u32, VaultError> {
        proposer.require_auth();
        if matches!(mode, GovernanceMode::AdminVote) {
            storage::require_admin(&env, &proposer)?;
        }

        let created_at = env.ledger().timestamp();
        let proposal_id = storage::next_proposal_id(&env);
        let action = match proposal_type {
            ProposalType::Pause => GovernanceAction::Pause,
            ProposalType::MaxDeposit => GovernanceAction::SetMaxDeposit(value),
            ProposalType::MaxLockDuration => GovernanceAction::SetMaxLockSecs(value as u64),
            ProposalType::FeeRate => GovernanceAction::SetFeeRate(value),
            ProposalType::FeatureFlag => GovernanceAction::ToggleFeature(value != 0),
        };

        storage::set_governance_proposal(&env, proposal_id, &GovernanceProposal {
            proposer: proposer.clone(),
            action,
            mode,
            created_at,
            voting_ends_at: created_at.saturating_add(crate::constants::GOVERNANCE_VOTING_PERIOD_SECS),
            executable_at: created_at
                .saturating_add(crate::constants::GOVERNANCE_VOTING_PERIOD_SECS)
                .saturating_add(crate::constants::GOVERNANCE_TIMELOCK_SECS),
            for_votes: 0,
            against_votes: 0,
            executed: false,
        });
        events::proposal_created(&env, proposal_id, &proposer);
        Ok(proposal_id)
    }

    pub fn propose_pause(env: Env, proposer: Address, mode: GovernanceMode) -> Result<u32, VaultError> {
        proposer.require_auth();
        if matches!(mode, GovernanceMode::AdminVote) {
            storage::require_admin(&env, &proposer)?;
        }
        let created_at = env.ledger().timestamp();
        let proposal_id = storage::next_proposal_id(&env);
        storage::set_governance_proposal(&env, proposal_id, &GovernanceProposal {
            proposer: proposer.clone(),
            action: GovernanceAction::Pause,
            mode,
            created_at,
            voting_ends_at: created_at.saturating_add(crate::constants::GOVERNANCE_VOTING_PERIOD_SECS),
            executable_at: created_at
                .saturating_add(crate::constants::GOVERNANCE_VOTING_PERIOD_SECS)
                .saturating_add(crate::constants::GOVERNANCE_TIMELOCK_SECS),
            for_votes: 0,
            against_votes: 0,
            executed: false,
        });
        events::governance_proposed(&env, proposal_id, &proposer);
        Ok(proposal_id)
    }

    pub fn vote(env: Env, proposal_id: u32, voter: Address, support: bool) -> Result<i128, VaultError> {
        voter.require_auth();
        let mut proposal = storage::get_governance_proposal(&env, proposal_id)
            .ok_or(VaultError::ProposalNotFound)?;
        if env.ledger().timestamp() >= proposal.voting_ends_at {
            return Err(VaultError::VotingEnded);
        }
        if storage::has_governance_vote(&env, proposal_id, &voter) {
            return Err(VaultError::AlreadyVoted);
        }

        let weight = match proposal.mode {
            GovernanceMode::AdminVote => {
                storage::require_admin(&env, &voter)?;
                1
            }
            GovernanceMode::CommunityVote => storage::get_voting_power(&env, &voter),
        };
        if weight <= 0 {
            return Err(VaultError::NoVotingPower);
        }
        if support {
            proposal.for_votes = proposal.for_votes.saturating_add(weight);
        } else {
            proposal.against_votes = proposal.against_votes.saturating_add(weight);
        }
        storage::set_governance_proposal(&env, proposal_id, &proposal);
        storage::set_governance_vote(&env, proposal_id, &voter);
        events::governance_voted(&env, proposal_id, &voter, support, weight);
        Ok(weight)
    }

    pub fn execute_proposal(env: Env, proposal_id: u32) -> Result<(), VaultError> {
        let mut proposal = storage::get_governance_proposal(&env, proposal_id)
            .ok_or(VaultError::ProposalNotFound)?;
        if proposal.executed {
            return Err(VaultError::ProposalAlreadyExecuted);
        }
        let now = env.ledger().timestamp();
        if now < proposal.voting_ends_at {
            return Err(VaultError::VotingStillOpen);
        }
        if now < proposal.executable_at {
            return Err(VaultError::TimelockActive);
        }
        if proposal.for_votes <= proposal.against_votes {
            return Err(VaultError::ProposalRejected);
        }
        match proposal.action {
            GovernanceAction::Pause => storage::set_paused(&env, true),
            GovernanceAction::SetMaxDeposit(value) => storage::set_max_deposit(&env, value),
            GovernanceAction::SetMaxLockSecs(value) => storage::set_max_lock_secs(&env, value),
            GovernanceAction::SetFeeRate(_value) => {},
            GovernanceAction::ToggleFeature(enabled) => {
                if enabled {
                    storage::set_paused(&env, false);
                }
            }
            GovernanceAction::SetFeeRecipient(recipient) => {
                storage::set_fee_recipient(&env, &recipient);
            }
        }
        proposal.executed = true;
        storage::set_governance_proposal(&env, proposal_id, &proposal);
        events::proposal_executed(&env, proposal_id);
        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<GovernanceProposal> {
        storage::get_governance_proposal(&env, proposal_id)
    }

    pub fn proposal_passed(env: Env, proposal_id: u32) -> Result<bool, VaultError> {
        let proposal = storage::get_governance_proposal(&env, proposal_id)
            .ok_or(VaultError::ProposalNotFound)?;
        Ok(proposal.for_votes > proposal.against_votes)
    }

    pub fn get_voting_power(env: Env, voter: Address) -> i128 {
        storage::get_voting_power(&env, &voter)
    }

    // ----------------------------------------------------------------
    //  Admin: Token Allowlist
    // ----------------------------------------------------------------

    pub fn add_allowed_token(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::set_token_allowed(&env, &token, true);
        Ok(())
    }

    pub fn remove_allowed_token(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::set_token_allowed(&env, &token, false);
        Ok(())
    }

    pub fn set_strict_mode(env: Env, admin: Address, strict: bool) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::set_strict_token_allowlist(&env, strict);
        Ok(())
    }

    pub fn toggle_strict_mode(env: Env, admin: Address) -> Result<bool, VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        let strict = !storage::is_strict_token_allowlist(&env);
        storage::set_strict_token_allowlist(&env, strict);
        Ok(strict)
    }

    pub fn is_strict_mode(env: Env) -> bool {
        storage::is_strict_token_allowlist(&env)
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

    pub fn get_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<VaultEntry> {
        storage::get_deposit_readonly(&env, &depositor, deposit_id)
    }

    pub fn get_ledger_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<LedgerVaultEntry> {
        storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id)
    }

    /// Returns whether a deposit is timestamp-based, ledger-based, or multi-token.
    /// Returns `None` if no deposit exists at the given `(depositor, deposit_id)`.
    pub fn get_deposit_type(env: Env, depositor: Address, deposit_id: u32) -> Option<DepositType> {
        if storage::get_deposit_readonly(&env, &depositor, deposit_id).is_some() {
            return Some(DepositType::TimeBased);
        }
        if storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id).is_some() {
            return Some(DepositType::LedgerBased);
        }
        if storage::get_multi_deposit_readonly(&env, &depositor, deposit_id).is_some() {
            return Some(DepositType::MultiToken);
        }
        None
    }

    /// Fetch a single deposit entry for each `(depositor, deposit_id)` pair.
    /// Clamped to `MAX_BATCH_SIZE` (25) entries per call.
    pub fn get_vault_batch(
        env: Env,
        depositors: Vec<Address>,
        deposit_id: u32,
    ) -> Vec<Option<VaultEntry>> {
        let limit = if depositors.len() > MAX_BATCH_SIZE {
            MAX_BATCH_SIZE
        } else {
            depositors.len()
        };
        let mut results = Vec::new(&env);
        for i in 0..limit {
            if let Some(depositor) = depositors.get(i) {
                let entry = storage::get_deposit_readonly(&env, &depositor, deposit_id);
                results.push_back(entry);
            }
        }
        results
    }

    /// Fetch multiple deposits for a single depositor in one RPC call.
    /// Limit: up to 25 deposit IDs per call.
    /// Returns `Vec` of `(deposit_id, Option<VaultEntry>)` pairs.
    pub fn get_deposit_batch(
        env: Env,
        depositor: Address,
        deposit_ids: Vec<u32>,
    ) -> Vec<(u32, Option<VaultEntry>)> {
        let limit = if deposit_ids.len() > MAX_BATCH_SIZE {
            MAX_BATCH_SIZE
        } else {
            deposit_ids.len()
        };
        let mut results = Vec::new(&env);
        for i in 0..limit {
            if let Some(id) = deposit_ids.get(i) {
                let entry = storage::get_deposit_readonly(&env, &depositor, id);
                results.push_back((id, entry));
            }
        }
        results
    }

    pub fn get_deposit_ids(env: Env, depositor: Address) -> Vec<u32> {
        storage::get_deposit_ids(&env, &depositor)
    }

    pub fn get_tax_loss_harvests(env: Env, depositor: Address) -> Vec<TaxLossHarvest> {
        storage::get_tax_loss_harvests(&env, &depositor)
    }

    pub fn get_time(env: Env) -> u64 {
        env.ledger().timestamp()
    }

    /// Returns time remaining for a deposit.
    /// For timestamp-based deposits: exact seconds remaining.
    /// For ledger-based deposits: estimated seconds (remaining_ledgers × 5).
    /// For multi-token deposits: exact seconds remaining.
    /// Returns 0 when unlocked or not found.
    pub fn time_remaining(env: Env, depositor: Address, deposit_id: u32) -> u64 {
        // Timestamp-based path.
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            return entry.unlock_time.saturating_sub(now);
        }

        // Ledger-based path.
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current = env.ledger().sequence();
            if current >= entry.unlock_ledger {
                return 0;
            }
            let remaining_ledgers = (entry.unlock_ledger - current) as u64;
            return remaining_ledgers.saturating_mul(storage::LEDGER_SECONDS);
        }

        // Multi-token path.
        if let Some(entry) = storage::get_multi_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            return entry.unlock_time.saturating_sub(now);
        }

        0
    }

    pub fn time_to_withdrawal(env: Env, depositor: Address, deposit_id: u32) -> u64 {
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            return entry
                .unlock_time
                .saturating_add(entry.withdrawal_delay_secs)
                .saturating_sub(env.ledger().timestamp());
        }
        0
    }

    pub fn get_accrued_interest(env: Env, depositor: Address, deposit_id: u32) -> i128 {
        storage::get_deposit_readonly(&env, &depositor, deposit_id)
            .and_then(|entry| calculate_interest(&entry, env.ledger().timestamp()).ok())
            .unwrap_or(0)
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

    pub fn flash_borrow(env: Env, borrower: Address, token: Address, amount: i128) -> Result<i128, VaultError> {
        borrower.require_auth();
        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        if storage::is_flash_loan_guard_active(&env) {
            return Err(VaultError::FlashLoanReentrancy);
        }

        let contract = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);
        let available = token_client.balance(&contract);
        if available < amount {
            return Err(VaultError::FlashLoanInsufficientLiquidity);
        }

        let fee = (amount * FLASH_LOAN_FEE_BPS as i128) / 10_000;
        let state = crate::types::FlashLoanState {
            borrower: borrower.clone(),
            token: token.clone(),
            amount,
            fee,
            repaid: false,
        };
        storage::set_flash_loan_state(&env, &borrower, &token, &state);
        storage::set_flash_loan_guard(&env, true);
        token_client.transfer(&contract, &borrower, &amount);
        Ok(amount)
    }

    pub fn flash_repay(env: Env, borrower: Address, token: Address, repayment: i128) -> Result<i128, VaultError> {
        borrower.require_auth();
        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        let state = storage::get_flash_loan_state(&env, &borrower, &token)
            .ok_or(VaultError::NoDepositFound)?;
        let due = state.amount.saturating_add(state.fee);
        if repayment < due {
            return Err(VaultError::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&borrower, &env.current_contract_address(), &repayment);

        let mut total_fee = state.fee;
        if total_fee > 0 {
            let mut assigned_total = 0i128;
            let depositor_list = storage::get_all_depositors_raw(&env);
            let mut total_weight = 0i128;
            let mut fee_map: soroban_sdk::Vec<(Address, i128)> = soroban_sdk::Vec::new(&env);

            for depositor in depositor_list.iter() {
                let mut weight = 0i128;
                let ids = storage::get_deposit_ids(&env, &depositor);
                for id in ids.iter() {
                    if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, id) {
                        if entry.token == token {
                            weight = weight.saturating_add(entry.amount);
                        }
                    }
                }
                if weight > 0 {
                    total_weight = total_weight.saturating_add(weight);
                    fee_map.push_back((depositor.clone(), weight));
                }
            }

            if total_weight > 0 {
                for entry in fee_map.iter() {
                    let (depositor, weight) = entry;
                    let share = (weight * total_fee) / total_weight;
                    if share > 0 {
                        let current_balance = storage::get_flash_loan_fee_balance(&env, &token, &depositor);
                        storage::set_flash_loan_fee_balance(&env, &token, &depositor, current_balance.saturating_add(share));
                        assigned_total = assigned_total.saturating_add(share);
                    }
                }
                if assigned_total < total_fee {
                    let mut remainder = total_fee.saturating_sub(assigned_total);
                    if let Some(first) = fee_map.first() {
                        let (depositor, _) = first;
                        let current_balance = storage::get_flash_loan_fee_balance(&env, &token, &depositor);
                        storage::set_flash_loan_fee_balance(&env, &token, &depositor, current_balance.saturating_add(remainder));
                    }
                }
            }
        }

        storage::remove_flash_loan_state(&env, &borrower, &token);
        storage::set_flash_loan_guard(&env, false);
        let refund = repayment.saturating_sub(due);
        if refund > 0 {
            token_client.transfer(&env.current_contract_address(), &borrower, &refund);
        }
        events::flash_loan_executed(&env, &borrower, &token, state.amount, state.fee, repayment);
        Ok(due)
    }

    pub fn claim_flash_loan_fees(env: Env, depositor: Address, token: Address) -> Result<i128, VaultError> {
        depositor.require_auth();
        let fee = storage::get_flash_loan_fee_balance(&env, &token, &depositor);
        if fee <= 0 {
            return Ok(0);
        }
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &depositor, &fee);
        storage::set_flash_loan_fee_balance(&env, &token, &depositor, 0);
        Ok(fee)
    }

    pub fn get_flash_loan_fee_balance(env: Env, token: Address, depositor: Address) -> i128 {
        storage::get_flash_loan_fee_balance(&env, &token, &depositor)
    }

    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        storage::is_token_allowed(&env, &token)
    }

    // ----------------------------------------------------------------
    //  Token vetting: Propose -> Review -> Approve
    // ----------------------------------------------------------------

    pub fn propose_token(env: Env, proposer: Address, token: Address) -> Result<(), VaultError> {
        proposer.require_auth();

        if storage::is_token_allowed(&env, &token) {
            return Err(VaultError::TokenAlreadyApproved);
        }

        let vetting = TokenVetting {
            proposer: proposer.clone(),
            proposed_at: env.ledger().timestamp(),
            reviewed: false,
            review_passed: false,
            reviewer: None,
            reviewed_at: None,
            approved: false,
        };
        storage::set_token_vetting(&env, &token, &vetting);
        events::token_proposed(&env, &token, &proposer);
        Ok(())
    }

    pub fn review_token(
        env: Env,
        reviewer: Address,
        token: Address,
        passed: bool,
    ) -> Result<(), VaultError> {
        reviewer.require_auth();
        storage::require_admin(&env, &reviewer)?;

        let mut vetting = storage::get_token_vetting(&env, &token)
            .ok_or(VaultError::TokenVettingNotFound)?;
        vetting.reviewed = true;
        vetting.review_passed = passed;
        vetting.reviewer = Some(reviewer.clone());
        vetting.reviewed_at = Some(env.ledger().timestamp());
        storage::set_token_vetting(&env, &token, &vetting);
        events::token_reviewed(&env, &token, &reviewer, passed);
        Ok(())
    }

    pub fn approve_token(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        let mut vetting = storage::get_token_vetting(&env, &token)
            .ok_or(VaultError::TokenVettingNotFound)?;
        if !vetting.reviewed || !vetting.review_passed {
            return Err(VaultError::TokenReviewRequired);
        }
        if vetting.approved {
            return Err(VaultError::TokenAlreadyApproved);
        }

        vetting.approved = true;
        storage::set_token_vetting(&env, &token, &vetting);
        storage::set_token_allowed(&env, &token, true);
        events::token_approved(&env, &token, &admin);
        Ok(())
    }

    pub fn get_token_vetting(env: Env, token: Address) -> Option<TokenVetting> {
        storage::get_token_vetting(&env, &token)
    }

    pub fn get_depositor_count(env: Env) -> u32 {
        storage::get_depositor_count(&env)
    }

    pub fn get_depositors(env: Env, offset: u32, limit: u32) -> Page {
        let (items, total_count) = storage::get_depositors_page(&env, offset, limit);
        Page { items, total_count }
    }

    pub fn is_initialized(env: Env) -> bool {
        storage::is_initialized(&env)
    }

    pub fn version(_env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_slice(&_env, env!("CARGO_PKG_VERSION"))
    }

    /// Admin-readable query: returns the cumulative emergency withdrawal amount
    /// for a specific ledger sequence number.
    /// 
    /// This allows admins to audit emergency withdrawal activity and ensure
    /// the per-ledger limit is being respected. Returns 0 if no emergency
    /// withdrawals have occurred in that ledger.
    pub fn get_emergency_withdrawal_total(env: Env, ledger: u32) -> i128 {
        storage::get_emergency_withdrawal_per_ledger(&env, ledger)
    }

    pub fn get_circuit_breaker_history(env: Env) -> Vec<CircuitBreakerActivation> {
        storage::get_circuit_breaker_history(&env)
    }

    // ----------------------------------------------------------------
    //  Read-only: Paginated flat deposits view
    // ----------------------------------------------------------------

    pub fn get_deposits_page(
        env: Env,
        offset: u32,
        limit: u32,
    ) -> Vec<(Address, u32, VaultEntry)> {
        let mut results: Vec<(Address, u32, VaultEntry)> = Vec::new(&env);
        let mut global_index: u32 = 0;
        let end_at = offset.saturating_add(limit);

        let depositor_list = storage::get_all_depositors_raw(&env);
        for depositor in depositor_list.iter() {
            if global_index >= end_at {
                break;
            }
            if !storage::depositor_is_active(&env, &depositor) {
                continue;
            }
            let ids = storage::get_deposit_ids(&env, &depositor);
            for id in ids.iter() {
                if global_index >= end_at {
                    break;
                }
                if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, id) {
                    if global_index >= offset {
                        results.push_back((depositor.clone(), id, entry));
                    }
                    global_index = global_index.saturating_add(1);
                }
            }
        }
        results
    }

    // ----------------------------------------------------------------
    //  Admin: Storage migration
    // ----------------------------------------------------------------

    pub fn get_storage_version(env: Env) -> Option<u32> {
        storage::get_storage_version(&env)
    }

    pub fn migrate(env: Env, admin: Address) -> Result<bool, VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        let current_version = storage::get_storage_version(&env).unwrap_or(0);

        if current_version >= STORAGE_VERSION {
            return Ok(false);
        }

        storage::set_storage_version(&env, STORAGE_VERSION);
        Ok(true)
    }

    // ----------------------------------------------------------------
    //  Issue #333: Recurring deposit subscriptions
    // ----------------------------------------------------------------

    /// Creates a recurring deposit subscription.
    ///
    /// The depositor commits to having `total_count` individual deposits of
    /// `amount` tokens created at intervals of `interval_secs` seconds.  The
    /// first execution is immediately due (caller can invoke `execute_subscription`
    /// right after creation, or wait `interval_secs`).
    ///
    /// # Parameters
    /// - `depositor`         — account that will fund every deposit tick; must sign.
    /// - `token`             — SAC token to lock.
    /// - `amount`            — amount per deposit tick (> 0, ≤ `max_deposit`).
    /// - `interval_secs`     — seconds between ticks (> 0).
    /// - `total_count`       — total number of ticks (> 0).
    /// - `lock_duration_secs`— lock duration per individual deposit (≥ `MIN_LOCK_DURATION_SECS`).
    /// - `penalty_bps`       — early-exit penalty for each produced deposit (0–10000).
    ///
    /// # Returns
    /// The new `subscription_id`.
    pub fn create_subscription(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        interval_secs: u64,
        total_count: u32,
        lock_duration_secs: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        // Validate params
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }
        if interval_secs == 0 || total_count == 0 {
            return Err(VaultError::InvalidSubscriptionParams);
        }
        if lock_duration_secs < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }
        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        if lock_duration_secs > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }
        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let sub_id = storage::next_subscription_id(&env, &depositor);
        let now = env.ledger().timestamp();

        let sub = DepositSubscription {
            depositor: depositor.clone(),
            token: token.clone(),
            amount,
            interval_secs,
            total_count,
            executed_count: 0,
            lock_duration_secs,
            penalty_bps,
            // First execution is due immediately.
            next_execution_time: now,
            paused: false,
            cancelled: false,
        };

        storage::set_subscription(&env, &depositor, sub_id, &sub);
        events::subscription_created(
            &env,
            &depositor,
            &token,
            sub_id,
            amount,
            interval_secs,
            total_count,
        );

        Ok(sub_id)
    }

    pub fn subscribe(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        interval_secs: u64,
        total_count: u32,
        lock_duration_secs: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        Self::create_subscription(
            env,
            depositor,
            token,
            amount,
            interval_secs,
            total_count,
            lock_duration_secs,
            penalty_bps,
        )
    }

    /// Cancels an active subscription.
    ///
    /// Only the depositor who created the subscription may cancel it.
    /// Already-executed deposits are unaffected — they remain locked until
    /// their individual `unlock_time`.
    pub fn cancel_subscription(
        env: Env,
        depositor: Address,
        sub_id: u32,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        let mut sub = storage::get_subscription(&env, &depositor, sub_id)
            .ok_or(VaultError::NoSubscriptionFound)?;

        if sub.cancelled {
            return Err(VaultError::SubscriptionCancelled);
        }

        if sub.paused {
            return Err(VaultError::SubscriptionPaused);
        }
        if sub.executed_count >= sub.total_count {
            return Err(VaultError::SubscriptionCompleted);
        }

        sub.cancelled = true;
        storage::set_subscription(&env, &depositor, sub_id, &sub);
        storage::record_subscription_execution(&env, &depositor, sub_id, &SubscriptionExecution {
            deposit_id,
            amount: sub.amount,
            executed_at: now,
        });
        storage::set_subscription_stats(&env, &depositor, sub_id, &SubscriptionStats {
            deposit_count: sub.executed_count,
            total_amount: sub.amount.saturating_mul(sub.executed_count as i128),
            last_execution_time: now,
        });

        events::subscription_cancelled(&env, &depositor, sub_id, sub.executed_count);
        Ok(())
    }

    /// Executes the next tick of a recurring deposit subscription.
    ///
    /// This is a *permissionless* function — anyone may call it as long as the
    /// subscription is active and the interval has elapsed.  This mirrors the
    /// "oracle-triggered" model described in issue #333: an off-chain keeper
    /// (or the depositor themselves) triggers execution at the right time.
    ///
    /// On each call the contract:
    /// 1. Validates the subscription is not cancelled / completed.
    /// 2. Checks `now >= next_execution_time`.
    /// 3. Transfers `amount` tokens from `depositor` to the contract.
    /// 4. Creates a `VaultEntry` locked for `lock_duration_secs` from `now`.
    /// 5. Updates `executed_count` and `next_execution_time`.
    ///
    /// # Returns
    /// The new `deposit_id` created for this tick.
    pub fn execute_subscription(
        env: Env,
        depositor: Address,
        sub_id: u32,
    ) -> Result<u32, VaultError> {
        // No auth required — permissionless execution by any caller (keeper/oracle).

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        let mut sub = storage::get_subscription(&env, &depositor, sub_id)
            .ok_or(VaultError::NoSubscriptionFound)?;

        if sub.cancelled {
            return Err(VaultError::SubscriptionCancelled);
        }
        if sub.executed_count >= sub.total_count {
            return Err(VaultError::SubscriptionCompleted);
        }

        let now = env.ledger().timestamp();
        if now < sub.next_execution_time {
            return Err(VaultError::SubscriptionNotDue);
        }

        // Transfer tokens from depositor into the contract.
        let token_client = token::Client::new(&env, &sub.token);
        token_client.transfer(&sub.depositor, &env.current_contract_address(), &sub.amount);

        // Create a locked VaultEntry for this tick.
        let unlock_time = now.saturating_add(sub.lock_duration_secs);
        let deposit_id = storage::next_deposit_id(&env, &depositor);
        let entry = VaultEntry {
            token: sub.token.clone(),
            amount: sub.amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps: sub.penalty_bps,
            compound_frequency_secs: 0,
            last_accrual_timestamp: now,
        };
        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);

        // Advance subscription state.
        sub.executed_count = sub.executed_count.saturating_add(1);
        sub.next_execution_time = now.saturating_add(sub.interval_secs);
        storage::set_subscription(&env, &depositor, sub_id, &sub);

        events::subscription_executed(
            &env,
            &depositor,
            &sub.token,
            sub_id,
            deposit_id,
            sub.executed_count,
        );
        events::deposit(&env, &depositor, &sub.token, sub.amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    pub fn pause_subscription(env: Env, depositor: Address, sub_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();
        let mut sub = storage::get_subscription(&env, &depositor, sub_id).ok_or(VaultError::NoSubscriptionFound)?;
        if sub.cancelled { return Err(VaultError::SubscriptionCancelled); }
        if sub.executed_count >= sub.total_count { return Err(VaultError::SubscriptionCompleted); }
        if sub.paused { return Err(VaultError::SubscriptionPaused); }
        sub.paused = true;
        storage::set_subscription(&env, &depositor, sub_id, &sub);
        events::subscription_paused(&env, &depositor, sub_id);
        Ok(())
    }

    pub fn resume_subscription(env: Env, depositor: Address, sub_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();
        let mut sub = storage::get_subscription(&env, &depositor, sub_id).ok_or(VaultError::NoSubscriptionFound)?;
        if sub.cancelled { return Err(VaultError::SubscriptionCancelled); }
        if sub.executed_count >= sub.total_count { return Err(VaultError::SubscriptionCompleted); }
        if !sub.paused { return Ok(()); }
        sub.paused = false;
        storage::set_subscription(&env, &depositor, sub_id, &sub);
        events::subscription_resumed(&env, &depositor, sub_id);
        Ok(())
    }

    pub fn get_subscription_history(env: Env, depositor: Address, sub_id: u32) -> Vec<SubscriptionExecution> {
        storage::get_subscription_history(&env, &depositor, sub_id)
    }

    pub fn get_subscription_stats(env: Env, depositor: Address, sub_id: u32) -> SubscriptionStats {
        storage::get_subscription_stats(&env, &depositor, sub_id)
    }

    /// Returns the `DepositSubscription` struct for the given `(depositor, sub_id)`,
    /// or `None` if not found.
    pub fn get_subscription(
        env: Env,
        depositor: Address,
        sub_id: u32,
    ) -> Option<DepositSubscription> {
        storage::get_subscription_readonly(&env, &depositor, sub_id)
    }

    /// Returns all subscription IDs ever created for `depositor` (includes
    /// cancelled and completed ones — filter by `cancelled` / `executed_count`).
    pub fn get_subscription_ids(env: Env, depositor: Address) -> Vec<u32> {
        storage::get_subscription_ids(&env, &depositor)
    }

    // ----------------------------------------------------------------
    //  Issue #334: Deposit insurance pool
    // ----------------------------------------------------------------

    /// Returns the current insurance pool balance for `token`.
    pub fn get_insurance_pool_balance(env: Env, token: Address) -> i128 {
        storage::get_insurance_pool_balance(&env, &token)
    }

    /// Files an insurance claim against the pool for a specific token.
    ///
    /// The claimant provides free-form `incident_evidence` (e.g. a description
    /// of what happened or an off-chain transaction hash).  The admin reviews
    /// and either approves or denies via `approve_claim` / `deny_claim`.
    ///
    /// There is no restriction on who can file a claim — any address may submit
    /// evidence.  Restricting claims to depositors only would require iterating
    /// every deposit, which is budget-unsafe.
    ///
    /// # Returns
    /// A new `claim_id`.
    pub fn claim_insurance(
        env: Env,
        claimant: Address,
        token: Address,
        amount_requested: i128,
        incident_evidence: String,
    ) -> Result<u32, VaultError> {
        claimant.require_auth();

        if amount_requested <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let claim_id = storage::next_claim_id(&env);
        let claim = InsuranceClaim {
            claim_id,
            claimant: claimant.clone(),
            token: token.clone(),
            amount_requested,
            incident_evidence,
            status: ClaimStatus::Pending,
        };

        storage::set_claim(&env, claim_id, &claim);
        events::insurance_claim_filed(&env, &claimant, &token, claim_id, amount_requested);

        Ok(claim_id)
    }

    /// Admin: approve an insurance claim and disburse funds from the pool.
    ///
    /// The full `amount_requested` is paid out from the pool balance for
    /// `claim.token`.  Returns `InsufficientInsurancePool` if the pool
    /// balance is lower than the requested amount.
    ///
    /// # Returns
    /// The amount disbursed.
    pub fn approve_claim(env: Env, admin: Address, claim_id: u32) -> Result<i128, VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        let mut claim = storage::get_claim(&env, claim_id).ok_or(VaultError::NoClaimFound)?;

        match claim.status {
            ClaimStatus::Approved | ClaimStatus::Denied => {
                return Err(VaultError::ClaimAlreadyResolved)
            }
            ClaimStatus::Pending => {}
        }

        // Deduct from pool balance first (checks-effects-interactions).
        storage::deduct_insurance_pool_balance(&env, &claim.token, claim.amount_requested)?;

        // Update claim status.
        claim.status = ClaimStatus::Approved;
        storage::set_claim(&env, claim_id, &claim);

        // Transfer tokens from contract to claimant.
        let token_client = token::Client::new(&env, &claim.token);
        token_client.transfer(
            &env.current_contract_address(),
            &claim.claimant,
            &claim.amount_requested,
        );

        events::insurance_claim_approved(
            &env,
            &admin,
            &claim.claimant,
            claim_id,
            claim.amount_requested,
        );

        Ok(claim.amount_requested)
    }

    /// Admin: deny an insurance claim.  No funds are moved.
    pub fn deny_claim(env: Env, admin: Address, claim_id: u32) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        let mut claim = storage::get_claim(&env, claim_id).ok_or(VaultError::NoClaimFound)?;

        match claim.status {
            ClaimStatus::Approved | ClaimStatus::Denied => {
                return Err(VaultError::ClaimAlreadyResolved)
            }
            ClaimStatus::Pending => {}
        }

        claim.status = ClaimStatus::Denied;
        storage::set_claim(&env, claim_id, &claim);

        events::insurance_claim_denied(&env, &admin, claim_id);
        Ok(())
    }

    /// Returns the `InsuranceClaim` for `claim_id`, or `None` if not found.
    pub fn get_claim(env: Env, claim_id: u32) -> Option<InsuranceClaim> {
        storage::get_claim_readonly(&env, claim_id)
    }

    // ================================================================
    //  NFT Evolution Query Functions
    // ================================================================

    /// Get the NFT evolution record for a deposit.
    /// Returns `None` if no NFT record exists yet.
    pub fn get_nft_evolution(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<crate::nft::NFTEvolutionRecord> {
        storage::get_nft_evolution_readonly(&env, &depositor, deposit_id)
    }

    /// Get the current evolution stage of a deposit's NFT.
    /// Returns `None` if the deposit does not exist or has no NFT record.
    pub fn get_nft_stage(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<crate::nft::EvolutionStage> {
        storage::get_nft_evolution_readonly(&env, &depositor, deposit_id)
            .map(|record| record.stage)
    }

    /// Get the rarity tier of a deposit's NFT.
    /// Returns `None` if the deposit does not exist or has no NFT record.
    pub fn get_nft_rarity(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<crate::nft::RarityTier> {
        storage::get_nft_evolution_readonly(&env, &depositor, deposit_id)
            .map(|record| record.rarity)
    }

    /// Get the metadata URI for a deposit's NFT.
    /// Returns `None` if the deposit does not exist or has no NFT record.
    pub fn get_nft_metadata_uri(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<String> {
        storage::get_nft_evolution_readonly(&env, &depositor, deposit_id)
            .map(|record| record.metadata_uri)
    }

    /// Get the evolution history (count) for a deposit's NFT.
    /// Returns `None` if the deposit does not exist or has no NFT record.
    pub fn get_nft_evolution_count(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<u32> {
        storage::get_nft_evolution_readonly(&env, &depositor, deposit_id)
            .map(|record| record.evolution_count)
    }
}


// ================================================================
//  MEV DETECTION & RECOVERY HELPERS (module-level)
// ================================================================

/// Compute Keccak256 hash of (token || amount || price || nonce) for commit-reveal.
/// This function uses the token's address bytes, concatenated with the amount,
/// price, and nonce values encoded as little-endian i128/u32.
fn compute_commit_hash(
    env: &Env,
    token: &Address,
    amount: i128,
    price: i128,
    nonce: u32,
) -> soroban_sdk::BytesN<32> {
    use soroban_sdk::Bytes;

    // Serialize: token (32 bytes) || amount (16 bytes) || price (16 bytes) || nonce (4 bytes)
    let mut data = Bytes::new(env);
    
    // Add token address (should be 32 bytes when serialized)
    let token_bytes = env.serialize_to_bytes(token).unwrap_or_else(|_| Bytes::new(env));
    for b in token_bytes.iter() {
        data.push_back(b);
    }
    
    // Add amount (i128 = 16 bytes, little-endian)
    for b in amount.to_le_bytes().iter() {
        data.push_back(*b);
    }
    
    // Add price (i128 = 16 bytes, little-endian)
    for b in price.to_le_bytes().iter() {
        data.push_back(*b);
    }
    
    // Add nonce (u32 = 4 bytes, little-endian)
    for b in nonce.to_le_bytes().iter() {
        data.push_back(*b);
    }
    
    // Compute Keccak256 hash
    soroban_sdk::crypto::keccak256(&data)
}

/// Detect MEV attacks by comparing current price against time-weighted average price (TWAP).
/// If price deviation exceeds MEV_PRICE_DEVIATION_THRESHOLD_BPS, record an attack detection.
fn detect_mev_attack(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    token: &Address,
    current_price: i128,
    now: u64,
) -> Result<(), VaultError> {
    if current_price <= 0 {
        return Err(VaultError::InvalidPriceData);
    }

    // Get price history for the token
    let price_history = storage::get_price_history(env, token, now);
    
    if price_history.len() == 0 {
        // Not enough history, assume no attack
        return Ok(());
    }

    // Calculate time-weighted average price (TWAP)
    let mut weighted_sum: i128 = 0;
    let mut total_weight: u64 = 0;
    
    for sample in price_history.iter() {
        let time_since = now.saturating_sub(sample.timestamp);
        // Give more weight to recent prices (decay over time)
        let weight = 3600u64.saturating_sub(time_since); // 1-hour decay window
        if weight > 0 {
            weighted_sum = weighted_sum.saturating_add((sample.price).saturating_mul(weight as i128));
            total_weight = total_weight.saturating_add(weight);
        }
    }

    let twap = if total_weight > 0 {
        weighted_sum / (total_weight as i128)
    } else {
        // Fallback: use average of all prices
        let mut sum: i128 = 0;
        for sample in price_history.iter() {
            sum = sum.saturating_add(sample.price);
        }
        if price_history.len() > 0 {
            sum / (price_history.len() as i128)
        } else {
            current_price
        }
    };

    // Check for price deviation
    let deviation_bps = if current_price > twap {
        ((current_price - twap) * 10_000) / twap
    } else {
        ((twap - current_price) * 10_000) / twap
    };

    if deviation_bps as u32 > constants::MEV_PRICE_DEVIATION_THRESHOLD_BPS {
        // MEV attack detected
        let detection_id = storage::next_mev_detection_id(env, depositor, deposit_id);
        
        // Estimate MEV recovered (simplified: difference between current and TWAP)
        let mev_amount = (current_price - twap).abs();
        
        let detection = MEVDetection {
            timestamp: now,
            depositor: depositor.clone(),
            deposit_id,
            price_deviation_bps: deviation_bps as u32,
            mev_recovered: mev_amount,
            resolved: false,
        };

        storage::set_mev_detection(env, depositor, deposit_id, detection_id, &detection);
        storage::set_mev_status(env, depositor, deposit_id, &MEVStatus::AttackDetected);

        // Add to MEV pool for redistribution
        let current_pool = storage::get_mev_pool(env);
        let new_pool = current_pool.saturating_add(mev_amount);
        storage::set_mev_pool(env, new_pool);

        // Credit depositor with recovered MEV
        let current_claimed = storage::get_mev_claimed(env, depositor);
        let new_claimed = current_claimed.saturating_add(mev_amount);
        storage::set_mev_claimed(env, depositor, new_claimed);

        events::mev_detected(env, depositor, deposit_id, deviation_bps as u32, mev_amount);
    }

    Ok(())
}

/// Calculate carbon footprint: amount × duration_seconds × CARBON_BASELINE
fn calculate_carbon_footprint(amount: i128, duration_secs: u64) -> i128 {
    (amount as i128)
        .saturating_mul(duration_secs as i128)
        .saturating_mul(constants::CARBON_BASELINE_PER_UNIT_SECOND)
}

/// Calculate carbon offset: carbon_footprint × (penalty_bps / 10_000)
fn calculate_carbon_offset(carbon_footprint: i128, penalty_bps: u32) -> i128 {
    carbon_footprint
        .saturating_mul(penalty_bps as i128)
        .saturating_div(10_000)
}

/// Check sustainability milestones and emit events
fn check_sustainability_milestones(
    env: &Env,
    depositor: &Address,
    total_carbon: i128,
    total_offset: i128,
    average_renewable: u32,
) {
    let bitmap = storage::get_milestone_bitmap(env, depositor);
    let mut new_bitmap = bitmap;

    // Milestone 1: Carbon Neutral (100% offset)
    if total_carbon > 0 && total_offset >= total_carbon && (bitmap & 1) == 0 {
        new_bitmap |= 1;
        events::sustainability_milestone(env, depositor, "carbon_neutral");
    }

    // Milestone 2: High Renewable (≥75%)
    if average_renewable >= 75 && (bitmap & 2) == 0 {
        new_bitmap |= 2;
        events::sustainability_milestone(env, depositor, "high_renewable");
    }

    // Milestone 3: Carbon Negative (offset > footprint)
    if total_offset > total_carbon && (bitmap & 4) == 0 {
        new_bitmap |= 4;
        events::sustainability_milestone(env, depositor, "carbon_negative");
    }

    // Milestone 4: Large Offset (≥1 billion grams = 1000 tonnes)
    if total_offset >= 1_000_000_000 && (bitmap & 8) == 0 {
        new_bitmap |= 8;
        events::sustainability_milestone(env, depositor, "large_offset");
    }

    if new_bitmap != bitmap {
        storage::set_milestone_bitmap(env, depositor, new_bitmap);
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
