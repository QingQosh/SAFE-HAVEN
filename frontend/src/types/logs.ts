// ============================================================
//  Contract Logs Type Definitions
// ============================================================

/** Supported contract operations */
export type ContractOperation =
  | 'deposit'
  | 'deposit_for'
  | 'deposit_by_ledger'
  | 'withdraw'
  | 'withdraw_to'
  | 'cancel_deposit'
  | 'register_staker'
  | 'claim_staker_rewards'
  | 'emergency_withdraw'
  | 'pause'
  | 'unpause'
  | 'transfer_admin'
  | 'accept_admin'
  | 'renounce_admin'
  | 'initialize'

/** Log entry status */
export type LogEntryStatus = 'pending' | 'success' | 'error'

/** A single contract operation log entry */
export interface ContractLogEntry {
  /** Unique identifier (timestamp + random) */
  id: string

  /** Type of operation */
  operation: ContractOperation

  /** Current status of the operation */
  status: LogEntryStatus

  /** When the operation was initiated (ISO string) */
  timestamp: string

  /** Stellar transaction hash (set after submission) */
  txHash?: string

  /** The wallet address that initiated the operation */
  initiator?: string

  /** Operation parameters (varies by operation type) */
  parameters: Record<string, unknown>

  /** Error message if status is 'error' */
  errorMessage?: string

  /** Additional details/notes */
  details?: string
}

/** Filters for log queries */
export interface LogFilters {
  operation?: ContractOperation
  status?: LogEntryStatus
  search?: string
  dateFrom?: Date
  dateTo?: Date
}
