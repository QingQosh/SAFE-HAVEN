import { useEffect, useState } from 'react'
import { isPaused } from '../lib/stellar'

/**
 * Full-screen overlay component displayed when the contract is paused.
 * Queries pause status every 10 seconds and allows users to refresh.
 */
export function PausedNotice() {
  const [isContractPaused, setIsContractPaused] = useState(false)
  const [isChecking, setIsChecking] = useState(false)

  async function checkPauseStatus() {
    setIsChecking(true)
    try {
      const paused = await isPaused()
      setIsContractPaused(paused)
    } catch (e) {
      console.error('Failed to check pause status:', e)
    } finally {
      setIsChecking(false)
    }
  }

  // Check pause status on mount
  useEffect(() => {
    void checkPauseStatus()
  }, [])

  // Poll for pause status every 10 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      void checkPauseStatus()
    }, 10_000)
    return () => clearInterval(interval)
  }, [])

  if (!isContractPaused) return null

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-slate-900 border border-red-700/60 rounded-2xl shadow-2xl max-w-md w-full p-8 text-center space-y-6">
        {/* Icon */}
        <div className="w-16 h-16 rounded-full bg-red-900/40 border border-red-700/60 flex items-center justify-center mx-auto">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            className="w-8 h-8 text-red-400"
          >
            <rect x="6" y="4" width="12" height="16" rx="1" />
            <path d="M9 9h6M9 15h6" strokeLinecap="round" />
          </svg>
        </div>

        {/* Heading */}
        <div>
          <h2 className="text-2xl font-bold text-white mb-2">Contract Paused</h2>
          <p className="text-slate-300">
            The SAFE-HAVEN contract is temporarily paused. New deposits and withdrawals are disabled.
          </p>
        </div>

        {/* Details */}
        <div className="bg-slate-800/50 rounded-lg p-4 space-y-2 text-sm text-left">
          <p className="text-slate-400">
            <strong className="text-slate-300">What's happening:</strong>
            <br />
            Administrative maintenance or security measures are in effect.
          </p>
          <p className="text-slate-400">
            <strong className="text-slate-300">Read-only access:</strong>
            <br />
            You can still view your deposits, but cannot execute new transactions.
          </p>
        </div>

        {/* Actions */}
        <div className="flex gap-3 flex-col sm:flex-row">
          <a
            href="https://github.com/kenedybok3/SAFE-HAVEN"
            target="_blank"
            rel="noopener noreferrer"
            className="btn-secondary flex-1 text-center"
          >
            Updates
          </a>
          <button
            onClick={() => void checkPauseStatus()}
            disabled={isChecking}
            className="btn-primary flex-1"
          >
            {isChecking ? (
              <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin inline-block" />
            ) : (
              'Try again'
            )}
          </button>
        </div>

        {/* Auto-refresh info */}
        <p className="text-xs text-slate-500">
          Status checked automatically every 10 seconds
        </p>
      </div>
    </div>
  )
}
