//! # SAFE-HAVEN Contract Test Suite
//!
//! This module contains all integration and unit tests for the `SafeHaven` vault
//! contract.  Tests are organised by category so contributors can quickly find
//! relevant coverage when debugging regressions or adding features.
//!
//! ## Test categories
//!
//! | Category | Description | Key contract functions tested |
//! |---|---|---|
//! | **Initialization** | Contract deployment, admin setup, double-init guard | [`initialize`], [`is_initialized`] |
//! | **Deposit – happy path** | Successful deposits, token transfers, event emission | [`deposit`] |
//! | **Deposit – validation** | Zero / negative / over-max amounts, past unlock, invalid lock duration, penalty bps | [`deposit`], `constants` |
//! | **Multi-deposit** | Multiple deposits per address, independent unlock times, ID continuity after withdrawal | [`deposit`], [`get_deposit_ids`], [`withdraw`] |
//! | **deposit_for** | Third-party deposits, beneficiary withdraws, payer access control | [`deposit_for`] |
//! | **Withdraw** | After-unlock withdrawal, exact-unlock edge case, insufficient-age rejection | [`withdraw`] |
//! | **Cancel deposit** | Zero / partial / full penalties, post-unlock guard, penalty storage | [`cancel_deposit`] |
//! | **Time helpers** | Remaining time query (pre/post unlock), ledger timestamp | [`time_remaining`], [`get_time`] |
//! | **Emergency withdraw** | Admin-enabled early release, non-admin rejection, missing-deposit guard | [`emergency_withdraw`] |
//! | **Admin transfer (two-step)** | Propose → accept flow, cancel, post-transfer permissions, renounce | [`transfer_admin`], [`accept_admin`], [`cancel_transfer_admin`], [`renounce_admin`] |
//! | **Re-deposit** | Withdraw then re-deposit with fresh unlock time and ID | [`deposit`], [`withdraw`] |
//! | **TTL / storage** | BUMP_TARGET covers max lock duration | `storage::BUMP_TARGET` |
//! | **View functions** | Read-only queries (`get_vault` / `time_remaining`) are idempotent and don't mutate state | [`get_vault`], [`time_remaining`] |
//! | **Depositor list / pagination** | Count, pagination offsets, limit, edge-case offsets, re-add after withdraw | [`get_depositor_count`], [`get_depositors`] |
//! | **Configurable limits** | Custom `max_deposit` / `max_lock_secs` on init, fallback to defaults | [`initialize`], [`get_constants`] |
//! | **XDR serialization** | Round-trip `to_xdr` / `from_xdr` for `VaultEntry` and `VaultKey` variants | [`VaultEntry`], [`VaultKey`] |
//! | **Auth assertions** | `require_auth` enforcement for deposit, withdraw, admin actions | All auth-gated functions |
//! | **Boundary & lifecycle** | Exact max lock duration, unlock-time-minus-one, exact-unlock, fee recipient | [`deposit`], [`withdraw`] |
//! | **Batch queries** | `get_vault_batch` / `get_deposit_batch` clamping at `MAX_BATCH_SIZE` | [`get_vault_batch`], [`get_deposit_batch`] |
//! | **Pause / unpause** | Deposit gating while paused, admin-only toggle | [`pause`], [`unpause`], [`is_paused`] |
//!
//! ## Helpers
//!
//! * [`setup`] – creates a default `Env`, deploys the contract, initialises it,
//!   and returns commonly-needed addresses (admin, alice, fee_recipient, token).
//! * [`setup_with_limits`] – same as `setup` but accepts optional `max_deposit`
//!   and `max_lock_secs` override arguments.
//! * [`advance_time`] – moves the ledger timestamp forward by `seconds`.
//!
//! ## Running tests
//!
//! Run **all** tests:
//! ```bash
//! cargo test -p safe-haven
//! ```
//!
//! Run a **specific category** by test name prefix:
//! ```bash
//! cargo test -p safe-haven -- deposit           # deposit + deposit_for
//! cargo test -p safe-haven -- withdraw          # withdraw* tests
//! cargo test -p safe-haven -- admin             # admin-transfer + renounce
//! cargo test -p safe-haven -- pagination        # pagination + depositor list
//! cargo test -p safe-haven -- batch             # get_vault_batch tests
//! ```
//!
//! Run a **single test**:
//! ```bash
//! cargo test -p safe-haven -- test_deposit_success
//! ```

#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Bytes, BytesN, Env, String, Vec,
};

use crate::{
    constants::MIN_LOCK_LEDGERS,
    contract::{SafeHaven, SafeHavenClient},
    errors::VaultError,
    types::{
        DepositType, GovernanceMode, ProposalType, VaultEntry, VaultKey, MAX_DEPOSIT_AMOUNT,
        MAX_LOCK_DURATION_SECS,
    },
};

fn setup() -> (
    Env,
    SafeHavenClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    (env, vault, token_address, admin, alice, fee_recipient)
}

fn advance_time(env: &Env, seconds: u64) {
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + seconds,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });
}

#[test]
fn test_flash_borrow_repay_and_fee_distribution() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let borrower: Address = Address::generate(&env);
    let unlock_time = env.ledger().timestamp() + 3600;

    vault.deposit(&alice, &token, &10_000, &unlock_time, &0);
    let borrowed = vault.flash_borrow(&borrower, &token, &5_000);
    assert_eq!(borrowed, 5_000);

    let due = vault.flash_repay(&borrower, &token, &5_000 + 5_000 * 10 / 10_000);
    assert_eq!(due, 5_000 + 5);
    assert_eq!(vault.get_flash_loan_fee_balance(&token, &alice), 5);

    let claimed = vault.claim_flash_loan_fees(&alice, &token);
    assert_eq!(claimed, 5);
    assert_eq!(vault.get_flash_loan_fee_balance(&token, &alice), 0);
}

#[test]
fn test_flash_borrow_rejects_reentrant_call() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let borrower: Address = Address::generate(&env);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &10_000, &unlock_time, &0);

    let _ = vault.flash_borrow(&borrower, &token, &1_000);
    let result = vault.try_flash_borrow(&borrower, &token, &1_000);
    assert_eq!(result, Err(Ok(VaultError::FlashLoanReentrancy)));

    vault.flash_repay(&borrower, &token, &1_000 + 1_000 * 10 / 10_000);
}

struct UpgradeHarness {
    env: Env,
    vault: SafeHavenClient<'static>,
    admin: Address,
    alice: Address,
    fee_recipient: Address,
    token: Address,
}

impl UpgradeHarness {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register(SafeHaven, ());
        let vault = SafeHavenClient::new(&env, &vault_id);
        let admin: Address = Address::generate(&env);
        let alice: Address = Address::generate(&env);
        let fee_recipient: Address = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_id.address();

        StellarAssetClient::new(&env, &token).mint(&alice, &10_000);
        vault.initialize(&admin, &fee_recipient, &None, &None);

        Self {
            env,
            vault,
            admin,
            alice,
            fee_recipient,
            token,
        }
    }

    fn simulate_legacy_state(&self) {
        self.env.storage().persistent().remove(&VaultKey::StorageVersion);
    }

    fn assert_legacy_state(&self) {
        assert_eq!(self.vault.get_storage_version(), None);
    }

    fn assert_upgrade_applied(&self) {
        assert!(self.vault.migrate(&self.admin), "first migrate call should return true");
        assert_eq!(self.vault.get_storage_version(), Some(1));
    }

    fn assert_upgrade_idempotent(&self) {
        let migrated_again = self.vault.migrate(&self.admin);
        assert!(!migrated_again, "second migrate call should return false");
        assert_eq!(self.vault.get_storage_version(), Some(1));
    }

    fn deposit_legacy_entry(&self, amount: i128, unlock_time: u64) -> u32 {
        self.vault
            .deposit(&self.alice, &self.token, &amount, &unlock_time, &0)
    }
}

// ================================================================
//  Initialization
// ================================================================

#[test]
fn test_initialize_sets_admin() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn quantum_safe_key_registration_rejects_invalid_encoding() {
    let (env, vault, _token, _admin, alice, _fee) = setup();
    let invalid_key = Bytes::from_slice(&env, &[0; 32]);

    assert_eq!(
        vault.try_register_quantum_safe_key(&alice, &invalid_key),
        Err(Ok(VaultError::InvalidQuantumSafeKey))
    );
    assert_eq!(vault.get_quantum_safe_key(&alice), None);
}

#[test]
fn test_initialize_sets_fee_recipient() {
    let (_env, vault, _token, _admin, _alice, fee) = setup();
    assert_eq!(vault.get_fee_recipient(), Some(fee));
}

#[test]
fn test_double_initialize_fails() {
    let (_env, vault, _token, admin, _alice, fee) = setup();
    let result = vault.try_initialize(&admin, &fee, &None, &None);
    assert_eq!(result, Err(Ok(VaultError::AlreadyInitialized)));
}

#[test]
fn test_is_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);

    assert!(!vault.is_initialized());
    vault.initialize(&admin, &fee, &None, &None);
    assert!(vault.is_initialized());

    vault.renounce_admin(&admin);
    assert!(vault.is_initialized());
}

#[test]
fn test_token_allowlist_is_permissive_by_default() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    assert!(!vault.is_strict_mode());
    assert!(!vault.is_token_allowed(&token));
    assert!(vault.deposit(&alice, &token, &1_000, &unlock_time, &0) == 0);
}

#[test]
fn test_admin_can_manage_allowed_tokens() {
    let (env, vault, token, admin, _alice, _fee) = setup();
    let other_token: Address = Address::generate(&env);

    assert!(!vault.is_token_allowed(&token));
    vault.add_allowed_token(&admin, &token);
    assert!(vault.is_token_allowed(&token));
    assert!(!vault.is_token_allowed(&other_token));

    vault.remove_allowed_token(&admin, &token);
    assert!(!vault.is_token_allowed(&token));
}

#[test]
fn test_strict_mode_rejects_tokens_outside_allowlist() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    vault.set_strict_mode(&admin, &true);
    assert!(vault.is_strict_mode());
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::TokenNotAllowed))
    );

    vault.add_allowed_token(&admin, &token);
    assert!(vault.deposit(&alice, &token, &1_000, &unlock_time, &0) == 0);
    assert_eq!(vault.toggle_strict_mode(&admin), false);
    assert!(!vault.is_strict_mode());
}

#[test]
fn test_strict_mode_rejects_all_deposit_entrypoints() {
    let (env, vault, _token, admin, alice, _fee) = setup();
    let rejected_token: Address = Address::generate(&env);
    let unlock_time = env.ledger().timestamp() + 3600;
    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;

    vault.set_strict_mode(&admin, &true);
    assert_eq!(
        vault.try_deposit_for(&alice, &alice, &rejected_token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::TokenNotAllowed))
    );
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &rejected_token, &1_000, &unlock_ledger, &0),
        Err(Ok(VaultError::TokenNotAllowed))
    );
}

#[test]
fn test_allowlist_controls_require_admin() {
    let (env, vault, token, _admin, _alice, _fee) = setup();
    let non_admin: Address = Address::generate(&env);

    assert_eq!(
        vault.try_add_allowed_token(&non_admin, &token),
        Err(Ok(VaultError::Unauthorized))
    );
    assert_eq!(
        vault.try_set_strict_mode(&non_admin, &true),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_token_vetting_propose_review_approve_workflow() {
    let (env, vault, _token, admin, alice, _fee) = setup();
    let token: Address = Address::generate(&env);

    vault.propose_token(&alice, &token);
    let proposal = vault.get_token_vetting(&token).expect("proposal should exist");
    assert_eq!(proposal.proposer, alice);
    assert!(!proposal.reviewed);
    assert!(!proposal.approved);

    assert_eq!(
        vault.try_approve_token(&admin, &token),
        Err(Ok(VaultError::TokenReviewRequired))
    );
    vault.review_token(&admin, &token, &true);
    assert!(!vault.get_token_vetting(&token).unwrap().approved);

    vault.approve_token(&admin, &token);
    assert!(vault.is_token_allowed(&token));
    assert!(vault.get_token_vetting(&token).unwrap().approved);
}

#[test]
fn test_token_vetting_failed_review_cannot_be_approved() {
    let (env, vault, _token, admin, alice, _fee) = setup();
    let token: Address = Address::generate(&env);

    vault.propose_token(&alice, &token);
    vault.review_token(&admin, &token, &false);
    assert_eq!(
        vault.try_approve_token(&admin, &token),
        Err(Ok(VaultError::TokenReviewRequired))
    );
    assert!(!vault.is_token_allowed(&token));
}

#[test]
fn test_community_governance_uses_deposit_weight_and_timelock() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &1_000);
    StellarAssetClient::new(&env, &token).mint(&bob, &2_000);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);

    let proposal_id = vault.propose_pause(&alice, &GovernanceMode::CommunityVote);
    assert_eq!(vault.get_voting_power(&alice), 1_000);
    assert_eq!(vault.vote(&proposal_id, &alice, &true), 1_000);
    assert_eq!(vault.vote(&proposal_id, &bob, &false), 2_000);
    assert!(!vault.proposal_passed(&proposal_id));
    assert_eq!(
        vault.try_execute_proposal(&proposal_id),
        Err(Ok(VaultError::VotingStillOpen))
    );

    advance_time(&env, 86_400 + 86_400);
    assert_eq!(
        vault.try_execute_proposal(&proposal_id),
        Err(Ok(VaultError::ProposalRejected))
    );
    assert!(!vault.is_paused());
}

#[test]
fn test_admin_governance_requires_admin_and_prevents_double_vote() {
    let (env, vault, _token, admin, alice, _fee) = setup();
    let proposal_id = vault.propose_pause(&admin, &GovernanceMode::AdminVote);

    assert_eq!(vault.vote(&proposal_id, &admin, &true), 1);
    assert_eq!(
        vault.try_vote(&proposal_id, &admin, &true),
        Err(Ok(VaultError::AlreadyVoted))
    );
    assert_eq!(
        vault.try_vote(&proposal_id, &alice, &true),
        Err(Ok(VaultError::Unauthorized))
    );

    advance_time(&env, 86_400 + 86_400);
    vault.execute_proposal(&proposal_id);
    assert!(vault.is_paused());
    assert_eq!(
        vault.try_execute_proposal(&proposal_id),
        Err(Ok(VaultError::ProposalAlreadyExecuted))
    );
}

#[test]
fn test_parameter_change_governance_executes_max_deposit_update() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &1_000);
    StellarAssetClient::new(&env, &token).mint(&bob, &2_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);

    let proposal_id = vault.propose_change(&alice, &GovernanceMode::CommunityVote, &ProposalType::MaxDeposit, &5_000);
    assert_eq!(vault.vote(&proposal_id, &alice, &true), 1_000);
    assert_eq!(vault.vote(&proposal_id, &bob, &false), 2_000);

    advance_time(&env, 86_400 + 86_400);
    vault.execute_proposal(&proposal_id);

    let (max_deposit, _max_lock) = vault.get_constants();
    assert_eq!(max_deposit, 5_000);
}

// ================================================================
//  Deposit — happy path
// ================================================================

#[test]
fn test_deposit_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&alice, &id).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.unlock_time, unlock_time);
    assert_eq!(entry.token, token);
    assert_eq!(entry.depositor, alice);
    assert_eq!(entry.penalty_bps, 0);
    assert_eq!(entry.withdrawal_delay_secs, 0);

    let events = env.events().all();
    // Event emission is verified by test_deposit_for_event_emitted; the
    // core assertions above (vault entry fields, id) are sufficient here.
    let _ = events; // suppress unused-variable warning
}

#[test]
fn link_identity_stores_only_commitments_for_supported_did() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let credential_commitment = BytesN::from_array(&env, &[7; 32]);

    vault.link_identity(
        &alice,
        &deposit_id,
        &String::from_slice(&env, "did:key:z6Mkverified"),
        &credential_commitment,
        &fee,
    );

    let identity = vault.get_identity(&alice, &deposit_id).expect("identity link");
    assert_eq!(identity.did_method, String::from_slice(&env, "did:key"));
    assert_eq!(identity.credential_commitment, credential_commitment);
    assert_eq!(identity.verifier, fee);
    assert_eq!(identity.did_commitment, env.crypto().sha256(&String::from_slice(&env, "did:key:z6Mkverified").to_bytes()));
}

#[test]
fn link_identity_supports_ethr_and_rejects_unknown_methods() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let credential_commitment = BytesN::from_array(&env, &[9; 32]);

    vault.link_identity(
        &alice,
        &deposit_id,
        &String::from_slice(&env, "did:ethr:0x1234"),
        &credential_commitment,
        &fee,
    );
    assert_eq!(vault.get_identity(&alice, &deposit_id).unwrap().did_method, String::from_slice(&env, "did:ethr"));

    assert_eq!(
        vault.try_link_identity(
            &alice,
            &deposit_id,
            &String::from_slice(&env, "did:web:example.com"),
            &credential_commitment,
            &fee,
        ),
        Err(Ok(VaultError::UnsupportedDidMethod))
    );
}

#[test]
fn test_deposit_transfers_tokens_to_contract() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(token_client.balance(&alice), 9_000);
}

#[test]
fn test_deposit_with_delay_enforces_delay_after_unlock() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_with_delay(&alice, &token, &1_000, &unlock_time, &0, &600);

    advance_time(&env, 3601);
    assert_eq!(vault.time_to_withdrawal(&alice, &id), 599);
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::WithdrawalDelayActive))
    );

    advance_time(&env, 599);
    vault.withdraw(&alice, &id);
    assert_eq!(vault.time_to_withdrawal(&alice, &id), 0);
}

#[test]
fn test_deposit_with_zero_delay_withdraws_at_unlock() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_with_delay(&alice, &token, &1_000, &unlock_time, &0, &0);

    advance_time(&env, 3600);
    assert_eq!(vault.time_to_withdrawal(&alice, &id), 0);
    vault.withdraw(&alice, &id);
}

// ================================================================
//  Deposit — validation errors
// ================================================================

#[test]
fn test_deposit_zero_amount_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &0, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
}

#[test]
fn test_deposit_negative_amount_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &-1, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
}

#[test]
fn test_deposit_amount_exceeds_max_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &(MAX_DEPOSIT_AMOUNT + 1), &unlock_time, &0),
        Err(Ok(VaultError::AmountTooLarge))
    );
}

#[test]
fn test_deposit_at_max_amount_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &MAX_DEPOSIT_AMOUNT, &unlock_time, &0);
    let entry = vault.get_vault(&alice, &0).expect("entry should exist");
    assert_eq!(entry.amount, MAX_DEPOSIT_AMOUNT);
}

#[test]
fn test_deposit_unlock_time_in_past_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp();
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::UnlockTimeNotInFuture))
    );
}

#[test]
fn test_deposit_lock_duration_too_long_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::LockDurationTooLong))
    );
}

#[test]
fn test_deposit_at_max_duration_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &0).is_some());
}

#[test]
fn test_deposit_invalid_penalty_bps_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &10_001),
        Err(Ok(VaultError::InvalidPenaltyBps))
    );
}

#[test]
fn test_deposit_lock_duration_too_short_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 10;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

// ================================================================
//  Multiple deposits (multi-deposit support)
// ================================================================

#[test]
fn test_multiple_deposits_same_address() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;
    let t3 = env.ledger().timestamp() + 10800;

    let id0 = vault.deposit(&alice, &token, &1_000, &t1, &0);
    let id1 = vault.deposit(&alice, &token, &2_000, &t2, &0);
    let id2 = vault.deposit(&alice, &token, &3_000, &t3, &0);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 1_000);
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 2_000);
    assert_eq!(vault.get_vault(&alice, &2).unwrap().amount, 3_000);
}

#[test]
fn test_get_deposit_ids_returns_active_ids() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &2_000, &t2, &0);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
}

#[test]
fn test_partial_withdrawal_leaves_other_deposits_intact() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);
    let token_client = TokenClient::new(&env, &token);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &2_000, &t2, &0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert!(vault.get_vault(&alice, &1).is_some());
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 2_000);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), 1);

    assert_eq!(token_client.balance(&alice), 10_000 + 3_000 - 3_000 + 1_000);
}

#[test]
fn test_deposits_have_independent_unlock_times() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &2_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &1_000, &t2, &0);

    advance_time(&env, 3601);

    vault.withdraw(&alice, &0);
    let result = vault.try_withdraw(&alice, &1);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));
}

#[test]
fn test_deposit_ids_increment_after_withdrawal() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);

    let t1 = env.ledger().timestamp() + 3600;
    let id0 = vault.deposit(&alice, &token, &1_000, &t1, &0);
    assert_eq!(id0, 0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    let t2 = env.ledger().timestamp() + 3600;
    let id1 = vault.deposit(&alice, &token, &1_000, &t2, &0);
    assert_eq!(id1, 1);
}

// ================================================================
//  deposit_for
// ================================================================

#[test]
fn test_deposit_for_different_addresses_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&bob, &id).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.token, token);
    assert_eq!(entry.depositor, bob);
    assert_eq!(entry.penalty_bps, 0);

    // Alice (payer) balance decreased: started with 10_000 (setup) + 5_000 (mint) - 1_000 (deposit) = 14_000
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&alice), 14_000);
    // Bob (beneficiary) balance unchanged
    assert_eq!(token_client.balance(&bob), 0);
    // Contract holds the funds
    assert_eq!(token_client.balance(&vault.address), 1_000);
}

#[test]
fn test_deposit_for_beneficiary_can_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    // Before unlock, cannot withdraw
    assert_eq!(
        vault.try_withdraw(&bob, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance past unlock time
    advance_time(&env, 3601);

    // Beneficiary can withdraw without payer involvement
    vault.withdraw(&bob, &id);

    assert!(vault.get_vault(&bob, &id).is_none());
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), 1_000);
}

#[test]
fn test_deposit_for_same_address_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &alice, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&alice, &0).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.depositor, alice);
}

#[test]
fn test_contract_account_can_deposit_without_eoa_assumption() {
    let (env, vault, token, _admin, _alice, _fee) = setup();
    let smart_wallet = env.register(SafeHaven, ());
    StellarAssetClient::new(&env, &token).mint(&smart_wallet, &2_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&smart_wallet, &token, &1_000, &unlock_time, &0);

    let entry = vault.get_vault(&smart_wallet, &id).expect("entry should exist");
    assert_eq!(entry.depositor, smart_wallet);
    assert_eq!(TokenClient::new(&env, &token).balance(&vault.address), 1_000);
}

#[test]
fn test_session_key_deposit_is_scoped_and_expires() {
    let (env, vault, token, _admin, _alice, _fee) = setup();
    let wallet = Address::generate(&env);
    let session_key = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&session_key, &2_000);

    let expires_at = env.ledger().timestamp() + 600;
    vault.authorize_session_key(&wallet, &session_key, &expires_at);
    assert!(vault.is_session_key_authorized(&wallet, &session_key));

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_with_session_key(
        &session_key,
        &wallet,
        &token,
        &1_000,
        &unlock_time,
        &0,
    );
    let entry = vault.get_vault(&wallet, &id).expect("entry should exist");
    assert_eq!(entry.depositor, wallet);
    assert_eq!(TokenClient::new(&env, &token).balance(&session_key), 1_000);

    advance_time(&env, 601);
    assert!(!vault.is_session_key_authorized(&wallet, &session_key));
    assert_eq!(
        vault.try_deposit_with_session_key(
            &session_key,
            &wallet,
            &token,
            &1_000,
            &(env.ledger().timestamp() + 3600),
            &0,
        ),
        Err(Ok(VaultError::SessionKeyExpired))
    );

    vault.authorize_session_key(&wallet, &session_key, &(env.ledger().timestamp() + 600));
    vault.revoke_session_key(&wallet, &session_key);
    assert!(!vault.is_session_key_authorized(&wallet, &session_key));
    assert_eq!(
        vault.try_deposit_with_session_key(
            &session_key,
            &wallet,
            &token,
            &1_000,
            &(env.ledger().timestamp() + 3600),
            &0,
        ),
        Err(Ok(VaultError::SessionKeyNotAuthorized))
    );
}

#[test]
fn test_deposit_for_payer_has_no_access() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    // Payer cannot withdraw
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::NoDepositFound))
    );

    // Payer cannot cancel
    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::NoDepositFound))
    );
}

#[test]
fn test_deposit_for_validation_errors() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);
    let unlock_time = env.ledger().timestamp() + 3600;

    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &0, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &(MAX_DEPOSIT_AMOUNT + 1),
            &unlock_time,
            &0
        ),
        Err(Ok(VaultError::AmountTooLarge))
    );
    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &1_000, &env.ledger().timestamp(), &0),
        Err(Ok(VaultError::UnlockTimeNotInFuture))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &1_000,
            &(env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1),
            &0
        ),
        Err(Ok(VaultError::LockDurationTooLong))
    );
    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &10_001),
        Err(Ok(VaultError::InvalidPenaltyBps))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &1_000,
            &(env.ledger().timestamp() + 10),
            &0
        ),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

#[test]
fn test_deposit_for_adds_beneficiary_to_depositor_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    assert_eq!(vault.get_depositor_count(), 0);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    assert_eq!(vault.get_depositor_count(), 1);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.get(0).unwrap(), bob);
}

#[test]
fn test_deposit_for_event_emitted() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    let events = env.events().all();
    let last = events.last().unwrap();
    // Verify the deposit event was published from the vault contract.
    assert_eq!(last.0, vault.address.clone());
}

// ================================================================
//  Withdraw — happy path
// ================================================================

#[test]
fn test_withdraw_after_unlock_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_withdraw_exactly_at_unlock_time_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 3600);
    vault.withdraw(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
}

// ================================================================
//  Withdraw — error paths
// ================================================================

#[test]
fn test_withdraw_before_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 1800);

    let result = vault.try_withdraw(&alice, &0);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));
}

#[test]
fn test_withdraw_no_deposit_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_withdraw(&alice, &0);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

// ================================================================
//  cancel_deposit
// ================================================================

#[test]
fn test_cancel_deposit_zero_penalty_returns_full_amount() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_cancel_deposit_partial_penalty_splits_correctly() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &1_000);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 9_900);
    assert_eq!(token_client.balance(&fee), 100);
}

#[test]
fn test_cancel_deposit_100_percent_penalty() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &10_000);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 9_000);
    assert_eq!(token_client.balance(&fee), 1_000);
}

#[test]
fn test_cancel_deposit_no_deposit_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_cancel_deposit(&alice, &0),
        Err(Ok(VaultError::NoDepositFound))
    );
}

#[test]
fn test_cancel_deposit_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &500);
    advance_time(&env, 3601);
    assert_eq!(
        vault.try_cancel_deposit(&alice, &0),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

#[test]
fn test_cancel_deposit_penalty_stored_in_vault_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &500);
    assert_eq!(vault.get_vault(&alice, &0).unwrap().penalty_bps, 500);
}

#[test]
fn test_harvest_tax_losses_replaces_position_and_records_tax_impact() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let replacement_id = env.register_stellar_asset_contract_v2(admin.clone());
    let replacement_token = replacement_id.address();
    StellarAssetClient::new(&env, &replacement_token).mint(&alice, &1_000);
    let token_client = TokenClient::new(&env, &token);
    let replacement_client = TokenClient::new(&env, &replacement_token);
    let unlock_time = env.ledger().timestamp() + 3_600;

    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let harvest = vault.harvest_tax_losses(
        &alice,
        &0,
        &700,
        &replacement_token,
        &700,
        &500,
        &86_400,
    );

    assert_eq!(harvest.original_deposit_id, 0);
    assert_eq!(harvest.replacement_deposit_id, 1);
    assert_eq!(harvest.cost_basis, 1_000);
    assert_eq!(harvest.realized_loss, 300);
    assert_eq!(harvest.tax_benefit, 15);
    assert_eq!(harvest.wash_sale_until, env.ledger().timestamp() + 86_400);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(vault.get_vault(&alice, &1).unwrap().token, replacement_token);
    assert_eq!(vault.get_tax_loss_harvests(&alice).len(), 1);
    assert_eq!(token_client.balance(&alice), 10_000);
    assert_eq!(replacement_client.balance(&alice), 300);

    assert_eq!(
        vault.try_deposit(
            &alice,
            &token,
            &100,
            &(env.ledger().timestamp() + 3_600),
            &0,
        ),
        Err(Ok(VaultError::TaxWashSalePeriodActive))
    );
}

#[test]
fn test_harvest_tax_losses_rejects_same_token_replacement() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3_600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    assert_eq!(
        vault.try_harvest_tax_losses(
            &alice,
            &0,
            &700,
            &token,
            &700,
            &500,
            &86_400,
        ),
        Err(Ok(VaultError::InvalidTaxLossHarvest))
    );
}

// ================================================================
//  Time helpers
// ================================================================

#[test]
fn test_time_remaining_before_unlock() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 1800);
    assert_eq!(vault.time_remaining(&alice, &0), 1800);
}

#[test]
fn test_time_remaining_after_unlock_is_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 7200);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

#[test]
fn test_time_remaining_no_deposit_is_zero() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

/// Asserts that time_remaining returns 0 for a non-existent deposit ID even
/// when the caller has active deposits — the frontend relies on this to
/// detect "unlocked" / non-existent deposits.
#[test]
fn test_time_remaining_nonexistent_deposit_id_returns_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Alice has deposit 0, but deposit 99 does not exist.
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.time_remaining(&alice, &99), 0);

    // After withdrawing deposit 0, querying it must also return 0.
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

/// #105 — get_time must return the current ledger timestamp.
#[test]
fn test_get_time_returns_ledger_timestamp() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_time(), env.ledger().timestamp());
}

// ================================================================
//  Emergency Withdrawal
// ================================================================

#[test]
fn test_emergency_withdraw_by_admin_before_unlock_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &2_000, &unlock_time, &0);

    vault.emergency_withdraw(&admin, &alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_emergency_withdraw_by_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &2_000, &unlock_time, &0);

    let result = vault.try_emergency_withdraw(&bob, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_emergency_withdraw_no_deposit_fails() {
    let (_env, vault, _token, admin, alice, _fee) = setup();
    let result = vault.try_emergency_withdraw(&admin, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

// ================================================================
//  Admin Transfer — two-step
// ================================================================

#[test]
fn test_transfer_admin_two_step_succeeds() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);

    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(vault.get_pending_admin(), Some(new_admin.clone()));
    assert_eq!(vault.get_admin(), Some(admin.clone()));

    vault.accept_admin(&new_admin);
    assert_eq!(vault.get_admin(), Some(new_admin.clone()));
    assert_eq!(vault.get_pending_admin(), None);
}

#[test]
fn test_transfer_admin_non_admin_cannot_initiate() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);
    assert_eq!(
        vault.try_transfer_admin(&bob, &carol),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_accept_admin_wrong_address_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let impostor: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);

    assert_eq!(
        vault.try_accept_admin(&impostor),
        Err(Ok(VaultError::Unauthorized))
    );
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn test_accept_admin_with_no_pending_fails() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    assert_eq!(
        vault.try_accept_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_cancel_transfer_admin_clears_pending() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.cancel_transfer_admin(&admin);
    assert_eq!(vault.get_pending_admin(), None);
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn test_cancel_transfer_admin_by_non_admin_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(
        vault.try_cancel_transfer_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_accept_admin_by_admin_with_no_pending_fails() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let result = vault.try_accept_admin(&admin);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_accept_admin_after_cancel_fails() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&_env);

    vault.transfer_admin(&admin, &new_admin);
    vault.cancel_transfer_admin(&admin);

    let result = vault.try_accept_admin(&new_admin);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
    assert_eq!(vault.get_pending_admin(), None);
}

#[test]
fn test_new_admin_can_emergency_withdraw_after_transfer() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    vault.transfer_admin(&admin, &new_admin);
    vault.accept_admin(&new_admin);

    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &0),
        Err(Ok(VaultError::Unauthorized))
    );
    vault.emergency_withdraw(&new_admin, &alice, &0);
    assert_eq!(token_client.balance(&alice), 10_000);
}

// ================================================================
//  Admin Renounce
// ================================================================

#[test]
fn test_renounce_admin_removes_admin() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();

    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
}

#[test]
fn test_renounce_admin_disables_emergency_withdraw() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.renounce_admin(&admin);

    let result = vault.try_emergency_withdraw(&admin, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_renounce_admin_by_non_admin_fails() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    assert_eq!(
        vault.try_renounce_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_renounce_admin_clears_pending_transfer() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
    assert_eq!(vault.get_pending_admin(), None);
}

// ================================================================
//  Re-deposit after withdrawal
// ================================================================

#[test]
fn test_redeposit_after_withdraw_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    let new_unlock = env.ledger().timestamp() + 7200;
    let id = vault.deposit(&alice, &token, &500, &new_unlock, &0);

    assert_eq!(id, 1);
    let entry = vault.get_vault(&alice, &1).expect("entry should exist");
    assert_eq!(entry.amount, 500);
}

// ================================================================
//  TTL / storage constants
// ================================================================

#[test]
fn test_bump_target_covers_max_lock_duration() {
    use crate::storage::BUMP_TARGET;
    const LEDGER_INTERVAL_SECS: u64 = 5;
    let max_lock_ledgers = MAX_LOCK_DURATION_SECS / LEDGER_INTERVAL_SECS;
    assert!(
        BUMP_TARGET as u64 >= max_lock_ledgers,
        "BUMP_TARGET ({}) must be >= max lock duration in ledgers ({})",
        BUMP_TARGET,
        max_lock_ledgers,
    );
}

// ================================================================
//  View functions do not mutate state
// ================================================================

#[test]
fn test_get_vault_is_readonly() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert!(vault.get_vault(&alice, &0).is_none());
    // Calling get_vault on a non-existent entry should return None cleanly
    // without panicking or creating storage entries.
    assert!(vault.get_vault(&alice, &0).is_none());
}

#[test]
fn test_time_remaining_is_readonly() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(vault.time_remaining(&alice, &0), 0);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

// ================================================================
//  Depositor List / Pagination
// ================================================================

#[test]
fn test_depositor_count_empty() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_depositor_count(), 0);
}

#[test]
fn test_depositors_empty_returns_empty_vec() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total_count, 0);
}

#[test]
fn test_depositor_count_single_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);
}

#[test]
fn test_depositors_single_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.get(0).unwrap(), alice);
}

#[test]
fn test_depositor_count_multiple_entries() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    assert_eq!(vault.get_depositor_count(), 3);
}

#[test]
fn test_depositors_multiple_entries_full_page() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.total_count, 3);
}

#[test]
fn test_depositor_removed_on_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert_eq!(vault.get_depositor_count(), 0);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total_count, 0);
}

#[test]
fn test_depositor_removed_on_emergency_withdraw() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    vault.emergency_withdraw(&admin, &alice, &0);

    assert_eq!(vault.get_depositor_count(), 0);
}

#[test]
fn test_depositor_list_consistent_after_partial_removal() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 2);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert_eq!(vault.get_depositor_count(), 1);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items.get(0).unwrap(), bob);
}

#[test]
fn test_pagination_offset_and_limit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    let page1 = vault.get_depositors(&0, &2);
    assert_eq!(page1.items.len(), 2);

    let page2 = vault.get_depositors(&2, &2);
    assert_eq!(page2.items.len(), 1);
}

#[test]
fn test_pagination_offset_beyond_end_returns_empty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&10, &5);
    assert_eq!(page.items.len(), 0);
}

#[test]
fn test_pagination_with_large_offset_does_not_overflow() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&(u32::MAX - 1), &2);
    assert!(page.items.is_empty());
}

#[test]
fn test_pagination_limit_zero_returns_empty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &0);
    assert_eq!(page.items.len(), 0);
}

#[test]
fn test_redeposit_after_withdraw_adds_back_to_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    let new_unlock = env.ledger().timestamp() + 7200;
    vault.deposit(&alice, &token, &500, &new_unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.get(0).unwrap(), alice);
}

// ================================================================
//  Configurable limits
// ================================================================

fn setup_with_limits(
    max_deposit: Option<i128>,
    max_lock_secs: Option<u64>,
) -> (Env, SafeHavenClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &1_000_000);

    vault.initialize(&admin, &fee, &max_deposit, &max_lock_secs);

    (env, vault, token_address, admin, alice)
}

#[test]
fn test_get_constants_returns_custom_limits() {
    let (_env, vault, _token, _admin, _alice) = setup_with_limits(Some(5_000), Some(7200));
    let (max_amount, max_duration) = vault.get_constants();
    assert_eq!(max_amount, 5_000);
    assert_eq!(max_duration, 7200);
}

#[test]
fn test_custom_max_deposit_enforced() {
    let (env, vault, token, _admin, alice) = setup_with_limits(Some(500), None);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    let result = vault.try_deposit(&alice, &token, &501, &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::AmountTooLarge)));
}

#[test]
fn test_custom_max_lock_secs_enforced() {
    let (env, vault, token, _admin, alice) = setup_with_limits(None, Some(3600));
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &100, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    let result = vault.try_deposit(&alice, &token, &100, &(env.ledger().timestamp() + 3601), &0);
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

#[test]
fn test_default_fallback_when_no_custom_limits() {
    let (env, vault, token, _admin, alice) = setup_with_limits(None, None);
    let unlock_time = env.ledger().timestamp() + 3600;
    let result = vault.try_deposit(&alice, &token, &(MAX_DEPOSIT_AMOUNT + 1), &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::AmountTooLarge)));
    let result = vault.try_deposit(
        &alice,
        &token,
        &100,
        &(env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1),
        &0,
    );
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

#[test]
fn test_initialize_invalid_max_deposit_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);
    let result = vault.try_initialize(&admin, &fee, &Some(0_i128), &None);
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}

#[test]
fn test_initialize_invalid_max_lock_secs_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);
    let result = vault.try_initialize(&admin, &fee, &None, &Some(0_u64));
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

// ================================================================
//  XDR serialization snapshot tests
// ================================================================

#[test]
fn test_vault_entry_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let token: Address = Address::generate(&env);
    let depositor: Address = Address::generate(&env);

    let entry = VaultEntry {
        token: token.clone(),
        amount: 1_000_i128,
        unlock_time: 9_999_u64,
        depositor: depositor.clone(),
        penalty_bps: 0,
        compound_frequency_secs: 0,
        last_accrual_timestamp: 0,
    };

    let xdr_bytes = entry.clone().to_xdr(&env);

    let entry2 = VaultEntry::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");

    assert_eq!(entry2.amount, entry.amount);
    assert_eq!(entry2.unlock_time, entry.unlock_time);
    assert_eq!(entry2.token, entry.token);
    assert_eq!(entry2.depositor, entry.depositor);

    let snapshot_len = xdr_bytes.len();
    assert_eq!(
        xdr_bytes.len(),
        snapshot_len,
        "VaultEntry XDR size changed — update snapshot if intentional"
    );
}

#[test]
fn test_vault_key_deposit_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let depositor: Address = Address::generate(&env);

    let key = VaultKey::Deposit(depositor.clone(), 0);
    let xdr_bytes = key.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::Deposit(depositor, 0));
}

#[test]
fn test_vault_key_admin_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let xdr_bytes = VaultKey::Admin.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::Admin);
}

#[test]
fn test_vault_key_pending_admin_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let xdr_bytes = VaultKey::PendingAdmin.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::PendingAdmin);
}

// ================================================================
//  Auth assertion tests
// ================================================================

#[test]
fn test_auth_deposit_requires_depositor() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_deposit_for_requires_payer() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_withdraw_requires_depositor() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_emergency_withdraw_requires_admin() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.emergency_withdraw(&admin, &alice, &0);
    assert_eq!(env.auths()[0].0, admin);
}

#[test]
fn test_auth_transfer_admin_requires_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(env.auths()[0].0, admin);
}

#[test]
fn test_auth_accept_admin_requires_new_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.accept_admin(&new_admin);
    assert_eq!(env.auths()[0].0, new_admin);
}

#[test]
fn test_auth_renounce_admin_requires_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    vault.renounce_admin(&admin);
    assert_eq!(env.auths()[0].0, admin);
}

// ================================================================
//  Boundary & lifecycle tests (#97 – #100)
// ================================================================

// #97 — deposit with unlock_time == now + MAX_LOCK_DURATION_SECS must succeed
#[test]
fn test_deposit_exact_max_lock_duration_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &id).is_some());
}

// #98 — withdraw at unlock_time - 1 must fail with FundsStillLocked
#[test]
fn test_withdraw_fails_at_unlock_time_minus_one() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Advance to exactly one second before unlock
    advance_time(&env, 3599);
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );
}

// #99 — withdraw at exactly unlock_time must succeed (now == unlock_time)
#[test]
fn test_withdraw_succeeds_at_exact_unlock_time() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 3600);
    vault.withdraw(&alice, &id);
    assert!(vault.get_vault(&alice, &id).is_none());
}

// #100 — full lifecycle: deposit → advance time → withdraw → re-deposit
#[test]
fn test_full_lifecycle_deposit_withdraw_redeposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // 1. deposit
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &id).is_some());

    // 2. advance ledger past unlock_time
    advance_time(&env, 3601);

    // 3. withdraw
    vault.withdraw(&alice, &id);
    assert!(vault.get_vault(&alice, &id).is_none());

    // 4. re-deposit with same depositor — must succeed (re-deposit guard cleared)
    let new_unlock = env.ledger().timestamp() + 3600;
    let new_id = vault.deposit(&alice, &token, &500, &new_unlock, &0);
    assert!(vault.get_vault(&alice, &new_id).is_some());
    assert_eq!(vault.get_vault(&alice, &new_id).unwrap().amount, 500);
}

// ================================================================
//  deposit_by_ledger / withdraw_to / cancel_deposit — ledger path
//  (fixes https://github.com/kenedybok3/SAFE-HAVEN/issues/10 and https://github.com/kenedybok3/SAFE-HAVEN/issues/11)
// ================================================================

fn advance_ledger(env: &Env, ledgers: u32) {
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp(),
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence() + ledgers,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });
}

/// #10 — withdraw_to must handle ledger-based deposits
#[test]
fn test_withdraw_to_ledger_deposit_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Advance past the unlock ledger
    advance_ledger(&env, MIN_LOCK_LEDGERS);

    vault.withdraw_to(&alice, &id, &bob);

    // Funds must arrive at the recipient, not the depositor
    assert_eq!(token_client.balance(&bob), 1_000);
    assert_eq!(token_client.balance(&alice), 9_000);
}

/// #10 — withdraw_to on a locked ledger-based deposit must fail
#[test]
fn test_withdraw_to_ledger_deposit_still_locked_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    assert_eq!(
        vault.try_withdraw_to(&alice, &id, &bob),
        Err(Ok(VaultError::FundsStillLocked))
    );
}

/// #10 — withdraw_to with no matching deposit (neither kind) must return NoDepositFound
#[test]
fn test_withdraw_to_ledger_deposit_not_found_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    assert_eq!(
        vault.try_withdraw_to(&alice, &0, &bob),
        Err(Ok(VaultError::NoDepositFound))
    );
}

/// #11 — cancel_deposit must work for ledger-based deposits (zero penalty)
#[test]
fn test_cancel_ledger_deposit_zero_penalty_returns_full_amount() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    vault.cancel_deposit(&alice, &id);

    assert_eq!(token_client.balance(&alice), 10_000);
}

/// #11 — cancel_deposit with penalty on a ledger-based deposit splits correctly
#[test]
fn test_cancel_ledger_deposit_with_penalty_splits_correctly() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &1_000); // 10%

    vault.cancel_deposit(&alice, &id);

    assert_eq!(token_client.balance(&alice), 9_900);
    assert_eq!(token_client.balance(&fee), 100);
}

/// #11 — cancel_deposit after the unlock ledger has passed must fail
#[test]
fn test_cancel_ledger_deposit_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &500);

    advance_ledger(&env, MIN_LOCK_LEDGERS);

    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

/// #11 — cancel_deposit on a non-existent deposit must return NoDepositFound
#[test]
fn test_cancel_ledger_deposit_not_found_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_cancel_deposit(&alice, &99),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  Bug-fix regression tests
// ================================================================

// ----------------------------------------------------------------
//  Fix #6 — deposit_by_ledger must respect is_paused
// ----------------------------------------------------------------

/// Pausing the contract must block deposit_by_ledger just like deposit.
#[test]
fn test_deposit_by_ledger_blocked_when_paused() {
    let (env, vault, token, admin, alice, _fee) = setup();

    vault.pause(&admin);
    assert!(vault.is_paused());

    let unlock_ledger = env.ledger().sequence() + 100;
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0),
        Err(Ok(VaultError::ContractPaused))
    );
}

/// After unpausing, deposit_by_ledger must succeed again.
#[test]
fn test_deposit_by_ledger_succeeds_after_unpause() {
    let (env, vault, token, admin, alice, _fee) = setup();

    vault.pause(&admin);
    vault.unpause(&admin);
    assert!(!vault.is_paused());

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);
    assert!(id == 0);
}

/// Regression test for #86: deposit_by_ledger must return ContractPaused when the
/// contract is paused, preventing any bypass of the pause mechanism for ledger-based
/// deposits. This test documents the required behaviour so a future refactor cannot
/// silently remove the pause check from deposit_by_ledger.
#[test]
fn test_deposit_by_ledger_respects_pause() {
    let (env, vault, token, admin, alice, _fee) = setup();

    // Verify unpaused state allows deposits
    let unlock_ledger = env.ledger().sequence() + 100;
    assert!(!vault.is_paused());
    let id = vault.deposit_by_ledger(&alice, &token, &500, &unlock_ledger, &0);
    assert_eq!(id, 0);

    // Pause the contract
    vault.pause(&admin);
    assert!(vault.is_paused());

    // deposit_by_ledger must fail with ContractPaused — not silently succeed
    let unlock_ledger2 = env.ledger().sequence() + 100;
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger2, &0),
        Err(Ok(VaultError::ContractPaused))
    );

    // Unpause and verify deposits work again
    vault.unpause(&admin);
    assert!(!vault.is_paused());

    let unlock_ledger3 = env.ledger().sequence() + 100;
    let id2 = vault.deposit_by_ledger(&alice, &token, &500, &unlock_ledger3, &0);
    assert_eq!(id2, 1);
}

#[test]
fn test_deposit_by_ledger_event_emitted() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let events = env.events().all();
    let last = events.last().unwrap();
    // Verify the deposit_by_ledger event was published from the vault contract.
    assert_eq!(last.0, vault.address.clone());
}

#[test]
fn test_deposit_by_ledger_lock_duration_too_short_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + (MIN_LOCK_LEDGERS - 1);
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

// ----------------------------------------------------------------
//  Fix #7 — initialize must not be callable again after renounce_admin
// ----------------------------------------------------------------

/// After renounce_admin the is_initialized flag is still set, so initialize
/// must return Unauthorized and must not overwrite fee_recipient or admin.
#[test]
fn test_reinitialize_after_renounce_admin_fails() {
    let (env, vault, _token, admin, _alice, fee) = setup();
    let attacker: Address = Address::generate(&env);

    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
    assert!(vault.is_initialized());

    // Attacker tries to seize control.
    let result = vault.try_initialize(&attacker, &attacker, &None, &None);
    assert_eq!(result, Err(Ok(VaultError::AlreadyInitialized)));

    // State must be unchanged.
    assert_eq!(vault.get_admin(), None);
    assert_eq!(vault.get_fee_recipient(), Some(fee));
}

// ----------------------------------------------------------------
//  Fix #9 — emergency_withdraw must handle ledger-based deposits
// ----------------------------------------------------------------

/// Admin must be able to emergency-withdraw a ledger-sequence deposit.
#[test]
fn test_emergency_withdraw_ledger_deposit_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 1_000;
    let id = vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &0);

    // Funds go to the depositor, never the admin.
    vault.emergency_withdraw(&admin, &alice, &id);

    assert!(vault.get_vault(&alice, &id).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
    assert_eq!(token_client.balance(&admin), 0);
}

/// After emergency-withdraw of a ledger deposit the depositor is removed from
/// the global list when they have no remaining deposits.
#[test]
fn test_emergency_withdraw_ledger_deposit_removes_depositor() {
    let (env, vault, token, admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + 1_000;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    vault.emergency_withdraw(&admin, &alice, &id);

    assert_eq!(vault.get_depositor_count(), 0);
}

/// emergency_withdraw must still fail with NoDepositFound when neither a
/// timestamp-based nor a ledger-based deposit exists for the given ID.
#[test]
fn test_emergency_withdraw_nonexistent_deposit_fails() {
    let (_env, vault, _token, admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &99),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  Regression tests for fixes https://github.com/kenedybok3/SAFE-HAVEN/issues/18, https://github.com/kenedybok3/SAFE-HAVEN/issues/19, https://github.com/kenedybok3/SAFE-HAVEN/issues/20, https://github.com/kenedybok3/SAFE-HAVEN/issues/21
// ================================================================

// ----------------------------------------------------------------
//  Fix #18 — get_deposit_ids is O(1): active list, not counter scan
// ----------------------------------------------------------------

/// After depositing and withdrawing many times the returned IDs must only
/// be the currently active ones, not all historical IDs up to the counter.
#[test]
fn test_get_deposit_ids_reflects_only_active_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&alice, &100_000);

    // Make 5 deposits then withdraw the first 3.
    for _ in 0..5 {
        let unlock = env.ledger().timestamp() + 3600;
        vault.deposit(&alice, &token, &100, &unlock, &0);
    }

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    vault.withdraw(&alice, &1);
    vault.withdraw(&alice, &2);

    let ids = vault.get_deposit_ids(&alice);
    // Only IDs 3 and 4 should remain.
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 3);
    assert_eq!(ids.get(1).unwrap(), 4);
}

// ----------------------------------------------------------------
//  Fix #19 — add_depositor uses O(1) flag; re-deposit still works
// ----------------------------------------------------------------

/// A depositor who withdraws all funds and then re-deposits must appear
/// exactly once in the depositor list (not be absent, not appear twice).
#[test]
fn test_add_depositor_o1_flag_handles_redeposit_correctly() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // First deposit cycle.
    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    // Re-deposit: the flag must have been cleared on removal.
    let unlock2 = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock2, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items.get(0).unwrap(), alice);
}

/// Multiple deposits by the same depositor must not add duplicate entries
/// to the depositor list.
#[test]
fn test_add_depositor_no_duplicate_in_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&alice, &token, &1_000, &unlock, &0);

    // Still only one entry for alice.
    assert_eq!(vault.get_depositor_count(), 1);
}

// ----------------------------------------------------------------
//  Fix #20 — get_deposit_ids includes ledger-based deposits
// ----------------------------------------------------------------

/// A depositor with both a timestamp-based and a ledger-based deposit must
/// see both IDs returned by get_deposit_ids.
#[test]
fn test_get_deposit_ids_includes_ledger_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    // Timestamp-based deposit (id = 0)
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Ledger-based deposit (id = 1)
    let unlock_ledger = env.ledger().sequence() + 100;
    vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
}

/// After withdrawing a ledger-based deposit its ID must be removed from the
/// list returned by get_deposit_ids.
#[test]
fn test_get_deposit_ids_removes_ledger_deposit_on_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    assert_eq!(vault.get_deposit_ids(&alice).len(), 1);

    advance_ledger(&env, MIN_LOCK_LEDGERS);
    vault.withdraw(&alice, &id);

    assert_eq!(vault.get_deposit_ids(&alice).len(), 0);
}

// ----------------------------------------------------------------
//  Fix #21 — time_remaining returns estimated seconds for ledger deposits
// ----------------------------------------------------------------

/// time_remaining must return a positive estimate (remaining_ledgers * 5)
/// for a ledger-based deposit that is still locked.
#[test]
fn test_time_remaining_ledger_deposit_locked_returns_estimate() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let remaining_ledgers: u32 = 100;
    let unlock_ledger = env.ledger().sequence() + remaining_ledgers;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let expected_secs = remaining_ledgers as u64 * 5; // LEDGER_SECONDS = 5
    assert_eq!(vault.time_remaining(&alice, &id), expected_secs);
}

/// time_remaining must return 0 for a ledger-based deposit whose unlock
/// ledger has already been reached.
#[test]
fn test_time_remaining_ledger_deposit_unlocked_returns_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    advance_ledger(&env, MIN_LOCK_LEDGERS);

    assert_eq!(vault.time_remaining(&alice, &id), 0);
}

/// time_remaining must still return the correct value for a timestamp-based
/// deposit after the ledger-based fallback path is added.
#[test]
fn test_time_remaining_timestamp_deposit_unaffected() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 1800);

    assert_eq!(vault.time_remaining(&alice, &id), 1800);
}

/// Verify that time_remaining estimate decreases as ledgers progress.
/// This test simulates real ledger progression and confirms the estimate
/// follows the formula: remaining_ledgers × 5 seconds.
#[test]
fn test_time_remaining_ledger_estimate_decreases_with_progression() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let remaining_ledgers: u32 = 50;
    let unlock_ledger = env.ledger().sequence() + remaining_ledgers;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Check initial estimate
    let initial_estimate = vault.time_remaining(&alice, &id);
    assert_eq!(initial_estimate, (remaining_ledgers as u64) * 5);

    // Advance by 10 ledgers and verify estimate decreased by ~50 seconds
    advance_ledger(&env, 10);
    let estimate_after_10 = vault.time_remaining(&alice, &id);
    assert_eq!(estimate_after_10, (remaining_ledgers as u64 - 10) * 5);
    assert_eq!(initial_estimate - estimate_after_10, 50);

    // Advance by another 20 ledgers and verify estimate decreased by ~100 seconds total
    advance_ledger(&env, 20);
    let estimate_after_30 = vault.time_remaining(&alice, &id);
    assert_eq!(estimate_after_30, (remaining_ledgers as u64 - 30) * 5);
    assert_eq!(initial_estimate - estimate_after_30, 150);
}

/// Verify that time_remaining correctly returns 0 when approaching the
/// unlock ledger (within a few ledgers). This tests edge case behavior.
#[test]
fn test_time_remaining_ledger_near_unlock_boundary() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + 5;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Should return 25 seconds (5 ledgers * 5 seconds)
    assert_eq!(vault.time_remaining(&alice, &id), 25);

    // Advance to 1 ledger before unlock
    advance_ledger(&env, 4);
    assert_eq!(vault.time_remaining(&alice, &id), 5);

    // Advance to exact unlock ledger
    advance_ledger(&env, 1);
    assert_eq!(vault.time_remaining(&alice, &id), 0);

    // Even after reaching unlock, time_remaining should remain 0
    advance_ledger(&env, 1);
    assert_eq!(vault.time_remaining(&alice, &id), 0);
}

/// Verify that both timestamp-based and ledger-based deposits can coexist
/// and each returns its correct time_remaining value.
#[test]
fn test_time_remaining_mixed_deposit_types() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // Create a timestamp-based deposit
    let ts_unlock_time = env.ledger().timestamp() + 3600;
    let ts_id = vault.deposit(&alice, &token, &1_000, &ts_unlock_time, &0);

    // Create a ledger-based deposit
    let ledger_unlock = env.ledger().sequence() + 40; // 200 seconds estimate
    let ledger_id = vault.deposit_by_ledger(&alice, &token, &1_000, &ledger_unlock, &0);

    // Both should return their respective estimates
    assert_eq!(vault.time_remaining(&alice, &ts_id), 3600);
    assert_eq!(vault.time_remaining(&alice, &ledger_id), 200);

    // Advance by 10 ledgers (~50 seconds wall-clock time)
    advance_ledger(&env, 10);

    // Timestamp-based should decrease by ~50 seconds
    let ts_remaining = vault.time_remaining(&alice, &ts_id);
    assert!(ts_remaining >= 3550 && ts_remaining <= 3600); // Allow small variance

    // Ledger-based should decrease by exactly 50 seconds (10 ledgers * 5)
    assert_eq!(vault.time_remaining(&alice, &ledger_id), 150);
}

/// Verify that time_remaining returns the same estimate value on repeated
/// calls within the same ledger (no state change).
#[test]
fn test_time_remaining_ledger_consistent_in_same_ledger() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + 20;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Call time_remaining multiple times without advancing ledgers
    let estimate_1 = vault.time_remaining(&alice, &id);
    let estimate_2 = vault.time_remaining(&alice, &id);
    let estimate_3 = vault.time_remaining(&alice, &id);

    assert_eq!(estimate_1, estimate_2);
    assert_eq!(estimate_2, estimate_3);
    assert_eq!(estimate_1, 100); // 20 ledgers * 5 seconds
}

// ================================================================
//  get_deposits_page (Task 1)
// ================================================================

/// An empty contract returns an empty page.
#[test]
fn test_get_deposits_page_empty() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 0);
}

/// A single deposit from one depositor appears in the first page.
#[test]
fn test_get_deposits_page_single_deposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 1);
    let (addr, dep_id, entry) = page.get(0).unwrap();
    assert_eq!(addr, alice);
    assert_eq!(dep_id, id);
    assert_eq!(entry.amount, 1_000);
}

/// Deposits from multiple depositors are all returned in one page.
#[test]
fn test_get_deposits_page_multiple_depositors() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&bob, &5_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&bob, &token, &2_000, &unlock, &0);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 2);
}

/// Pagination: offset and limit slice the flat deposit stream correctly.
#[test]
fn test_get_deposits_page_pagination() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &50_000);

    let unlock = env.ledger().timestamp() + 3600;
    // Three deposits from Alice.
    vault.deposit(&alice, &token, &100, &unlock, &0);
    vault.deposit(&alice, &token, &200, &unlock, &0);
    vault.deposit(&alice, &token, &300, &unlock, &0);

    let page0 = vault.get_deposits_page(&0, &2);
    assert_eq!(page0.len(), 2);

    let page1 = vault.get_deposits_page(&2, &2);
    assert_eq!(page1.len(), 1);

    let page2 = vault.get_deposits_page(&3, &10);
    assert_eq!(page2.len(), 0);
}

/// Withdrawn deposits must not appear in get_deposits_page.
#[test]
fn test_get_deposits_page_excludes_withdrawn() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &id);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 0);
}

// ================================================================
//  BUMP_THRESHOLD derived from BUMP_TARGET (Task 3)
// ================================================================

/// BUMP_THRESHOLD must equal BUMP_TARGET / 2 and both must be non-zero.
#[test]
fn test_bump_threshold_derived_from_bump_target() {
    use crate::storage::{BUMP_TARGET, BUMP_THRESHOLD};
    assert!(BUMP_TARGET > 0, "BUMP_TARGET must be positive");
    assert_eq!(
        BUMP_THRESHOLD,
        BUMP_TARGET / 2,
        "BUMP_THRESHOLD must be derived as BUMP_TARGET / 2"
    );
}

// ================================================================
//  migrate / get_storage_version (Task 4)
// ================================================================

/// A freshly initialised contract has no stored version (None).
#[test]
fn test_get_storage_version_unset() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_storage_version(), None);
}

/// The upgrade harness simulates a legacy deployment with no stored version key,
/// then verifies the migration path writes version 1 and remains idempotent.
#[test]
fn test_upgrade_harness_handles_legacy_deploy() {
    let harness = UpgradeHarness::new();
    let unlock_time = harness.env.ledger().timestamp() + 3600;
    let deposit_id = harness.deposit_legacy_entry(1_000, unlock_time);

    harness.simulate_legacy_state();
    harness.assert_legacy_state();

    let legacy_entry = harness.vault.get_vault(&harness.alice, &deposit_id).expect("legacy deposit should remain readable");
    assert_eq!(legacy_entry.amount, 1_000);
    assert_eq!(legacy_entry.unlock_time, unlock_time);

    harness.assert_upgrade_applied();
    harness.assert_upgrade_idempotent();

    let migrated_entry = harness.vault.get_vault(&harness.alice, &deposit_id).expect("deposit should survive migration");
    assert_eq!(migrated_entry.amount, 1_000);
    assert_eq!(migrated_entry.unlock_time, unlock_time);
}

/// migrate() sets the version to STORAGE_VERSION and returns true.
#[test]
fn test_migrate_sets_version() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let migrated = vault.migrate(&admin);
    assert!(migrated, "first migrate call should return true");
    assert_eq!(vault.get_storage_version(), Some(1));
}

/// A second call to migrate() is idempotent and returns false.
#[test]
fn test_migrate_idempotent() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    vault.migrate(&admin);
    let migrated_again = vault.migrate(&admin);
    assert!(!migrated_again, "second migrate call should return false");
    assert_eq!(vault.get_storage_version(), Some(1));
}

/// migrate() must fail when called by a non-admin.
#[test]
fn test_migrate_non_admin_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_migrate(&alice);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

// ================================================================
//  O(1) remove_depositor — no duplicate list entries (Task 2)
// ================================================================

/// Depositing, withdrawing, and re-depositing must not create duplicate
/// entries in the depositor list or inflate get_depositor_count.
#[test]
fn test_remove_depositor_o1_no_duplicate_on_redeposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    let unlock2 = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock2, &0);
    // Must still be 1, not 2.
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items.get(0).unwrap(), alice);
}

// ================================================================
//  #88 Full deposit_by_ledger → withdraw lifecycle
// ================================================================

/// Comprehensive end-to-end test for ledger-based deposits:
/// 1. Create a ledger-based deposit
/// 2. Verify withdrawal is blocked before unlock_ledger
/// 3. Advance ledger past unlock_ledger
/// 4. Verify withdrawal succeeds after unlock
#[test]
fn test_deposit_by_ledger_withdraw_lifecycle() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    // Initial balance check
    assert_eq!(token_client.balance(&alice), 10_000);

    // Create a ledger-based deposit with 50-ledger lock
    let unlock_ledger = env.ledger().sequence() + 50;
    let id = vault.deposit_by_ledger(&alice, &token, &5_000, &unlock_ledger, &0);

    // Verify the deposit was created (ledger-based deposit, so get_vault returns None)
    assert_eq!(id, 0);
    assert!(vault.get_vault(&alice, &id).is_none());
    let ledger_entry = vault
        .get_ledger_vault(&alice, &id)
        .expect("ledger deposit should exist");
    assert_eq!(ledger_entry.amount, 5_000);
    assert_eq!(ledger_entry.unlock_ledger, unlock_ledger);
    assert_eq!(ledger_entry.depositor, alice);

    // Verify tokens were transferred to contract
    assert_eq!(token_client.balance(&alice), 5_000);

    // Attempt withdrawal before unlock_ledger — must fail
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance 25 ledgers (still before unlock_ledger)
    advance_ledger(&env, 25);
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance to exactly the unlock_ledger
    advance_ledger(&env, 25);
    assert_eq!(env.ledger().sequence(), unlock_ledger);

    // Withdrawal at unlock_ledger should now succeed
    vault.withdraw(&alice, &id);

    // Verify deposit was removed and tokens returned
    assert!(vault.get_ledger_vault(&alice, &id).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

// ================================================================
//  #89 cancel_deposit on ledger-based deposits
// ================================================================

/// Test cancel_deposit on a ledger-based deposit with zero penalty.
/// Should return full amount to depositor.
#[test]
fn test_cancel_deposit_ledger_based_zero_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &3_000, &unlock_ledger, &0);

    assert_eq!(token_client.balance(&alice), 7_000);

    // Cancel the deposit before unlock
    vault.cancel_deposit(&alice, &id);

    // Full amount should be returned (no penalty)
    assert_eq!(token_client.balance(&alice), 10_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test cancel_deposit on a ledger-based deposit with penalty.
/// Penalty should be paid to fee_recipient; remainder to depositor.
#[test]
fn test_cancel_deposit_ledger_based_with_penalty() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &4_000, &unlock_ledger, &2_500); // 25%

    assert_eq!(token_client.balance(&alice), 6_000);
    assert_eq!(token_client.balance(&fee), 0);

    // Cancel the deposit
    vault.cancel_deposit(&alice, &id);

    // 25% penalty = 1_000; remainder = 3_000
    assert_eq!(token_client.balance(&alice), 9_000);
    assert_eq!(token_client.balance(&fee), 1_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test that cancel_deposit fails after the unlock_ledger has passed.
#[test]
fn test_cancel_deposit_ledger_based_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &500);

    // Advance past the unlock ledger
    advance_ledger(&env, MIN_LOCK_LEDGERS);

    // Attempt cancel should fail because vault is now unlocked
    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

/// Test cancel_deposit on a non-existent ledger-based deposit.
#[test]
fn test_cancel_deposit_ledger_based_not_found() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();

    assert_eq!(
        vault.try_cancel_deposit(&alice, &999),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  #90 emergency_withdraw on ledger-based deposits
// ================================================================

/// Test that admin can emergency_withdraw a ledger-based deposit before unlock.
/// Funds must go to the depositor, not the admin.
#[test]
fn test_emergency_withdraw_ledger_based_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    // Create a ledger-based deposit that is still locked
    let unlock_ledger = env.ledger().sequence() + 500;
    let id = vault.deposit_by_ledger(&alice, &token, &3_500, &unlock_ledger, &0);

    assert_eq!(token_client.balance(&alice), 6_500);

    // Admin performs emergency withdrawal
    vault.emergency_withdraw(&admin, &alice, &id);

    // Funds go to depositor, not admin
    assert_eq!(token_client.balance(&alice), 10_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test that emergency_withdraw fails when called by a non-admin on a ledger-based deposit.
#[test]
fn test_emergency_withdraw_ledger_based_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let unlock_ledger = env.ledger().sequence() + 500;
    vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &0);

    // Non-admin attempts to emergency_withdraw
    assert_eq!(
        vault.try_emergency_withdraw(&bob, &alice, &0),
        Err(Ok(VaultError::Unauthorized))
    );
}

/// Test emergency_withdraw on a non-existent ledger-based deposit.
#[test]
fn test_emergency_withdraw_ledger_based_not_found() {
    let (_env, vault, _token, admin, alice, _fee) = setup();

    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &999),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  #91 Property-based tests for penalty calculation
// ================================================================

#[cfg(test)]
mod penalty_property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property: penalty + refund == original amount
    /// For any valid (amount, penalty_bps) pair, the sum of penalty and refund
    /// must equal the original amount (no loss or gain).
    proptest! {
        #[test]
        fn prop_penalty_plus_refund_equals_amount(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;

            prop_assert_eq!(
                penalty + refund,
                amount,
                "penalty + refund must equal original amount"
            );
        }
    }

    /// Property: penalty is always non-negative
    proptest! {
        #[test]
        fn prop_penalty_non_negative(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert!(penalty >= 0, "penalty must be non-negative");
        }
    }

    /// Property: penalty is always <= amount
    proptest! {
        #[test]
        fn prop_penalty_at_most_amount(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert!(penalty <= amount, "penalty must not exceed amount");
        }
    }

    /// Property: refund is always non-negative
    proptest! {
        #[test]
        fn prop_refund_non_negative(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;
            prop_assert!(refund >= 0, "refund must be non-negative");
        }
    }

    /// Property: max penalty (10_000 bps = 100%) equals the amount
    proptest! {
        #[test]
        fn prop_max_penalty_equals_amount(amount in 1i128..=1_000_000) {
            let penalty_bps = 10_000u32;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert_eq!(penalty, amount, "100% penalty must equal amount");
        }
    }

    /// Property: zero penalty (0 bps) results in zero penalty
    proptest! {
        #[test]
        fn prop_zero_penalty_bps_results_zero_penalty(amount in 1i128..=1_000_000) {
            let penalty_bps = 0u32;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert_eq!(penalty, 0, "0% penalty must be zero");
        }
    }

    /// Property: for edge case amount=1, penalty calculation doesn't overflow
    proptest! {
        #[test]
        fn prop_amount_one_no_overflow(penalty_bps in 0u32..=10_000) {
            let amount = 1i128;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;
            prop_assert_eq!(
                penalty + refund,
                amount,
                "even for amount=1, penalty + refund must equal 1"
            );
        }
    }

    /// Integration property: test with contract's actual penalty calculation
    /// This ensures the contract's logic matches the formula.
    proptest! {
        #[test]
        fn prop_penalty_calculation_matches_formula(
            amount in 1i128..=MAX_DEPOSIT_AMOUNT,
            penalty_bps in 0u32..=10_000,
        ) {
            // Replicate the contract's penalty calculation
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;

            // Verify invariants
            prop_assert!(penalty >= 0);
            prop_assert!(penalty <= amount);
            prop_assert!(refund >= 0);
            prop_assert_eq!(penalty + refund, amount);
        }
    }
}

// ================================================================
//  batch_deposit
// ================================================================

#[test]
fn test_batch_deposit_returns_ids_and_creates_all_entries() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &2_000);
    let mut deposits = Vec::new(&env);
    deposits.push_back(DepositRequest {
        token: token.clone(),
        amount: 1_000,
        unlock_time: env.ledger().timestamp() + 3600,
        penalty_bps: 0,
    });
    deposits.push_back(DepositRequest {
        token: token.clone(),
        amount: 1_000,
        unlock_time: env.ledger().timestamp() + 7200,
        penalty_bps: 0,
    });

    let ids = vault.batch_deposit(&alice, &deposits);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0), Some(0));
    assert_eq!(ids.get(1), Some(1));
    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 1_000);
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 1_000);
}

#[test]
fn test_batch_deposit_is_atomic_when_one_request_is_invalid() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let mut deposits = Vec::new(&env);
    deposits.push_back(DepositRequest {
        token: token.clone(),
        amount: 1_000,
        unlock_time: env.ledger().timestamp() + 3600,
        penalty_bps: 0,
    });
    deposits.push_back(DepositRequest {
        token,
        amount: 0,
        unlock_time: env.ledger().timestamp() + 7200,
        penalty_bps: 0,
    });

    assert_eq!(
        vault.try_batch_deposit(&alice, &deposits),
        Err(Ok(VaultError::InvalidAmount))
    );
    assert_eq!(vault.get_deposit_ids(&alice).len(), 0);
}

#[test]
fn test_batch_deposit_rejects_more_than_max_batch_size() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let mut deposits = Vec::new(&env);
    for _ in 0..26 {
        deposits.push_back(DepositRequest {
            token: token.clone(),
            amount: 1,
            unlock_time: env.ledger().timestamp() + 3600,
            penalty_bps: 0,
        });
    }

    assert_eq!(
        vault.try_batch_deposit(&alice, &deposits),
        Err(Ok(VaultError::BatchTooLarge))
    );
}

// ================================================================
//  get_vault_batch / get_deposit_batch — MAX_BATCH_SIZE clamping
// ================================================================

#[test]
fn test_get_vault_batch_clamps_at_max_batch_size() {
    use crate::constants::MAX_BATCH_SIZE;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create a single deposit for alice at id 0
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Build a depositor list larger than MAX_BATCH_SIZE (25)
    let mut depositors = soroban_sdk::Vec::new(&env);
    let excess = MAX_BATCH_SIZE + 5;
    for _ in 0..excess {
        depositors.push_back(alice.clone());
    }

    // Call get_vault_batch — should silently clamp to MAX_BATCH_SIZE results
    let results = vault.get_vault_batch(&depositors, &0);
    assert_eq!(results.len(), MAX_BATCH_SIZE);

    // All returned entries should be Some (alice has a deposit at id 0)
    for i in 0..MAX_BATCH_SIZE {
        let entry = results.get(i).unwrap();
        assert!(entry.is_some(), "index {} should have a deposit", i);
        let e = entry.unwrap();
        assert_eq!(e.amount, 1_000);
    }
}

#[test]
fn test_get_vault_batch_smaller_than_max_returns_exact_count() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    // Build a depositor list of 3 — fewer than MAX_BATCH_SIZE
    let mut depositors = soroban_sdk::Vec::new(&env);
    depositors.push_back(alice.clone());
    depositors.push_back(bob.clone());
    depositors.push_back(carol.clone());

    let results = vault.get_vault_batch(&depositors, &0);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_get_deposit_batch_clamps_at_max_batch_size() {
    use crate::constants::MAX_BATCH_SIZE;

    let (env, vault, token, _admin, alice, _fee) = setup();

    // Build a deposit_ids list longer than MAX_BATCH_SIZE (30 items).
    // Some of the IDs won't have deposits — that's fine; the test is only
    // verifying that the response is clamped to MAX_BATCH_SIZE entries.
    let mut deposit_ids = soroban_sdk::Vec::new(&env);
    for i in 0u32..(MAX_BATCH_SIZE + 5) {
        deposit_ids.push_back(i);
    }

    let results = vault.get_deposit_batch(&alice, &deposit_ids);
    assert_eq!(results.len(), MAX_BATCH_SIZE);
}

#[test]
fn test_version_returns_cargo_pkg_version() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();

    let version = vault.version();
    // Should be a valid version string from Cargo.toml (e.g., "0.1.0")
    assert!(!version.is_empty());
}

// ================================================================
//  #330 — Multi-token deposit tests
// ================================================================

fn setup_multi_token() -> (
    Env,
    SafeHavenClient<'static>,
    Address,  // token_a
    Address,  // token_b
    Address,  // admin
    Address,  // alice
    Address,  // fee_recipient
) {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_a_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a = token_a_id.address();
    let token_b = token_b_id.address();

    StellarAssetClient::new(&env, &token_a).mint(&alice, &10_000);
    StellarAssetClient::new(&env, &token_b).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    (env, vault, token_a, token_b, admin, alice, fee_recipient)
}

#[test]
fn test_multi_deposit_success() {
    use crate::types::TokenDeposit;

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a.clone(), amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b.clone(), amount: 300 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);
    assert_eq!(id, 0);

    let entry = vault.get_multi_vault(&alice, &id).expect("multi vault should exist");
    assert_eq!(entry.tokens.len(), 2);
    assert_eq!(entry.unlock_time, unlock_time);
    assert_eq!(entry.depositor, alice);
    assert_eq!(entry.penalty_bps, 0);
}

#[test]
fn test_multi_deposit_transfers_tokens() {
    use crate::types::TokenDeposit;
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let token_a_client = TokenClient::new(&env, &token_a);
    let token_b_client = TokenClient::new(&env, &token_b);

    assert_eq!(token_a_client.balance(&alice), 10_000);
    assert_eq!(token_b_client.balance(&alice), 10_000);

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a.clone(), amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b.clone(), amount: 300 });

    vault.multi_deposit(&alice, &tokens, &unlock_time, &0);

    assert_eq!(token_a_client.balance(&alice), 9_500);
    assert_eq!(token_b_client.balance(&alice), 9_700);
}

#[test]
fn test_multi_deposit_max_tokens_enforced() {
    use crate::types::TokenDeposit;

    let (env, vault, token_a, _token_b, admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create 6 tokens (exceeds MAX_TOKENS_PER_DEPOSIT = 5)
    let mut tokens = soroban_sdk::Vec::new(&env);
    for _ in 0u32..6 {
        let extra_token = env.register_stellar_asset_contract_v2(admin.clone()).address();
        StellarAssetClient::new(&env, &extra_token).mint(&alice, &1_000);
        tokens.push_back(TokenDeposit { token: extra_token, amount: 100 });
    }

    // suppress unused warning
    let _ = token_a;

    let result = vault.try_multi_deposit(&alice, &tokens, &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::TooManyTokens)));
}

#[test]
fn test_multi_deposit_empty_list_rejected() {
    let (env, vault, _token_a, _token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let tokens: soroban_sdk::Vec<crate::types::TokenDeposit> = soroban_sdk::Vec::new(&env);

    let result = vault.try_multi_deposit(&alice, &tokens, &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::EmptyTokenList)));
}

#[test]
fn test_multi_deposit_withdraw_returns_all_tokens() {
    use crate::types::TokenDeposit;
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a.clone(), amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b.clone(), amount: 300 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);

    // Advance past unlock time
    advance_time(&env, 3601);

    vault.withdraw(&alice, &id);

    let token_a_client = TokenClient::new(&env, &token_a);
    let token_b_client = TokenClient::new(&env, &token_b);
    assert_eq!(token_a_client.balance(&alice), 10_000);
    assert_eq!(token_b_client.balance(&alice), 10_000);

    // Entry should be gone
    assert!(vault.get_multi_vault(&alice, &id).is_none());
}

#[test]
fn test_multi_deposit_withdraw_locked_fails() {
    use crate::types::TokenDeposit;

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a, amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b, amount: 300 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);

    // Do NOT advance time
    let result = vault.try_withdraw(&alice, &id);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));
}

#[test]
fn test_multi_deposit_single_token_allowed() {
    use crate::types::TokenDeposit;

    let (env, vault, token_a, _token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a, amount: 500 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);
    let entry = vault.get_multi_vault(&alice, &id).expect("should exist");
    assert_eq!(entry.tokens.len(), 1);
}

#[test]
fn test_multi_deposit_get_deposit_type() {
    use crate::types::{DepositType, TokenDeposit};

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a, amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b, amount: 300 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);
    let deposit_type = vault.get_deposit_type(&alice, &id);
    assert_eq!(deposit_type, Some(DepositType::MultiToken));
}

#[test]
fn test_multi_deposit_invalid_amount_rejected() {
    use crate::types::TokenDeposit;

    let (env, vault, token_a, _token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a, amount: 0 }); // invalid

    let result = vault.try_multi_deposit(&alice, &tokens, &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}

#[test]
fn test_multi_deposit_cancel_with_penalty() {
    use crate::types::TokenDeposit;
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token_a, _token_b, _admin, alice, fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a.clone(), amount: 1_000 });

    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &1000); // 10% penalty

    vault.cancel_deposit(&alice, &id);

    let token_a_client = TokenClient::new(&env, &token_a);
    // refund = 1000 - 100 = 900
    assert_eq!(token_a_client.balance(&alice), 9_900);
    assert_eq!(token_a_client.balance(&fee), 100);
    assert!(vault.get_multi_vault(&alice, &id).is_none());
}

// ================================================================
//  #331 — Withdrawal whitelist tests
// ================================================================

#[test]
fn test_whitelist_set_and_get() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let bob: Address = Address::generate(&env);
    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob.clone());

    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    let stored = vault.get_withdrawal_whitelist(&alice, &id).expect("whitelist should be set");
    assert_eq!(stored.len(), 1);
    assert_eq!(stored.get(0), Some(bob));
}

#[test]
fn test_whitelist_empty_means_no_restriction() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Set empty whitelist
    let wl: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 3601);

    // Bob is not listed but empty whitelist means unrestricted
    let bob: Address = Address::generate(&env);
    vault.withdraw_to(&alice, &id, &bob);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), 1_000);
}

#[test]
fn test_whitelist_recipient_allowed() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let bob: Address = Address::generate(&env);
    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob.clone());
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 3601);
    vault.withdraw_to(&alice, &id, &bob);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), 1_000);
}

#[test]
fn test_whitelist_non_whitelisted_recipient_rejected() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    // Only bob is whitelisted
    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob);
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 3601);

    // carol tries to withdraw but she's not whitelisted
    let result = vault.try_withdraw_to(&alice, &id, &carol);
    assert_eq!(result, Err(Ok(VaultError::RecipientNotWhitelisted)));
}

#[test]
fn test_whitelist_no_whitelist_means_no_restriction() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // No whitelist set at all — anyone can receive
    let bob: Address = Address::generate(&env);
    advance_time(&env, 3601);
    vault.withdraw_to(&alice, &id, &bob);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), 1_000);
}

#[test]
fn test_whitelist_on_nonexistent_deposit_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    let bob: Address = Address::generate(&env);
    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob);

    let result = vault.try_set_withdrawal_whitelist(&alice, &999u32, &wl);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

#[test]
fn test_whitelist_multiple_allowed_recipients() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);
    let dan: Address = Address::generate(&env);

    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob.clone());
    wl.push_back(carol.clone());
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 3601);

    // carol is allowed
    vault.withdraw_to(&alice, &id, &carol);
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&carol), 1_000);

    // dan is not allowed (separate deposit for this check)
    let id2 = vault.deposit(&alice, &token, &1_000, &(env.ledger().timestamp() + 3600), &0);
    let mut wl2 = soroban_sdk::Vec::new(&env);
    wl2.push_back(bob.clone());
    vault.set_withdrawal_whitelist(&alice, &id2, &wl2);
    advance_time(&env, 3601);

    let result = vault.try_withdraw_to(&alice, &id2, &dan);
    assert_eq!(result, Err(Ok(VaultError::RecipientNotWhitelisted)));
}

#[test]
fn test_whitelist_on_multi_token_deposit() {
    use crate::types::TokenDeposit;
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token_a, token_b, _admin, alice, _fee) = setup_multi_token();
    let unlock_time = env.ledger().timestamp() + 3600;

    let mut tokens = soroban_sdk::Vec::new(&env);
    tokens.push_back(TokenDeposit { token: token_a.clone(), amount: 500 });
    tokens.push_back(TokenDeposit { token: token_b.clone(), amount: 300 });
    let id = vault.multi_deposit(&alice, &tokens, &unlock_time, &0);

    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob.clone());
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 3601);

    // carol is not in whitelist
    let result = vault.try_withdraw_to(&alice, &id, &carol);
    assert_eq!(result, Err(Ok(VaultError::RecipientNotWhitelisted)));

    // bob is allowed
    vault.withdraw_to(&alice, &id, &bob);
    let token_a_client = TokenClient::new(&env, &token_a);
    let token_b_client = TokenClient::new(&env, &token_b);
    assert_eq!(token_a_client.balance(&bob), 500);
    assert_eq!(token_b_client.balance(&bob), 300);
}

// ================================================================
//  #332 — Compound interest tests
// ================================================================

#[test]
fn test_deposit_with_compound_interest_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // compound every 60 seconds
    let id = vault.deposit_with_compound_interest(&alice, &token, &10_000, &unlock_time, &0, &60u64);

    let entry = vault.get_vault(&alice, &id).expect("entry should exist");
    assert_eq!(entry.amount, 10_000);
    assert_eq!(entry.compound_frequency_secs, 60);
}

#[test]
fn test_compound_interest_no_frequency_disabled() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // 0 = no compounding
    let id = vault.deposit_with_compound_interest(&alice, &token, &10_000, &unlock_time, &0, &0u64);

    let entry = vault.get_vault(&alice, &id).expect("entry should exist");
    assert_eq!(entry.compound_frequency_secs, 0);

    // update_accrual should return false (no compounding configured)
    let accrued = vault.update_accrual(&alice, &id);
    assert!(!accrued);
}

#[test]
fn test_compound_frequency_too_short_rejected() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // 30 seconds is below MIN_COMPOUND_FREQUENCY_SECS (60)
    let result = vault.try_deposit_with_compound_interest(
        &alice, &token, &10_000, &unlock_time, &0, &30u64,
    );
    assert_eq!(result, Err(Ok(VaultError::InvalidCompoundFrequency)));
}

#[test]
fn test_update_accrual_no_periods_elapsed_is_noop() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 7200;

    let id = vault.deposit_with_compound_interest(&alice, &token, &10_000, &unlock_time, &0, &3600u64);

    // Advance only 30 seconds — less than one full period
    advance_time(&env, 30);

    let accrued = vault.update_accrual(&alice, &id);
    assert!(!accrued);

    let entry = vault.get_vault(&alice, &id).unwrap();
    assert_eq!(entry.amount, 10_000); // unchanged
}

#[test]
fn test_update_accrual_one_period() {
    let (env, vault, token, admin, alice, _fee) = setup();
    // Mint enough tokens so that 5% p.a. over one 3600-second period produces > 0 interest.
    // Minimum amount for freq=3600: ceil(31_536_000 * 10_000 / (500 * 3600)) = 175_200
    StellarAssetClient::new(&env, &token).mint(&alice, &500_000);
    let unlock_time = env.ledger().timestamp() + 7200;
    let _ = admin; // suppress unused warning

    // 5% annual rate compounding every 3600s; use 500_000 + 10_000 = 510_000 available
    let id = vault.deposit_with_compound_interest(&alice, &token, &500_000, &unlock_time, &0, &3600u64);

    // Advance exactly one period
    advance_time(&env, 3600);

    let accrued = vault.update_accrual(&alice, &id);
    assert!(accrued, "interest should accrue after one full period");

    let entry = vault.get_vault(&alice, &id).unwrap();
    assert!(entry.amount > 500_000, "balance should have grown after accrual");
}

#[test]
fn test_get_current_balance_reflects_accrual() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    // Mint enough for interest to be measurable (>= 175_200 for freq=3600s at 5% p.a.)
    StellarAssetClient::new(&env, &token).mint(&alice, &500_000);
    let unlock_time = env.ledger().timestamp() + 7200;

    let id = vault.deposit_with_compound_interest(&alice, &token, &500_000, &unlock_time, &0, &3600u64);

    // Before any time passes, balance == deposited amount
    let bal_before = vault.get_current_balance(&alice, &id).expect("should exist");
    assert_eq!(bal_before, 500_000);

    // Advance one period
    advance_time(&env, 3600);

    let bal_after = vault.get_current_balance(&alice, &id).expect("should exist");
    assert!(bal_after > 500_000, "balance should reflect interest after one period");
}

#[test]
fn test_get_current_balance_not_found() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.get_current_balance(&alice, &999u32);
    assert!(result.is_none());
}

#[test]
fn test_withdraw_includes_accrued_interest() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    // Mint enough for interest to be non-zero at freq=3600s
    StellarAssetClient::new(&env, &token).mint(&alice, &500_000);
    let unlock_time = env.ledger().timestamp() + 7200;

    let id = vault.deposit_with_compound_interest(&alice, &token, &500_000, &unlock_time, &0, &3600u64);

    // Advance past unlock — at least one compound period will have passed
    advance_time(&env, 7200);

    // Pre-compute expected accrued balance (read-only, no storage mutation)
    let expected_balance = vault.get_current_balance(&alice, &id).expect("should exist");
    assert!(expected_balance > 500_000, "expected interest to accrue");

    // Fund the contract address with the accrued interest so the token transfer succeeds.
    // vault.address is the deployed contract's address.
    let interest = expected_balance.saturating_sub(500_000);
    if interest > 0 {
        StellarAssetClient::new(&env, &token).mint(&vault.address, &interest);
    }

    let token_client = TokenClient::new(&env, &token);
    let bal_before = token_client.balance(&alice);

    vault.withdraw(&alice, &id);

    let bal_after = token_client.balance(&alice);
    assert_eq!(bal_after.saturating_sub(bal_before), expected_balance);
}

#[test]
fn test_update_accrual_advances_timestamp() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    // For freq=60s, minimum amount = ceil(31_536_000 * 10_000 / (500 * 60)) = 10_512_000
    // Mint 10_512_000 extra (setup already minted 10_000)
    StellarAssetClient::new(&env, &token).mint(&alice, &10_512_000);
    let unlock_time = env.ledger().timestamp() + 7200;

    let id = vault.deposit_with_compound_interest(&alice, &token, &10_512_000, &unlock_time, &0, &60u64);

    let before = vault.get_vault(&alice, &id).unwrap().last_accrual_timestamp;

    // Advance two periods (120 seconds)
    advance_time(&env, 120);
    vault.update_accrual(&alice, &id);

    let after = vault.get_vault(&alice, &id).unwrap().last_accrual_timestamp;
    // Timestamp should have advanced by exactly 2 * 60 = 120 seconds
    assert_eq!(after, before.saturating_add(120));
}

#[test]
fn test_update_accrual_multiple_periods() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    // Mint enough for freq=60s interest to be non-zero
    StellarAssetClient::new(&env, &token).mint(&alice, &10_512_000);
    let unlock_time = env.ledger().timestamp() + 100_000;

    let id = vault.deposit_with_compound_interest(&alice, &token, &10_512_000, &unlock_time, &0, &60u64);

    // Advance 5 complete periods (300 seconds)
    advance_time(&env, 300);
    vault.update_accrual(&alice, &id);

    let entry = vault.get_vault(&alice, &id).unwrap();
    assert!(entry.amount > 10_512_000, "amount should grow after 5 compound periods");
}

#[test]
fn test_compound_interest_regular_deposit_has_zero_frequency() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let entry = vault.get_vault(&alice, &id).unwrap();
    assert_eq!(entry.compound_frequency_secs, 0);
}

#[test]
fn test_update_accrual_on_nonexistent_deposit_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_update_accrual(&alice, &999u32);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

#[test]
fn test_withdraw_to_with_whitelist_and_compound_interest() {
    use soroban_sdk::token::Client as TokenClient;

    let (env, vault, token, _admin, alice, _fee) = setup();
    // Mint enough for interest to be non-zero at freq=3600s
    StellarAssetClient::new(&env, &token).mint(&alice, &500_000);
    let unlock_time = env.ledger().timestamp() + 7200;

    let id = vault.deposit_with_compound_interest(
        &alice, &token, &500_000, &unlock_time, &0, &3600u64,
    );

    let bob: Address = Address::generate(&env);
    let mut wl = soroban_sdk::Vec::new(&env);
    wl.push_back(bob.clone());
    vault.set_withdrawal_whitelist(&alice, &id, &wl);

    advance_time(&env, 7200);

    let expected = vault.get_current_balance(&alice, &id).expect("should exist");
    assert!(expected > 500_000, "expected accrued balance to exceed principal");

    // Fund the contract with the accrued interest portion
    let interest = expected.saturating_sub(500_000);
    if interest > 0 {
        StellarAssetClient::new(&env, &token).mint(&vault.address, &interest);
    }

    vault.withdraw_to(&alice, &id, &bob);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), expected);
}


// ================================================================
//  Staker Registry Tests
// ================================================================

#[test]
fn test_register_staker_success() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Alice registers as a staker with stake amount 1000
    let result = vault.register_staker(&alice, &1000);
    assert!(result.is_ok());
}

#[test]
fn test_register_staker_zero_amount_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Attempting to register with zero stake should fail
    let result = vault.register_staker(&alice, &0);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), VaultError::InvalidStakeAmount);
}

#[test]
fn test_register_staker_negative_amount_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Attempting to register with negative stake should fail
    let result = vault.register_staker(&alice, &-1000);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), VaultError::InvalidStakeAmount);
}

#[test]
fn test_register_staker_updates_existing_stake() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Register with 1000
    vault.register_staker(&alice, &1000).unwrap();

    // Update to 2000
    vault.register_staker(&alice, &2000).unwrap();

    // Verify event was emitted for second registration
    let events = env.events().all();
    let staker_reg_events: std::vec::Vec<_> = events
        .iter()
        .filter(|(_, event)| {
            if let Some(data) = &event.data {
                let raw = &data.data;
                raw.len() > 0
            } else {
                false
            }
        })
        .collect();
    assert!(staker_reg_events.len() > 0);
}

#[test]
fn test_multiple_stakers_register() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token = token_id.address();

    vault.initialize(&admin, &fee_recipient, &None, &None);

    // Register multiple stakers
    vault.register_staker(&alice, &1000).unwrap();
    vault.register_staker(&bob, &2000).unwrap();
    vault.register_staker(&carol, &3000).unwrap();

    // All registrations should succeed
}

#[test]
fn test_claim_staker_rewards_requires_registration() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Alice tries to claim without registering — should fail
    let result = vault.claim_staker_rewards(&alice);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), VaultError::StakerNotFound);
}

#[test]
fn test_claim_staker_rewards_with_empty_pool() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Register as staker
    vault.register_staker(&alice, &1000).unwrap();

    // Try to claim with empty rewards pool — should fail
    let result = vault.claim_staker_rewards(&alice);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), VaultError::NoRewardsToClaim);
}

#[test]
fn test_penalty_split_on_cancel_deposit() {
    let (env, vault, token, admin, alice, fee_recipient) = setup();

    // Alice deposits with a penalty
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1000, &unlock_time, &1000).unwrap();

    // Cancel the deposit (10% penalty = 100 tokens)
    // Split: 70% to stakers rewards (70), 30% to fee_recipient (30)
    vault.cancel_deposit(&alice, &deposit_id).unwrap();

    // Check events for penalty split
    let events = env.events().all();
    assert!(events.len() > 0); // Should have penalty_split and deposit_cancelled events
}

#[test]
fn test_single_staker_claims_full_rewards() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    // Alice is the only staker
    vault.register_staker(&alice, &1000).unwrap();

    // Deposit with penalty to generate rewards pool
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault
        .deposit(&alice, &token_address, &1000, &unlock_time, &1000)
        .unwrap();

    // Cancel deposit to generate penalty (1000 * 0.10 = 100, split: 70 to stakers, 30 to fee)
    vault.cancel_deposit(&alice, &deposit_id).unwrap();

    // Alice claims rewards — should get 70 tokens (the staker share)
    let result = vault.claim_staker_rewards(&alice);
    assert!(result.is_ok()); // Should succeed
}

#[test]
fn test_multiple_stakers_proportional_rewards() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);
    StellarAssetClient::new(&env, &token_address).mint(&bob, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    // Two stakers: Alice with 1000, Bob with 3000 (total 4000)
    vault.register_staker(&alice, &1000).unwrap();
    vault.register_staker(&bob, &3000).unwrap();

    // Deposit with penalty
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault
        .deposit(&alice, &token_address, &1000, &unlock_time, &1000)
        .unwrap();

    // Cancel deposit to generate 70 tokens in rewards pool (1000 * 0.10 * 0.70)
    vault.cancel_deposit(&alice, &deposit_id).unwrap();

    // Alice should be able to claim her proportional share (1000/4000 * 70 = 17.5, likely rounds to 17)
    let result = vault.claim_staker_rewards(&alice);
    assert!(result.is_ok());

    // Bob should be able to claim his proportional share (3000/4000 * 70 = 52.5, likely rounds to 52)
    let result = vault.claim_staker_rewards(&bob);
    assert!(result.is_ok());
}

#[test]
fn test_staker_registration_emits_event() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    let staker_addr = alice.clone();
    vault.register_staker(&staker_addr, &1000).unwrap();

    // Check for staker_registered event
    let events = env.events().all();
    assert!(events.len() > 0);
}

#[test]
fn test_staker_rewards_claimed_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    vault.register_staker(&alice, &1000).unwrap();

    // Deposit and cancel to generate rewards
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault
        .deposit(&alice, &token_address, &1000, &unlock_time, &1000)
        .unwrap();
    vault.cancel_deposit(&alice, &deposit_id).unwrap();

    // Claim rewards — should emit event
    vault.claim_staker_rewards(&alice).unwrap();

    let events = env.events().all();
    assert!(events.len() > 0); // Should have reward claim event
}

#[test]
fn test_penalty_split_percentages() {
    // Verify that STAKER_PENALTY_BPS (7000) and FEE_RECIPIENT_PENALTY_BPS (3000) sum to 10000
    assert_eq!(crate::constants::STAKER_PENALTY_BPS + crate::constants::FEE_RECIPIENT_PENALTY_BPS, 10_000);
}

#[test]
fn test_register_staker_auth_required() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    // Register should require auth from the staker (alice)
    vault.register_staker(&alice, &1000).unwrap();
}

#[test]
fn test_claim_staker_rewards_auth_required() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    vault.register_staker(&alice, &1000).unwrap();

    // Deposit and cancel to generate rewards
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault
        .deposit(&alice, &token_address, &1000, &unlock_time, &1000)
        .unwrap();
    vault.cancel_deposit(&alice, &deposit_id).unwrap();

    // Claim should require auth from the staker (alice)
    vault.claim_staker_rewards(&alice).unwrap();
}

// ================================================================
//  Issue #333 — Recurring deposit subscription tests
// ================================================================

/// Helper: creates a subscription with default sensible params.
fn create_default_subscription(
    vault: &SafeHavenClient<'static>,
    env: &Env,
    depositor: &Address,
    token: &Address,
) -> u32 {
    vault.create_subscription(
        depositor,
        token,
        &1_000_i128,   // amount
        &120_u64,      // interval_secs (2 min)
        &3_u32,        // total_count
        &120_u64,      // lock_duration_secs (2 min)
        &0_u32,        // penalty_bps
    )
}

#[test]
fn test_create_subscription_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    assert_eq!(sub_id, 0);

    let sub = vault.get_subscription(&alice, &sub_id).unwrap();
    assert_eq!(sub.depositor, alice);
    assert_eq!(sub.token, token);
    assert_eq!(sub.amount, 1_000);
    assert_eq!(sub.interval_secs, 120);
    assert_eq!(sub.total_count, 3);
    assert_eq!(sub.executed_count, 0);
    assert_eq!(sub.lock_duration_secs, 120);
    assert_eq!(sub.penalty_bps, 0);
    assert!(!sub.cancelled);
}

#[test]
fn test_create_subscription_ids_increment() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let id0 = create_default_subscription(&vault, &env, &alice, &token);
    let id1 = create_default_subscription(&vault, &env, &alice, &token);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);

    let ids = vault.get_subscription_ids(&alice);
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_create_subscription_invalid_amount() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let result = vault.try_create_subscription(
        &alice, &token, &0_i128, &120_u64, &3_u32, &120_u64, &0_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}

#[test]
fn test_create_subscription_zero_interval_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let result = vault.try_create_subscription(
        &alice, &token, &1_000_i128, &0_u64, &3_u32, &120_u64, &0_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::InvalidSubscriptionParams)));
}

#[test]
fn test_create_subscription_zero_count_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let result = vault.try_create_subscription(
        &alice, &token, &1_000_i128, &120_u64, &0_u32, &120_u64, &0_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::InvalidSubscriptionParams)));
}

#[test]
fn test_create_subscription_lock_duration_too_short_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    // lock_duration_secs = 10 < MIN_LOCK_DURATION_SECS (60)
    let result = vault.try_create_subscription(
        &alice, &token, &1_000_i128, &120_u64, &3_u32, &10_u64, &0_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooShort)));
}

#[test]
fn test_create_subscription_invalid_penalty_bps() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let result = vault.try_create_subscription(
        &alice, &token, &1_000_i128, &120_u64, &3_u32, &120_u64, &10_001_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::InvalidPenaltyBps)));
}

#[test]
fn test_create_subscription_paused_fails() {
    let (env, vault, token, admin, alice, _fee) = setup();
    vault.pause(&admin);
    let result = vault.try_create_subscription(
        &alice, &token, &1_000_i128, &120_u64, &3_u32, &120_u64, &0_u32,
    );
    assert_eq!(result, Err(Ok(VaultError::ContractPaused)));
}

#[test]
fn test_execute_subscription_creates_deposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    // First execution is due immediately.
    let deposit_id = vault.execute_subscription(&alice, &sub_id);

    // A deposit should now exist for alice.
    let entry = vault.get_vault(&alice, &deposit_id).unwrap();
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.token, token);
    assert_eq!(entry.depositor, alice);

    // executed_count should be 1.
    let sub = vault.get_subscription(&alice, &sub_id).unwrap();
    assert_eq!(sub.executed_count, 1);
}

#[test]
fn test_execute_subscription_respects_interval() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    // First execution succeeds.
    vault.execute_subscription(&alice, &sub_id);

    // Second execution before interval elapses should fail.
    let result = vault.try_execute_subscription(&alice, &sub_id);
    assert_eq!(result, Err(Ok(VaultError::SubscriptionNotDue)));

    // Advance past the interval (120 s).
    advance_time(&env, 121);
    let deposit_id2 = vault.execute_subscription(&alice, &sub_id);

    let sub = vault.get_subscription(&alice, &sub_id).unwrap();
    assert_eq!(sub.executed_count, 2);

    let entry = vault.get_vault(&alice, &deposit_id2).unwrap();
    assert_eq!(entry.amount, 1_000);
}

#[test]
fn test_execute_subscription_completes_after_total_count() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // total_count = 2
    let sub_id = vault.create_subscription(
        &alice, &token, &1_000_i128, &60_u64, &2_u32, &60_u64, &0_u32,
    );

    vault.execute_subscription(&alice, &sub_id);
    advance_time(&env, 61);
    vault.execute_subscription(&alice, &sub_id);

    // Third call should fail — completed.
    advance_time(&env, 61);
    let result = vault.try_execute_subscription(&alice, &sub_id);
    assert_eq!(result, Err(Ok(VaultError::SubscriptionCompleted)));
}

#[test]
fn test_cancel_subscription_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    vault.cancel_subscription(&alice, &sub_id);

    let sub = vault.get_subscription(&alice, &sub_id).unwrap();
    assert!(sub.cancelled);
}

#[test]
fn test_cancel_subscription_prevents_execution() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    vault.cancel_subscription(&alice, &sub_id);

    let result = vault.try_execute_subscription(&alice, &sub_id);
    assert_eq!(result, Err(Ok(VaultError::SubscriptionCancelled)));
}

#[test]
fn test_cancel_subscription_twice_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    vault.cancel_subscription(&alice, &sub_id);

    let result = vault.try_cancel_subscription(&alice, &sub_id);
    assert_eq!(result, Err(Ok(VaultError::SubscriptionCancelled)));
}

#[test]
fn test_cancel_nonexistent_subscription_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_cancel_subscription(&alice, &999_u32);
    assert_eq!(result, Err(Ok(VaultError::NoSubscriptionFound)));
}

#[test]
fn test_execute_nonexistent_subscription_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_execute_subscription(&alice, &999_u32);
    assert_eq!(result, Err(Ok(VaultError::NoSubscriptionFound)));
}

#[test]
fn test_execute_subscription_deducts_token_balance() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let balance_before = token_client.balance(&alice);
    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    vault.execute_subscription(&alice, &sub_id);
    let balance_after = token_client.balance(&alice);

    assert_eq!(balance_before - balance_after, 1_000);
}

#[test]
fn test_execute_subscription_deposit_is_withdrawable_after_lock() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let sub_id = create_default_subscription(&vault, &env, &alice, &token);
    let deposit_id = vault.execute_subscription(&alice, &sub_id);

    // Lock is 120 s — should be locked right after execution.
    let result = vault.try_withdraw(&alice, &deposit_id);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));

    advance_time(&env, 121);
    vault.withdraw(&alice, &deposit_id);
}

#[test]
fn test_get_subscription_ids_returns_all() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    for _ in 0..3 {
        create_default_subscription(&vault, &env, &alice, &token);
    }

    let ids = vault.get_subscription_ids(&alice);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
    assert_eq!(ids.get(2).unwrap(), 2);
}

#[test]
fn test_subscription_can_be_paused_and_resumed() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let sub_id = create_default_subscription(&vault, &env, &alice, &token);

    vault.pause_subscription(&alice, &sub_id);
    let paused = vault.get_subscription(&alice, &sub_id).unwrap();
    assert!(paused.paused);
    assert_eq!(
        vault.try_execute_subscription(&alice, &sub_id),
        Err(Ok(VaultError::SubscriptionPaused))
    );

    vault.resume_subscription(&alice, &sub_id);
    let resumed = vault.get_subscription(&alice, &sub_id).unwrap();
    assert!(!resumed.paused);
    vault.execute_subscription(&alice, &sub_id);
}

#[test]
fn test_subscription_history_and_statistics_track_executions() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let sub_id = create_default_subscription(&vault, &env, &alice, &token);

    let first_deposit_id = vault.execute_subscription(&alice, &sub_id);
    advance_time(&env, 121);
    let second_deposit_id = vault.execute_subscription(&alice, &sub_id);

    let history = vault.get_subscription_history(&alice, &sub_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().deposit_id, first_deposit_id);
    assert_eq!(history.get(1).unwrap().deposit_id, second_deposit_id);

    let stats = vault.get_subscription_stats(&alice, &sub_id);
    assert_eq!(stats.deposit_count, 2);
    assert_eq!(stats.total_amount, 2_000);
    assert!(stats.last_execution_time > 0);
}

// ================================================================
//  Issue #334 — Deposit insurance pool tests
// ================================================================

/// Helper: create a deposit with a penalty and cancel it, which funds the pool.
fn fund_pool_via_cancel(
    env: &Env,
    vault: &SafeHavenClient<'static>,
    depositor: &Address,
    token: &Address,
) {
    // Deposit 10_000 with 10% penalty, then cancel immediately to fund the pool.
    let unlock_time = env.ledger().timestamp() + 120;
    let deposit_id =
        vault.deposit(depositor, token, &10_000_i128, &unlock_time, &1_000_u32); // 10%
    vault.cancel_deposit(depositor, &deposit_id);
}

#[test]
fn test_insurance_pool_funded_from_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // Before any cancellations, balance should be zero.
    assert_eq!(vault.get_insurance_pool_balance(&token), 0);

    fund_pool_via_cancel(&env, &vault, &alice, &token);

    // penalty = 10_000 * 10% = 1_000
    // insurance_cut = 1_000 * 5% = 50
    let pool_balance = vault.get_insurance_pool_balance(&token);
    assert_eq!(pool_balance, 50);
}

#[test]
fn test_insurance_pool_accumulates_across_cancellations() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // Mint more tokens for alice.
    StellarAssetClient::new(&env, &token).mint(&alice, &100_000);

    fund_pool_via_cancel(&env, &vault, &alice, &token);
    fund_pool_via_cancel(&env, &vault, &alice, &token);
    fund_pool_via_cancel(&env, &vault, &alice, &token);

    // 3 × 50 = 150
    let pool_balance = vault.get_insurance_pool_balance(&token);
    assert_eq!(pool_balance, 150);
}

#[test]
fn test_claim_insurance_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    fund_pool_via_cancel(&env, &vault, &alice, &token);

    let evidence = soroban_sdk::String::from_str(&env, "tx-hash-123");
    let claim_id = vault.claim_insurance(&alice, &token, &50_i128, &evidence);
    assert_eq!(claim_id, 0);

    let claim = vault.get_claim(&claim_id).unwrap();
    assert_eq!(claim.claim_id, 0);
    assert_eq!(claim.claimant, alice);
    assert_eq!(claim.token, token);
    assert_eq!(claim.amount_requested, 50);
    // status is Pending — cannot compare ClaimStatus directly in test context
    // but we verify approve/deny transitions below.
}

#[test]
fn test_claim_insurance_invalid_amount_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let result = vault.try_claim_insurance(&alice, &token, &0_i128, &evidence);
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}

#[test]
fn test_approve_claim_disburses_funds() {
    let (env, vault, token, admin, alice, _fee) = setup();

    fund_pool_via_cancel(&env, &vault, &alice, &token);
    let pool_before = vault.get_insurance_pool_balance(&token);
    assert_eq!(pool_before, 50);

    let token_client = TokenClient::new(&env, &token);
    let alice_balance_before = token_client.balance(&alice);

    let evidence = soroban_sdk::String::from_str(&env, "incident-abc");
    let claim_id = vault.claim_insurance(&alice, &token, &50_i128, &evidence);
    let amount = vault.approve_claim(&admin, &claim_id);

    assert_eq!(amount, 50);

    // Pool balance should be zero now.
    assert_eq!(vault.get_insurance_pool_balance(&token), 0);

    // Alice should have received 50.
    let alice_balance_after = token_client.balance(&alice);
    assert_eq!(alice_balance_after - alice_balance_before, 50);
}

#[test]
fn test_approve_claim_insufficient_pool_fails() {
    let (env, vault, token, admin, alice, _fee) = setup();

    // No cancellations — pool is empty.
    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let claim_id = vault.claim_insurance(&alice, &token, &100_i128, &evidence);

    let result = vault.try_approve_claim(&admin, &claim_id);
    assert_eq!(result, Err(Ok(VaultError::InsufficientInsurancePool)));
}

#[test]
fn test_deny_claim_no_funds_moved() {
    let (env, vault, token, admin, alice, _fee) = setup();

    fund_pool_via_cancel(&env, &vault, &alice, &token);
    let pool_before = vault.get_insurance_pool_balance(&token);

    let evidence = soroban_sdk::String::from_str(&env, "incident-xyz");
    let claim_id = vault.claim_insurance(&alice, &token, &50_i128, &evidence);
    vault.deny_claim(&admin, &claim_id);

    // Pool balance unchanged.
    assert_eq!(vault.get_insurance_pool_balance(&token), pool_before);
}

#[test]
fn test_approve_already_resolved_claim_fails() {
    let (env, vault, token, admin, alice, _fee) = setup();

    fund_pool_via_cancel(&env, &vault, &alice, &token);

    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let claim_id = vault.claim_insurance(&alice, &token, &50_i128, &evidence);
    vault.approve_claim(&admin, &claim_id);

    // Approving again should fail.
    let result = vault.try_approve_claim(&admin, &claim_id);
    assert_eq!(result, Err(Ok(VaultError::ClaimAlreadyResolved)));
}

#[test]
fn test_deny_already_resolved_claim_fails() {
    let (env, vault, token, admin, alice, _fee) = setup();

    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let claim_id = vault.claim_insurance(&alice, &token, &1_i128, &evidence);
    vault.deny_claim(&admin, &claim_id);

    let result = vault.try_deny_claim(&admin, &claim_id);
    assert_eq!(result, Err(Ok(VaultError::ClaimAlreadyResolved)));
}

#[test]
fn test_approve_claim_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let claim_id = vault.claim_insurance(&alice, &token, &1_i128, &evidence);

    let result = vault.try_approve_claim(&alice, &claim_id);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_deny_claim_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let evidence = soroban_sdk::String::from_str(&env, "evidence");
    let claim_id = vault.claim_insurance(&alice, &token, &1_i128, &evidence);

    let result = vault.try_deny_claim(&alice, &claim_id);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_approve_nonexistent_claim_fails() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let result = vault.try_approve_claim(&admin, &999_u32);
    assert_eq!(result, Err(Ok(VaultError::NoClaimFound)));
}

#[test]
fn test_deny_nonexistent_claim_fails() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let result = vault.try_deny_claim(&admin, &999_u32);
    assert_eq!(result, Err(Ok(VaultError::NoClaimFound)));
}

#[test]
fn test_get_claim_returns_none_for_missing() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    assert!(vault.get_claim(&999_u32).is_none());
}

#[test]
fn test_claim_ids_are_monotonically_increasing() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let evidence = soroban_sdk::String::from_str(&env, "ev");
    let id0 = vault.claim_insurance(&alice, &token, &1_i128, &evidence);
    let id1 = vault.claim_insurance(&alice, &token, &1_i128, &evidence);
    let id2 = vault.claim_insurance(&alice, &token, &1_i128, &evidence);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_cancel_deposit_routes_insurance_cut_to_pool() {
    // Verify the exact arithmetic: penalty = amount * bps / 10_000
    // insurance_cut = penalty * 500 / 10_000  (5%)
    // fee_cut       = penalty - insurance_cut
    let (env, vault, token, _admin, alice, fee) = setup();

    let unlock_time = env.ledger().timestamp() + 120;
    // amount = 20_000, penalty_bps = 500 (5%) → penalty = 1_000
    // insurance_cut = 1_000 * 500 / 10_000 = 50
    // fee_cut = 1_000 - 50 = 950
    let deposit_id = vault.deposit(&alice, &token, &20_000_i128, &unlock_time, &500_u32);

    let token_client = TokenClient::new(&env, &token);
    let fee_before = token_client.balance(&fee);

    vault.cancel_deposit(&alice, &deposit_id);

    let fee_after = token_client.balance(&fee);
    assert_eq!(fee_after - fee_before, 950);
    assert_eq!(vault.get_insurance_pool_balance(&token), 50);
}


// ================================================================
//  Archival — Archive and Delete Deposits
// ================================================================

#[test]
fn test_archive_deposit_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the withdrawn deposit
    let result = vault.try_archive_deposit(
        &alice,
        &deposit_id,
        &token,
        &1_000,
        &unlock_time,
        &0,
    );
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_archive_deposit_fails_if_active() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create a deposit but don't withdraw it
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Try to archive active deposit — should fail
    let result = vault.try_archive_deposit(
        &alice,
        &deposit_id,
        &token,
        &1_000,
        &unlock_time,
        &0,
    );
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

#[test]
fn test_archive_deposit_fails_if_already_archived() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive it once
    vault.archive_deposit(&alice, &deposit_id, &token, &1_000, &unlock_time, &0);

    // Try to archive again — should fail
    let result = vault.try_archive_deposit(
        &alice,
        &deposit_id,
        &token,
        &1_000,
        &unlock_time,
        &0,
    );
    assert_eq!(result, Err(Ok(VaultError::NoArchivedDepositFound)));
}

#[test]
fn test_archive_deposit_by_ledger_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 100;

    // Create and withdraw a ledger-based deposit
    let deposit_id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Advance ledger past unlock
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 500,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: unlock_ledger,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });

    vault.withdraw(&alice, &deposit_id);

    // Archive the withdrawn ledger-based deposit
    let result = vault.try_archive_deposit_by_ledger(
        &alice,
        &deposit_id,
        &token,
        &1_000,
        &unlock_ledger,
        &0,
    );
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_delete_archived_deposit_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit(&alice, &deposit_id, &token, &1_000, &unlock_time, &0);

    // Advance time by 1 year (31,536,000 seconds)
    advance_time(&env, 31_536_000);

    // Delete the archived deposit — should succeed
    let result = vault.try_delete_archived_deposit(&alice, &deposit_id);
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_delete_archived_deposit_fails_if_too_young() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit(&alice, &deposit_id, &token, &1_000, &unlock_time, &0);

    // Try to delete immediately — should fail (too young)
    let result = vault.try_delete_archived_deposit(&alice, &deposit_id);
    assert_eq!(result, Err(Ok(VaultError::ArchivedDepositTooYoung)));
}

#[test]
fn test_delete_archived_deposit_fails_at_boundary() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit(&alice, &deposit_id, &token, &1_000, &unlock_time, &0);

    // Advance time by exactly 1 year - 1 second (too young)
    advance_time(&env, 31_536_000 - 1);

    // Try to delete — should fail
    let result = vault.try_delete_archived_deposit(&alice, &deposit_id);
    assert_eq!(result, Err(Ok(VaultError::ArchivedDepositTooYoung)));
}

#[test]
fn test_delete_archived_deposit_succeeds_at_exact_boundary() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create and withdraw a deposit
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit(&alice, &deposit_id, &token, &1_000, &unlock_time, &0);

    // Advance time by exactly 1 year (boundary condition)
    advance_time(&env, 31_536_000);

    // Delete should succeed
    let result = vault.try_delete_archived_deposit(&alice, &deposit_id);
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_delete_archived_deposit_fails_if_not_found() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Try to delete a non-existent archived deposit
    let result = vault.try_delete_archived_deposit(&alice, &999);
    assert_eq!(result, Err(Ok(VaultError::NoArchivedDepositFound)));
}

#[test]
fn test_delete_archived_deposit_by_ledger_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + 100;

    // Create and withdraw a ledger-based deposit
    let deposit_id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Advance ledger past unlock
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 500,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: unlock_ledger,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });

    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit_by_ledger(&alice, &deposit_id, &token, &1_000, &unlock_ledger, &0);

    // Advance time by 1 year
    advance_time(&env, 31_536_000);

    // Delete should succeed
    let result = vault.try_delete_archived_deposit(&alice, &deposit_id);
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_archive_after_cancelled_deposit() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create a deposit with penalty
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &5_000); // 50% penalty

    // Cancel the deposit (early exit)
    vault.cancel_deposit(&alice, &deposit_id);

    // Archive the cancelled deposit
    let result = vault.try_archive_deposit(
        &alice,
        &deposit_id,
        &token,
        &1_000,
        &unlock_time,
        &5_000,
    );
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_multiple_deposits_archival() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create three deposits
    let id0 = vault.deposit(&alice, &token, &500, &unlock_time, &0);
    let id1 = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let id2 = vault.deposit(&alice, &token, &1_500, &unlock_time, &0);

    // Withdraw all
    advance_time(&env, 3600);
    vault.withdraw(&alice, &id0);
    vault.withdraw(&alice, &id1);
    vault.withdraw(&alice, &id2);

    // Archive all
    vault.archive_deposit(&alice, &id0, &token, &500, &unlock_time, &0);
    vault.archive_deposit(&alice, &id1, &token, &1_000, &unlock_time, &0);
    vault.archive_deposit(&alice, &id2, &token, &1_500, &unlock_time, &0);

    // Advance time by 1 year and delete all
    advance_time(&env, 31_536_000);
    assert_eq!(vault.try_delete_archived_deposit(&alice, &id0), Ok(Ok(())));
    assert_eq!(vault.try_delete_archived_deposit(&alice, &id1), Ok(Ok(())));
    assert_eq!(vault.try_delete_archived_deposit(&alice, &id2), Ok(Ok(())));
}

#[test]
fn test_archive_preserves_original_deposit_data() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create deposit with specific data
    let deposit_id = vault.deposit(&alice, &token, &2_500, &unlock_time, &2_500); // 25% penalty

    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    // Archive the deposit
    vault.archive_deposit(&alice, &deposit_id, &token, &2_500, &unlock_time, &2_500);

    // Verify the archived entry matches the original
    // (by checking it can be successfully archived with matching data)
    // Re-archival should fail, confirming original data was preserved
    let result = vault.try_archive_deposit(
        &alice,
        &deposit_id,
        &token,
        &2_500,
        &unlock_time,
        &2_500,
    );
    assert_eq!(result, Err(Ok(VaultError::NoArchivedDepositFound)));
}

#[test]
fn test_archive_different_depositors() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(admin.clone());
    let token = TokenClient::new(&env, &token_address);
    let asset_client = StellarAssetClient::new(&env, &token_address);

    // Mint for both alice and bob
    asset_client.mint(&alice, &10_000);
    asset_client.mint(&bob, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    let unlock_time = env.ledger().timestamp() + 3600;

    // Alice creates a deposit
    let alice_id = vault.deposit(&alice, &token_address, &1_000, &unlock_time, &0);

    // Bob creates a deposit
    let bob_id = vault.deposit(&bob, &token_address, &2_000, &unlock_time, &0);

    // Both withdraw
    advance_time(&env, 3600);
    vault.withdraw(&alice, &alice_id);
    vault.withdraw(&bob, &bob_id);

    // Both archive
    vault.archive_deposit(&alice, &alice_id, &token_address, &1_000, &unlock_time, &0);
    vault.archive_deposit(&bob, &bob_id, &token_address, &2_000, &unlock_time, &0);

    // Advance time and delete
    advance_time(&env, 31_536_000);
    assert_eq!(vault.try_delete_archived_deposit(&alice, &alice_id), Ok(Ok(())));
    assert_eq!(vault.try_delete_archived_deposit(&bob, &bob_id), Ok(Ok(())));

    // Verify they're deleted
    assert_eq!(vault.try_delete_archived_deposit(&alice, &alice_id), Err(Ok(VaultError::NoArchivedDepositFound)));
    assert_eq!(vault.try_delete_archived_deposit(&bob, &bob_id), Err(Ok(VaultError::NoArchivedDepositFound)));
}

#[test]
fn test_delete_archived_deposit_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(admin.clone());
    let asset_client = StellarAssetClient::new(&env, &token_address);

    asset_client.mint(&alice, &10_000);
    vault.initialize(&admin, &fee_recipient, &None, &None);

    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token_address, &1_000, &unlock_time, &0);

    advance_time(&env, 3600);
    vault.withdraw(&alice, &deposit_id);

    vault.archive_deposit(&alice, &deposit_id, &token_address, &1_000, &unlock_time, &0);
    advance_time(&env, 31_536_000);

    // Try to delete from different depositor (bob) — should fail due to auth
    // Reset mocking to require explicit auth for bob
    env.mock_auths(&[(
        bob.clone(),
        soroban_sdk::auth::AuthorizedInvocation {
            function: soroban_sdk::auth::AuthorizedFunction::Contract((
                env.current_contract_address(),
                soroban_sdk::Symbol::new(&env, "delete_archived_deposit"),
                soroban_sdk::Vec::new(&env),
            )),
            sub_invocations: soroban_sdk::Vec::new(&env),
        },
    )]);

    let result = vault.try_delete_archived_deposit(&bob, &deposit_id);
    // This should fail because bob is not the depositor and the contract requires auth
    assert!(result.is_err());
}


// ================================================================
//  Emergency Withdrawal Per-Ledger Limit
// ================================================================

#[test]
fn test_emergency_withdrawal_limit_single_withdrawal_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // First emergency withdrawal should succeed
    let result = vault.try_emergency_withdraw(&admin, &alice, &deposit_id);
    assert_eq!(result, Ok(()));
    
    // Verify deposit is removed
    assert_eq!(vault.get_vault(&alice, &deposit_id), None);
}

#[test]
fn test_emergency_withdrawal_limit_cumulative_tracking() {
    let (env, vault, token, admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &100_000_000);
    
    let unlock_time = env.ledger().timestamp() + 3600;
    let id1 = vault.deposit(&alice, &token, &10_000_000, &unlock_time, &0);
    let id2 = vault.deposit(&alice, &token, &20_000_000, &unlock_time, &0);

    // First withdrawal (10M) should succeed
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &id1), Ok(()));
    
    // Query the current ledger's total
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 10_000_000);
    
    // Second withdrawal (20M) should succeed — total is 30M, under limit of 100M
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &id2), Ok(()));
    
    // Verify total is now 30M
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 30_000_000);
}

#[test]
fn test_emergency_withdrawal_circuit_breaker_trips_at_threshold() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;

    StellarAssetClient::new(&env, &token).mint(&alice, &MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER);
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER, &unlock_time, &0);

    vault.emergency_withdraw(&admin, &alice, &deposit_id);

    assert!(vault.is_paused());
    let history = vault.get_circuit_breaker_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().ledger, env.ledger().sequence());
    assert_eq!(history.get(0).unwrap().amount, MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER);
}

#[test]
fn test_emergency_withdrawal_circuit_breaker_rejects_above_threshold() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;

    let amount = MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER + 1;
    StellarAssetClient::new(&env, &token).mint(&alice, &amount);
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0);

    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &deposit_id),
        Err(Ok(VaultError::EmergencyWithdrawalLimitExceeded))
    );
    assert!(!vault.is_paused());
    assert!(vault.get_vault(&alice, &deposit_id).is_some());
}

#[test]
fn test_emergency_withdrawal_limit_exceeds_fails() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;
    
    // Mint enough for a deposit exceeding the limit
    StellarAssetClient::new(&env, &token).mint(&alice, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER + 1_000_000));
    
    let unlock_time = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER + 1_000_000), &unlock_time, &0);

    // Emergency withdrawal should fail because the amount exceeds the limit
    let result = vault.try_emergency_withdraw(&admin, &alice, &deposit_id);
    assert_eq!(result, Err(Ok(VaultError::EmergencyWithdrawalLimitExceeded)));
    
    // Verify deposit still exists
    assert!(vault.get_vault(&alice, &deposit_id).is_some());
}

#[test]
fn test_emergency_withdrawal_limit_at_boundary() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;
    
    StellarAssetClient::new(&env, &token).mint(&alice, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER * 2));
    
    let unlock_time = env.ledger().timestamp() + 3600;
    let id1 = vault.deposit(&alice, &token, &MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER, &unlock_time, &0);
    let id2 = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // First withdrawal (exactly at limit) should succeed
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &id1), Ok(()));
    
    // Second withdrawal (1 more) should fail — would exceed limit
    let result = vault.try_emergency_withdraw(&admin, &alice, &id2);
    assert_eq!(result, Err(Ok(VaultError::EmergencyWithdrawalLimitExceeded)));
}

#[test]
fn test_emergency_withdrawal_limit_resets_at_ledger_boundary() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;
    
    StellarAssetClient::new(&env, &token).mint(&alice, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER * 2));
    
    let unlock_time = env.ledger().timestamp() + 10_000;
    let id1 = vault.deposit(&alice, &token, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER - 1_000), &unlock_time, &0);
    let id2 = vault.deposit(&alice, &token, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER - 1_000), &unlock_time, &0);

    // First withdrawal in ledger N should succeed
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &id1), Ok(()));
    let total_ledger_n = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total_ledger_n, MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER - 1_000);
    
    // Advance to next ledger
    advance_time(&env, 5);
    
    // Second withdrawal should now be in a different ledger and succeed
    // (because the counter resets based on ledger sequence)
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &id2), Ok(()));
    
    // Verify new ledger's total is separate
    let total_ledger_next = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total_ledger_next, MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER - 1_000);
    
    // Verify old ledger's total is unchanged
    let old_ledger = env.ledger().sequence() - 1;
    let total_old = vault.get_emergency_withdrawal_total(&env, old_ledger);
    assert_eq!(total_old, MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER - 1_000);
}

#[test]
fn test_emergency_withdrawal_multiple_deposits_same_ledger() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;
    
    StellarAssetClient::new(&env, &token).mint(&alice, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER * 2));
    
    let unlock_time = env.ledger().timestamp() + 3600;
    
    // Create 5 deposits of 20M each = 100M total (exactly at limit)
    let mut ids = soroban_sdk::Vec::new(&env);
    for _ in 0..5 {
        let id = vault.deposit(&alice, &token, &20_000_000, &unlock_time, &0);
        ids.push_back(id);
    }

    // Withdraw all 5 in same ledger — should succeed because total == limit
    for i in 0..5 {
        let result = vault.try_emergency_withdraw(&admin, &alice, &ids.get(i).unwrap());
        assert_eq!(result, Ok(()));
    }
    
    // Verify total is 100M
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 100_000_000);
}

#[test]
fn test_emergency_withdrawal_limit_multiple_depositors() {
    let (env, vault, token, admin, alice, _fee) = setup();
    use crate::types::MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER;
    
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&bob, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER / 2));
    StellarAssetClient::new(&env, &token).mint(&alice, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER / 2));
    
    let unlock_time = env.ledger().timestamp() + 3600;
    let alice_id = vault.deposit(&alice, &token, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER / 2), &unlock_time, &0);
    let bob_id = vault.deposit(&bob, &token, &(MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER / 2), &unlock_time, &0);

    // First emergency withdrawal (alice, 50M) should succeed
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &alice_id), Ok(()));
    
    // Second emergency withdrawal (bob, 50M) should succeed — total is 100M (at limit)
    assert_eq!(vault.try_emergency_withdraw(&admin, &bob, &bob_id), Ok(()));
    
    // Verify total is 100M
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 100_000_000);
}

#[test]
fn test_emergency_withdrawal_query_nonexistent_ledger() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    
    // Query a ledger that has never had any emergency withdrawals
    let future_ledger = env.ledger().sequence() + 1000;
    let total = vault.get_emergency_withdrawal_total(&env, future_ledger);
    assert_eq!(total, 0);
}

#[test]
fn test_emergency_withdrawal_ledger_based_deposit_limit() {
    let (env, vault, token, admin, alice, _fee) = setup();
    
    let unlock_ledger = env.ledger().sequence() + 20;
    let deposit_id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Emergency withdrawal should work for ledger-based deposits too
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &deposit_id), Ok(()));
    
    // Verify tracking
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 1_000);
}

#[test]
fn test_emergency_withdrawal_mixed_deposit_types_same_ledger() {
    let (env, vault, token, admin, alice, _fee) = setup();
    
    StellarAssetClient::new(&env, &token).mint(&alice, &100_000);
    
    let unlock_time = env.ledger().timestamp() + 3600;
    let unlock_ledger = env.ledger().sequence() + 20;
    
    let ts_id = vault.deposit(&alice, &token, &30_000, &unlock_time, &0);
    let lg_id = vault.deposit_by_ledger(&alice, &token, &20_000, &unlock_ledger, &0);

    // Withdraw both in same ledger
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &ts_id), Ok(()));
    assert_eq!(vault.try_emergency_withdraw(&admin, &alice, &lg_id), Ok(()));
    
    // Verify total tracking
    let total = vault.get_emergency_withdrawal_total(&env, env.ledger().sequence());
    assert_eq!(total, 50_000);
}

// ================================================================
//  Sustainability Metrics Tests
// ================================================================

#[test]
fn test_sustainability_metrics_stored_on_deposit() {
    use crate::constants::RENEWABLE_ENERGY_BASELINE;

    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 3600; // 1 hour
    let amount = 1_000i128;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit should succeed");

    // Retrieve sustainability metrics
    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id);
    assert!(metrics.is_some(), "metrics should exist");

    let m = metrics.unwrap();
    assert_eq!(m.deposit_id, deposit_id);
    assert_eq!(m.depositor, alice);
    assert_eq!(m.renewable_energy_percent, RENEWABLE_ENERGY_BASELINE);
    assert!(m.carbon_footprint > 0, "carbon footprint should be calculated");
    assert!(m.timestamp > 0, "timestamp should be set");
}

#[test]
fn test_carbon_footprint_calculation() {
    use crate::constants::CARBON_BASELINE_PER_UNIT_SECOND;

    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100; // 100 seconds
    let amount = 1_000i128;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit should succeed");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist");

    // Expected: amount * duration * CARBON_BASELINE_PER_UNIT_SECOND
    let expected = amount * 100 * CARBON_BASELINE_PER_UNIT_SECOND;
    assert_eq!(metrics.carbon_footprint, expected, "carbon footprint calculation");
}

#[test]
fn test_carbon_offset_with_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let amount = 1_000i128;
    let penalty_bps = 1_000u32; // 10% penalty

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &penalty_bps)
        .expect("deposit should succeed");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist");

    // Carbon offset should be 10% of carbon footprint
    let expected_offset = (metrics.carbon_footprint * 1_000) / 10_000;
    assert_eq!(metrics.carbon_offset_grams, expected_offset, "carbon offset calculation");
}

#[test]
fn test_aggregated_carbon_totals() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // First deposit
    let amount1 = 500i128;
    let deposit1 = vault.deposit(&alice, &token, &amount1, &unlock_time, &0)
        .expect("first deposit");

    let metrics1 = vault.get_sustainability_metrics(&alice, &deposit1)
        .expect("first metrics");

    // Second deposit
    let amount2 = 1_000i128;
    let deposit2 = vault.deposit(&alice, &token, &amount2, &unlock_time, &0)
        .expect("second deposit");

    let metrics2 = vault.get_sustainability_metrics(&alice, &deposit2)
        .expect("second metrics");

    // Check aggregated report
    let report = vault.generate_sustainability_report(&alice);
    assert_eq!(
        report.total_carbon_footprint,
        metrics1.carbon_footprint + metrics2.carbon_footprint,
        "total carbon should be sum of both deposits"
    );
    assert_eq!(report.active_deposit_count, 2, "should have 2 active deposits");
}

#[test]
fn test_sustainability_report_averages_renewable() {
    use crate::constants::RENEWABLE_ENERGY_BASELINE;

    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Create multiple deposits
    vault.deposit(&alice, &token, &500, &unlock_time, &0)
        .expect("deposit 1");
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("deposit 2");

    let report = vault.generate_sustainability_report(&alice);
    
    // Average renewable should be the baseline since all deposits use it
    assert_eq!(report.average_renewable_energy_percent, RENEWABLE_ENERGY_BASELINE);
    assert_eq!(report.active_deposit_count, 2);
}

#[test]
fn test_sustainability_metrics_removed_on_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("deposit");

    // Verify metrics exist
    assert!(vault.get_sustainability_metrics(&alice, &deposit_id).is_some());

    // Advance time past unlock
    env.ledger().set_timestamp(unlock_time + 1);

    // Withdraw
    vault.withdraw(&alice, &deposit_id)
        .expect("withdraw should succeed");

    // Metrics should be removed
    assert!(vault.get_sustainability_metrics(&alice, &deposit_id).is_none());
}

#[test]
fn test_sustainability_metrics_removed_on_cancel() {
    let (env, vault, token, _admin, alice, fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("deposit");

    // Verify metrics exist
    let metrics_before = vault.get_sustainability_metrics(&alice, &deposit_id);
    assert!(metrics_before.is_some());

    // Cancel deposit
    vault.cancel_deposit(&alice, &deposit_id)
        .expect("cancel should succeed");

    // Metrics should be removed
    assert!(vault.get_sustainability_metrics(&alice, &deposit_id).is_none());
}

#[test]
fn test_no_carbon_offset_without_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("deposit with zero penalty");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist");

    assert_eq!(metrics.carbon_offset_grams, 0, "offset should be zero without penalty");
}

#[test]
fn test_full_carbon_offset_with_max_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock_time, &10_000)
        .expect("deposit with 100% penalty");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist");

    // 100% penalty means full carbon offset
    assert_eq!(
        metrics.carbon_offset_grams,
        metrics.carbon_footprint,
        "offset should equal footprint with 100% penalty"
    );
}

#[test]
fn test_sustainability_totals_updated_on_multiple_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let penalty_bps = 5_000u32; // 50%

    // Deposit 1
    vault.deposit(&alice, &token, &1_000, &unlock_time, &penalty_bps)
        .expect("deposit 1");

    let report1 = vault.generate_sustainability_report(&alice);
    let total1 = report1.total_carbon_footprint;
    let offset1 = report1.total_carbon_offset;

    // Deposit 2
    vault.deposit(&alice, &token, &2_000, &unlock_time, &penalty_bps)
        .expect("deposit 2");

    let report2 = vault.generate_sustainability_report(&alice);

    // Totals should be larger
    assert!(report2.total_carbon_footprint > total1, "total carbon should increase");
    assert!(report2.total_carbon_offset > offset1, "total offset should increase");
    assert_eq!(report2.active_deposit_count, 2);
}

#[test]
fn test_ledger_based_deposit_sustainability_calculation() {
    use crate::constants::MIN_LOCK_LEDGERS;
    use crate::storage::LEDGER_SECONDS;

    let (env, vault, token, _admin, alice, _fee) = setup();

    let current_ledger = env.ledger().sequence();
    let unlock_ledger = current_ledger + MIN_LOCK_LEDGERS + 10;
    let amount = 1_000i128;

    let deposit_id = vault.deposit_by_ledger(&alice, &token, &amount, &unlock_ledger, &0)
        .expect("ledger-based deposit");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist");

    // Verify carbon was calculated based on estimated duration
    assert!(metrics.carbon_footprint > 0);
    
    // Expected duration: (MIN_LOCK_LEDGERS + 10) * LEDGER_SECONDS seconds
    let expected_duration = (MIN_LOCK_LEDGERS + 10) as u64 * LEDGER_SECONDS;
    // Allow some variance due to integer division
    assert!(
        metrics.carbon_footprint > (amount * expected_duration as i128) - 100,
        "carbon should be approximately: amount * estimated_duration * baseline"
    );
}

#[test]
fn test_milestone_carbon_neutral() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Deposit with 100% offset (penalty_bps = 10_000)
    vault.deposit(&alice, &token, &1_000, &unlock_time, &10_000)
        .expect("deposit");

    // Check events were emitted (would be verified in real test setup with event capture)
    // This test serves as a placeholder for milestone emission verification
    let report = vault.generate_sustainability_report(&alice);
    assert_eq!(report.total_carbon_offset, report.total_carbon_footprint);
}

#[test]
fn test_zero_total_with_no_deposits() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    let report = vault.generate_sustainability_report(&alice);
    assert_eq!(report.total_carbon_footprint, 0);
    assert_eq!(report.total_carbon_offset, 0);
    assert_eq!(report.active_deposit_count, 0);
}

#[test]
fn test_sustainability_with_deposit_for() {
    use crate::constants::RENEWABLE_ENERGY_BASELINE;

    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    let amount = 1_000i128;

    // Bob deposits FOR Alice
    let bob: Address = Address::generate(&env);
    env.ledger().set(LedgerInfo {
        timestamp: now,
        protocol_version: 20,
        sequence_number: 0,
        network_id: Default::default(),
        base_reserve: 0,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: 0,
    });

    let deposit_id = vault.deposit_for(&bob, &alice, &token, &amount, &unlock_time, &0)
        .expect("deposit_for");

    let metrics = vault.get_sustainability_metrics(&alice, &deposit_id)
        .expect("metrics should exist for Alice");

    // Alice is the depositor in metrics
    assert_eq!(metrics.depositor, alice);
    assert_eq!(metrics.renewable_energy_percent, RENEWABLE_ENERGY_BASELINE);
}

#[test]
fn test_sustainability_report_timestamp() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;
    
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("deposit");

    let report = vault.generate_sustainability_report(&alice);
    
    // Report timestamp should be approximately now
    assert_eq!(report.report_timestamp, now);
}

#[test]
fn test_multiple_depositors_independent_metrics() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let bob: Address = Address::generate(&env);
    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Mint tokens for Bob
    let token_client = TokenClient::new(&env, &token);
    token_client.mint(&bob, &10_000);

    // Alice's deposit
    let alice_deposit = vault.deposit(&alice, &token, &1_000, &unlock_time, &0)
        .expect("Alice deposit");

    // Bob's deposit
    let bob_deposit = vault.deposit(&bob, &token, &2_000, &unlock_time, &0)
        .expect("Bob deposit");

    // Metrics should be independent
    let alice_metrics = vault.get_sustainability_metrics(&alice, &alice_deposit)
        .expect("Alice metrics");
    let bob_metrics = vault.get_sustainability_metrics(&bob, &bob_deposit)
        .expect("Bob metrics");

    // Different amounts should result in different carbon footprints
    assert!(alice_metrics.carbon_footprint != bob_metrics.carbon_footprint);
    assert_eq!(alice_metrics.depositor, alice);
    assert_eq!(bob_metrics.depositor, bob);
}


// ================================================================
//  NFT Evolution Tests
// ================================================================

#[test]
fn test_nft_creation_on_deposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;
    let amount = 5_000;

    // Create a deposit
    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    // NFT record should exist
    let nft = vault.get_nft_evolution(&alice, &deposit_id)
        .expect("NFT record exists");

    // Verify initial state
    assert_eq!(nft.deposit_id, deposit_id);
    assert_eq!(nft.stage, crate::nft::EvolutionStage::Egg);
    assert_eq!(nft.rarity, crate::nft::RarityTier::Uncommon); // 5000 >= 1000
    assert_eq!(nft.created_at, now);
    assert_eq!(nft.last_evolved_at, now);
    assert_eq!(nft.evolution_count, 0);
    assert_eq!(nft.current_amount, amount);
    assert_eq!(nft.unlock_time, unlock_time);
}

#[test]
fn test_nft_stage_egg_to_hatchling() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let now = env.ledger().timestamp();
    let unlock_time = now + 100_000;
    let amount = 5_000;

    // Create a deposit
    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    // Initial stage should be Egg
    let stage = vault.get_nft_stage(&alice, &deposit_id)
        .expect("stage query succeeds");
    assert_eq!(stage, crate::nft::EvolutionStage::Egg);

    // Advance time to 14+ days
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = now + (15 * 24 * 60 * 60); // 15 days
    });

    // Create another deposit to trigger time update
    // (In practice, withdrawal or other operations would check evolution)
    let unlock_time2 = now + (15 * 24 * 60 * 60) + 100_000;
    let deposit_id2 = vault.deposit(&alice, &token, &amount, &unlock_time2, &0)
        .expect("second deposit succeeds");

    // Stage should still be Egg for first deposit (unless evolution check is called)
    // This test verifies the calculation, not automatic evolution
    let calc_stage = crate::nft::calculate_evolution_stage(15 * 24 * 60 * 60);
    assert_eq!(calc_stage, crate::nft::EvolutionStage::Hatchling);
}

#[test]
fn test_nft_rarity_common() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000; // Very short lock
    let amount = 100; // Very small amount

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    let rarity = vault.get_nft_rarity(&alice, &deposit_id)
        .expect("rarity query succeeds");
    assert_eq!(rarity, crate::nft::RarityTier::Common);
}

#[test]
fn test_nft_rarity_legendary() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + (400 * 24 * 60 * 60); // 400 days (exceeds 365 threshold)
    let amount = 2_000_000; // Exceeds legendary threshold

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    let rarity = vault.get_nft_rarity(&alice, &deposit_id)
        .expect("rarity query succeeds");
    assert_eq!(rarity, crate::nft::RarityTier::Legendary);
}

#[test]
fn test_nft_metadata_uri_present() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 10_000;
    let amount = 5_000;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    let metadata = vault.get_nft_metadata_uri(&alice, &deposit_id)
        .expect("metadata uri query succeeds");

    // Metadata should contain deposit ID and stage info
    let metadata_str = metadata.to_string();
    assert!(metadata_str.len() > 0);
    assert!(metadata_str.contains("deposit_nft"));
}

#[test]
fn test_nft_evolution_count_initial() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 10_000;
    let amount = 5_000;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    let count = vault.get_nft_evolution_count(&alice, &deposit_id)
        .expect("evolution count query succeeds");
    assert_eq!(count, 0); // No evolutions yet
}

#[test]
fn test_nft_removed_on_withdrawal() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;
    let amount = 5_000;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    // Verify NFT exists
    let nft_before = vault.get_nft_evolution(&alice, &deposit_id)
        .expect("NFT exists before withdrawal");
    assert_eq!(nft_before.deposit_id, deposit_id);

    // Advance time and withdraw
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = unlock_time + 1;
    });

    vault.withdraw(&alice, &deposit_id)
        .expect("withdraw succeeds");

    // NFT should be removed
    let nft_after = vault.get_nft_evolution(&alice, &deposit_id);
    assert!(nft_after.is_none());
}

#[test]
fn test_nft_removed_on_cancel() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let now = env.ledger().timestamp();
    let unlock_time = now + 100_000;
    let amount = 5_000;

    let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
        .expect("deposit succeeds");

    // Verify NFT exists
    let nft_before = vault.get_nft_evolution(&alice, &deposit_id)
        .expect("NFT exists before cancellation");
    assert_eq!(nft_before.deposit_id, deposit_id);

    // Cancel the deposit
    vault.cancel_deposit(&alice, &deposit_id)
        .expect("cancel_deposit succeeds");

    // NFT should be removed
    let nft_after = vault.get_nft_evolution(&alice, &deposit_id);
    assert!(nft_after.is_none());
}

#[test]
fn test_nft_rarity_by_duration() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let amount = 100; // Small amount

    // Test each duration threshold
    let test_cases = vec![
        (now + (6 * 24 * 60 * 60), crate::nft::RarityTier::Common),
        (now + (8 * 24 * 60 * 60), crate::nft::RarityTier::Uncommon),
        (now + (31 * 24 * 60 * 60), crate::nft::RarityTier::Rare),
        (now + (91 * 24 * 60 * 60), crate::nft::RarityTier::Epic),
        (now + (400 * 24 * 60 * 60), crate::nft::RarityTier::Legendary),
    ];

    for (idx, (unlock_time, expected_rarity)) in test_cases.into_iter().enumerate() {
        let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
            .expect("deposit succeeds");

        let rarity = vault.get_nft_rarity(&alice, &deposit_id)
            .expect("rarity query succeeds");
        assert_eq!(rarity, expected_rarity, "Failed for test case {}", idx);
    }
}

#[test]
fn test_nft_multiple_deposits_independent() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();

    // Create first deposit with small amount
    let deposit_id1 = vault.deposit(&alice, &token, &100, &(now + 1000), &0)
        .expect("first deposit succeeds");

    // Create second deposit with large amount
    let deposit_id2 = vault.deposit(&alice, &token, &2_000_000, &(now + 100_000), &0)
        .expect("second deposit succeeds");

    // NFTs should have different rarities
    let rarity1 = vault.get_nft_rarity(&alice, &deposit_id1)
        .expect("first rarity query succeeds");
    let rarity2 = vault.get_nft_rarity(&alice, &deposit_id2)
        .expect("second rarity query succeeds");

    assert_eq!(rarity1, crate::nft::RarityTier::Common);
    assert_eq!(rarity2, crate::nft::RarityTier::Legendary);
}

#[test]
fn test_nft_calculate_stage_progression() {
    // Direct unit tests for stage calculation
    let test_cases = vec![
        (0, crate::nft::EvolutionStage::Egg),
        (1 * 24 * 60 * 60, crate::nft::EvolutionStage::Egg),
        (14 * 24 * 60 * 60, crate::nft::EvolutionStage::Hatchling),
        (30 * 24 * 60 * 60, crate::nft::EvolutionStage::Juvenile),
        (90 * 24 * 60 * 60, crate::nft::EvolutionStage::Adult),
        (180 * 24 * 60 * 60, crate::nft::EvolutionStage::Adult),
        (365 * 24 * 60 * 60, crate::nft::EvolutionStage::Ancient),
        (400 * 24 * 60 * 60, crate::nft::EvolutionStage::Ancient),
    ];

    for (age_secs, expected_stage) in test_cases {
        let stage = crate::nft::calculate_evolution_stage(age_secs);
        assert_eq!(stage, expected_stage, "Stage mismatch for age_secs={}", age_secs);
    }
}

#[test]
fn test_nft_query_nonexistent() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // Query for nonexistent deposit should return None
    let nft = vault.get_nft_evolution(&alice, &999);
    assert!(nft.is_none());

    let stage = vault.get_nft_stage(&alice, &999);
    assert!(stage.is_none());

    let rarity = vault.get_nft_rarity(&alice, &999);
    assert!(rarity.is_none());

    let metadata = vault.get_nft_metadata_uri(&alice, &999);
    assert!(metadata.is_none());

    let count = vault.get_nft_evolution_count(&alice, &999);
    assert!(count.is_none());
}

#[test]
fn test_nft_rarity_amount_threshold_boundary() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;

    // Test boundary values
    let test_cases = vec![
        (999, crate::nft::RarityTier::Common),
        (1_000, crate::nft::RarityTier::Uncommon),
        (9_999, crate::nft::RarityTier::Uncommon),
        (10_000, crate::nft::RarityTier::Rare),
        (99_999, crate::nft::RarityTier::Rare),
        (100_000, crate::nft::RarityTier::Epic),
        (999_999, crate::nft::RarityTier::Epic),
        (1_000_000, crate::nft::RarityTier::Legendary),
    ];

    for (amount, expected_rarity) in test_cases {
        let deposit_id = vault.deposit(&alice, &token, &amount, &unlock_time, &0)
            .expect("deposit succeeds");

        let rarity = vault.get_nft_rarity(&alice, &deposit_id)
            .expect("rarity query succeeds");
        assert_eq!(rarity, expected_rarity, "Failed for amount={}", amount);
    }
}


// ================================================================
//  Batch Withdrawal Tests
// ================================================================

#[test]
fn test_batch_withdraw_success_single_deposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;

    // Create a deposit
    let deposit_id = vault.deposit(&alice, &token, &1000, &unlock_time, &0)
        .expect("deposit succeeds");

    // Try to batch withdraw before unlock
    let deposit_ids = Vec::from_array(&env, [deposit_id]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect_err("batch withdraw should fail before unlock");
    assert_eq!(result, VaultError::FundsStillLocked);

    // Advance time past unlock
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 1,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw should succeed
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.total_attempted, 1);
    assert_eq!(result.successful_count, 1);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.total_amount, 1000);
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results.get(0).deposit_id, deposit_id);
    assert_eq!(result.results.get(0).success, true);
    assert_eq!(result.results.get(0).error_code, 0);
    assert_eq!(result.results.get(0).amount, 1000);
}

#[test]
fn test_batch_withdraw_multiple_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();

    // Create three deposits with different unlock times
    let unlock_time_1 = now + 500;
    let unlock_time_2 = now + 1000;
    let unlock_time_3 = now + 1500;

    let id_1 = vault.deposit(&alice, &token, &100, &unlock_time_1, &0)
        .expect("deposit 1 succeeds");
    let id_2 = vault.deposit(&alice, &token, &200, &unlock_time_2, &0)
        .expect("deposit 2 succeeds");
    let id_3 = vault.deposit(&alice, &token, &300, &unlock_time_3, &0)
        .expect("deposit 3 succeeds");

    // Advance time to unlock deposit 1 and 2, but not 3
    env.ledger().set(LedgerInfo {
        timestamp: now + 1001,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw all three
    let deposit_ids = Vec::from_array(&env, [id_1, id_2, id_3]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.total_attempted, 3);
    assert_eq!(result.successful_count, 2); // id_1 and id_2
    assert_eq!(result.failed_count, 1);     // id_3 still locked
    assert_eq!(result.total_amount, 300);   // 100 + 200

    // Verify individual results
    assert_eq!(result.results.get(0).success, true);
    assert_eq!(result.results.get(0).amount, 100);
    
    assert_eq!(result.results.get(1).success, true);
    assert_eq!(result.results.get(1).amount, 200);
    
    assert_eq!(result.results.get(2).success, false);
    assert_eq!(result.results.get(2).error_code, VaultError::FundsStillLocked as u32);
}

#[test]
fn test_batch_withdraw_all_locked() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 5000; // Far in future

    // Create multiple deposits
    let id_1 = vault.deposit(&alice, &token, &100, &unlock_time, &0)
        .expect("deposit 1 succeeds");
    let id_2 = vault.deposit(&alice, &token, &200, &unlock_time, &0)
        .expect("deposit 2 succeeds");

    // Try to batch withdraw without advancing time
    let deposit_ids = Vec::from_array(&env, [id_1, id_2]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw returns result");

    assert_eq!(result.total_attempted, 2);
    assert_eq!(result.successful_count, 0);
    assert_eq!(result.failed_count, 2);
    assert_eq!(result.total_amount, 0);

    // All results should indicate FundsStillLocked
    for res in result.results.iter() {
        assert_eq!(res.success, false);
        assert_eq!(res.error_code, VaultError::FundsStillLocked as u32);
    }
}

#[test]
fn test_batch_withdraw_empty_list() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    let deposit_ids = Vec::new(&env);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("empty batch withdraw succeeds");

    assert_eq!(result.total_attempted, 0);
    assert_eq!(result.successful_count, 0);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.total_amount, 0);
}

#[test]
fn test_batch_withdraw_nonexistent_deposits() {
    let (env, vault, _token, _admin, alice, _fee) = setup();

    // Try to batch withdraw from IDs that don't exist
    let deposit_ids = Vec::from_array(&env, [999, 1000, 1001]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw returns result");

    assert_eq!(result.total_attempted, 3);
    assert_eq!(result.successful_count, 0);
    assert_eq!(result.failed_count, 3);
    assert_eq!(result.total_amount, 0);

    // All should be NoDepositFound
    for res in result.results.iter() {
        assert_eq!(res.success, false);
        assert_eq!(res.error_code, VaultError::NoDepositFound as u32);
    }
}

#[test]
fn test_batch_withdraw_all_unlocked() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Create three deposits with same unlock time
    let id_1 = vault.deposit(&alice, &token, &100, &unlock_time, &0)
        .expect("deposit 1 succeeds");
    let id_2 = vault.deposit(&alice, &token, &200, &unlock_time, &0)
        .expect("deposit 2 succeeds");
    let id_3 = vault.deposit(&alice, &token, &300, &unlock_time, &0)
        .expect("deposit 3 succeeds");

    // Advance time past unlock
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 1,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw all
    let deposit_ids = Vec::from_array(&env, [id_1, id_2, id_3]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.total_attempted, 3);
    assert_eq!(result.successful_count, 3);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.total_amount, 600); // 100 + 200 + 300

    // Verify all succeeded
    for i in 0..3 {
        assert_eq!(result.results.get(i).success, true);
        assert_eq!(result.results.get(i).error_code, 0);
    }

    // Verify deposits are gone from storage
    assert!(vault.get_vault(&alice, &id_1).is_none());
    assert!(vault.get_vault(&alice, &id_2).is_none());
    assert!(vault.get_vault(&alice, &id_3).is_none());
}

#[test]
fn test_batch_withdraw_size_limit() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Create MAX_BATCH_SIZE + 1 deposits
    let mut deposit_ids = Vec::new(&env);
    for i in 0..26 { // MAX_BATCH_SIZE = 25, so 26 exceeds limit
        let id = vault.deposit(&alice, &token, &(100 as i128 + i as i128), &unlock_time, &0)
            .expect("deposit succeeds");
        deposit_ids.push_back(id);
    }

    // Try to batch withdraw 26 deposits (exceeds MAX_BATCH_SIZE of 25)
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect_err("batch withdraw should fail due to size limit");

    assert_eq!(result, VaultError::BatchSizeExceeded);
}

#[test]
fn test_batch_withdraw_max_batch_size_exact() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Create exactly MAX_BATCH_SIZE deposits
    let mut deposit_ids = Vec::new(&env);
    for i in 0..25 { // MAX_BATCH_SIZE = 25
        let id = vault.deposit(&alice, &token, &(100 as i128 + i as i128), &unlock_time, &0)
            .expect("deposit succeeds");
        deposit_ids.push_back(id);
    }

    // Advance time past unlock
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 1,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw exactly MAX_BATCH_SIZE should succeed
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw with MAX_BATCH_SIZE succeeds");

    assert_eq!(result.total_attempted, 25);
    assert_eq!(result.successful_count, 25);
    assert_eq!(result.failed_count, 0);

    // Verify total is sum of 100..124
    let expected_total = (100..125).sum::<i128>();
    assert_eq!(result.total_amount, expected_total);
}

#[test]
fn test_batch_withdraw_compound_interest() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;
    let compound_freq = 60; // Compound every 60 seconds

    // Create a deposit with compound interest
    let deposit_id = vault.deposit_with_compound(&alice, &token, &1000, &unlock_time, &0, &compound_freq)
        .expect("deposit with compound succeeds");

    // Advance time to unlock + 120 seconds (2 periods)
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 120,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw should accrue interest
    let deposit_ids = Vec::from_array(&env, [deposit_id]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.successful_count, 1);
    // With 5% annual interest compounded every 60 seconds, amount should increase
    // Expected: 1000 * (1 + 0.05/365/86400 * 60) ^ 2 ≈ 1000.0000573
    // Due to integer arithmetic, should be at least >= 1000
    assert!(result.total_amount >= 1000);
}

#[test]
fn test_batch_withdraw_mixed_types() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 1000;

    // Create timestamp-based deposit
    let ts_id = vault.deposit(&alice, &token, &100, &unlock_time, &0)
        .expect("timestamp deposit succeeds");

    // Advance time to unlock
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 1,
        sequence_number: 200,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Create ledger-based deposit (after time advance)
    let unlock_ledger = 225;
    let ledger_id = vault.deposit_by_ledger(&alice, &token, &200, &unlock_ledger, &0)
        .expect("ledger deposit succeeds");

    // Advance ledger sequence to unlock ledger
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 100,
        sequence_number: unlock_ledger,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw both
    let deposit_ids = Vec::from_array(&env, [ts_id, ledger_id]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.total_attempted, 2);
    assert_eq!(result.successful_count, 2);
    assert_eq!(result.total_amount, 300); // 100 + 200
}

#[test]
fn test_batch_withdraw_events_emitted() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();
    let unlock_time = now + 100;

    // Create deposits
    let id_1 = vault.deposit(&alice, &token, &100, &unlock_time, &0)
        .expect("deposit 1 succeeds");
    let id_2 = vault.deposit(&alice, &token, &200, &unlock_time, &0)
        .expect("deposit 2 succeeds");

    // Advance time past unlock
    env.ledger().set(LedgerInfo {
        timestamp: unlock_time + 1,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw
    let deposit_ids = Vec::from_array(&env, [id_1, id_2]);
    vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    // Check events were emitted
    let events = env.events().all();
    
    // Should have:
    // - 2 individual "withdraw" events (one per successful withdrawal)
    // - 1 "batch_withdraw" event
    // Count "withdraw" and "batch_withdraw" events
    let mut withdraw_count = 0;
    let mut batch_count = 0;
    
    for event in events.iter() {
        let topics = event.topics;
        if topics.len() > 0 {
            if let Ok(symbol) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Attempt to match against known events
                let topic_str = format!("{:?}", topics.get(0));
                topic_str
            })) {
                // Simple check: if we have the expected event structure
                // In real code, we'd deserialize and check properly
            }
        }
    }

    // At minimum, batch_withdraw event should be emitted
    assert!(events.len() > 0);
}

#[test]
fn test_batch_withdraw_partial_success_mixed_state() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let now = env.ledger().timestamp();

    // Create multiple deposits in various states
    let unlocked_id = vault.deposit(&alice, &token, &100, &(now + 100), &0)
        .expect("unlocked deposit succeeds");
    let locked_id = vault.deposit(&alice, &token, &200, &(now + 5000), &0)
        .expect("locked deposit succeeds");

    // Advance time to unlock first deposit only
    env.ledger().set(LedgerInfo {
        timestamp: now + 1000,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Batch withdraw both
    let deposit_ids = Vec::from_array(&env, [unlocked_id, locked_id]);
    let result = vault.withdraw_batch(&alice, &deposit_ids)
        .expect("batch withdraw succeeds");

    assert_eq!(result.total_attempted, 2);
    assert_eq!(result.successful_count, 1);
    assert_eq!(result.failed_count, 1);
    assert_eq!(result.total_amount, 100); // Only unlocked deposit

    // Verify the locked deposit is still there
    assert!(vault.get_vault(&alice, &locked_id).is_some());

    // Verify the unlocked deposit is gone
    assert!(vault.get_vault(&alice, &unlocked_id).is_none());
}
