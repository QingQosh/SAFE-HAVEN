use soroban_sdk::{Address, Env};

use crate::types::{Deposit, DataKey};

/// TTL bump threshold (in ledgers). When an entry's remaining TTL drops below
/// this value, the next write will extend it back up to `BUMP_TARGET`.
///
/// ~30 days at 5s/ledger (30 * 24 * 60 * 12 = 518_400 ledgers). This buffer
/// ensures entries are renewed well before they can expire, even if writes
/// are infrequent.
pub const BUMP_THRESHOLD: u32 = 518_400;

/// TTL bump target (in ledgers). Every storage write extends the entry's TTL
/// to at least this value.
///
/// ~5 years at 5s/ledger (5 * 365 * 24 * 60 * 12 = 31_536_000 ledgers),
/// plus a ~1 year buffer to cover the maximum lock duration and any drift
/// between write cadence and expiry.
pub const BUMP_TARGET: u32 = 37_843_200;

/// Persist a deposit and extend its TTL so it survives the full lock duration.
pub fn set_deposit(env: &Env, deposit: &Deposit) {
    let key = DataKey::Deposit(deposit.id);
    env.storage().persistent().set(&key, deposit);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Load a deposit, renewing its TTL on read so active entries never expire
/// prematurely.
pub fn get_deposit(env: &Env, id: u64) -> Option<Deposit> {
    let key = DataKey::Deposit(id);
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    env.storage().persistent().get(&key)
}

/// Remove a deposit entry.
pub fn remove_deposit(env: &Env, id: u64) {
    let key = DataKey::Deposit(id);
    env.storage().persistent().remove(&key);
}

/// Persist the vault address and extend its TTL.
pub fn set_vault(env: &Env, vault: &Address) {
    let key = DataKey::Vault;
    env.storage().persistent().set(&key, vault);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Load the vault address, renewing its TTL on read.
pub fn get_vault(env: &Env) -> Option<Address> {
    let key = DataKey::Vault;
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    env.storage().persistent().get(&key)
}
