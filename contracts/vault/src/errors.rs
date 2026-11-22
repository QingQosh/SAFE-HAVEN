use soroban_sdk::contracterror;

/// Maximum allowed deposit amount: 10^15 units.
///
/// Deposits above this limit are rejected with [`VaultError::AmountTooLarge`]
/// to prevent arithmetic overflow in downstream calculations. This constant is
/// exported for frontend use so clients can validate amounts before submitting
/// a transaction.
pub const MAX_DEPOSIT: i128 = 1_000_000_000_000_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    /// Caller is not authorized to perform this action.
    Unauthorized = 1,
    /// The vault has not been initialized.
    NotInitialized = 2,
    /// The vault has already been initialized.
    AlreadyInitialized = 3,
    /// The requested amount is invalid (e.g. zero or negative).
    InvalidAmount = 4,
    /// Insufficient balance for the requested operation.
    InsufficientBalance = 5,
    /// The lock period has not yet elapsed.
    StillLocked = 6,
    /// The provided ledger is invalid.
    InvalidLedger = 7,
    /// The deposit amount exceeds [`MAX_DEPOSIT`] (10^15 units).
    AmountTooLarge = 8,
}
