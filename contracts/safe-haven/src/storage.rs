use soroban_sdk::{token, Address, Bytes, Env, Vec};

use crate::types::{DepositSubscription, MultiTokenVaultEntry, SubscriptionExecution, SubscriptionStats, TaxLossHarvest, VaultEntry, VaultKey, LedgerVaultEntry, MAX_LOCK_DURATION_SECS};
use crate::types::CircuitBreakerActivation;

// ================================================================
// LEDGER_SECONDS: Average time between Stellar ledger closes
// ================================================================
// Stellar's consensus protocol produces a new ledger approximately every 5 seconds.
// This constant is used to estimate wall-clock time remaining for ledger-based deposits
// via the formula: estimated_seconds = remaining_ledgers × LEDGER_SECONDS
//
// Important: This is an APPROXIMATION, not a guarantee.
// - Actual ledger close times vary by ±1-2 seconds due to network conditions
// - The 5-second value is a Stellar network consensus target, not a hard limit
// - Use this for UI display and rough estimates only, not for precise scheduling
// - For exact time limits, prefer timestamp-based deposits instead
//
// Used by:
// - contract.rs::time_remaining() — converts ledger count to estimated seconds for ledger-based deposits
// - constants.rs::MIN_LOCK_LEDGERS — computes minimum lock duration in ledgers (60 seconds ÷ 5)
// - storage.rs::BUMP_TARGET — ensures TTL coverage extends past maximum lock duration
pub const LEDGER_SECONDS: u64 = 5;

// How many ledgers to extend TTL to cover the maximum allowed lock duration.
// BUMP_THRESHOLD is derived from BUMP_TARGET so both stay in sync automatically
// when MAX_LOCK_DURATION_SECS changes — no silent inconsistency.
pub const BUMP_TARGET: u32 =
    ((MAX_LOCK_DURATION_SECS + LEDGER_SECONDS - 1) / LEDGER_SECONDS) as u32;
pub const BUMP_THRESHOLD: u32 = BUMP_TARGET / 2;

// ----------------------------------------------------------------
//  Deposit counter helpers
// ----------------------------------------------------------------

// `next_deposit_id` returns the next unused deposit ID for a depositor.
// The counter is monotonic and is never decremented when deposits are
// removed. If all deposits are withdrawn, the counter still advances
// from its historical maximum, so a new deposit after a full drain will
// receive the next unused ID and cannot collide with any prior deposit.
//
// The active deposit IDs are tracked separately in `ActiveDepositIds`, so
// `get_deposit_ids` does not rely on scanning `0..counter` or checking
// whether each historical deposit key exists. This keeps active ID
// enumeration bounded by the number of open deposits rather than by the
// historical counter value.
pub fn next_deposit_id(env: &Env, depositor: &Address) -> u32 {
    let key = VaultKey::DepositCounter(depositor.clone());
    let id: u32 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().set(&key, &(id.saturating_add(1)));
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    id
}

pub fn peek_next_deposit_id(env: &Env, depositor: &Address) -> u32 {
    let key = VaultKey::DepositCounter(depositor.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_quantum_safe_public_key(env: &Env, account: &Address, public_key: &Bytes) {
    let key = VaultKey::QuantumSafePublicKey(account.clone());
    env.storage().persistent().set(&key, public_key);
}

pub fn get_quantum_safe_public_key(env: &Env, account: &Address) -> Option<Bytes> {
    let key = VaultKey::QuantumSafePublicKey(account.clone());
    env.storage().persistent().get(&key)
}

pub fn set_quantum_safe_metadata(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    metadata: &Bytes,
) {
    let key = VaultKey::QuantumSafeMetadata(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, metadata);
}

pub fn get_quantum_safe_metadata(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<Bytes> {
    let key = VaultKey::QuantumSafeMetadata(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn set_identity(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    identity: &IdentityLink,
) {
    let key = VaultKey::Identity(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, identity);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_identity(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<IdentityLink> {
    let key = VaultKey::Identity(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn remove_identity(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::Identity(depositor.clone(), deposit_id);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }
}

// ----------------------------------------------------------------
//  Active deposit ID list helpers
// ----------------------------------------------------------------

fn get_active_ids(env: &Env, depositor: &Address) -> Vec<u32> {
    let key = VaultKey::ActiveDepositIds(depositor.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

fn save_active_ids(env: &Env, depositor: &Address, ids: &Vec<u32>) {
    let key = VaultKey::ActiveDepositIds(depositor.clone());
    env.storage().persistent().set(&key, ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Append `deposit_id` to the active ID list for `depositor`. Called
/// immediately after a new deposit entry is written to storage.
pub fn add_active_deposit_id(env: &Env, depositor: &Address, deposit_id: u32) {
    let mut ids = get_active_ids(env, depositor);
    ids.push_back(deposit_id);
    save_active_ids(env, depositor, &ids);
}

/// Remove `deposit_id` from the active ID list for `depositor`. Called
/// immediately after a deposit entry is removed from storage.
pub fn remove_active_deposit_id(env: &Env, depositor: &Address, deposit_id: u32) {
    let ids = get_active_ids(env, depositor);
    let mut new_ids: Vec<u32> = Vec::new(env);
    for id in ids.iter() {
        if id != deposit_id {
            new_ids.push_back(id);
        }
    }
    save_active_ids(env, depositor, &new_ids);
}

/// O(1) single storage read — returns all active deposit IDs for
/// `depositor`, regardless of whether they are timestamp- or
/// ledger-based.
pub fn get_deposit_ids(env: &Env, depositor: &Address) -> Vec<u32> {
    get_active_ids(env, depositor)
}

pub fn set_session_key(env: &Env, wallet: &Address, session_key: &Address, expires_at: u64) {
    let key = VaultKey::SessionKey(wallet.clone(), session_key.clone());
    env.storage().persistent().set(&key, &expires_at);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_session_key_expiry(env: &Env, wallet: &Address, session_key: &Address) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&VaultKey::SessionKey(wallet.clone(), session_key.clone()))
}

pub fn remove_session_key(env: &Env, wallet: &Address, session_key: &Address) {
    env.storage()
        .persistent()
        .remove(&VaultKey::SessionKey(wallet.clone(), session_key.clone()));
}

pub fn get_voting_power(env: &Env, voter: &Address) -> i128 {
    let mut total = 0i128;
    for deposit_id in get_active_ids(env, voter).iter() {
        if let Some(entry) = get_deposit_readonly(env, voter, deposit_id) {
            total = total.saturating_add(entry.amount);
        } else if let Some(entry) = get_deposit_by_ledger_readonly(env, voter, deposit_id) {
            total = total.saturating_add(entry.amount);
        }
    }
    total
}

// ----------------------------------------------------------------
//  Deposit helpers (single-token, timestamp-based)
// ----------------------------------------------------------------

pub fn set_deposit(env: &Env, depositor: &Address, deposit_id: u32, entry: &VaultEntry) {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    add_active_deposit_id(env, depositor, deposit_id);
}

pub fn get_deposit(env: &Env, depositor: &Address, deposit_id: u32) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    let entry: Option<VaultEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    entry
}

pub fn get_deposit_readonly(env: &Env, depositor: &Address, deposit_id: u32) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn remove_deposit(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_identity(env, depositor, deposit_id);
    remove_active_deposit_id(env, depositor, deposit_id);
}

pub fn add_tax_loss_harvest(env: &Env, depositor: &Address, harvest: &TaxLossHarvest) {
    let key = VaultKey::TaxLossHarvests(depositor.clone());
    let mut records: Vec<TaxLossHarvest> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    records.push_back(harvest.clone());
    env.storage().persistent().set(&key, &records);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_tax_loss_harvests(env: &Env, depositor: &Address) -> Vec<TaxLossHarvest> {
    env.storage()
        .persistent()
        .get(&VaultKey::TaxLossHarvests(depositor.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_tax_wash_sale_until(
    env: &Env,
    depositor: &Address,
    token: &Address,
    until: u64,
) {
    let key = VaultKey::TaxWashSaleUntil(depositor.clone(), token.clone());
    env.storage().persistent().set(&key, &until);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_tax_wash_sale_until(
    env: &Env,
    depositor: &Address,
    token: &Address,
) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&VaultKey::TaxWashSaleUntil(depositor.clone(), token.clone()))
}

// ----------------------------------------------------------------
//  Ledger-based deposit helpers
// ----------------------------------------------------------------

pub fn set_deposit_by_ledger(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    entry: &LedgerVaultEntry,
) {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    add_active_deposit_id(env, depositor, deposit_id);
}

pub fn get_deposit_by_ledger_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<LedgerVaultEntry> {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn remove_deposit_by_ledger(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_identity(env, depositor, deposit_id);
    remove_active_deposit_id(env, depositor, deposit_id);
}

// ----------------------------------------------------------------
//  Multi-token deposit helpers (issue #330)
// ----------------------------------------------------------------

/// Write a `MultiTokenVaultEntry` to persistent storage.
pub fn set_multi_deposit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    entry: &MultiTokenVaultEntry,
) {
    let key = VaultKey::MultiDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    add_active_deposit_id(env, depositor, deposit_id);
}

/// Read a `MultiTokenVaultEntry` (mutable path — extends TTL).
pub fn get_multi_deposit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<MultiTokenVaultEntry> {
    let key = VaultKey::MultiDeposit(depositor.clone(), deposit_id);
    let entry: Option<MultiTokenVaultEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    entry
}

/// Read a `MultiTokenVaultEntry` (read-only — does not extend TTL).
pub fn get_multi_deposit_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<MultiTokenVaultEntry> {
    let key = VaultKey::MultiDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Remove a `MultiTokenVaultEntry` from storage.
pub fn remove_multi_deposit(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::MultiDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_identity(env, depositor, deposit_id);
    remove_active_deposit_id(env, depositor, deposit_id);
}

// ----------------------------------------------------------------
//  Withdrawal whitelist helpers (issue #331)
// ----------------------------------------------------------------

/// Persist a whitelist for a specific deposit. An empty Vec means "no restriction".
pub fn set_withdrawal_whitelist(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    whitelist: &Vec<Address>,
) {
    let key = VaultKey::WithdrawalWhitelist(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, whitelist);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Read the whitelist for a deposit. Returns `None` if no whitelist has been configured.
pub fn get_withdrawal_whitelist(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<Vec<Address>> {
    let key = VaultKey::WithdrawalWhitelist(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Remove the whitelist when a deposit is withdrawn / cancelled.
pub fn remove_withdrawal_whitelist(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::WithdrawalWhitelist(depositor.clone(), deposit_id);
    // Only remove if present (avoid a panic on missing key).
    if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }
}

// ----------------------------------------------------------------
//  Admin helpers
// ----------------------------------------------------------------

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&VaultKey::Admin, admin);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Admin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::Admin)
}

pub fn remove_admin(env: &Env) {
    env.storage().persistent().remove(&VaultKey::Admin);
}

pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::PendingAdmin, pending);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::PendingAdmin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::PendingAdmin)
}

pub fn remove_pending_admin(env: &Env) {
    env.storage().persistent().remove(&VaultKey::PendingAdmin);
}

// ----------------------------------------------------------------
//  Initialized flag
// ----------------------------------------------------------------

pub fn set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&VaultKey::Initialized, &true);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Initialized, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::Initialized)
        .unwrap_or(false)
}

// ----------------------------------------------------------------
//  Runtime limits helpers
// ----------------------------------------------------------------

pub fn set_max_deposit(env: &Env, v: i128) {
    env.storage().persistent().set(&VaultKey::MaxDeposit, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxDeposit, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_max_deposit(env: &Env) -> Option<i128> {
    env.storage().persistent().get(&VaultKey::MaxDeposit)
}

pub fn set_max_lock_secs(env: &Env, v: u64) {
    env.storage().persistent().set(&VaultKey::MaxLockSecs, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxLockSecs, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Returns the runtime-configured max lock duration, or `None` to use the compile-time default.
pub fn get_max_lock_secs(env: &Env) -> Option<u64> {
    env.storage().persistent().get(&VaultKey::MaxLockSecs)
}

// ----------------------------------------------------------------
//  Fee recipient helpers
// ----------------------------------------------------------------

/// Persists the `fee_recipient` address and bumps TTL. Called once during `initialize`.
pub fn set_fee_recipient(env: &Env, recipient: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::FeeRecipient, recipient);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::FeeRecipient, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_fee_recipient(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::FeeRecipient)
}

/// Guard flag to prevent nested flash-loan re-entry in the same transaction.
pub fn set_flash_loan_guard(env: &Env, active: bool) {
    env.storage().persistent().set(&VaultKey::FlashLoanGuard, &active);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::FlashLoanGuard, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_flash_loan_guard_active(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::FlashLoanGuard)
        .unwrap_or(false)
}

pub fn set_flash_loan_state(
    env: &Env,
    borrower: &Address,
    token: &Address,
    state: &crate::types::FlashLoanState,
) {
    let key = VaultKey::FlashLoanState(borrower.clone(), token.clone());
    env.storage().persistent().set(&key, state);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_flash_loan_state(
    env: &Env,
    borrower: &Address,
    token: &Address,
) -> Option<crate::types::FlashLoanState> {
    let key = VaultKey::FlashLoanState(borrower.clone(), token.clone());
    env.storage().persistent().get(&key)
}

pub fn remove_flash_loan_state(env: &Env, borrower: &Address, token: &Address) {
    let key = VaultKey::FlashLoanState(borrower.clone(), token.clone());
    env.storage().persistent().remove(&key);
}

pub fn set_flash_loan_fee_balance(
    env: &Env,
    token: &Address,
    depositor: &Address,
    balance: i128,
) {
    let key = VaultKey::FlashLoanFeeBalance(token.clone(), depositor.clone());
    env.storage().persistent().set(&key, &balance);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_flash_loan_fee_balance(env: &Env, token: &Address, depositor: &Address) -> i128 {
    let key = VaultKey::FlashLoanFeeBalance(token.clone(), depositor.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

// ----------------------------------------------------------------
//  Depositor list helpers
// ----------------------------------------------------------------

fn get_depositor_list(env: &Env) -> soroban_sdk::Vec<Address> {
    env.storage()
        .persistent()
        .get(&VaultKey::DepositorList)
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}

fn save_depositor_list(env: &Env, list: &soroban_sdk::Vec<Address>) {
    env.storage()
        .persistent()
        .set(&VaultKey::DepositorList, list);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::DepositorList, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn add_depositor(env: &Env, depositor: &Address) {
    let flag_key = VaultKey::DepositorFlag(depositor.clone());
    let in_list_key = VaultKey::DepositorInList(depositor.clone());

    if env
        .storage()
        .persistent()
        .get::<VaultKey, bool>(&flag_key)
        .unwrap_or(false)
    {
        return;
    }

    env.storage().persistent().set(&flag_key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&flag_key, BUMP_THRESHOLD, BUMP_TARGET);

    if !env
        .storage()
        .persistent()
        .get::<VaultKey, bool>(&in_list_key)
        .unwrap_or(false)
    {
        env.storage().persistent().set(&in_list_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&in_list_key, BUMP_THRESHOLD, BUMP_TARGET);

        let mut list = get_depositor_list(env);
        list.push_back(depositor.clone());
        save_depositor_list(env, &list);
    }
}

pub fn remove_depositor(env: &Env, depositor: &Address) {
    let flag_key = VaultKey::DepositorFlag(depositor.clone());
    env.storage().persistent().remove(&flag_key);
}

pub fn get_depositor_count(env: &Env) -> u32 {
    let list = get_depositor_list(env);
    let mut count: u32 = 0;
    for addr in list.iter() {
        let flag_key = VaultKey::DepositorFlag(addr.clone());
        if env
            .storage()
            .persistent()
            .get::<VaultKey, bool>(&flag_key)
            .unwrap_or(false)
        {
            count = count.saturating_add(1);
        }
    }
    count
}

// ----------------------------------------------------------------
//  Paused flag helpers
// ----------------------------------------------------------------

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&VaultKey::Paused, &paused);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Paused, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::Paused)
        .unwrap_or(false)
}

// ----------------------------------------------------------------
//  Token allowlist helpers
// ----------------------------------------------------------------

pub fn set_token_allowed(env: &Env, token: &Address, allowed: bool) {
    let key = VaultKey::AllowedToken(token.clone());
    if allowed {
        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    } else {
        env.storage().persistent().remove(&key);
    }
}

pub fn is_token_allowed(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::AllowedToken(token.clone()))
        .unwrap_or(false)
}

pub fn set_strict_token_allowlist(env: &Env, strict: bool) {
    let key = VaultKey::StrictTokenAllowlist;
    env.storage().persistent().set(&key, &strict);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_strict_token_allowlist(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::StrictTokenAllowlist)
        .unwrap_or(false)
}

// ----------------------------------------------------------------
//  Token vetting workflow helpers
// ----------------------------------------------------------------

pub fn set_token_vetting(env: &Env, token: &Address, vetting: &TokenVetting) {
    let key = VaultKey::TokenVetting(token.clone());
    env.storage().persistent().set(&key, vetting);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_token_vetting(env: &Env, token: &Address) -> Option<TokenVetting> {
    env.storage()
        .persistent()
        .get(&VaultKey::TokenVetting(token.clone()))
}

pub fn next_proposal_id(env: &Env) -> u32 {
    let id: u32 = env.storage().persistent().get(&VaultKey::ProposalCounter).unwrap_or(0);
    env.storage().persistent().set(&VaultKey::ProposalCounter, &(id + 1));
    env.storage().persistent().extend_ttl(&VaultKey::ProposalCounter, BUMP_THRESHOLD, BUMP_TARGET);
    id
}

pub fn set_governance_proposal(env: &Env, id: u32, proposal: &GovernanceProposal) {
    let key = VaultKey::GovernanceProposal(id);
    env.storage().persistent().set(&key, proposal);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_governance_proposal(env: &Env, id: u32) -> Option<GovernanceProposal> {
    env.storage().persistent().get(&VaultKey::GovernanceProposal(id))
}

pub fn has_governance_vote(env: &Env, id: u32, voter: &Address) -> bool {
    env.storage().persistent().get::<VaultKey, bool>(&VaultKey::GovernanceVote(id, voter.clone())).unwrap_or(false)
}

pub fn set_governance_vote(env: &Env, id: u32, voter: &Address) {
    let key = VaultKey::GovernanceVote(id, voter.clone());
    env.storage().persistent().set(&key, &true);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Returns the raw append-only depositor list (may contain stale entries after
/// O(1) removes).  Callers that need only *active* depositors should filter with
/// `depositor_is_active`.
pub fn get_all_depositors_raw(env: &Env) -> soroban_sdk::Vec<Address> {
    get_depositor_list(env)
}

/// Returns `true` if `depositor` currently has an active existence flag.
pub fn depositor_is_active(env: &Env, depositor: &Address) -> bool {
    let flag_key = VaultKey::DepositorFlag(depositor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&flag_key)
        .unwrap_or(false)
}

pub fn get_depositors_page(env: &Env, offset: u32, limit: u32) -> (soroban_sdk::Vec<Address>, u32) {
    let list = get_depositor_list(env);
    let mut page: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(env);
    let mut active_seen: u32 = 0;
    let end_at = offset.saturating_add(limit);

    let mut total_count: u32 = 0;
    for addr in list.iter() {
        let flag_key = VaultKey::DepositorFlag(addr.clone());
        if env
            .storage()
            .persistent()
            .get::<VaultKey, bool>(&flag_key)
            .unwrap_or(false)
        {
            total_count = total_count.saturating_add(1);
        }
    }

    for addr in list.iter() {
        let flag_key = VaultKey::DepositorFlag(addr.clone());
        if !env
            .storage()
            .persistent()
            .get::<VaultKey, bool>(&flag_key)
            .unwrap_or(false)
        {
            continue;
        }
        if active_seen >= offset && active_seen < end_at {
            page.push_back(addr.clone());
        }
        active_seen = active_seen.saturating_add(1);
        if active_seen >= end_at {
            break;
        }
    }
    (page, total_count)
}

// ----------------------------------------------------------------
//  Admin authorization check helper
// ----------------------------------------------------------------

/// Returns `Ok(())` if `caller` matches the stored admin, else `Err(VaultError::Unauthorized)`.
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), crate::errors::VaultError> {
    match get_admin(env) {
        Some(ref stored) if stored == caller => Ok(()),
        _ => Err(crate::errors::VaultError::Unauthorized),
    }
}

// ----------------------------------------------------------------
//  Storage version helpers
// ----------------------------------------------------------------

/// Write the current schema version into persistent storage.
pub fn set_storage_version(env: &Env, version: u32) {
    env.storage()
        .persistent()
        .set(&VaultKey::StorageVersion, &version);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::StorageVersion, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Read the schema version that was last written by `migrate()`.
/// Returns `None` for contracts deployed before versioning was introduced.
pub fn get_storage_version(env: &Env) -> Option<u32> {
    env.storage().persistent().get(&VaultKey::StorageVersion)
}

// ----------------------------------------------------------------
//  Withdrawal limit per epoch helpers
// ----------------------------------------------------------------

/// Calculate the current epoch number based on the ledger sequence.
/// Each epoch spans EPOCH_SIZE_LEDGERS ledgers.
pub fn get_current_epoch(env: &Env) -> u32 {
    let current_ledger = env.ledger().sequence();
    current_ledger / EPOCH_SIZE_LEDGERS
}

/// Get the withdrawal count for a depositor in a given epoch.
pub fn get_withdrawal_count(env: &Env, depositor: &Address, epoch: u32) -> u32 {
    let key = VaultKey::WithdrawalCountPerEpoch(depositor.clone(), epoch);
    env.storage()
        .persistent()
        .get::<VaultKey, u32>(&key)
        .unwrap_or(0)
}

/// Increment the withdrawal count for a depositor in the current epoch.
/// Returns the new count after incrementing.
pub fn increment_withdrawal_count(env: &Env, depositor: &Address, epoch: u32) -> u32 {
    let key = VaultKey::WithdrawalCountPerEpoch(depositor.clone(), epoch);
    let current_count: u32 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_count = current_count.saturating_add(1);

    env.storage().persistent().set(&key, &new_count);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);

    new_count
}

/// Check if a depositor has exceeded the withdrawal limit for the current epoch.
/// Returns `Ok(())` if within limit, `Err(WithdrawalLimitExceeded)` if exceeded.
pub fn check_withdrawal_limit(
    env: &Env,
    depositor: &Address,
) -> Result<(), crate::errors::VaultError> {
    let current_epoch = get_current_epoch(env);
    let current_count = get_withdrawal_count(env, depositor, current_epoch);

    if current_count >= WITHDRAWAL_LIMIT_PER_EPOCH {
        Err(crate::errors::VaultError::WithdrawalLimitExceeded)
    } else {
        Ok(())
    }
}

/// Clean up old epoch counters to prevent unbounded storage growth.
/// Removes withdrawal counters for epochs older than `num_epochs_to_keep`.
/// This is called after a withdrawal to remove the counter from the previous epoch.
pub fn cleanup_old_epochs(env: &Env, depositor: &Address, current_epoch: u32) {
    // Keep withdrawal counts for the current epoch and the previous epoch only.
    // Earlier epochs are cleaned up to prevent unbounded storage growth.
    if current_epoch > 1 {
        let previous_epoch = current_epoch.saturating_sub(1);
        // Remove counters from epochs before the previous one
        let epoch_to_remove = previous_epoch.saturating_sub(1);
        if epoch_to_remove > 0 {
            for epoch in 0..epoch_to_remove {
                let key = VaultKey::WithdrawalCountPerEpoch(depositor.clone(), epoch);
                env.storage().persistent().remove(&key);
            }
        }
    }
}


// ----------------------------------------------------------------
//  Staker registry helpers
// ----------------------------------------------------------------

use crate::types::StakerEntry;

/// Set a staker's stake amount. Creates or updates the entry.
pub fn set_staker(env: &Env, staker: &Address, stake_amount: i128) {
    let key = VaultKey::Staker(staker.clone());
    env.storage().persistent().set(&key, &stake_amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get a staker's stake amount. Returns `None` if not registered.
pub fn get_staker(env: &Env, staker: &Address) -> Option<i128> {
    let key = VaultKey::Staker(staker.clone());
    env.storage().persistent().get(&key)
}

/// Get staker list (append-only, may contain stale entries after removes).
fn get_staker_list(env: &Env) -> Vec<Address> {
    let key = VaultKey::StakerList;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

/// Save the staker list.
fn save_staker_list(env: &Env, stakers: &Vec<Address>) {
    let key = VaultKey::StakerList;
    env.storage().persistent().set(&key, stakers);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Add a staker to the staker list (O(1) if not already in list).
pub fn add_staker_to_list(env: &Env, staker: &Address) {
    // Check if already in list using StakerInList flag
    let in_list_key = VaultKey::StakerInList(staker.clone());
    if env
        .storage()
        .persistent()
        .get::<VaultKey, bool>(&in_list_key)
        .unwrap_or(false)
    {
        return; // Already in list, skip
    }

    // Add to list
    let mut list = get_staker_list(env);
    list.push_back(staker.clone());
    save_staker_list(env, &list);

    // Mark as in list
    env.storage().persistent().set(&in_list_key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&in_list_key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the total amount staked by all stakers.
pub fn get_total_staked(env: &Env) -> i128 {
    let key = VaultKey::TotalStaked;
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Update the total staked amount.
pub fn set_total_staked(env: &Env, total: i128) {
    let key = VaultKey::TotalStaked;
    env.storage().persistent().set(&key, &total);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the rewards pool (accumulated penalties for stakers).
pub fn get_rewards_pool(env: &Env) -> i128 {
    let key = VaultKey::RewardsPool;
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Update the rewards pool.
pub fn set_rewards_pool(env: &Env, amount: i128) {
    let key = VaultKey::RewardsPool;
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the rewards claimed by a staker (cumulative).
pub fn get_staker_rewards_claimed(env: &Env, staker: &Address) -> i128 {
    let key = VaultKey::StakerRewardsClaimed(staker.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Update the rewards claimed by a staker.
pub fn set_staker_rewards_claimed(env: &Env, staker: &Address, amount: i128) {
    let key = VaultKey::StakerRewardsClaimed(staker.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the staker list (may contain stale entries). Active check via `get_staker`.
pub fn get_stakers_list(env: &Env) -> Vec<Address> {
    get_staker_list(env)
}

// ----------------------------------------------------------------
//  Emergency Lockdown helpers
// ----------------------------------------------------------------

/// Set the emergency lockdown flag to `true` to activate lockdown.
pub fn set_emergency_lockdown(env: &Env, is_locked: bool) {
    let key = VaultKey::EmergencyLockdown;
    env.storage().persistent().set(&key, &is_locked);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the current emergency lockdown status. Returns `false` if not set.
pub fn is_emergency_lockdown(env: &Env) -> bool {
    let key = VaultKey::EmergencyLockdown;
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&key)
        .unwrap_or(false)
}

/// Set the timestamp when lockdown was activated.
pub fn set_lockdown_activated_at(env: &Env, timestamp: u64) {
    let key = VaultKey::LockdownActivatedAt;
    env.storage().persistent().set(&key, &timestamp);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the timestamp when lockdown was activated.
pub fn get_lockdown_activated_at(env: &Env) -> Option<u64> {
    let key = VaultKey::LockdownActivatedAt;
    env.storage().persistent().get(&key)
}

/// Set the reason for lockdown.
pub fn set_lockdown_reason(env: &Env, reason: &soroban_sdk::String) {
    let key = VaultKey::LockdownReason;
    env.storage().persistent().set(&key, reason);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the reason for lockdown.
pub fn get_lockdown_reason(env: &Env) -> Option<soroban_sdk::String> {
    let key = VaultKey::LockdownReason;
    env.storage().persistent().get(&key)
}

/// Add a lockdown entry to the history list.
pub fn add_lockdown_history(env: &Env, entry: &crate::types::LockdownEntry) {
    let key = VaultKey::LockdownHistory;
    let mut history: Vec<crate::types::LockdownEntry> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    history.push_back(entry.clone());
    env.storage().persistent().set(&key, &history);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get the lockdown history.
pub fn get_lockdown_history(env: &Env) -> Vec<crate::types::LockdownEntry> {
    let key = VaultKey::LockdownHistory;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

// ----------------------------------------------------------------
//  Archived deposit helpers
// ----------------------------------------------------------------

/// Archive a timestamp-based deposit by moving it to archived storage
/// and recording the archive timestamp.
pub fn set_archived_deposit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    entry: &VaultEntry,
    archive_timestamp: u64,
) {
    let key = VaultKey::ArchivedDeposit(depositor.clone(), deposit_id);
    let archived = ArchivedVaultEntry {
        token: entry.token.clone(),
        amount: entry.amount,
        unlock_time: entry.unlock_time,
        depositor: depositor.clone(),
        penalty_bps: entry.penalty_bps,
        archive_timestamp,
    };
    env.storage().persistent().set(&key, &archived);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieve an archived timestamp-based deposit without bumping TTL.
pub fn get_archived_deposit_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<ArchivedVaultEntry> {
    let key = VaultKey::ArchivedDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Delete an archived timestamp-based deposit from storage.
pub fn remove_archived_deposit(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::ArchivedDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
}

/// Archive a ledger-based deposit by moving it to archived storage
/// and recording the archive timestamp.
pub fn set_archived_deposit_by_ledger(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    entry: &LedgerVaultEntry,
    archive_timestamp: u64,
) {
    let key = VaultKey::ArchivedDepositByLedger(depositor.clone(), deposit_id);
    let archived = ArchivedLedgerVaultEntry {
        token: entry.token.clone(),
        amount: entry.amount,
        unlock_ledger: entry.unlock_ledger,
        depositor: depositor.clone(),
        penalty_bps: entry.penalty_bps,
        archive_timestamp,
    };
    env.storage().persistent().set(&key, &archived);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieve an archived ledger-based deposit without bumping TTL.
pub fn get_archived_deposit_by_ledger_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<ArchivedLedgerVaultEntry> {
    let key = VaultKey::ArchivedDepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Delete an archived ledger-based deposit from storage.
pub fn remove_archived_deposit_by_ledger(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::ArchivedDepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
}


// ----------------------------------------------------------------
//  Emergency withdrawal per-ledger tracking helpers
// ----------------------------------------------------------------

/// Add `amount` to the cumulative emergency withdrawal total for the current ledger.
/// Returns the new total after adding.
pub fn add_emergency_withdrawal(env: &Env, amount: i128) -> i128 {
    let ledger = env.ledger().sequence();
    let key = crate::types::VaultKey::EmergencyWithdrawalPerLedger(ledger);
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = current.saturating_add(amount);
    env.storage().persistent().set(&key, &new_total);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    new_total
}

/// Get the cumulative emergency withdrawal amount for a specific ledger.
/// Returns 0 if no withdrawals have been made in that ledger.
pub fn get_emergency_withdrawal_per_ledger(env: &Env, ledger: u32) -> i128 {
    let key = crate::types::VaultKey::EmergencyWithdrawalPerLedger(ledger);
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Get the cumulative emergency withdrawal amount for the current ledger.
pub fn get_current_ledger_emergency_withdrawal(env: &Env) -> i128 {
    get_emergency_withdrawal_per_ledger(env, env.ledger().sequence())
}

pub fn record_circuit_breaker_activation(env: &Env, activation: &CircuitBreakerActivation) {
    let key = VaultKey::CircuitBreakerHistory;
    let mut history: Vec<CircuitBreakerActivation> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    history.push_back(activation.clone());
    env.storage().persistent().set(&key, &history);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_circuit_breaker_history(env: &Env) -> Vec<CircuitBreakerActivation> {
    env.storage()
        .persistent()
        .get(&VaultKey::CircuitBreakerHistory)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn next_subscription_id(env: &Env, depositor: &Address) -> u32 {
    let key = VaultKey::SubscriptionCounter(depositor.clone());
    let id: u32 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().set(&key, &id.saturating_add(1));
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    id
}

pub fn set_subscription(env: &Env, depositor: &Address, id: u32, subscription: &DepositSubscription) {
    let key = VaultKey::Subscription(depositor.clone(), id);
    env.storage().persistent().set(&key, subscription);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    let ids_key = VaultKey::SubscriptionIds(depositor.clone());
    let mut ids: Vec<u32> = env.storage().persistent().get(&ids_key).unwrap_or_else(|| Vec::new(env));
    let already_present = ids.iter().any(|existing| existing == id);
    if !already_present {
        ids.push_back(id);
        env.storage().persistent().set(&ids_key, &ids);
    }
    env.storage().persistent().extend_ttl(&ids_key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_subscription(env: &Env, depositor: &Address, id: u32) -> Option<DepositSubscription> {
    let key = VaultKey::Subscription(depositor.clone(), id);
    let subscription = env.storage().persistent().get(&key);
    if subscription.is_some() {
        env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    subscription
}

pub fn get_subscription_readonly(env: &Env, depositor: &Address, id: u32) -> Option<DepositSubscription> {
    env.storage().persistent().get(&VaultKey::Subscription(depositor.clone(), id))
}

pub fn get_subscription_ids(env: &Env, depositor: &Address) -> Vec<u32> {
    env.storage().persistent().get(&VaultKey::SubscriptionIds(depositor.clone())).unwrap_or_else(|| Vec::new(env))
}

pub fn record_subscription_execution(env: &Env, depositor: &Address, id: u32, execution: &SubscriptionExecution) {
    let key = VaultKey::SubscriptionHistory(depositor.clone(), id);
    let mut history: Vec<SubscriptionExecution> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
    history.push_back(execution.clone());
    env.storage().persistent().set(&key, &history);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_subscription_history(env: &Env, depositor: &Address, id: u32) -> Vec<SubscriptionExecution> {
    env.storage().persistent().get(&VaultKey::SubscriptionHistory(depositor.clone(), id)).unwrap_or_else(|| Vec::new(env))
}

pub fn set_subscription_stats(env: &Env, depositor: &Address, id: u32, stats: &SubscriptionStats) {
    let key = VaultKey::SubscriptionStats(depositor.clone(), id);
    env.storage().persistent().set(&key, stats);
    env.storage().persistent().extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_subscription_stats(env: &Env, depositor: &Address, id: u32) -> SubscriptionStats {
    env.storage().persistent().get(&VaultKey::SubscriptionStats(depositor.clone(), id)).unwrap_or(SubscriptionStats { deposit_count: 0, total_amount: 0, last_execution_time: 0 })
}

// ----------------------------------------------------------------
//  Upgrade helpers
// ----------------------------------------------------------------

/// Sets the current contract version string
pub fn set_contract_version(env: &Env, version: &str) {
    env.storage()
        .persistent()
        .set(&VaultKey::ContractVersion, version);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::ContractVersion, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieves the current contract version string
pub fn get_contract_version(env: &Env) -> Option<String> {
    env.storage()
        .persistent()
        .get(&VaultKey::ContractVersion)
}

/// Stores the previous contract version for rollback support
pub fn set_previous_contract_version(env: &Env, version: &str) {
    env.storage()
        .persistent()
        .set(&VaultKey::PreviousContractVersion, version);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::PreviousContractVersion, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieves the previous contract version for rollback support
pub fn get_previous_contract_version(env: &Env) -> Option<String> {
    env.storage()
        .persistent()
        .get(&VaultKey::PreviousContractVersion)
}

/// Sets a pending upgrade proposal
pub fn set_upgrade(env: &Env, upgrade: &crate::types::UpgradeEntry) {
    env.storage()
        .persistent()
        .set(&VaultKey::PendingUpgrade, upgrade);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::PendingUpgrade, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieves the current pending upgrade proposal
pub fn get_upgrade(env: &Env) -> Option<crate::types::UpgradeEntry> {
    env.storage()
        .persistent()
        .get(&VaultKey::PendingUpgrade)
}

/// Removes the pending upgrade proposal from storage
pub fn remove_upgrade(env: &Env) {
    env.storage()
        .persistent()
        .remove(&VaultKey::PendingUpgrade);
}

pub fn set_faucet_asset(env: &Env, asset: &FaucetAsset, token: &Address, max_amount: i128) {
    env.storage().persistent().set(&VaultKey::FaucetToken(asset.clone()), token);
    env.storage().persistent().set(&VaultKey::FaucetMaxAmount(asset.clone()), &max_amount);
    env.storage().persistent().extend_ttl(&VaultKey::FaucetToken(asset.clone()), BUMP_THRESHOLD, BUMP_TARGET);
    env.storage().persistent().extend_ttl(&VaultKey::FaucetMaxAmount(asset.clone()), BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_faucet_status(env: &Env, asset: &FaucetAsset) -> FaucetStatus {
    let token = env.storage().persistent().get(&VaultKey::FaucetToken(asset.clone()));
    let max_amount = env.storage().persistent().get(&VaultKey::FaucetMaxAmount(asset.clone())).unwrap_or(0);
    let request_count = env.storage().persistent().get(&VaultKey::FaucetRequestCount(asset.clone())).unwrap_or(0);
    let distributed = env.storage().persistent().get(&VaultKey::FaucetDistributed(asset.clone())).unwrap_or(0);
    let balance = token.as_ref().map(|address: &Address| token::Client::new(env, address).balance(&env.current_contract_address())).unwrap_or(0);
    FaucetStatus { token, balance, max_amount, request_count, distributed }
}

pub fn get_faucet_last_request(env: &Env, account: &Address) -> Option<u64> {
    env.storage().persistent().get(&VaultKey::FaucetLastRequest(account.clone()))
}

pub fn record_faucet_request(env: &Env, account: &Address, asset: &FaucetAsset, amount: i128, now: u64) {
    let last_key = VaultKey::FaucetLastRequest(account.clone());
    env.storage().persistent().set(&last_key, &now);
    env.storage().persistent().extend_ttl(&last_key, BUMP_THRESHOLD, BUMP_TARGET);
    let count_key = VaultKey::FaucetRequestCount(asset.clone());
    let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
    env.storage().persistent().set(&count_key, &count.saturating_add(1));
    let total_key = VaultKey::FaucetDistributed(asset.clone());
    let total: i128 = env.storage().persistent().get(&total_key).unwrap_or(0);
    env.storage().persistent().set(&total_key, &total.saturating_add(amount));
}

// ----------------------------------------------------------------
//  Sustainability metrics helpers
// ----------------------------------------------------------------

/// Store sustainability metrics for a specific deposit.
pub fn set_sustainability_metrics(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    metrics: &SustainabilityMetrics,
) {
    let key = VaultKey::SustainabilityMetrics(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, metrics);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieve sustainability metrics for a specific deposit.
pub fn get_sustainability_metrics(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<SustainabilityMetrics> {
    let key = VaultKey::SustainabilityMetrics(depositor.clone(), deposit_id);
    let metrics: Option<SustainabilityMetrics> = env.storage().persistent().get(&key);
    if metrics.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    metrics
}

/// Retrieve sustainability metrics without extending TTL (read-only).
pub fn get_sustainability_metrics_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<SustainabilityMetrics> {
    let key = VaultKey::SustainabilityMetrics(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Remove sustainability metrics for a specific deposit.
pub fn remove_sustainability_metrics(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::SustainabilityMetrics(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
}

/// Get total carbon footprint for a depositor.
pub fn get_total_carbon_footprint(env: &Env, depositor: &Address) -> i128 {
    let key = VaultKey::TotalCarbonFootprint(depositor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, i128>(&key)
        .unwrap_or(0)
}

/// Set total carbon footprint for a depositor.
pub fn set_total_carbon_footprint(env: &Env, depositor: &Address, total: i128) {
    let key = VaultKey::TotalCarbonFootprint(depositor.clone());
    env.storage().persistent().set(&key, &total);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Get total carbon offset for a depositor.
pub fn get_total_carbon_offset(env: &Env, depositor: &Address) -> i128 {
    let key = VaultKey::TotalCarbonOffset(depositor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, i128>(&key)
        .unwrap_or(0)
}

/// Set total carbon offset for a depositor.
pub fn set_total_carbon_offset(env: &Env, depositor: &Address, total: i128) {
    let key = VaultKey::TotalCarbonOffset(depositor.clone());
    env.storage().persistent().set(&key, &total);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Check if a depositor has achieved a specific milestone.
/// Returns a bitmap where each bit represents a different milestone type.
pub fn get_milestone_bitmap(env: &Env, depositor: &Address) -> u32 {
    let key = VaultKey::MilestoneAchieved(depositor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, u32>(&key)
        .unwrap_or(0)
}

/// Set milestone bitmap for a depositor.
pub fn set_milestone_bitmap(env: &Env, depositor: &Address, bitmap: u32) {
    let key = VaultKey::MilestoneAchieved(depositor.clone());
    env.storage().persistent().set(&key, &bitmap);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}


// ================================================================
//  NFT Evolution helpers
// ================================================================

/// Store an NFT evolution record for a deposit.
pub fn set_nft_evolution(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    record: &crate::nft::NFTEvolutionRecord,
) {
    let key = crate::types::VaultKey::NFTEvolution(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Retrieve an NFT evolution record (mutable path — extends TTL).
pub fn get_nft_evolution(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<crate::nft::NFTEvolutionRecord> {
    let key = crate::types::VaultKey::NFTEvolution(depositor.clone(), deposit_id);
    let record: Option<crate::nft::NFTEvolutionRecord> = env.storage().persistent().get(&key);
    if record.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    record
}

/// Retrieve an NFT evolution record (read-only — does not extend TTL).
pub fn get_nft_evolution_readonly(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<crate::nft::NFTEvolutionRecord> {
    let key = crate::types::VaultKey::NFTEvolution(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

/// Remove an NFT evolution record from storage.
pub fn remove_nft_evolution(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = crate::types::VaultKey::NFTEvolution(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
}
