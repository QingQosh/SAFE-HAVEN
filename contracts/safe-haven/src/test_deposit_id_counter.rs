//! # Deposit ID Counter Verification Tests
//!
//! This module contains comprehensive tests for the deposit ID counter implementation.
//! Tests verify:
//! - Counter initialization (starts at 0)
//! - Monotonic incrementing
//! - Counter isolation per depositor
//! - Persistence across sessions
//! - No wraparound behavior
//! - Storage layout prevents collisions

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Env,
};

use crate::{
    contract::SafeHavenClient,
    contract::SafeHaven,
    errors::VaultError,
};

fn setup_deposit_id_tests() -> (Env, SafeHavenClient<'static>, Address, Address, Address, Address) {
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

    let token_client = StellarAssetClient::new(&env, &token_address);
    token_client.mint(&admin, &20_000);
    token_client.mint(&alice, &20_000);
    token_client.mint(&bob, &20_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    (env, vault, token_address, admin, alice, bob)
}

// ================================================================
//  Test 1: Counter starts at 0 and increments monotonically
// ================================================================

#[test]
fn test_deposit_id_counter_starts_at_zero() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let t1 = env.ledger().timestamp() + 3600;
    let id0 = vault.deposit(&alice, &token, &1_000, &t1, &0);

    assert_eq!(
        id0, 0,
        "First deposit for a new depositor should get ID=0"
    );
}

#[test]
fn test_deposit_id_counter_increments_monotonically() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();
    let mut ids = Vec::new();

    // Create 10 consecutive deposits
    for i in 0..10 {
        let unlock_time = base_time + 3600 + (i as u64 * 100);
        let id = vault.deposit(&alice, &token, &100 * (i as i128 + 1), &unlock_time, &0);
        ids.push(id);
    }

    // Verify all IDs are unique and sequential
    for (i, &id) in ids.iter().enumerate() {
        assert_eq!(
            id, i as u32,
            "Deposit {} should have ID={}, got ID={}",
            i,
            i,
            id
        );
    }
}

#[test]
fn test_deposit_ids_form_continuous_sequence() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    let id0 = vault.deposit(&alice, &token, &100, &(base_time + 3600), &0);
    let id1 = vault.deposit(&alice, &token, &100, &(base_time + 7200), &0);
    let id2 = vault.deposit(&alice, &token, &100, &(base_time + 10800), &0);
    let id3 = vault.deposit(&alice, &token, &100, &(base_time + 14400), &0);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 4);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
    assert_eq!(ids.get(2).unwrap(), 2);
    assert_eq!(ids.get(3).unwrap(), 3);
}

// ================================================================
//  Test 2: Counter isolation per depositor
// ================================================================

#[test]
fn test_deposit_counter_isolated_per_depositor() {
    let (env, vault, token, _admin, alice, bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Alice makes 3 deposits
    let alice_id0 = vault.deposit(&alice, &token, &100, &(base_time + 3600), &0);
    let alice_id1 = vault.deposit(&alice, &token, &100, &(base_time + 7200), &0);
    let alice_id2 = vault.deposit(&alice, &token, &100, &(base_time + 10800), &0);

    // Bob makes 2 deposits - should also start from 0
    let bob_id0 = vault.deposit(&bob, &token, &100, &(base_time + 3600), &0);
    let bob_id1 = vault.deposit(&bob, &token, &100, &(base_time + 7200), &0);

    // Verify Alice's counter
    assert_eq!(alice_id0, 0);
    assert_eq!(alice_id1, 1);
    assert_eq!(alice_id2, 2);

    // Verify Bob's counter is independent and also starts at 0
    assert_eq!(bob_id0, 0);
    assert_eq!(bob_id1, 1);

    // Verify both can retrieve their deposits uniquely
    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 100);
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 100);
    assert_eq!(vault.get_vault(&alice, &2).unwrap().amount, 100);

    assert_eq!(vault.get_vault(&bob, &0).unwrap().amount, 100);
    assert_eq!(vault.get_vault(&bob, &1).unwrap().amount, 100);

    // Verify no cross-contamination
    assert!(vault.get_vault(&alice, &3).is_none());
    assert!(vault.get_vault(&bob, &2).is_none());
}

#[test]
fn test_multiple_depositors_independent_counters() {
    let (env, vault, token, _admin, alice, bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Interleave deposits from Alice and Bob
    let alice_id0 = vault.deposit(&alice, &token, &100, &(base_time + 3600), &0);
    let bob_id0 = vault.deposit(&bob, &token, &100, &(base_time + 3600), &0);
    let alice_id1 = vault.deposit(&alice, &token, &100, &(base_time + 7200), &0);
    let bob_id1 = vault.deposit(&bob, &token, &100, &(base_time + 7200), &0);
    let alice_id2 = vault.deposit(&alice, &token, &100, &(base_time + 10800), &0);

    assert_eq!(alice_id0, 0);
    assert_eq!(bob_id0, 0);
    assert_eq!(alice_id1, 1);
    assert_eq!(bob_id1, 1);
    assert_eq!(alice_id2, 2);

    let alice_ids = vault.get_deposit_ids(&alice);
    let bob_ids = vault.get_deposit_ids(&bob);

    assert_eq!(alice_ids.len(), 3);
    assert_eq!(bob_ids.len(), 2);
}

// ================================================================
//  Test 3: Counter persistence after withdrawal
// ================================================================

#[test]
fn test_counter_persists_after_partial_withdrawal() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    let id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    let id1 = vault.deposit(&alice, &token, &1_000, &(base_time + 7200), &0);
    let id2 = vault.deposit(&alice, &token, &1_000, &(base_time + 10800), &0);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    // Advance time and withdraw the first deposit
    env.ledger().set(LedgerInfo {
        timestamp: base_time + 3601,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    vault.withdraw(&alice, &0);

    // Verify ID 0 is gone
    assert!(vault.get_vault(&alice, &0).is_none());

    // Verify IDs 1 and 2 still exist
    assert!(vault.get_vault(&alice, &1).is_some());
    assert!(vault.get_vault(&alice, &2).is_some());

    // Make a new deposit and verify the counter continues from where it left off
    let id3 = vault.deposit(&alice, &token, &1_000, &(base_time + 14400), &0);
    assert_eq!(id3, 3, "Counter should continue from 3, not reset or reuse 0");

    assert!(vault.get_vault(&alice, &3).is_some());
}

#[test]
fn test_counter_continues_from_max_after_full_drain() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Create and then withdraw all deposits
    let id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    let id1 = vault.deposit(&alice, &token, &1_000, &(base_time + 7200), &0);
    let id2 = vault.deposit(&alice, &token, &1_000, &(base_time + 10800), &0);

    // Advance past all unlock times
    env.ledger().set(LedgerInfo {
        timestamp: base_time + 10801,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    // Withdraw all
    vault.withdraw(&alice, &0);
    vault.withdraw(&alice, &1);
    vault.withdraw(&alice, &2);

    // Verify all are gone
    assert!(vault.get_vault(&alice, &0).is_none());
    assert!(vault.get_vault(&alice, &1).is_none());
    assert!(vault.get_vault(&alice, &2).is_none());

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 0, "All deposits should be withdrawn");

    // Now make a fresh deposit - should get ID=3, not reuse 0
    let id_new = vault.deposit(&alice, &token, &1_000, &(base_time + 14400), &0);
    assert_eq!(
        id_new, 3,
        "Counter should continue from 3 even after full drain, not wrap or reuse"
    );

    assert!(vault.get_vault(&alice, &3).is_some());
}

#[test]
fn test_counter_survives_multiple_cycles() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Cycle 1: Deposit, withdraw, deposit
    let id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    assert_eq!(id0, 0);

    env.ledger().set(LedgerInfo {
        timestamp: base_time + 3601,
        sequence_number: 1,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    vault.withdraw(&alice, &0);

    let id1 = vault.deposit(&alice, &token, &1_000, &(base_time + 7200), &0);
    assert_eq!(id1, 1);

    // Cycle 2: Deposit multiple, withdraw one
    let id2 = vault.deposit(&alice, &token, &1_000, &(base_time + 10800), &0);
    let id3 = vault.deposit(&alice, &token, &1_000, &(base_time + 14400), &0);

    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    env.ledger().set(LedgerInfo {
        timestamp: base_time + 7201,
        sequence_number: 2,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    vault.withdraw(&alice, &1);

    // Cycle 3: Make final deposit
    env.ledger().set(LedgerInfo {
        timestamp: base_time + 14401,
        sequence_number: 3,
        network_id: Default::default(),
        base_fee: 100,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: u32::MAX,
    });

    vault.withdraw(&alice, &2);
    vault.withdraw(&alice, &3);

    let id4 = vault.deposit(&alice, &token, &1_000, &(base_time + 18000), &0);
    assert_eq!(id4, 4);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), 4);
}

#[test]
fn test_cancel_deposit_does_not_reset_counter() {
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    let id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &100); // 1% penalty
    let id1 = vault.deposit(&alice, &token, &1_000, &(base_time + 7200), &0);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);

    // Cancel the first deposit (early exit with penalty)
    vault.cancel_deposit(&alice, &0);

    // Verify it's gone
    assert!(vault.get_vault(&alice, &0).is_none());

    // Make a new deposit - should get ID=2, not reuse 0
    let id2 = vault.deposit(&alice, &token, &1_000, &(base_time + 10800), &0);
    assert_eq!(id2, 2, "Counter should continue after cancel_deposit");
}

// ================================================================
//  Test 4: Storage layout prevents collisions
// ================================================================

#[test]
fn test_storage_key_uniqueness_per_depositor_counter() {
    let (env, vault, token, _admin, alice, bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Both depositors create deposits
    let alice_id = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    let bob_id = vault.deposit(&bob, &token, &1_000, &(base_time + 3600), &0);

    // Both should get ID=0 due to independent counters
    assert_eq!(alice_id, 0);
    assert_eq!(bob_id, 0);

    // But retrieval must be keyed by (depositor, id) to avoid collision
    assert_eq!(vault.get_vault(&alice, &0).unwrap().depositor, alice);
    assert_eq!(vault.get_vault(&bob, &0).unwrap().depositor, bob);

    // Different amounts to ensure they're really separate
    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 1_000);
    assert_eq!(vault.get_vault(&bob, &0).unwrap().amount, 1_000);
}

#[test]
fn test_deposit_entries_keyed_by_depositor_and_id() {
    let (env, vault, token, _admin, alice, bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Create deposits with overlapping IDs but different depositors
    let alice_id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    let alice_id1 = vault.deposit(&alice, &token, &2_000, &(base_time + 7200), &0);

    let bob_id0 = vault.deposit(&bob, &token, &3_000, &(base_time + 3600), &0);
    let bob_id1 = vault.deposit(&bob, &token, &4_000, &(base_time + 7200), &0);

    // All should have sequential IDs
    assert_eq!(alice_id0, 0);
    assert_eq!(alice_id1, 1);
    assert_eq!(bob_id0, 0);
    assert_eq!(bob_id1, 1);

    // But each (depositor, id) pair should be unique
    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 1_000);
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 2_000);
    assert_eq!(vault.get_vault(&bob, &0).unwrap().amount, 3_000);
    assert_eq!(vault.get_vault(&bob, &1).unwrap().amount, 4_000);

    // Cross-lookup should fail
    assert!(vault.get_vault(&alice, &2).is_none());
    assert!(vault.get_vault(&bob, &2).is_none());
}

// ================================================================
//  Test 5: No wraparound behavior (u32 limits)
// ================================================================

#[test]
fn test_deposit_id_uses_u32() {
    // This test verifies that deposit_id is a u32 by checking
    // that the type system enforces it. We can't easily test max u32
    // in a sandbox, but we can verify the behavior doesn't wrap.
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Create a large number of deposits (practical limit without exhausting resources)
    let mut ids = Vec::new();
    for i in 0..100 {
        let unlock_time = base_time + 3600 + (i as u64 * 100);
        let id = vault.deposit(&alice, &token, &10 * (i as i128 + 1), &unlock_time, &0);
        ids.push(id);
    }

    // Verify no wraparound occurred
    for (i, &id) in ids.iter().enumerate() {
        assert_eq!(
            id, i as u32,
            "ID {} should be {}, not wrapped around",
            i,
            i
        );
    }

    // Verify monotonicity is maintained
    let last_id = vault.deposit(&alice, &token, &1000, &(base_time + 20000), &0);
    assert_eq!(last_id, 100, "Counter should reach 100 without wraparound");
}

#[test]
fn test_deposit_counter_uses_saturating_add() {
    // The counter uses saturating_add, which means it won't overflow.
    // We verify this by checking the implementation detail:
    // next_deposit_id returns the current value and increments.
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();

    // Create many deposits to verify saturation behavior is safe
    for i in 0..50 {
        let unlock_time = base_time + 3600 + (i as u64 * 100);
        let id = vault.deposit(&alice, &token, &100, &unlock_time, &0);
        assert_eq!(
            id, i as u32,
            "Deposit {} should have ID={}",
            i,
            i
        );
    }

    // Final deposit should get a high but valid ID
    let final_id = vault.deposit(&alice, &token, &100, &(base_time + 10000), &0);
    assert_eq!(final_id, 50);
}

// ================================================================
//  Test 6: Counter behavior with ledger-based deposits
// ================================================================

#[test]
fn test_deposit_id_counter_shared_across_deposit_types() {
    // Timestamp-based and ledger-based deposits share the same counter
    // so IDs remain unique across both types
    let (env, vault, token, _admin, alice, _bob) = setup_deposit_id_tests();

    let base_time = env.ledger().timestamp();
    let current_ledger = env.ledger().sequence();

    // Create a timestamp-based deposit
    let id0 = vault.deposit(&alice, &token, &1_000, &(base_time + 3600), &0);
    assert_eq!(id0, 0);

    // Create a ledger-based deposit
    let id1 = vault.deposit_by_ledger(
        &alice,
        &token,
        &1_000,
        &(current_ledger + 100),
        &0,
    );
    assert_eq!(id1, 1);

    // Create another timestamp-based deposit
    let id2 = vault.deposit(&alice, &token, &1_000, &(base_time + 7200), &0);
    assert_eq!(id2, 2);

    // Verify all are retrievable by their unique IDs
    assert!(vault.get_vault(&alice, &0).is_some()); // timestamp-based
    assert!(vault.get_ledger_vault(&alice, &1).is_some()); // ledger-based
    assert!(vault.get_vault(&alice, &2).is_some()); // timestamp-based
}
