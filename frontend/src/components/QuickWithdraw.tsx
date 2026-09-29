import { useState, useEffect } from 'react'
import { formatCountdown, formatBps } from '../lib/format'
import type { VaultEntry } from '../types'

interface QuickWithdrawProps {
  deposit: VaultEntry & { timeRemaining: number; depositId: number }
  onWithdraw: (depositId: number) => Promise<void>
  onCancel: (depositId: number) => Promise<void>
  isLoading?: boolean
}

export function QuickWithdraw({ deposit, onWithdraw, onCancel, isLoading }: QuickWithdrawProps) {
  const [timeRemaining, setTimeRemaining] = useState(deposit.timeRemaining)
  const [countdownText, setCountdownText] = useState('')

  useEffect(() => {
    setTimeRemaining(deposit.timeRemaining)
  }, [deposit.timeRemaining])

  // Live countdown
  useEffect(() => {
    if (timeRemaining <= 0) {
      setCountdownText('Ready to withdraw')
      return
    }

    const updateCountdown = () => {
      setTimeRemaining((prev) => {
        if (prev <= 1) {
          setCountdownText('Ready to withdraw')
          return 0
        }
        setCountdownText(formatCountdown(prev - 1))
        return prev - 1
      })
    }

    const interval = setInterval(updateCountdown, 1000)
    setCountdownText(formatCountdown(timeRemaining))
    return () => clearInterval(interval)
  }, [timeRemaining])

  const isUnlocked = timeRemaining === 0
  const penaltyApplies = !isUnlocked && deposit.penaltyBps > 0

  return (
    <div className="bg-gradient-to-br from-slate-800/60 to-slate-800/30 border border-slate-700/40 rounded-lg p-4 space-y-3">
      {/* Deposit ID & Amount */}
      <div className="flex justify-between items-start">
        <div>
          <p className="text-xs text-slate-500 uppercase tracking-wide font-medium">Deposit #{deposit.depositId}</p>
          <p className="text-lg font-semibold text-slate-100 mt-1">{deposit.amount} XLM</p>
        </div>
        {isUnlocked ? (
          <div className="px-2 py-1 bg-green-900/40 border border-green-700/60 rounded text-xs font-medium text-green-400">
            🔓 Unlocked
          </div>
        ) : (
          <div className={`px-2 py-1 rounded text-xs font-medium ${
            penaltyApplies
              ? 'bg-orange-900/40 border border-orange-700/60 text-orange-400'
              : 'bg-amber-900/40 border border-amber-700/60 text-amber-400'
          }`}>
            🔒 Locked
          </div>
        )}
      </div>

      {/* Countdown / Status */}
      <div className="text-center py-2">
        {isUnlocked ? (
          <p className="text-green-400 font-semibold text-sm">Ready to withdraw</p>
        ) : (
          <div>
            <p className="text-xs text-slate-500 uppercase tracking-wide font-medium mb-0.5">Time remaining</p>
            <p className="font-mono text-sm text-slate-200">{countdownText}</p>
          </div>
        )}
      </div>

      {/* Penalty Warning */}
      {penaltyApplies && (
        <div className="bg-orange-900/20 border border-orange-700/30 rounded p-2 text-xs">
          <p className="font-medium text-orange-300">⚠️ Early exit penalty</p>
          <p className="text-orange-200/80 text-xs mt-0.5">Exit now and lose {formatBps(deposit.penaltyBps)}</p>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex gap-2 pt-2">
        {isUnlocked ? (
          <>
            <button
              onClick={() => onWithdraw(deposit.depositId)}
              disabled={isLoading}
              className="flex-1 py-2 px-3 bg-green-600 hover:bg-green-700 disabled:opacity-50 rounded-lg text-sm font-medium transition-colors"
            >
              {isLoading ? '⏳' : '✓'} Withdraw
            </button>
          </>
        ) : (
          <>
            <button
              onClick={() => onCancel(deposit.depositId)}
              disabled={isLoading}
              className="flex-1 py-2 px-3 bg-slate-700/40 hover:bg-slate-700/60 disabled:opacity-50 border border-slate-600/40 rounded-lg text-sm font-medium transition-colors text-slate-300"
            >
              {isLoading ? '⏳' : '⏹'} Exit now
            </button>
          </>
        )}
      </div>
    </div>
  )
}
