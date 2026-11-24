//! Deposit contract storage with TTL management.
//!
//! ## TTL configuration
//!
//! Soroban storage entries expire unless their time-to-live (TTL) is extended.
//! Every deposit must persist for the full lock duration, so each storage write
//! extends the entry TTL using [`extend_ttl`].
//!
//! - `BUMP_THRESHOLD`: minimum remaining TTL (in ledgers) before an entry is
//!   eligible for extension. Chosen so entries are renewed well before expiry.
//! - `BUMP_TARGET`: TTL (in ledgers) applied on extension. Set to cover the
//!   maximum lock duration (~5 years) plus a safety buffer.
//!
//! Ledgers close roughly every 5 seconds, so ~5 years is about
//! `5 * 365 * 24 * 60 * 60 / 5 = 31_536_000` ledgers. `BUMP_TARGET` adds a
//! buffer on top of that to guarantee entries outlive the longest lock.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Minimum remaining TTL (in ledgers) before an entry is extended.
/// ~30 days of ledgers, giving ample buffer before expiry.
const BUMP_THRESHOLD: u32 = 518_400;

/// TTL (in ledgers) applied when extending. Covers the maximum lock duration
/// (~5 years) plus a safety buffer.
const BUMP_TARGET: u32 = 33_000_000;

#[contracttype]
#[derive(Clone)]
pub struct Deposit {
    pub owner: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
}

fn deposit_key(owner: &Address) -> DepositKey {
    DepositKey::Deposit(owner.clone())
}

#[contracttype]
#[derive(Clone)]
pub enum DepositKey {
    Deposit(Address),
    /// Monotonic counter used to hand out deposit IDs.
    DepositCounter,
}

/// Persist a deposit and extend its TTL so it survives the full lock duration.
fn set_deposit(env: &Env, owner: &Address, deposit: &Deposit) {
    let key = deposit_key(owner);
    env.storage().persistent().set(&key, deposit);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

fn get_deposit(env: &Env, owner: &Address) -> Option<Deposit> {
    let key = deposit_key(owner);
    let deposit: Option<Deposit> = env.storage().persistent().get(&key);
    if deposit.is_some() {
        // Renew TTL on read so active deposits never expire prematurely.
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    deposit
}

/// Read the current deposit ID counter (0 if none issued yet).
fn get_deposit_counter(env: &Env) -> u64 {
    let key = DepositKey::DepositCounter;
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Increment and durably persist the deposit ID counter, returning the new ID.
fn next_deposit_id(env: &Env) -> u64 {
    let key = DepositKey::DepositCounter;
    let id = get_deposit_counter(env) + 1;
    env.storage().persistent().set(&key, &id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    id
}

#[contract]
pub struct DepositContract;

#[contractimpl]
impl DepositContract {
    /// Create a deposit, extending TTL on write.
    pub fn deposit(env: Env, owner: Address, amount: i128, unlock_ledger: u32) {
        owner.require_auth();
        let deposit = Deposit {
            owner: owner.clone(),
            amount,
            unlock_ledger,
        };
        set_deposit(&env, &owner, &deposit);
    }

    /// Create a deposit locked until `unlock_ledger`, returning its deposit ID.
    ///
    /// The deposit is stored durably with TTL extension so it survives the
    /// full lock duration. IDs are drawn from the shared deposit counter.
    pub fn deposit_by_ledger(
        env: Env,
        owner: Address,
        amount: i128,
        unlock_ledger: u32,
    ) -> u64 {
        owner.require_auth();
        let deposit = Deposit {
            owner: owner.clone(),
            amount,
            unlock_ledger,
        };
        set_deposit(&env, &owner, &deposit);
        next_deposit_id(&env)
    }

    /// Fetch a deposit, renewing its TTL if present.
    pub fn get_vault(env: Env, owner: Address) -> Option<Deposit> {
        get_deposit(&env, &owner)
    }

    /// Fetch a ledger-based deposit, returning the stored `unlock_ledger`.
    pub fn get_vault_by_ledger(env: Env, owner: Address) -> Option<Deposit> {
        get_deposit(&env, &owner)
    }
}

// NOTE (issue #831): This contract stores only deposit records and holds no
// sustainability metrics or aggregate state. Metrics are owned by the
// safe-haven contract, which creates them on deposit and removes them on
// withdraw/cancel. Because nothing here caches or mirrors those metrics, the
// deposit contract cannot retain stale sustainability state and requires no
// lifecycle hooks for metric cleanup.

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn deposit_extends_ttl() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DepositContract);
        let client = DepositContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);

        client.deposit(&owner, &1_000, &(env.ledger().sequence() + 100));

        let key = deposit_key(&owner);
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= BUMP_TARGET, "deposit TTL must cover lock duration");
    }

    #[test]
    fn bump_target_covers_max_lock() {
        // ~5 years of ledgers at 5s close time.
        const FIVE_YEARS_LEDGERS: u32 = 31_536_000;
        assert!(BUMP_TARGET > FIVE_YEARS_LEDGERS);
        assert!(BUMP_THRESHOLD < BUMP_TARGET);
    }

    #[test]
    fn deposit_by_ledger_stores_ledger_and_returns_id() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DepositContract);
        let client = DepositContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        let unlock_ledger = env.ledger().sequence() + 500;

        let id = client.deposit_by_ledger(&owner, &1_000, &unlock_ledger);
        assert_eq!(id, 1, "first deposit ID must be 1");

        let stored = client
            .get_vault_by_ledger(&owner)
            .expect("ledger-based deposit must exist");
        assert_eq!(stored.unlock_ledger, unlock_ledger);
        assert_eq!(stored.amount, 1_000);
        assert_eq!(stored.owner, owner);

        let key = deposit_key(&owner);
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= BUMP_TARGET, "deposit TTL must cover lock duration");
    }

    #[test]
    fn deposit_by_ledger_ids_increment() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DepositContract);
        let client = DepositContractClient::new(&env, &contract_id);
        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);
        let unlock_ledger = env.ledger().sequence() + 100;

        assert_eq!(client.deposit_by_ledger(&owner_a, &10, &unlock_ledger), 1);
        assert_eq!(client.deposit_by_ledger(&owner_b, &20, &unlock_ledger), 2);
    }
}
