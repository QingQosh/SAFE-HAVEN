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

    /// Fetch a deposit, renewing its TTL if present.
    pub fn get_vault(env: Env, owner: Address) -> Option<Deposit> {
        get_deposit(&env, &owner)
    }
}

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
}
