//! Vault constants shared across the contract and exported for frontend use.

/// Maximum amount (in base units) that may be deposited in a single call.
///
/// Deposits are capped at 10^15 units to prevent arithmetic overflow in
/// downstream calculations. Any deposit path (`deposit`, `deposit_for`,
/// `deposit_by_ledger`) must reject amounts greater than this value with
/// `AmountTooLarge` (error code 8) before mutating state.
pub const MAX_DEPOSIT: i128 = 1_000_000_000_000_000;
