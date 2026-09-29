import { useState } from 'react'
import type { Deposit } from '../types'
import { stroopsToXlm, formatUnlockDate, formatBps, shortAddr } from '../lib/format'
import { CONFIG } from '../config'

interface DuplicateDepositWarningProps {
  isOpen: boolean
  pendingDeposit: {
    token: string
    amount: bigint
    unlockTime: number
    penaltyBps: number
  }
  duplicates: Deposit[]
  onProceed: (suppressFuture: boolean) => void
  onModify: () => void
  onCancel: () => void
}

const STORAGE_KEY = 'safe-haven-duplicate-warnings-dismissed'
const ALWAYS_ALLOW_KEY = 'safe-haven-allow-all-duplicates'

export function DuplicateDepositWarning({
  isOpen,
  pendingDeposit,
  duplicates,
  onProceed,
  onModify,
  onCancel,
}: DuplicateDepositWarningProps) {
  const [suppressFuture, setSuppressFuture] = useState(false)
  const [alwaysAllow, setAlwaysAllow] = useState(false)

  if (!isOpen) return null

  const isXlm = pendingDeposit.token === CONFIG.NATIVE_TOKEN
  const exactMatch = duplicates.some(d => 
    d.token === pendingDeposit.token &&
    d.amount === pendingDeposit.amount &&
    d.unlockTime === pendingDeposit.unlockTime &&
    d.penaltyBps === pendingDeposit.penaltyBps
  )

  const handleProceed = () => {
    if (alwaysAllow) {
      localStorage.setItem(ALWAYS_ALLOW_KEY, 'true')
    } else if (suppressFuture) {
      // Save the specific deposit signature to suppress future warnings
      saveDismissedSignature(pendingDeposit)
    }
    onProceed(suppressFuture || alwaysAllow)
  }

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-slate-800 rounded-lg border border-slate-700 shadow-2xl w-full max-w-lg max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="sticky top-0 bg-slate-800 border-b border-slate-700 p-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <svg className="w-6 h-6 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
            </svg>
            <h2 className="text-xl font-bold text-slate-100">
              {exactMatch ? 'Exact Duplicate Detected' : 'Similar Deposit Found'}
            </h2>
          </div>
          <button
            onClick={onCancel}
            className="text-slate-400 hover:text-slate-200 transition-colors"
            aria-label="Close"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Warning Message */}
          <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
            <p className="text-sm text-slate-300">
              {exactMatch ? (
                <>You already have an <span className="font-semibold text-yellow-400">identical deposit</span> with the same parameters.</>
              ) : (
                <>You have <span className="font-semibold text-yellow-400">{duplicates.length} similar deposit{duplicates.length > 1 ? 's' : ''}</span> with comparable parameters.</>
              )}
            </p>
            <p className="text-xs text-slate-400 mt-2">
              This might be unintentional. Review the details below before proceeding.
            </p>
          </div>

          {/* Pending Deposit Summary */}
          <div>
            <h3 className="text-sm font-semibold text-slate-300 mb-2">New Deposit</h3>
            <div className="bg-stellar-900/30 border border-stellar-700/40 rounded-lg p-3 space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-slate-400">Amount</span>
                <span className="font-semibold text-stellar-400">{stroopsToXlm(pendingDeposit.amount)} {isXlm ? 'XLM' : 'tokens'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Unlock</span>
                <span className="text-slate-100">{formatUnlockDate(pendingDeposit.unlockTime)}</span>
              </div>
              {pendingDeposit.penaltyBps > 0 && (
                <div className="flex justify-between">
                  <span className="text-slate-400">Penalty</span>
                  <span className="text-orange-400">{formatBps(pendingDeposit.penaltyBps)}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-slate-400">Token</span>
                <span className="text-slate-100 font-mono text-xs">{isXlm ? 'XLM' : shortAddr(pendingDeposit.token)}</span>
              </div>
            </div>
          </div>

          {/* Existing Duplicates */}
          <div>
            <h3 className="text-sm font-semibold text-slate-300 mb-2">Existing Similar Deposits</h3>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {duplicates.map((deposit) => {
                const matchIndicators = []
                if (deposit.token === pendingDeposit.token) matchIndicators.push('token')
                if (deposit.amount === pendingDeposit.amount) matchIndicators.push('amount')
                if (Math.abs(deposit.unlockTime - pendingDeposit.unlockTime) < 3600) matchIndicators.push('unlock time')
                if (deposit.penaltyBps === pendingDeposit.penaltyBps) matchIndicators.push('penalty')

                const isExactMatch = matchIndicators.length === 4

                return (
                  <div
                    key={deposit.depositId}
                    className={`rounded-lg p-3 text-xs space-y-1.5 ${
                      isExactMatch
                        ? 'bg-red-900/20 border border-red-700/40'
                        : 'bg-slate-900/50 border border-slate-700/60'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-slate-200">Deposit #{deposit.depositId}</span>
                      {isExactMatch && (
                        <span className="text-xs bg-red-500/20 text-red-400 px-2 py-0.5 rounded">Exact match</span>
                      )}
                    </div>
                    <div className="grid grid-cols-2 gap-1.5 text-xs">
                      <div>
                        <span className="text-slate-500">Amount: </span>
                        <span className={deposit.amount === pendingDeposit.amount ? 'text-yellow-400 font-medium' : 'text-slate-300'}>
                          {stroopsToXlm(deposit.amount)}
                        </span>
                      </div>
                      <div>
                        <span className="text-slate-500">Penalty: </span>
                        <span className={deposit.penaltyBps === pendingDeposit.penaltyBps ? 'text-yellow-400 font-medium' : 'text-slate-300'}>
                          {formatBps(deposit.penaltyBps)}
                        </span>
                      </div>
                    </div>
                    <div className="text-slate-500">
                      Unlocks: <span className={Math.abs(deposit.unlockTime - pendingDeposit.unlockTime) < 3600 ? 'text-yellow-400 font-medium' : 'text-slate-300'}>
                        {formatUnlockDate(deposit.unlockTime)}
                      </span>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>

          {/* Options */}
          <div className="space-y-2">
            <label className="flex items-start gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={suppressFuture}
                onChange={(e) => {
                  setSuppressFuture(e.target.checked)
                  if (e.target.checked) setAlwaysAllow(false)
                }}
                className="mt-0.5 w-4 h-4 rounded border-slate-600 bg-slate-700 text-stellar-500 focus:ring-stellar-500 focus:ring-offset-slate-800"
              />
              <span className="text-xs text-slate-300">
                Don't warn me again for this specific deposit configuration
              </span>
            </label>
            
            <label className="flex items-start gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={alwaysAllow}
                onChange={(e) => {
                  setAlwaysAllow(e.target.checked)
                  if (e.target.checked) setSuppressFuture(false)
                }}
                className="mt-0.5 w-4 h-4 rounded border-slate-600 bg-slate-700 text-stellar-500 focus:ring-stellar-500 focus:ring-offset-slate-800"
              />
              <span className="text-xs text-slate-300">
                Always allow duplicates (disable all future warnings)
              </span>
            </label>
          </div>
        </div>

        {/* Footer Actions */}
        <div className="sticky bottom-0 bg-slate-800 border-t border-slate-700 p-4 flex flex-col gap-2">
          <div className="flex gap-2">
            <button
              onClick={handleProceed}
              className="flex-1 px-4 py-2 bg-stellar-500 hover:bg-stellar-600 text-white rounded-lg transition-colors font-medium"
            >
              Proceed Anyway
            </button>
            <button
              onClick={onModify}
              className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-100 rounded-lg transition-colors font-medium"
            >
              Modify
            </button>
          </div>
          <button
            onClick={onCancel}
            className="w-full px-4 py-2 bg-slate-900 hover:bg-slate-800 text-slate-300 rounded-lg transition-colors font-medium border border-slate-700"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}

/**
 * Check if duplicate warnings are globally disabled.
 */
export function isDuplicateWarningsDisabled(): boolean {
  return localStorage.getItem(ALWAYS_ALLOW_KEY) === 'true'
}

/**
 * Save a dismissed deposit signature to suppress future warnings.
 */
function saveDismissedSignature(deposit: {
  token: string
  amount: bigint
  unlockTime: number
  penaltyBps: number
}) {
  try {
    const signature = `${deposit.token}:${deposit.amount}:${deposit.unlockTime}:${deposit.penaltyBps}`
    const stored = localStorage.getItem(STORAGE_KEY)
    const dismissed: string[] = stored ? JSON.parse(stored) : []
    
    if (!dismissed.includes(signature)) {
      dismissed.push(signature)
      // Keep only last 50 to avoid storage bloat
      if (dismissed.length > 50) {
        dismissed.shift()
      }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(dismissed))
    }
  } catch (e) {
    console.warn('Failed to save dismissed duplicate warning:', e)
  }
}

/**
 * Check if a deposit signature has been dismissed.
 */
export function isDepositDismissed(deposit: {
  token: string
  amount: bigint
  unlockTime: number
  penaltyBps: number
}): boolean {
  try {
    const signature = `${deposit.token}:${deposit.amount}:${deposit.unlockTime}:${deposit.penaltyBps}`
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return false
    
    const dismissed: string[] = JSON.parse(stored)
    return dismissed.includes(signature)
  } catch {
    return false
  }
}

/**
 * Find duplicate or similar deposits.
 * 
 * Similarity criteria:
 * - Same token
 * - Amount within 10% or exactly equal
 * - Unlock time within 1 hour or exactly equal
 */
export function findDuplicateDeposits(
  deposits: Deposit[],
  pending: {
    token: string
    amount: bigint
    unlockTime: number
    penaltyBps: number
  }
): Deposit[] {
  return deposits.filter(deposit => {
    // Must be same token
    if (deposit.token !== pending.token) return false
    
    // Check amount similarity (within 10% or exact match)
    const amountDiff = deposit.amount > pending.amount
      ? Number((deposit.amount - pending.amount) * 100n / deposit.amount)
      : Number((pending.amount - deposit.amount) * 100n / pending.amount)
    
    const amountSimilar = amountDiff <= 10 || deposit.amount === pending.amount
    
    // Check unlock time similarity (within 1 hour)
    const timeDiff = Math.abs(deposit.unlockTime - pending.unlockTime)
    const timeSimilar = timeDiff < 3600 // 1 hour in seconds
    
    return amountSimilar && timeSimilar
  })
}
