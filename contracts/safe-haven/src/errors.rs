use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    InvalidAmount = 1,
    UnlockTimeNotInFuture = 2,
    NoDepositFound = 3,
    FundsStillLocked = 4,
    DepositAlreadyExists = 5,
    LockDurationTooLong = 6,
    Unauthorized = 7,
    AmountTooLarge = 8,
    InvalidPenaltyBps = 9,
    InvalidAdmin = 10,
    LockDurationTooShort = 11,
    ContractPaused = 12,
    VaultAlreadyUnlocked = 13,
    MissingFeeRecipient = 14,
    /// `initialize` was called on an already-initialized contract.
    /// The `is_initialized` flag is the sole re-initialization guard (closes #46).
    AlreadyInitialized = 15,
    /// Too many tokens in a multi-token deposit — exceeds MAX_TOKENS_PER_DEPOSIT (issue #330).
    TooManyTokens = 16,
    /// A multi-token deposit must contain at least one token (issue #330).
    EmptyTokenList = 17,
    /// Recipient is not on the withdrawal whitelist (issue #331).
    RecipientNotWhitelisted = 18,
    /// Compound frequency must be >= 60 seconds if non-zero (issue #332).
    InvalidCompoundFrequency = 19,
    /// MEV attack detected: price deviation > 200 bps from TWAP
    MevDetected = 20,
    /// Price sampling too frequent: must wait >= 1 hour between samples
    SamplingTooFrequent = 21,
    /// Session key not found
    SessionKeyNotFound = 22,
    /// Session key has been revoked
    SessionKeyRevoked = 23,
    /// Session key has expired
    SessionKeyExpired = 24,
    /// Session key exceeds maximum deposit amount
    SessionKeyInsufficientLimit = 25,
    /// Emergency withdrawal would exceed the per-ledger hard cap.
    EmergencyWithdrawalLimitExceeded = 26,
    /// Sponsorship fund not initialized
    SponsorshipNotInitialized = 27,
    /// User not eligible for sponsorship
    NotEligibleForSponsorship = 28,
    /// Sponsorship fund insufficient
    InsufficientSponsorshipFund = 29,
    /// User exceeded daily sponsorship limit
    SponsorshipDailyLimitExceeded = 30,
    /// User transaction cooldown still active
    SponsorshipCooldownActive = 31,
    /// Potential sybil attack detected
    PotentialSybilAttack = 32,
    /// Sponsorship configuration error
    InvalidSponsorshipConfig = 33,
    /// Batch size exceeds MAX_BATCH_SIZE
    BatchSizeExceeded = 34,
    /// Renewable energy percentage must be 0-100
    InvalidRenewableEnergyPercent = 35,
    /// Recipient address is invalid (zero address)
    InvalidRecipient = 28,
    /// Session key expiry must be in the future.
    InvalidSessionKeyExpiry = 29,
    /// Session key is not authorized for this wallet.
    SessionKeyNotAuthorized = 30,
    InvalidRecipient = 36,
    /// Carbon footprint arithmetic would overflow i128 (issue #77).
    /// Returned by the checked carbon calculation helpers.
    CarbonOverflow = 37,
}
