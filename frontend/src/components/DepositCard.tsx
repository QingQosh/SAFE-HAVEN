import { useState } from 'react'
import type { Deposit } from '../types'
import { formatUnlockDate, formatCountdown, formatBps, shortAddr, explorerAddrUrl, formatTokenWithUsd, formatPriceUpdate } from '../lib/format'
import { usePrice } from '../hooks/usePrice'
import { CONFIG } from '../config'

interface DepositCardProps {
  deposit: Deposit
  onWithdraw: (depositId: number) => void
  onCancel: (depositId: number) => void
  txPending: boolean
}

export function DepositCard({ deposit, onWithdraw, onCancel, txPending }: DepositCardProps) {
  const [showDetails, setShowDetails] = useState(false)
  const { getPrice } = usePrice()

  const isXlm   = deposit.token === CONFIG.NATIVE_TOKEN
  const isUnlocked = deposit.timeRemaining !== null && deposit.timeRemaining === 0 && deposit.unlockVerified
  const isPendingVerification = deposit.timeRemaining === 0 && !deposit.unlockVerified
  const hasPenalty = deposit.penaltyBps > 0

  const penaltyAmount = isUnlocked
    ? 0n
    : (deposit.amount * BigInt(deposit.penaltyBps)) / 10_000n

  const refundAmount = deposit.amount - penaltyAmount

  // Get price for token (only XLM for now)
  const priceData = isXlm ? getPrice('native') : null
  const priceUsd = priceData?.usd
  const priceUpdateStr = priceData ? formatPriceUpdate(priceData.lastUpdated) : null

  return (
    <div className="card p-4 md:p-5 hover:border-slate-600/80 transition-colors">
      {/* Top row */}
      <div className="flex items-start justify-between gap-2 md:gap-3">
        <div className="flex items-center gap-2 md:gap-3 min-w-0 flex-1">
          {/* Token icon placeholder */}
          <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-stellar-900/60 border border-stellar-700/40 flex items-center justify-center flex-shrink-0 text-stellar-400 font-bold text-xs md:text-sm">
            {isXlm ? 'XLM' : '?'}
          </div>
          <div className="min-w-0">
            <p className="font-semibold text-base">
              {formatTokenWithUsd(deposit.amount, isXlm ? 'XLM' : 'tokens', priceUsd)}
            </p>
            <p className="text-xs text-slate-400 truncate">Deposit #{deposit.depositId}</p>
            {priceUpdateStr && (
              <p className="text-xs text-slate-500">{priceUpdateStr}</p>
            )}
          </div>
        </div>

        {/* Status badge */}
        {isUnlocked ? (
          <span className="badge-green flex-shrink-0 text-xs">
            <span className="w-1.5 h-1.5 rounded-full bg-green-400" />
            <span className="hidden sm:inline">Unlocked</span>
          </span>
        ) : isPendingVerification ? (
          <span className="badge-yellow flex-shrink-0 text-xs">
            <span className="w-3 h-3 border-2 border-yellow-400/40 border-t-yellow-400 rounded-full animate-spin" />
          </span>
        ) : (
          <span className="badge-yellow flex-shrink-0 countdown-active text-xs">
            <span className="w-1.5 h-1.5 rounded-full bg-yellow-400" />
            <span className="hidden sm:inline">{formatCountdown(deposit.timeRemaining)}</span>
          </span>
        )}
      </div>

      {/* Unlock date */}
      <div className="mt-2 md:mt-3 text-xs md:text-sm text-slate-400">
        {isUnlocked ? (
          <span className="text-green-400">Ready to withdraw</span>
        ) : isPendingVerification ? (
          <span className="text-yellow-400/80">Verifying on-chain…</span>
        ) : (
          <>Unlocks <span className="text-slate-200">{formatUnlockDate(deposit.unlockTime)}</span></>
        )}
      </div>

      {/* Penalty info (if any) */}
      {hasPenalty && !isUnlocked && (
        <div className="mt-2 text-xs text-orange-400 bg-orange-900/20 rounded-lg px-3 py-2 border border-orange-800/30">
          Early exit penalty: {formatBps(deposit.penaltyBps)} — you'd receive ~{formatTokenWithUsd(refundAmount, isXlm ? 'XLM' : 'tokens', priceUsd)}
        </div>
      )}

      {/* Expandable details */}
      {showDetails && (
        <div className="mt-3 md:mt-4 pt-3 md:pt-4 border-t border-slate-700/60 grid grid-cols-2 gap-y-2 gap-x-3 text-xs">
          <span className="text-slate-500">Token</span>
          <a
            href={explorerAddrUrl(deposit.token)}
            target="_blank"
            rel="noopener noreferrer"
            className="text-stellar-400 hover:text-stellar-300 font-mono truncate text-xs"
          >
            {shortAddr(deposit.token)}
          </a>
          <span className="text-slate-500">Depositor</span>
          <a
            href={explorerAddrUrl(deposit.depositor)}
            target="_blank"
            rel="noopener noreferrer"
            className="text-stellar-400 hover:text-stellar-300 font-mono truncate text-xs"
          >
            {shortAddr(deposit.depositor)}
          </a>
          <span className="text-slate-500">Penalty BPS</span>
          <span className="text-slate-300 text-xs">{deposit.penaltyBps}</span>
          <span className="text-slate-500">Unlock time</span>
          <span className="text-slate-300 font-mono text-xs truncate">{deposit.unlockTime}</span>
        </div>
      )}

      {/* Actions */}
      <div className="mt-3 md:mt-4 flex flex-col sm:flex-row gap-2">
        {isUnlocked ? (
          <button
            className="btn-primary w-full text-sm py-2.5 min-h-10"
            onClick={() => onWithdraw(deposit.depositId)}
            disabled={txPending}
          >
            {txPending ? (
              <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
                <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            )}
            <span className="hidden sm:inline">Withdraw</span>
          </button>
        ) : isPendingVerification ? (
          <button className="btn-primary w-full text-sm py-2.5 min-h-10" disabled>
            <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            <span className="hidden sm:inline">Verifying…</span>
          </button>
        ) : (
          <button
            className={`text-sm py-2.5 min-h-10 ${hasPenalty ? 'btn-danger' : 'btn-secondary'} flex-1`}
            onClick={() => onCancel(deposit.depositId)}
            disabled={txPending}
          >
            Cancel {hasPenalty ? `(${formatBps(deposit.penaltyBps)})` : ''}
          </button>
        )}

        <button
          className="btn-secondary text-xs px-3 py-2.5 min-h-10"
          onClick={() => setShowDetails((v) => !v)}
        >
          {showDetails ? 'Less' : 'Details'}
        </button>
      </div>
    </div>
  )
}
