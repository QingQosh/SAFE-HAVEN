import { useState, useEffect } from 'react'
import type { Deposit } from '../types'
import { formatTokenWithUsd, formatUnlockDate, shortAddr } from '../lib/format'
import { usePrice } from '../hooks/usePrice'
import { CONFIG } from '../config'

interface WithdrawalConfirmationProps {
  isOpen: boolean
  deposit: Deposit
  recipient?: string
  estimatedGas: string
  onConfirm: () => void
  onCancel: () => void
}

interface FavoriteRecipient {
  address: string
  label: string
}

const COUNTDOWN_SECONDS = 10
const STORAGE_KEY = 'safe-haven-favorite-recipients'

export function WithdrawalConfirmation({
  isOpen,
  deposit,
  recipient,
  estimatedGas,
  onConfirm,
  onCancel,
}: WithdrawalConfirmationProps) {
  const [confirmText, setConfirmText] = useState('')
  const [countdown, setCountdown] = useState(COUNTDOWN_SECONDS)
  const [saveAsFavorite, setSaveAsFavorite] = useState(false)
  const [favoriteLabel, setFavoriteLabel] = useState('')
  const { getPrice } = usePrice()

  const isXlm = deposit.token === CONFIG.NATIVE_TOKEN
  const priceData = isXlm ? getPrice('native') : null
  const priceUsd = priceData?.usd
  
  const isWithdrawToRecipient = !!recipient && recipient !== deposit.depositor
  const confirmTextMatch = confirmText.toUpperCase() === 'CONFIRM'
  const canSubmit = confirmTextMatch && countdown === 0

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setConfirmText('')
      setCountdown(COUNTDOWN_SECONDS)
      setSaveAsFavorite(false)
      setFavoriteLabel('')
    }
  }, [isOpen])

  // Countdown timer
  useEffect(() => {
    if (!isOpen || countdown === 0) return

    const timer = setInterval(() => {
      setCountdown((prev) => Math.max(0, prev - 1))
    }, 1000)

    return () => clearInterval(timer)
  }, [isOpen, countdown])

  const handleConfirm = () => {
    if (!canSubmit) return

    // Save favorite recipient if requested
    if (saveAsFavorite && isWithdrawToRecipient && recipient && favoriteLabel.trim()) {
      saveFavoriteRecipient(recipient, favoriteLabel.trim())
    }

    onConfirm()
  }

  const saveFavoriteRecipient = (address: string, label: string) => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      const favorites: FavoriteRecipient[] = stored ? JSON.parse(stored) : []
      
      // Don't add duplicates
      if (!favorites.some(f => f.address === address)) {
        favorites.push({ address, label })
        localStorage.setItem(STORAGE_KEY, JSON.stringify(favorites))
      }
    } catch (e) {
      console.warn('Failed to save favorite recipient:', e)
    }
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-slate-800 rounded-lg border border-slate-700 shadow-2xl w-full max-w-md max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="sticky top-0 bg-slate-800 border-b border-slate-700 p-4 flex items-center justify-between">
          <h2 className="text-xl font-bold text-slate-100">Confirm Withdrawal</h2>
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
          {/* Risk Warning */}
          <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
            <div className="flex gap-2">
              <svg className="w-5 h-5 text-yellow-500 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
              </svg>
              <div>
                <p className="text-sm font-semibold text-yellow-500">Important</p>
                <p className="text-xs text-slate-300 mt-1">
                  This transaction is irreversible. Please verify all details before confirming.
                </p>
              </div>
            </div>
          </div>

          {/* Transaction Summary */}
          <div className="space-y-3">
            <h3 className="text-sm font-semibold text-slate-300">Transaction Details</h3>
            
            <div className="bg-slate-900/50 rounded-lg p-3 space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-slate-400">Deposit ID</span>
                <span className="text-slate-100 font-mono">#{deposit.depositId}</span>
              </div>
              
              <div className="flex justify-between text-sm">
                <span className="text-slate-400">Amount</span>
                <span className="text-slate-100 font-semibold">
                  {formatTokenWithUsd(deposit.amount, isXlm ? 'XLM' : 'tokens', priceUsd)}
                </span>
              </div>

              <div className="flex justify-between text-sm">
                <span className="text-slate-400">Token</span>
                <span className="text-slate-100 font-mono text-xs">
                  {isXlm ? 'XLM (Native)' : shortAddr(deposit.token)}
                </span>
              </div>

              <div className="flex justify-between text-sm">
                <span className="text-slate-400">Unlocked</span>
                <span className="text-green-400">{formatUnlockDate(deposit.unlockTime)}</span>
              </div>

              <div className="flex justify-between text-sm">
                <span className="text-slate-400">From</span>
                <span className="text-slate-100 font-mono text-xs">{shortAddr(deposit.depositor)}</span>
              </div>

              {isWithdrawToRecipient && recipient && (
                <div className="flex justify-between text-sm border-t border-slate-700 pt-2">
                  <span className="text-slate-400">To</span>
                  <span className="text-stellar-400 font-mono text-xs">{shortAddr(recipient)}</span>
                </div>
              )}

              <div className="flex justify-between text-sm border-t border-slate-700 pt-2">
                <span className="text-slate-400">Estimated Gas</span>
                <span className="text-slate-100">{estimatedGas}</span>
              </div>
            </div>
          </div>

          {/* Save as Favorite (only for withdraw_to) */}
          {isWithdrawToRecipient && recipient && (
            <div className="bg-slate-900/50 rounded-lg p-3 space-y-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={saveAsFavorite}
                  onChange={(e) => setSaveAsFavorite(e.target.checked)}
                  className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-stellar-500 focus:ring-stellar-500 focus:ring-offset-slate-800"
                />
                <span className="text-sm text-slate-300">Save recipient as favorite</span>
              </label>
              
              {saveAsFavorite && (
                <input
                  type="text"
                  value={favoriteLabel}
                  onChange={(e) => setFavoriteLabel(e.target.value)}
                  placeholder="Enter label (e.g., 'Savings Wallet')"
                  className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-lg text-sm text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-stellar-500"
                  maxLength={50}
                />
              )}
            </div>
          )}

          {/* Confirmation Input */}
          <div className="space-y-2">
            <label className="block text-sm font-semibold text-slate-300">
              Type <span className="text-stellar-400 font-mono">CONFIRM</span> to proceed
            </label>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              placeholder="Type CONFIRM"
              className="w-full px-4 py-2 bg-slate-900 border border-slate-600 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-stellar-500 font-mono uppercase"
              autoComplete="off"
              autoFocus
            />
            {confirmText && !confirmTextMatch && (
              <p className="text-xs text-red-400">Must type exactly: CONFIRM</p>
            )}
          </div>

          {/* Countdown Timer */}
          <div className="bg-slate-900/50 rounded-lg p-3 flex items-center justify-center gap-2">
            {countdown > 0 ? (
              <>
                <div className="w-2 h-2 rounded-full bg-yellow-400 animate-pulse" />
                <span className="text-sm text-slate-300">
                  Please wait <span className="font-bold text-yellow-400">{countdown}s</span> before confirming
                </span>
              </>
            ) : (
              <>
                <div className="w-2 h-2 rounded-full bg-green-400" />
                <span className="text-sm text-green-400 font-semibold">Ready to proceed</span>
              </>
            )}
          </div>
        </div>

        {/* Footer Actions */}
        <div className="sticky bottom-0 bg-slate-800 border-t border-slate-700 p-4 flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-100 rounded-lg transition-colors font-medium"
          >
            Cancel
          </button>
          <button
            onClick={handleConfirm}
            disabled={!canSubmit}
            className={`flex-1 px-4 py-2 rounded-lg font-medium transition-all ${
              canSubmit
                ? 'bg-stellar-500 hover:bg-stellar-600 text-white shadow-lg shadow-stellar-500/30'
                : 'bg-slate-700 text-slate-500 cursor-not-allowed'
            }`}
          >
            {canSubmit ? 'Confirm Withdrawal' : 'Waiting...'}
          </button>
        </div>
      </div>
    </div>
  )
}
