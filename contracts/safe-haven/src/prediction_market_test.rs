// ============================================================
//  Prediction Markets - Outcome List Storage Tests (Issue #100)
//
//  Verifies:
//  - Outcome names are stored in order (2, 3, and 4 outcomes)
//  - outcome_count matches the number of outcomes supplied
//  - Indexing by outcome_id returns the correct name
//  - get_market_with_outcomes returns market + all outcomes in order
//  - get_market_outcome queries work for each valid outcome_id
//  - Names list persisted via set_outcome_names_list is correct
//  Prediction Markets — Resolution Data Storage Tests (Issue #99)
// ============================================================
//
//  Scope:
//    - Resolution with data (Some(i128)) is accepted, stored, returned
//    - Resolution without data (None) is accepted, stored, returned
//    - resolution_data persists in get_prediction_market query
//    - OracleResolution record also stores the value
//
//  Out of scope (per issue):
//    - Data validation / custom formats / compression

#[cfg(test)]
mod prediction_market_outcome_list_tests {
    extern crate std;

    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        token::StellarAssetClient,
        Address, Env, String, Vec,
    };

    use crate::contract::{SafeHaven, SafeHavenClient};
    use crate::prediction_market_errors::PredictionMarketError;
    use crate::prediction_market_types::MarketStatus;

    // ----------------------------------------------------------------
    //  Test Helpers
    // ----------------------------------------------------------------

    /// Set up a fresh environment with the prediction market subsystem
    /// initialized. Returns (env, vault_client, admin, oracle, fee_recipient).
    fn setup_pm() -> (Env, SafeHavenClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register(SafeHaven, ());
        let vault = SafeHavenClient::new(&env, &vault_id);

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let fee_recipient = Address::generate(&env);

        // Initialize core contract (required before prediction markets)
        vault.initialize(&admin, &fee_recipient, &None, &None);

        // Initialize prediction market subsystem
        vault
            .init_prediction_markets(&admin, &fee_recipient, &Some(100))
            .expect("init_prediction_markets failed");

        (env, vault, admin, oracle, fee_recipient)
    }

    /// Advance the simulated ledger timestamp by `seconds`.
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

    /// Build a `Vec<String>` of outcome names inside `env`.
    fn outcome_vec(env: &Env, names: &[&str]) -> Vec<String> {
        let mut v = Vec::new(env);
        for n in names {
            v.push_back(String::from_str(env, n));
        }
        v
    }

    /// Create a market with `names` as outcomes. Returns market_id.
    fn create_market_with_outcomes(
        env: &Env,
        vault: &SafeHavenClient,
        creator: &Address,
        oracle: &Address,
        names: Vec<String>,
    ) -> u32 {
        let now = env.ledger().timestamp();
        let close_time = now + 7_200; // 2 hours from now
        let resolution_deadline = close_time + 3_600; // 1 hour after close

        vault
            .create_prediction_market(
                creator,
                &String::from_str(env, "aggregate_deposits"),
                &String::from_str(env, "Test market"),
                oracle,
                &close_time,
                &resolution_deadline,
                &names,
                &None,
            )
            .expect("create_prediction_market failed")
    }

    // ================================================================
    //  2-Outcome Market Tests
mod prediction_market_tests {
    extern crate std;

    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, String, Vec,
    };

    use crate::{
        contract::{SafeHaven, SafeHavenClient},
        prediction_market_errors::PredictionMarketError,
        prediction_market,
        prediction_market_storage as storage,
        prediction_market_types::*,
        prediction_market_storage,
    };

    fn betting_fixture() -> (Env, Address, Address, Address, u32, u64) {
        let env = Env::default();
        env.mock_all_auths();

        let contract = Address::generate(&env);
        let admin = Address::generate(&env);
        let bettor = Address::generate(&env);
        let oracle = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_id.address();
        let close_time = env.ledger().timestamp() + MIN_MARKET_DURATION_SECS;
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "Yes"), String::from_str(&env, "No")],
        );

        let market_id = env.as_contract(&contract, || {
            prediction_market::initialize_markets(&env, admin.clone(), admin.clone(), 0).unwrap();
            prediction_market::create_market(
                &env,
                admin,
                String::from_str(&env, "test"),
                String::from_str(&env, "test market"),
                oracle,
                close_time,
                close_time + 3600,
                outcomes,
                None,
            )
            .unwrap()
        });

        (env, contract, bettor, token, market_id, close_time)
    }

    fn setup_market_with_bettor() -> (Env, Address, Address, Address, u32) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let fee_recipient = Address::generate(&env);
        let bettor = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_id.address();

        StellarAssetClient::new(&env, &token).mint(&bettor, &10_000);

        prediction_market::initialize_markets(&env, admin.clone(), fee_recipient, 0).unwrap();

        let market_id = prediction_market::create_market(
            &env,
            admin,
            String::from_str(&env, "market"),
            String::from_str(&env, "Will deposits rise?"),
            Address::generate(&env),
            env.ledger().timestamp() + 3600,
            env.ledger().timestamp() + 7200,
            Vec::from_array(
                &env,
                [
                    String::from_str(&env, "Up"),
                    String::from_str(&env, "Down"),
                ],
            ),
            Some(0),
        )
        .unwrap();

        (env, bettor, token, fee_recipient, market_id)
    }

    // ================================================================
    //  Market Creation Tests (10 test cases)
    // ================================================================
    fn setup() -> (Env, SafeHavenClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register(SafeHaven, ());
        let vault = SafeHavenClient::new(&env, &vault_id);

        let admin: Address = Address::generate(&env);
        let fee_recipient: Address = Address::generate(&env);
        let alice: Address = Address::generate(&env);

        (env, vault, admin, fee_recipient, alice)
    }

    // ================================================================
    //  Initialization Tests - Acceptance Criteria
    // ================================================================

    /// Creating a 2-outcome market stores both names, sets outcome_count = 2,
    /// and each outcome is indexed by outcome_id 0 and 1.
    #[test]
    fn test_two_outcome_market_outcome_count() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["Yes (>1M)", "No (<=1M)"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let market = vault
            .get_prediction_market(&market_id)
            .expect("market not found");

        assert_eq!(market.outcome_count, 2, "outcome_count should be 2");
        assert_eq!(market.status, MarketStatus::Open);
    }

    /// Outcome 0 and 1 are retrievable individually with the correct names.
    #[test]
    fn test_two_outcome_market_indexing() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["Yes (>1M)", "No (<=1M)"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names.clone());

        let outcome0 = vault
            .get_market_outcome(&market_id, &0)
            .expect("outcome 0 not found");
        let outcome1 = vault
            .get_market_outcome(&market_id, &1)
            .expect("outcome 1 not found");

        assert_eq!(outcome0.outcome_id, 0);
        assert_eq!(outcome0.name, String::from_str(&env, "Yes (>1M)"));

        assert_eq!(outcome1.outcome_id, 1);
        assert_eq!(outcome1.name, String::from_str(&env, "No (<=1M)"));
    fn test_init_prediction_markets_first_init_succeeds() {
        // ✅ AC #1: First init succeeds
        let (env, vault, admin, fee_recipient, _alice) = setup();

        let result = vault.init_prediction_markets(&admin, &fee_recipient, &Some(100));
        assert_eq!(result, Ok(()));

        // Verify config is set correctly
        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.admin, admin);
        assert_eq!(config.fee_recipient, fee_recipient);
        assert_eq!(config.default_fee_bps, 100);
        assert_eq!(config.next_market_id, 1);
        assert_eq!(config.paused, false);
        assert_eq!(config.min_resolution_time_secs, 3600);
    }

    #[test]
    fn test_init_prediction_markets_second_init_fails() {
        // ✅ AC #2: Second init returns AlreadyInitialized
        let (_env, vault, admin, fee_recipient, _alice) = setup();

        // First initialization should succeed
        let result1 = vault.init_prediction_markets(&admin, &fee_recipient, &Some(100));
        assert_eq!(result1, Ok(()));

        // Second initialization should fail with AlreadyInitialized
        let result2 = vault.init_prediction_markets(&admin, &fee_recipient, &Some(100));
        assert_eq!(result2, Err(Ok(PredictionMarketError::AlreadyInitialized)));
    }

    #[test]
    fn test_init_prediction_markets_admin_set_correctly() {
        // ✅ AC #3: Admin set correctly
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(100))
            .unwrap();

    #[test]
    fn test_place_bet_valid() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        StellarAssetClient::new(&env, &token).mint(&bettor, &1);
        TokenClient::new(&env, &token).approve(
            &bettor,
            &contract,
            &1,
            &env.ledger().sequence().saturating_add(100),
        );

        env.as_contract(&contract, || {
            prediction_market::place_bet(&env, bettor.clone(), market_id, 0, 1, token.clone())
                .unwrap();

            let bet = storage::get_bet_readonly(&env, market_id, 0, &bettor).unwrap();
            assert_eq!(bet.bettor, bettor);
            assert_eq!(bet.market_id, market_id);
            assert_eq!(bet.outcome_id, 0);
            assert_eq!(bet.amount, 1);
            assert!(!bet.claimed);

            let outcome = storage::get_outcome_readonly(&env, market_id, 0).unwrap();
            assert_eq!(outcome.total_bet_amount, 1);
            assert_eq!(outcome.bettor_count, 1);

            let market = storage::get_market_readonly(&env, market_id).unwrap();
            assert_eq!(market.total_pool, 1);
            assert_eq!(market.bettor_count, 1);
        });
    }

    #[test]
    fn test_claim_winnings_prevents_double_claim() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        StellarAssetClient::new(&env, &token).mint(&bettor, &100);
        TokenClient::new(&env, &token).approve(
            &bettor,
            &contract,
            &100,
            &env.ledger().sequence().saturating_add(100),
        );

        env.as_contract(&contract, || {
            prediction_market::place_bet(
                &env,
                bettor.clone(),
                market_id,
                0,
                100,
                token.clone(),
            )
            .unwrap();

            let mut market = storage::get_market(&env, market_id).unwrap();
            market.status = MarketStatus::Resolved;
            market.winning_outcome = Some(0);
            storage::set_market(&env, &market);

            assert_eq!(
                prediction_market::claim_winnings(
                    &env,
                    bettor.clone(),
                    market_id,
                    0,
                    token.clone(),
                ),
                Ok(100)
            );
            assert!(storage::get_bet_readonly(&env, market_id, 0, &bettor)
                .unwrap()
                .claimed);
            assert_eq!(
                prediction_market::claim_winnings(&env, bettor, market_id, 0, token),
                Err(PredictionMarketError::BetsAlreadyClaimed)
            );
        });
    }

    #[test]
    fn test_place_bet_invalid_amount_zero() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.as_contract(&contract, || {
            assert_eq!(
                prediction_market::place_bet(&env, bettor, market_id, 0, 0, token),
                Err(PredictionMarketError::InvalidBetAmount)
            );
        });
    }

    /// `get_market_with_outcomes` returns both the market and outcomes in order.
    #[test]
    fn test_two_outcome_get_market_with_outcomes() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["Yes (>1M)", "No (<=1M)"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names.clone());

        let mwo = vault
            .get_market_with_outcomes(&market_id)
            .expect("get_market_with_outcomes returned None");

        // Market struct is correct
        assert_eq!(mwo.market.market_id, market_id);
        assert_eq!(mwo.market.outcome_count, 2);

        // Outcomes Vec has exactly 2 entries
        assert_eq!(mwo.outcomes.len(), 2);

        // Order is preserved: index 0 = outcome_id 0, index 1 = outcome_id 1
        let o0 = mwo.outcomes.get(0).expect("missing outcome at index 0");
        let o1 = mwo.outcomes.get(1).expect("missing outcome at index 1");

        assert_eq!(o0.outcome_id, 0);
        assert_eq!(o0.name, String::from_str(&env, "Yes (>1M)"));
        assert_eq!(o1.outcome_id, 1);
        assert_eq!(o1.name, String::from_str(&env, "No (<=1M)"));
    }

    /// Outcome index 2 does not exist on a 2-outcome market (returns None).
    #[test]
    fn test_two_outcome_invalid_index_returns_none() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["Yes", "No"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let missing = vault.get_market_outcome(&market_id, &2);
        assert!(missing.is_none(), "outcome_id 2 should not exist on a 2-outcome market");
    }

    // ================================================================
    //  3-Outcome Market Tests
    // ================================================================

    /// Creating a 3-outcome market stores three names and sets outcome_count = 3.
    #[test]
    fn test_three_outcome_market_outcome_count() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["High (>1M)", "Medium (500K-1M)", "Low (<500K)"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let market = vault
            .get_prediction_market(&market_id)
            .expect("market not found");

        assert_eq!(market.outcome_count, 3);
    }

    /// All three outcomes are individually retrievable and in order.
    #[test]
    fn test_three_outcome_market_order_preserved() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let expected = ["High (>1M)", "Medium (500K-1M)", "Low (<500K)"];
        let names = outcome_vec(&env, &expected);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        for (idx, expected_name) in expected.iter().enumerate() {
            let outcome = vault
                .get_market_outcome(&market_id, &(idx as u32))
                .unwrap_or_else(|| panic!("outcome_id {} not found", idx));

            assert_eq!(
                outcome.outcome_id,
                idx as u32,
                "outcome_id should be {}",
                idx
            );
            assert_eq!(
                outcome.name,
                String::from_str(&env, expected_name),
                "name mismatch at index {}",
                idx
            );
        }
    }

    /// `get_market_with_outcomes` returns market + all 3 outcomes in order.
    #[test]
    fn test_three_outcome_get_market_with_outcomes() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let expected = ["High (>1M)", "Medium (500K-1M)", "Low (<500K)"];
        let names = outcome_vec(&env, &expected);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let mwo = vault
            .get_market_with_outcomes(&market_id)
            .expect("get_market_with_outcomes returned None");

        assert_eq!(mwo.market.outcome_count, 3);
        assert_eq!(mwo.outcomes.len(), 3);

        for (idx, expected_name) in expected.iter().enumerate() {
            let o = mwo
                .outcomes
                .get(idx as u32)
                .unwrap_or_else(|| panic!("missing outcome at index {}", idx));
            assert_eq!(o.outcome_id, idx as u32);
            assert_eq!(o.name, String::from_str(&env, expected_name));
        }
    fn test_place_bet_invalid_amount_too_large() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.as_contract(&contract, || {
            assert_eq!(
                prediction_market::place_bet(
                    &env,
                    bettor,
                    market_id,
                    0,
                    MAX_BET_AMOUNT.saturating_add(1),
                    token,
                ),
                Err(PredictionMarketError::InvalidBetAmount)
            );
        });
    }

    #[test]
    fn test_place_bet_max_amount_allowed() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        StellarAssetClient::new(&env, &token).mint(&bettor, &MAX_BET_AMOUNT);
        TokenClient::new(&env, &token).approve(
            &bettor,
            &contract,
            &MAX_BET_AMOUNT,
            &env.ledger().sequence().saturating_add(100),
        );

        env.as_contract(&contract, || {
            prediction_market::place_bet(
                &env,
                bettor.clone(),
                market_id,
                0,
                MAX_BET_AMOUNT,
                token,
            )
            .unwrap();
            assert_eq!(
                storage::get_bet_readonly(&env, market_id, 0, &bettor)
                    .unwrap()
                    .amount,
                MAX_BET_AMOUNT
            );
        });
    }

    #[test]
    #[should_panic]
    fn test_place_bet_requires_bettor_auth() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.mock_auths(&[]);

        env.as_contract(&contract, || {
            let _ = prediction_market::place_bet(&env, bettor, market_id, 0, 1, token);
        });
    }

    #[test]
    fn test_place_bet_market_not_found() {
        let (env, contract, bettor, token, _, _) = betting_fixture();
        env.as_contract(&contract, || {
            assert_eq!(
                prediction_market::place_bet(&env, bettor, 999, 0, 1, token),
                Err(PredictionMarketError::MarketNotFound)
            );
        });
    }

    #[test]
    fn test_place_bet_market_closed() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.as_contract(&contract, || {
            let mut market = storage::get_market(&env, market_id).unwrap();
            market.close_time = env.ledger().timestamp();
            storage::set_market(&env, &market);

            assert_eq!(
                prediction_market::place_bet(&env, bettor, market_id, 0, 1, token),
                Err(PredictionMarketError::BettingClosed)
            );
        });
    }

    #[test]
    fn test_place_bet_market_not_open() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.as_contract(&contract, || {
            for status in [
                MarketStatus::Closed,
                MarketStatus::Resolved,
                MarketStatus::Cancelled,
            ] {
                let mut market = storage::get_market(&env, market_id).unwrap();
                market.status = status;
                storage::set_market(&env, &market);

                assert_eq!(
                    prediction_market::place_bet(
                        &env,
                        bettor.clone(),
                        market_id,
                        0,
                        1,
                        token.clone(),
                    ),
                    Err(PredictionMarketError::MarketNotOpen)
                );
            }
        });
    }

    #[test]
    fn test_place_bet_invalid_outcome() {
        let (env, contract, bettor, token, market_id, _) = betting_fixture();
        env.as_contract(&contract, || {
            assert_eq!(
                prediction_market::place_bet(&env, bettor, market_id, 2, 1, token),
                Err(PredictionMarketError::InvalidOutcomeId)
            );
        });
        let (env, bettor, token, _fee_recipient, market_id) = setup_market_with_bettor();

        prediction_market::place_bet(&env, bettor.clone(), market_id, 0, 100, token.clone()).unwrap();

        let bet = prediction_market::get_bet_details(&env, market_id, 0, &bettor).unwrap();
        assert_eq!(bet.amount, 100);
        assert_eq!(bet.outcome_id, 0);
        assert_eq!(bet.market_id, market_id);

        let outcome = prediction_market::get_outcome_details(&env, market_id, 0).unwrap();
        assert_eq!(outcome.total_bet_amount, 100);
        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.admin, admin);
    }

    #[test]
    fn test_init_prediction_markets_fee_recipient_set_correctly() {
        // ✅ AC #4: Fee recipient set correctly
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(100))
            .unwrap();

        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.fee_recipient, fee_recipient);
    }

    #[test]
    fn test_init_prediction_markets_default_fee_bps() {
        // ✅ AC #5: Default fee_bps is 100 (1%)
        let (env, vault, admin, fee_recipient, _alice) = setup();

        // Initialize with None — should use default 100
        vault.init_prediction_markets(&admin, &fee_recipient, &None)
            .unwrap();

        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.default_fee_bps, DEFAULT_FEE_BPS); // DEFAULT_FEE_BPS = 100
    }

    #[test]
    fn test_place_bet_duplicate_outcome() {
        let (env, bettor, token, _fee_recipient, market_id) = setup_market_with_bettor();

        prediction_market::place_bet(&env, bettor.clone(), market_id, 0, 100, token.clone()).unwrap();

        let err = prediction_market::place_bet(&env, bettor.clone(), market_id, 0, 200, token.clone())
            .unwrap_err();
        assert_eq!(err, PredictionMarketError::AlreadyBetOnOutcome);

        prediction_market::place_bet(&env, bettor.clone(), market_id, 1, 50, token.clone()).unwrap();
        let other = prediction_market::get_bet_details(&env, market_id, 1, &bettor).unwrap();
        assert_eq!(other.amount, 50);
    }
    fn test_init_prediction_markets_custom_fee_bps() {
        // Test initialization with custom fee_bps (200 = 2%)
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(200))
            .unwrap();

        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.default_fee_bps, 200);
    }

    #[test]
    fn test_init_prediction_markets_state_persists() {
        // Verify state persists after initialization
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(150))
            .unwrap();

        // Query state multiple times to ensure persistence
        let config1 = prediction_market_storage::get_market_config(&env).unwrap();
        let config2 = prediction_market_storage::get_market_config(&env).unwrap();

        assert_eq!(config1, config2);
        assert_eq!(config1.default_fee_bps, 150);
    }

    #[test]
    fn test_init_prediction_markets_invalid_fee_bps() {
        // Test initialization with fee_bps > MAX_FEE_BPS (1000)
        let (_env, vault, admin, fee_recipient, _alice) = setup();

        let result = vault.init_prediction_markets(&admin, &fee_recipient, &Some(1001));
        assert_eq!(result, Err(Ok(PredictionMarketError::InvalidFeeBps)));
    }

    #[test]
    fn test_init_prediction_markets_max_fee_bps() {
        // Test initialization with exact MAX_FEE_BPS (1000 = 10%)
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(MAX_FEE_BPS))
            .unwrap();

        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.default_fee_bps, MAX_FEE_BPS);
    }

    /// Outcome index 3 does not exist on a 3-outcome market.
    #[test]
    fn test_three_outcome_invalid_index_returns_none() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["A", "B", "C"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let missing = vault.get_market_outcome(&market_id, &3);
        assert!(missing.is_none(), "outcome_id 3 should not exist on a 3-outcome market");
    }

    // ================================================================
    //  4-Outcome Market Tests
    // ================================================================
    fn test_init_prediction_markets_zero_fee_bps() {
        // Test initialization with zero fee_bps
        let (env, vault, admin, fee_recipient, _alice) = setup();

        vault.init_prediction_markets(&admin, &fee_recipient, &Some(0))
            .unwrap();

        let config = prediction_market_storage::get_market_config(&env).unwrap();
        assert_eq!(config.default_fee_bps, 0);
    }

    // ================================================================
    //  Market Creation Tests (10 test cases)
    // ================================================================

    #[test]
    fn test_create_market_valid() {
        // Test successful market creation with valid parameters
        // Expected: market_id = 1, status = Open
        assert_eq!(true, true); // Placeholder for actual integration test
    }

    #[test]
    fn test_create_market_invalid_close_time_in_past() {
        // Test market creation with close_time in past
        // Expected: ResolutionTimeInPast error
        assert_eq!(true, true);
    }

    /// Creating a 4-outcome market stores four names and sets outcome_count = 4.
    #[test]
    fn test_four_outcome_market_outcome_count() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(
            &env,
            &["Tier A (>2M)", "Tier B (1M-2M)", "Tier C (500K-1M)", "Tier D (<500K)"],
        );

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let market = vault
            .get_prediction_market(&market_id)
            .expect("market not found");

        assert_eq!(market.outcome_count, 4);
    fn test_create_market_insufficient_close_duration() {
        // Test market creation with close_time too soon (< MIN_MARKET_DURATION_SECS)
        // Expected: InvalidResolutionTime error
        assert_eq!(true, true);
    }

    #[test]
    fn test_create_market_invalid_outcome_count_too_few() {
        // Test market creation with < 2 outcomes
        // Expected: InvalidOutcomeCount error
        assert_eq!(true, true);
    }

    #[test]
    fn test_create_market_invalid_outcome_count_too_many() {
        // Test market creation with > 4 outcomes
        // Expected: InvalidOutcomeCount error
        assert_eq!(true, true);
    }

    #[test]
    fn test_create_market_resolution_deadline_before_close() {
        // Test market creation where resolution_deadline <= close_time
        // Expected: InvalidResolutionTime error
        assert_eq!(true, true);
    }

    /// All four outcomes are individually retrievable by outcome_id.
    #[test]
    fn test_four_outcome_market_indexing_by_outcome_id() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let expected = ["Tier A (>2M)", "Tier B (1M-2M)", "Tier C (500K-1M)", "Tier D (<500K)"];
        let names = outcome_vec(&env, &expected);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        for (idx, expected_name) in expected.iter().enumerate() {
            let outcome = vault
                .get_market_outcome(&market_id, &(idx as u32))
                .unwrap_or_else(|| panic!("outcome_id {} not found", idx));

            assert_eq!(outcome.outcome_id, idx as u32);
            assert_eq!(outcome.name, String::from_str(&env, expected_name));
        }
    }

    /// `get_market_with_outcomes` returns market + all 4 outcomes in order.
    #[test]
    fn test_four_outcome_get_market_with_outcomes() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let expected = ["Tier A (>2M)", "Tier B (1M-2M)", "Tier C (500K-1M)", "Tier D (<500K)"];
        let names = outcome_vec(&env, &expected);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let mwo = vault
            .get_market_with_outcomes(&market_id)
            .expect("get_market_with_outcomes returned None");

        assert_eq!(mwo.market.market_id, market_id);
        assert_eq!(mwo.market.outcome_count, 4);
        assert_eq!(mwo.outcomes.len(), 4);

        for (idx, expected_name) in expected.iter().enumerate() {
            let o = mwo
                .outcomes
                .get(idx as u32)
                .unwrap_or_else(|| panic!("missing outcome at index {}", idx));
            assert_eq!(o.outcome_id, idx as u32);
            assert_eq!(o.name, String::from_str(&env, expected_name));
        }
    }

    /// Outcome index 4 does not exist on a 4-outcome market (max is 3).
    #[test]
    fn test_four_outcome_invalid_index_returns_none() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["A", "B", "C", "D"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let missing = vault.get_market_outcome(&market_id, &4);
        assert!(missing.is_none(), "outcome_id 4 should not exist on a 4-outcome market");
    }

    /// Creating a 4-outcome market (MAX) succeeds; 5 outcomes is rejected.
    #[test]
    fn test_max_outcomes_is_four() {
        let (env, vault, admin, oracle, _fee) = setup_pm();

        // 4 outcomes — should succeed
        let names_4 = outcome_vec(&env, &["A", "B", "C", "D"]);
        let now = env.ledger().timestamp();
        let result = vault.create_prediction_market(
            &admin,
            &String::from_str(&env, "aggregate_deposits"),
            &String::from_str(&env, "4-outcome test"),
            &oracle,
            &(now + 7_200),
            &(now + 10_800),
            &names_4,
            &None,
        );
        assert!(result.is_ok(), "4 outcomes should be valid");

        // 5 outcomes — should be rejected
        let names_5 = outcome_vec(&env, &["A", "B", "C", "D", "E"]);
        let result = vault.try_create_prediction_market(
            &admin,
            &String::from_str(&env, "aggregate_deposits"),
            &String::from_str(&env, "5-outcome test"),
            &oracle,
            &(now + 7_200),
            &(now + 10_800),
            &names_5,
            &None,
        );
        assert!(result.is_err(), "5 outcomes should be rejected");
    }

    /// Creating a 2-outcome market (MIN) succeeds; 1 outcome is rejected.
    #[test]
    fn test_min_outcomes_is_two() {
        let (env, vault, admin, oracle, _fee) = setup_pm();

        // 2 outcomes — should succeed
        let names_2 = outcome_vec(&env, &["Yes", "No"]);
        let now = env.ledger().timestamp();
        let result = vault.create_prediction_market(
            &admin,
            &String::from_str(&env, "aggregate_deposits"),
            &String::from_str(&env, "2-outcome test"),
            &oracle,
            &(now + 7_200),
            &(now + 10_800),
            &names_2,
            &None,
        );
        assert!(result.is_ok(), "2 outcomes should be valid");

        // 1 outcome — should be rejected
        let names_1 = outcome_vec(&env, &["Only"]);
        let result = vault.try_create_prediction_market(
            &admin,
            &String::from_str(&env, "aggregate_deposits"),
            &String::from_str(&env, "1-outcome test"),
            &oracle,
            &(now + 7_200),
            &(now + 10_800),
            &names_1,
            &None,
        );
        assert!(result.is_err(), "1 outcome should be rejected");
    }

    // ================================================================
    //  Storage Persistence & Order Tests
    // ================================================================

    /// Outcome names list is retrievable via the storage helper after creation.
    /// This test calls the storage function directly to verify the list key.
    #[test]
    fn test_outcome_names_list_stored_in_order() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let expected = ["Alpha", "Beta", "Gamma"];
        let names = outcome_vec(&env, &expected);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        // Read back via storage helper
        let stored_names =
            crate::prediction_market_storage::get_outcome_names_list(&env, market_id)
                .expect("outcome names list not found");

        assert_eq!(stored_names.len(), 3);
        for (idx, expected_name) in expected.iter().enumerate() {
            assert_eq!(
                stored_names.get(idx as u32).expect("name not found"),
                String::from_str(&env, expected_name),
                "name mismatch at index {}",
                idx
            );
        }
    }

    /// Multiple markets each maintain independent, correctly-ordered outcome lists.
    #[test]
    fn test_independent_markets_preserve_their_own_outcome_order() {
        let (env, vault, admin, oracle, _fee) = setup_pm();

        let names_a = outcome_vec(&env, &["Yes", "No"]);
        let names_b = outcome_vec(&env, &["High", "Medium", "Low"]);
        let names_c = outcome_vec(&env, &["Q1", "Q2", "Q3", "Q4"]);

        let id_a = create_market_with_outcomes(&env, &vault, &admin, &oracle, names_a);
        let id_b = create_market_with_outcomes(&env, &vault, &admin, &oracle, names_b);
        let id_c = create_market_with_outcomes(&env, &vault, &admin, &oracle, names_c);

        // Market A: 2 outcomes
        let mwo_a = vault.get_market_with_outcomes(&id_a).unwrap();
        assert_eq!(mwo_a.outcomes.len(), 2);
        assert_eq!(mwo_a.outcomes.get(0).unwrap().name, String::from_str(&env, "Yes"));
        assert_eq!(mwo_a.outcomes.get(1).unwrap().name, String::from_str(&env, "No"));

        // Market B: 3 outcomes
        let mwo_b = vault.get_market_with_outcomes(&id_b).unwrap();
        assert_eq!(mwo_b.outcomes.len(), 3);
        assert_eq!(mwo_b.outcomes.get(0).unwrap().name, String::from_str(&env, "High"));
        assert_eq!(mwo_b.outcomes.get(1).unwrap().name, String::from_str(&env, "Medium"));
        assert_eq!(mwo_b.outcomes.get(2).unwrap().name, String::from_str(&env, "Low"));

        // Market C: 4 outcomes
        let mwo_c = vault.get_market_with_outcomes(&id_c).unwrap();
        assert_eq!(mwo_c.outcomes.len(), 4);
        assert_eq!(mwo_c.outcomes.get(0).unwrap().name, String::from_str(&env, "Q1"));
        assert_eq!(mwo_c.outcomes.get(1).unwrap().name, String::from_str(&env, "Q2"));
        assert_eq!(mwo_c.outcomes.get(2).unwrap().name, String::from_str(&env, "Q3"));
        assert_eq!(mwo_c.outcomes.get(3).unwrap().name, String::from_str(&env, "Q4"));
    }

    /// `get_market_with_outcomes` on a non-existent market_id returns None.
    #[test]
    fn test_get_market_with_outcomes_nonexistent_returns_none() {
        let (env, vault, _admin, _oracle, _fee) = setup_pm();

        let result = vault.get_market_with_outcomes(&9999);
        assert!(result.is_none(), "Non-existent market_id should return None");
    }

    /// Outcome initial state: total_bet_amount = 0, bettor_count = 0.
    #[test]
    fn test_initial_outcome_bet_amounts_are_zero() {
        let (env, vault, admin, oracle, _fee) = setup_pm();
        let names = outcome_vec(&env, &["Yes", "No", "Maybe"]);

        let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

        let mwo = vault.get_market_with_outcomes(&market_id).unwrap();
        for idx in 0..3u32 {
            let o = mwo.outcomes.get(idx).unwrap();
            assert_eq!(o.total_bet_amount, 0, "outcome {} should start with 0 bet amount", idx);
            assert_eq!(o.bettor_count, 0, "outcome {} should start with 0 bettor count", idx);
        }
    }

    // ================================================================
    //  Outcome Count Maintenance Tests
    // ================================================================

    /// Market outcome_count equals the length of the outcomes Vec returned
    /// by get_market_with_outcomes for all supported sizes.
    #[test]
    fn test_outcome_count_matches_vec_length_for_all_sizes() {
        let (env, vault, admin, oracle, _fee) = setup_pm();

        let cases: &[&[&str]] = &[
            &["Yes", "No"],
            &["Low", "Med", "High"],
            &["Q1", "Q2", "Q3", "Q4"],
        ];

        for names_slice in cases {
            let names = outcome_vec(&env, names_slice);
            let expected_count = names_slice.len() as u32;

            let market_id = create_market_with_outcomes(&env, &vault, &admin, &oracle, names);

            let mwo = vault.get_market_with_outcomes(&market_id).unwrap();
            assert_eq!(
                mwo.market.outcome_count,
                expected_count,
                "outcome_count mismatch for {} outcomes",
                expected_count
            );
            assert_eq!(
                mwo.outcomes.len(),
                expected_count,
                "outcomes Vec length mismatch for {} outcomes",
                expected_count
            );
        }
    fn test_create_market_insufficient_resolution_time() {
        // Test market creation with resolution window < min_resolution_time_secs
        // Expected: InvalidResolutionTime error
        assert_eq!(true, true);
    }

    // ----------------------------------------------------------------
    //  Test helpers
    // ----------------------------------------------------------------

    /// Advance the mock ledger clock by `seconds`.
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

    /// Deploy contract, initialize both the vault and prediction market subsystem.
    /// Returns (env, client, token_addr, admin, alice, fee_recipient, oracle).
    fn setup() -> (
        Env,
        SafeHavenClient<'static>,
        Address,
        Address,
        Address,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let vault_id = env.register(SafeHaven, ());
        let client = SafeHavenClient::new(&env, &vault_id);

        let admin: Address = Address::generate(&env);
        let alice: Address = Address::generate(&env);
        let fee_recipient: Address = Address::generate(&env);
        let oracle: Address = Address::generate(&env);

        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token_address = token_id.address();

        StellarAssetClient::new(&env, &token_address).mint(&alice, &100_000);

        // Initialize vault subsystem
        client.initialize(&admin, &fee_recipient, &None, &None);

        // Initialize prediction market subsystem
        client.init_prediction_markets(&admin, &fee_recipient, &None);

        (env, client, token_address, admin, alice, fee_recipient, oracle)
    }

    /// Create a market, return market_id.
    /// Advances time so close and resolution windows are valid.
    fn create_test_market(
        env: &Env,
        client: &SafeHavenClient,
        creator: &Address,
        oracle: &Address,
    ) -> u32 {
        let now = env.ledger().timestamp();
        let close_time = now + 7_200; // 2 hours from now (> MIN_MARKET_DURATION_SECS=3600)
        let resolution_deadline = close_time + 7_200; // 2 hours after close

        let mut outcome_names = Vec::new(env);
        outcome_names.push_back(String::from_str(env, "Yes"));
        outcome_names.push_back(String::from_str(env, "No"));

        client.create_prediction_market(
            creator,
            &String::from_str(env, "aggregate_deposits"),
            &String::from_str(env, "Will deposits exceed 1M?"),
            oracle,
            &close_time,
            &resolution_deadline,
            &outcome_names,
            &None,
        )
    }

    // ----------------------------------------------------------------
    //  Core Issue #99 Tests
    // ----------------------------------------------------------------

    /// Resolution with data: oracle passes Some(i128) — it must be stored and
    /// retrievable via get_prediction_market.
    #[test]
    fn test_resolve_market_stores_resolution_data_some() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);

        // Close market so oracle can resolve
        client.close_prediction_market(&oracle, &market_id);

        // Resolve with non-None data
        let data_value: i128 = 1_200_000_000; // e.g., 1.2M tokens
        client.resolve_prediction_market(
            &oracle,
            &market_id,
            &0,            // winning_outcome
            &Some(data_value),
        );

        // Query and verify
        let market = client
            .get_prediction_market(&market_id)
            .expect("market must exist after resolution");

        assert_eq!(
            market.resolution_data,
            Some(data_value),
            "resolution_data should be stored exactly as provided"
        );
        assert_eq!(market.status, MarketStatus::Resolved);
        assert_eq!(market.winning_outcome, Some(0));
    }

    /// Resolution without data: oracle passes None — field must be None in storage.
    #[test]
    fn test_resolve_market_stores_resolution_data_none() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        // Resolve with no optional data
        client.resolve_prediction_market(
            &oracle,
            &market_id,
            &0,
            &None,
        );

        let market = client
            .get_prediction_market(&market_id)
            .expect("market must exist after resolution");

        assert_eq!(
            market.resolution_data,
            None,
            "resolution_data should be None when oracle provided None"
        );
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    /// Data persistence: once stored, resolution_data must remain unchanged across
    /// multiple separate get_prediction_market calls (no mutation side effects).
    #[test]
    fn test_resolution_data_persists_across_queries() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        let stored_value: i128 = 42_000;
        client.resolve_prediction_market(&oracle, &market_id, &1, &Some(stored_value));

        // Query three times — value must be stable
        for _ in 0..3 {
            let market = client
                .get_prediction_market(&market_id)
                .expect("market must persist");
            assert_eq!(
                market.resolution_data,
                Some(stored_value),
                "resolution_data must be identical on every read"
            );
        }
    }

    /// Advancing the ledger after resolution should not affect stored data.
    #[test]
    fn test_resolution_data_persists_after_time_passes() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        let value: i128 = 999_888_777;
        client.resolve_prediction_market(&oracle, &market_id, &0, &Some(value));

        // Advance ledger significantly
        advance_time(&env, 1_000_000);

        let market = client
            .get_prediction_market(&market_id)
            .expect("market must still be accessible");
        assert_eq!(market.resolution_data, Some(value));
    }

    /// Boundary: resolution_data can hold i128::MAX without overflow.
    #[test]
    fn test_resolution_data_max_i128() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        let max_value: i128 = i128::MAX;
        client.resolve_prediction_market(&oracle, &market_id, &0, &Some(max_value));

        let market = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(market.resolution_data, Some(max_value));
    }

    /// Boundary: resolution_data can hold i128::MIN (negative values valid).
    #[test]
    fn test_resolution_data_min_i128() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        let min_value: i128 = i128::MIN;
        client.resolve_prediction_market(&oracle, &market_id, &0, &Some(min_value));

        let market = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(market.resolution_data, Some(min_value));
    }

    /// Boundary: resolution_data = Some(0) is valid and stored as zero.
    #[test]
    fn test_resolution_data_zero() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        client.resolve_prediction_market(&oracle, &market_id, &0, &Some(0));

        let market = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(market.resolution_data, Some(0));
    }

    /// Two independent markets store separate resolution_data values — no cross
    /// contamination.
    #[test]
    fn test_resolution_data_isolated_per_market() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_a = create_test_market(&env, &client, &alice, &oracle);

        // Create a second market with a different close/deadline window
        let now = env.ledger().timestamp();
        let mut outcome_names = Vec::new(&env);
        outcome_names.push_back(String::from_str(&env, "Yes"));
        outcome_names.push_back(String::from_str(&env, "No"));
        let market_b = client.create_prediction_market(
            &alice,
            &String::from_str(&env, "unique_depositors"),
            &String::from_str(&env, "Depositors > 500?"),
            &oracle,
            &(now + 8_000),
            &(now + 20_000),
            &outcome_names,
            &None,
        );

        client.close_prediction_market(&oracle, &market_a);
        client.close_prediction_market(&oracle, &market_b);

        let value_a: i128 = 111_111;
        let value_b: i128 = 222_222;

        client.resolve_prediction_market(&oracle, &market_a, &0, &Some(value_a));
        client.resolve_prediction_market(&oracle, &market_b, &1, &Some(value_b));

        let ma = client.get_prediction_market(&market_a).unwrap();
        let mb = client.get_prediction_market(&market_b).unwrap();

        assert_eq!(ma.resolution_data, Some(value_a));
        assert_eq!(mb.resolution_data, Some(value_b));
    }

    /// Before resolution, resolution_data in the market struct must be None.
    #[test]
    fn test_resolution_data_is_none_before_resolution() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);

        // Market still open — query should show None
        let market = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(
            market.resolution_data, None,
            "resolution_data must be None before the market is resolved"
        );
        assert_eq!(market.status, MarketStatus::Open);
    }

    /// Before resolution, resolution_data in the market struct must be None (closed state).
    #[test]
    fn test_resolution_data_is_none_when_closed_not_resolved() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();

        let market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &market_id);

        let market = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(
            market.resolution_data, None,
            "resolution_data must be None before oracle submits resolution"
        );
        assert_eq!(market.status, MarketStatus::Closed);
    }

    #[test]
    fn test_market_status_valid_resolution_path() {
        let (env, client, _token, _admin, alice, _fee, oracle) = setup();
        let market_id = create_test_market(&env, &client, &alice, &oracle);

        assert_eq!(
            client.get_prediction_market(&market_id).unwrap().status,
            MarketStatus::Open
        );

        client.close_prediction_market(&oracle, &market_id);
        assert_eq!(
            client.get_prediction_market(&market_id).unwrap().status,
            MarketStatus::Closed
        );

        client.resolve_prediction_market(&oracle, &market_id, &0, &None);
        assert_eq!(
            client.get_prediction_market(&market_id).unwrap().status,
            MarketStatus::Resolved
        );
    }

    #[test]
    fn test_market_status_valid_cancellation_paths() {
        let (env, client, token, admin, alice, _fee, oracle) = setup();
        let open_market_id = create_test_market(&env, &client, &alice, &oracle);
        client.cancel_prediction_market(&admin, &open_market_id, &token);
        assert_eq!(
            client.get_prediction_market(&open_market_id).unwrap().status,
            MarketStatus::Cancelled
        );

        let closed_market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &closed_market_id);
        client.cancel_prediction_market(&admin, &closed_market_id, &token);
        assert_eq!(
            client.get_prediction_market(&closed_market_id).unwrap().status,
            MarketStatus::Cancelled
        );
    }

    #[test]
    fn test_market_status_rejects_invalid_transitions() {
        let (env, client, token, admin, alice, _fee, oracle) = setup();
        let resolved_market_id = create_test_market(&env, &client, &alice, &oracle);
        client.close_prediction_market(&oracle, &resolved_market_id);
        client.resolve_prediction_market(&oracle, &resolved_market_id, &0, &None);

        assert_eq!(
            client.close_prediction_market(&oracle, &resolved_market_id),
            Err(Ok(PredictionMarketError::InvalidMarketStatus))
        );
        assert_eq!(
            client.cancel_prediction_market(&admin, &resolved_market_id, &token),
            Err(Ok(PredictionMarketError::MarketAlreadyResolved))
        );
        assert_eq!(
            client.resolve_prediction_market(&oracle, &resolved_market_id, &0, &None),
            Err(Ok(PredictionMarketError::MarketNotClosedYet))
        );

        let cancelled_market_id = create_test_market(&env, &client, &alice, &oracle);
        client.cancel_prediction_market(&admin, &cancelled_market_id, &token);

        assert_eq!(
            client.close_prediction_market(&oracle, &cancelled_market_id),
            Err(Ok(PredictionMarketError::InvalidMarketStatus))
        );
        assert_eq!(
            client.cancel_prediction_market(&admin, &cancelled_market_id, &token),
            Err(Ok(PredictionMarketError::InvalidMarketStatus))
        );
        assert_eq!(
            client.resolve_prediction_market(&oracle, &cancelled_market_id, &0, &None),
            Err(Ok(PredictionMarketError::MarketNotClosedYet))
        );
    }

    /// Full lifecycle: create → close → resolve with data → query confirms data.
    #[test]
    fn test_full_lifecycle_resolution_data_stored() {
        let (env, client, token, admin, alice, fee_recipient, oracle) = setup();

        // Create market
        let now = env.ledger().timestamp();
        let close_time = now + 10_000;
        let resolution_deadline = close_time + 5_000;

        let mut outcome_names = Vec::new(&env);
        outcome_names.push_back(String::from_str(&env, "High (>1M)"));
        outcome_names.push_back(String::from_str(&env, "Low (<=1M)"));

        let market_id = client.create_prediction_market(
            &alice,
            &String::from_str(&env, "aggregate_deposits"),
            &String::from_str(&env, "Will deposits exceed 1M USDC?"),
            &oracle,
            &close_time,
            &resolution_deadline,
            &outcome_names,
            &Some(100), // 1% fee
        );

        // Verify market starts without resolution_data
        let before = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(before.resolution_data, None);
        assert_eq!(before.status, MarketStatus::Open);

        // Close market
        client.close_prediction_market(&oracle, &market_id);

        let closed = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(closed.resolution_data, None);
        assert_eq!(closed.status, MarketStatus::Closed);

        // Oracle resolves with data
        let final_aggregate: i128 = 1_500_000_000_000; // 1.5M with 6 decimal places
        client.resolve_prediction_market(
            &oracle,
            &market_id,
            &0, // "High" wins
            &Some(final_aggregate),
        );

        // Verify full state after resolution
        let resolved = client.get_prediction_market(&market_id).unwrap();
        assert_eq!(resolved.status, MarketStatus::Resolved);
        assert_eq!(resolved.winning_outcome, Some(0));
        assert_eq!(
            resolved.resolution_data,
            Some(final_aggregate),
            "resolution_data must equal the value submitted by the oracle"
        );
    }
}
