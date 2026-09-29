// ============================================================
//  Prediction Markets - Error Codes
// ============================================================

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PredictionMarketError {
    // Market creation errors
    MarketNotFound = 1000,
    MarketNotOpen = 1001,
    InvalidMarketStatus = 1002,
    InvalidOutcomeCount = 1003,
    InvalidResolutionTime = 1004,
    ResolutionTimeInPast = 1005,
    InvalidFeeBps = 1006,
    
    // Betting errors
    BettingClosed = 1007,
    InvalidBetAmount = 1008,
    InvalidOutcomeId = 1009,
    BetNotFound = 1010,
    AlreadyBetOnOutcome = 1011,
    InsufficientFunds = 1012,
    
    // Resolution errors
    MarketNotClosedYet = 1013,
    MarketAlreadyResolved = 1014,
    InvalidWinningOutcome = 1015,
    ResolutionDeadlineExceeded = 1016,
    UnauthorizedOracle = 1017,
    OracleSubmissionExists = 1018,
    
    // Claiming errors
    NoWinningBets = 1019,
    BetsAlreadyClaimed = 1020,
    InvalidClaimAmount = 1021,
    ClaimingBeforeResolution = 1022,
    
    // Authorization errors
    Unauthorized = 1023,
    InvalidAdmin = 1024,
    
    // Market operation errors
    MarketPaused = 1025,
    InvalidMarketType = 1026,
    InvalidOracle = 1027,
    ResolutionDataMissing = 1028,
    
    // Generic errors
    InvalidArgument = 1029,
    InternalError = 1030,
}
