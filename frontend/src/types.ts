// ============================================================
//  Shared TypeScript types — mirrors the Soroban contract types
// ============================================================

/** Mirrors the Rust VaultEntry struct */
export interface VaultEntry {
  token: string       // Stellar address
  amount: bigint      // in token base units (stroops for XLM)
  unlockTime: number  // Unix timestamp (seconds)
  depositor: string   // Stellar address
  penaltyBps: number  // 0–10_000 (basis points)
  compoundFrequencySecs: number  // Compound interest frequency (0 = no compounding)
  lastAccrualTimestamp: number   // Last compound accrual timestamp
}

/** Available ways to plan a withdrawal */
export type WithdrawalStrategy = 'lump-sum' | 'linear' | 'cliff'

/** A deposit with its ID attached */
export interface Deposit extends VaultEntry {
  depositId: number
  /** Seconds remaining until unlock (0 if unlocked, null during initial load) */
  timeRemaining: number | null
  /**
   * Whether the chain has confirmed this deposit is truly unlocked.
   *
   * - `true`  for all freshly-loaded deposits (timeRemaining was computed from
   *           a live getLedgerTime() call, so the value is chain-authoritative).
   * - `true`  after a getTimeRemaining() call confirms the value is 0.
   * - `false` while the local ticker has just ticked down to 0 but a chain
   *           re-verification is still in-flight. The Withdraw button must be
   *           hidden while this is false to prevent premature FundsStillLocked
   *           errors due to clock drift or tab-throttling.
   */
  unlockVerified: boolean
}

/** Result of wallet connection */
export interface WalletInfo {
  address: string
  displayAddress: string
  networkMismatch?: boolean  // True if wallet network differs from app network
  walletNetwork?: string     // Network passphrase from Freighter (for display)
}

/** Tab pages */
export type PageTab = 'dashboard' | 'deposit' | 'withdraw' | 'yield' | 'settings' | 'admin'

export type RecoveryContactType = 'email' | 'wallet'

export interface RecoveryContact {
  id: string
  type: RecoveryContactType
  value: string
  addedAt: number
}

export interface RecoveryRequest {
  recoveryContactId: string
  newWallet: string
  verificationCode: string
  initiatedAt: number
  unlockAt: number
  verifiedAt: number | null
}

/** Loading states for async operations */
export type TxStatus = 'idle' | 'signing' | 'submitting' | 'confirming' | 'success' | 'error'

/** Contract call result wrapper */
export interface ContractResult<T> {
  success: boolean
  data?: T
  error?: string
  txHash?: string
}

/** Discriminated union for wallet signing results */
export type SigningResult = 
  | { signed: true; xdr: string }           // Successfully signed
  | { signed: false; rejected: true }       // User rejected the signing request
  | { signed: false; rejected: false; error: string } // Signing failed with error
