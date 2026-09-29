use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Maximum number of tokens allowed in a single multi-token deposit (issue #330).
pub const MAX_TOKENS_PER_DEPOSIT: u32 = 5;
/// Emergency withdrawals at or above this cumulative amount in one ledger trip the circuit breaker.
pub const MAX_EMERGENCY_WITHDRAWAL_PER_LEDGER: i128 = 100_000_000;

/// Current storage schema version. Bump this constant when the on-chain
/// layout of a `contracttype` struct changes so `migrate()` can detect
/// and upgrade stale entries.
pub const STORAGE_VERSION: u32 = 1;

/// Fraction of the penalty fee reserved for the insurance pool (5 = 5%).
pub const INSURANCE_POOL_BPS: u32 = 500; // 5% in basis points

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositRequest {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub penalty_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultKey {
    Deposit(Address, u32),
    DepositByLedger(Address, u32),
    /// Multi-token deposit entry (issue #330).
    MultiDeposit(Address, u32),
    /// Withdrawal whitelist for a deposit (issue #331).
    WithdrawalWhitelist(Address, u32),
    DepositCounter(Address),
    /// Stores a `Vec<u32>` of active deposit IDs for a depositor (both timestamp- and
    /// ledger-based). Maintained alongside the counter so `get_deposit_ids` is O(1).
    ActiveDepositIds(Address),
    Admin,
    PendingAdmin,
    Initialized,
    DepositorList,
    /// Boolean existence flag per depositor — O(1) duplicate check in `add_depositor`.
    DepositorFlag(Address),
    /// Set-once flag recording that an address was appended to `DepositorList`.
    /// Never deleted, so re-deposits don't create duplicate list entries even
    /// after the corresponding `DepositorFlag` has been cleared by `remove_depositor`.
    DepositorInList(Address),
    FeeRecipient,
    MaxDeposit,
    MaxLockSecs,
    Paused,
    /// Boolean membership flag for the token allowlist.
    AllowedToken(Address),
    /// When true, deposits may use only tokens in the allowlist.
    StrictTokenAllowlist,
    /// Stores the token vetting workflow state.
    TokenVetting(Address),
    ProposalCounter,
    GovernanceProposal(u32),
    GovernanceVote(u32, Address),
    NextUpgradeId,
    UpgradeProposal(u32),
    UpgradeVote(u32, Address),
    UpgradeVeto(u32, Address),
    /// Persists the schema version written by the last `migrate()` call (or 1
    /// for contracts that were initialized before versioning was introduced).
    StorageVersion,
    /// Guards flash-loan execution against re-entrant nested calls.
    FlashLoanGuard,
    /// Active borrower state for a single-token flash loan.
    FlashLoanState(Address, Address),
    /// Fee share owed to a depositor for a token after flash-loan repayment.
    FlashLoanFeeBalance(Address, Address),
    /// Staker entry: maps staker address to their stake amount
    Staker(Address),
    /// List of all registered stakers
    StakerList,
    /// Flag to track if a staker is in the StakerList (prevents duplicates)
    StakerInList(Address),
    /// Total amount staked by all stakers
    TotalStaked,
    /// Rewards pool for stakers (accumulated from penalties)
    RewardsPool,
    /// Rewards claimed by a staker (track cumulative for auditing)
    StakerRewardsClaimed(Address),
    /// NFT evolution record: maps (depositor, deposit_id) to NFTEvolutionRecord
    NFTEvolution(Address, u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    /// Compound interest accrual frequency in seconds (0 = no compounding). (issue #332)
    pub compound_frequency_secs: u64,
    /// Timestamp of last compound accrual (issue #332).
    pub last_accrual_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaxLossHarvest {
    pub depositor: Address,
    pub original_token: Address,
    pub replacement_token: Address,
    pub original_deposit_id: u32,
    pub replacement_deposit_id: u32,
    pub cost_basis: i128,
    pub current_value: i128,
    pub realized_loss: i128,
    pub tax_benefit: i128,
    pub harvested_at: u64,
    pub wash_sale_until: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub depositor: Address,
    pub penalty_bps: u32,
}

/// A single token+amount pair used in multi-token deposits (issue #330).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDeposit {
    pub token: Address,
    pub amount: i128,
}

/// Vault entry that holds multiple token deposits (issue #330).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiTokenVaultEntry {
    /// Each element is a (token, amount) pair. Length ≤ MAX_TOKENS_PER_DEPOSIT.
    pub tokens: Vec<TokenDeposit>,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    /// Compound interest accrual frequency in seconds (0 = no compounding). (issue #332)
    pub compound_frequency_secs: u64,
    /// Timestamp of last compound accrual (issue #332).
    pub last_accrual_timestamp: u64,
}

/// The deposit type discriminant returned by `get_deposit_type`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositType {
    /// Timestamp-based (`VaultEntry`) single-token deposit.
    TimeBased,
    /// Ledger-sequence-based (`LedgerVaultEntry`) deposit.
    LedgerBased,
    /// Multi-token timestamp-based deposit (`MultiTokenVaultEntry`). (issue #330)
    MultiToken,
}

/// Paginated query result for depositor addresses.
/// (Soroban `#[contracttype]` does not support generics.)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    /// The items in this page
    pub items: soroban_sdk::Vec<Address>,
    /// Total number of active items across all pages
    pub total_count: u32,
}

/// Staker entry: tracks stake amount and optionally last claim timestamp
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakerEntry {
    pub staker: Address,
    pub stake_amount: i128,
}

/// Sponsorship fund configuration and state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SponsorshipFund {
    /// Balance of native tokens available for sponsorship
    pub balance: i128,

    /// Address that manages the sponsorship fund (typically admin)
    pub sponsor_address: Address,

    /// Maximum tokens to sponsor per transaction
    pub max_per_txn: i128,

    /// Maximum tokens to sponsor per user per day
    pub max_per_user_day: i128,

    /// Minimum native balance required to be eligible for sponsorship (KYC-lite)
    pub min_eligible_balance: i128,

    /// Cooldown period (in seconds) between sponsored transactions per user
    pub cooldown_seconds: u64,

    /// Timestamp of last update (for tracking replenishment frequency)
    pub last_replenished: u64,
}

/// Per-user sponsorship tracking for a specific day
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SponsorshipUsage {
    /// Cumulative amount sponsored to this user today
    pub amount_used_today: i128,

    /// Timestamp of the last sponsored transaction for this user
    pub last_sponsored_time: u64,

    /// Counter of sponsored transactions for this user (for sybil detection)
    pub transaction_count: u32,
}

/// Result of sponsorship eligibility check
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SponsorshipEligibility {
    /// User is eligible if they meet all criteria
    pub is_eligible: bool,

    /// Reason if not eligible (empty string if eligible)
    pub reason: soroban_sdk::String,

    /// Amount available for this user today
    pub available_today: i128,
}

/// Lockdown history entry to track emergency lockdowns
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockdownEntry {
    pub activated_at: u64,
    pub deactivated_at: Option<u64>,
    pub admin: Address,
    pub reason: String,
    pub duration_secs: Option<u64>,
}

// ----------------------------------------------------------------
//  ACL — Access Control List
// ----------------------------------------------------------------

/// Granular permission types for the SAFE-HAVEN Access Control List.
///
/// Each variant maps to a single bit in a `u32` bitmask stored per address.
/// The admin always bypasses ACL checks — these permissions apply to
/// non-admin callers only.
///
/// # Bit layout
/// | Permission        | Bit position | Mask value |
/// |---|---|---|
/// | `ViewVault`       | 0            | 0x0001 |
/// | `Deposit`         | 1            | 0x0002 |
/// | `Withdraw`        | 2            | 0x0004 |
/// | `CancelDeposit`   | 3            | 0x0008 |
/// | `Manage`          | 4            | 0x0010 |
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PermissionType {
    /// Allows the address to query vault entries via `get_vault` / `get_vault_batch`.
    ViewVault = 0,
    /// Allows the address to create deposits on behalf of themselves (`deposit`, `deposit_for`,
    /// `deposit_by_ledger`, `multi_deposit`).
    Deposit = 1,
    /// Allows the address to call `withdraw` and `withdraw_to` for their own deposits.
    Withdraw = 2,
    /// Allows the address to call `cancel_deposit` for their own deposits.
    CancelDeposit = 3,
    /// Allows the address to call admin-adjacent operations such as `emergency_withdraw`,
    /// `pause`, and `unpause`.  Typically granted only to trusted operator addresses.
    Manage = 4,
}

impl PermissionType {
    /// Returns the bit mask for this permission.
    pub fn mask(self) -> u32 {
        1u32 << (self as u32)
    }
}

// ----------------------------------------------------------------
//  Batch Withdrawal
// ----------------------------------------------------------------

/// Result of withdrawing from a single deposit in a batch operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalResult {
    /// The deposit ID that was attempted to withdraw from.
    pub deposit_id: u32,
    /// Whether this withdrawal succeeded (true) or failed (false).
    pub success: bool,
    /// Error code if failed (0 = success, non-zero = VaultError code).
    pub error_code: u32,
    /// Amount withdrawn if successful, 0 otherwise.
    pub amount: i128,
}

/// Aggregated result of a batch withdrawal operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchWithdrawalResult {
    /// Individual results for each deposit ID in the batch.
    pub results: Vec<WithdrawalResult>,
    /// Total number of deposits attempted.
    pub total_attempted: u32,
    /// Number of successful withdrawals.
    pub successful_count: u32,
    /// Number of failed withdrawals.
    pub failed_count: u32,
    /// Total amount withdrawn across all successful withdrawals.
    pub total_amount: i128,
}
