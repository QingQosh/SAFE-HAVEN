use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposit {
    pub owner: Address,
    pub amount: i128,
    pub unlock_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultError {
    FundsStillLocked,
    NoDeposit,
    InvalidAmount,
}

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    /// Deposit funds and lock them until `unlock_time` (ledger timestamp, seconds).
    pub fn deposit(env: Env, owner: Address, amount: i128, unlock_time: u64) -> Result<(), VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        owner.require_auth();
        let deposit = Deposit {
            owner: owner.clone(),
            amount,
            unlock_time,
        };
        env.storage().persistent().set(&owner, &deposit);
        Ok(())
    }

    /// Withdraw funds once the lock time has expired.
    ///
    /// Timestamp semantics: `unlock_time` is compared against
    /// `env.ledger().timestamp()` (seconds, ledger precision). The boundary is
    /// inclusive: withdrawal succeeds when `current_time >= unlock_time`, so a
    /// withdrawal exactly at `unlock_time` is allowed. Any earlier timestamp
    /// returns `FundsStillLocked` and leaves the deposit state untouched.
    pub fn withdraw(env: Env, owner: Address) -> Result<(), VaultError> {
        owner.require_auth();

        let deposit: Deposit = env
            .storage()
            .persistent()
            .get(&owner)
            .ok_or(VaultError::NoDeposit)?;

        // Critical security check: reject early withdrawals. Inclusive boundary
        // so withdrawal exactly at unlock_time succeeds.
        let current_time = env.ledger().timestamp();
        if current_time < deposit.unlock_time {
            return Err(VaultError::FundsStillLocked);
        }

        // Validation passed: clear state before transferring tokens.
        env.storage().persistent().remove(&owner);

        let token_client = token::Client::new(&env, &deposit.owner);
        token_client.transfer(&env.current_contract_address(), &owner, &deposit.amount);

        Ok(())
    }
}
