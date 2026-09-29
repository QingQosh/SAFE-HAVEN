import { useState, useEffect } from 'react'
import { useWallet } from '../context/WalletContext'
import { useDeposits } from '../hooks/useDeposits'
import { usePrice } from '../hooks/usePrice'
import {
  calculateAggregateYield,
  calculateYieldSummary,
  compareYieldVsPenalty,
  generateYieldProjections,
} from '../lib/yield'
import { stroopsToXlm, formatBps, shortAddr } from '../lib/format'
import { CONFIG } from '../config'
import type { Deposit } from '../types'

export function YieldDashboard() {
  const { wallet, isRestoringSession } = useWallet()
  const { deposits, loading, error, refresh } = useDeposits(wallet?.address ?? null)
  const { getPrice } = usePrice()
  
  const [currentTime, setCurrentTime] = useState(Math.floor(Date.now() / 1000))
  const [selectedDeposit, setSelectedDeposit] = useState<Deposit | null>(null)

  // Update current time every second for real-time projections
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(Math.floor(Date.now() / 1000))
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  if (!wallet && !isRestoringSession) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-center">
        <div className="w-16 h-16 rounded-2xl bg-stellar-900/40 border border-stellar-700/40 flex items-center justify-center mb-4">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-8 h-8 text-stellar-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
          </svg>
        </div>
        <h2 className="text-xl font-semibold mb-2">Connect your wallet</h2>
        <p className="text-slate-400 text-sm max-w-xs">Connect your Freighter wallet to view your yield dashboard.</p>
      </div>
    )
  }

  if (isRestoringSession || loading) {
    return <LoadingSkeleton />
  }

  if (error) {
    return (
      <div className="card p-6 text-center text-red-400">
        <p className="font-medium">Failed to load yield data</p>
        <p className="text-xs text-slate-500 mt-1">{error}</p>
        <button onClick={refresh} className="btn-secondary mt-4 text-sm">
          Retry
        </button>
      </div>
    )
  }

  const compoundingDeposits = deposits.filter(d => d.compoundFrequencySecs > 0)
  
  if (compoundingDeposits.length === 0) {
    return (
      <div className="card p-10 text-center">
        <div className="w-16 h-16 rounded-2xl bg-slate-700/40 mx-auto mb-4 flex items-center justify-center">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-8 h-8 text-slate-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
          </svg>
        </div>
        <p className="text-slate-400">No yield-bearing deposits found</p>
        <p className="text-slate-500 text-sm mt-2">Create a deposit with compound interest enabled to see yield analytics here.</p>
      </div>
    )
  }

  const aggregate = calculateAggregateYield(compoundingDeposits, currentTime)
  const priceData = getPrice('native')
  const xlmPriceUsd = priceData?.usd

  return (
    <div className="space-y-4 md:space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold">Yield Dashboard</h1>
          <p className="text-sm text-slate-400 mt-1">Track your compound interest earnings in real-time</p>
        </div>
        <div className="flex gap-2">
          <button onClick={refresh} className="btn-secondary text-xs px-3 py-2" disabled={loading}>
            {loading ? (
              <span className="w-3 h-3 border-2 border-current/30 border-t-current rounded-full animate-spin" />
            ) : (
              <svg viewBox="0 0 20 20" fill="currentColor" className="w-3.5 h-3.5">
                <path fillRule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clipRule="evenodd" />
              </svg>
            )}
            <span className="hidden sm:inline ml-1.5">Refresh</span>
          </button>
          <button onClick={() => exportYieldReport(compoundingDeposits, aggregate, currentTime)} className="btn-primary text-xs px-3 py-2">
            <svg viewBox="0 0 20 20" fill="currentColor" className="w-3.5 h-3.5">
              <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
            </svg>
            <span className="hidden sm:inline ml-1.5">Export Report</span>
          </button>
        </div>
      </div>

      {/* Aggregate Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard
          label="Total Principal"
          value={stroopsToXlm(aggregate.totalPrincipal)}
          unit="XLM"
          usdValue={xlmPriceUsd ? Number(stroopsToXlm(aggregate.totalPrincipal)) * xlmPriceUsd : undefined}
        />
        <StatCard
          label="Interest Earned"
          value={stroopsToXlm(aggregate.totalInterestEarned)}
          unit="XLM"
          usdValue={xlmPriceUsd ? Number(stroopsToXlm(aggregate.totalInterestEarned)) * xlmPriceUsd : undefined}
          accent="green"
        />
        <StatCard
          label="Current Balance"
          value={stroopsToXlm(aggregate.totalCurrentBalance)}
          unit="XLM"
          usdValue={xlmPriceUsd ? Number(stroopsToXlm(aggregate.totalCurrentBalance)) * xlmPriceUsd : undefined}
        />
        <StatCard
          label="Weighted APY"
          value={aggregate.weightedAPY.toFixed(2)}
          unit="%"
          accent="stellar"
        />
      </div>

      {/* Projected Yield at Unlock */}
      <div className="card p-4 md:p-5">
        <h3 className="font-semibold text-lg mb-3">Projected at Unlock</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-xs text-slate-400 mb-1">Total Amount</p>
            <p className="text-2xl font-bold text-slate-100">
              {stroopsToXlm(aggregate.totalProjectedAtUnlock)} <span className="text-base text-slate-400">XLM</span>
            </p>
            {xlmPriceUsd && (
              <p className="text-sm text-slate-500 mt-1">
                ${(Number(stroopsToXlm(aggregate.totalProjectedAtUnlock)) * xlmPriceUsd).toFixed(2)}
              </p>
            )}
          </div>
          <div>
            <p className="text-xs text-slate-400 mb-1">Total Interest</p>
            <p className="text-2xl font-bold text-green-400">
              +{stroopsToXlm(aggregate.totalProjectedInterest)} <span className="text-base text-green-400/70">XLM</span>
            </p>
            {xlmPriceUsd && (
              <p className="text-sm text-slate-500 mt-1">
                +${(Number(stroopsToXlm(aggregate.totalProjectedInterest)) * xlmPriceUsd).toFixed(2)}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Yield Breakdown by Deposit */}
      <div className="card p-4 md:p-5">
        <h3 className="font-semibold text-lg mb-4">Yield Breakdown by Deposit</h3>
        <div className="space-y-3">
          {compoundingDeposits.map(deposit => (
            <YieldDepositRow
              key={deposit.depositId}
              deposit={deposit}
              currentTime={currentTime}
              xlmPriceUsd={xlmPriceUsd}
              onSelect={() => setSelectedDeposit(deposit)}
            />
          ))}
        </div>
      </div>

      {/* Selected Deposit Details */}
      {selectedDeposit && (
        <YieldDetailCard
          deposit={selectedDeposit}
          currentTime={currentTime}
          xlmPriceUsd={xlmPriceUsd}
          onClose={() => setSelectedDeposit(null)}
        />
      )}
    </div>
  )
}

function StatCard({ label, value, unit, usdValue, accent }: {
  label: string
  value: string
  unit: string
  usdValue?: number
  accent?: 'green' | 'stellar'
}) {
  const valueClass =
    accent === 'green' ? 'text-green-400' :
    accent === 'stellar' ? 'text-stellar-400' :
    'text-slate-100'

  return (
    <div className="card p-3 md:p-4">
      <p className="text-xs text-slate-500 uppercase tracking-wide mb-1">{label}</p>
      <p className={`text-lg md:text-2xl font-bold ${valueClass}`}>
        {value} <span className="text-sm md:text-base opacity-70">{unit}</span>
      </p>
      {usdValue !== undefined && (
        <p className="text-xs text-slate-500 mt-1">${usdValue.toFixed(2)} USD</p>
      )}
    </div>
  )
}

function YieldDepositRow({ deposit, currentTime, xlmPriceUsd, onSelect }: {
  deposit: Deposit
  currentTime: number
  xlmPriceUsd?: number
  onSelect: () => void
}) {
  const summary = calculateYieldSummary(deposit, currentTime)
  const isXlm = deposit.token === CONFIG.NATIVE_TOKEN

  return (
    <button
      onClick={onSelect}
      className="w-full card p-3 hover:border-slate-600/80 transition-colors text-left"
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <div className="w-10 h-10 rounded-full bg-stellar-900/60 border border-stellar-700/40 flex items-center justify-center flex-shrink-0">
            <span className="text-stellar-400 font-bold text-xs">{isXlm ? 'XLM' : '?'}</span>
          </div>
          <div className="min-w-0 flex-1">
            <p className="font-semibold text-sm">
              Deposit #{deposit.depositId}
            </p>
            <p className="text-xs text-slate-400 truncate">
              {stroopsToXlm(summary.principal)} XLM principal
            </p>
          </div>
        </div>
        <div className="text-right flex-shrink-0">
          <p className="text-sm font-semibold text-green-400">
            +{stroopsToXlm(summary.interestEarned)}
          </p>
          <p className="text-xs text-slate-500">
            {summary.apy.toFixed(2)}% APY
          </p>
        </div>
      </div>
    </button>
  )
}

function YieldDetailCard({ deposit, currentTime, xlmPriceUsd, onClose }: {
  deposit: Deposit
  currentTime: number
  xlmPriceUsd?: number
  onClose: () => void
}) {
  const summary = calculateYieldSummary(deposit, currentTime)
  const comparison = compareYieldVsPenalty(deposit, currentTime)
  const projections = generateYieldProjections(deposit, currentTime)
  const isXlm = deposit.token === CONFIG.NATIVE_TOKEN

  return (
    <div className="card p-4 md:p-5">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-lg">Deposit #{deposit.depositId} Details</h3>
        <button onClick={onClose} className="text-slate-400 hover:text-slate-200 transition-colors">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Current Status */}
      <div className="grid grid-cols-2 md:grid-cols-3 gap-3 mb-4">
        <div className="bg-slate-900/50 rounded-lg p-3">
          <p className="text-xs text-slate-400 mb-1">Principal</p>
          <p className="font-semibold text-sm">{stroopsToXlm(summary.principal)} XLM</p>
          {xlmPriceUsd && (
            <p className="text-xs text-slate-500">${(Number(stroopsToXlm(summary.principal)) * xlmPriceUsd).toFixed(2)}</p>
          )}
        </div>
        <div className="bg-slate-900/50 rounded-lg p-3">
          <p className="text-xs text-slate-400 mb-1">Current Balance</p>
          <p className="font-semibold text-sm">{stroopsToXlm(summary.currentBalance)} XLM</p>
          {xlmPriceUsd && (
            <p className="text-xs text-slate-500">${(Number(stroopsToXlm(summary.currentBalance)) * xlmPriceUsd).toFixed(2)}</p>
          )}
        </div>
        <div className="bg-green-900/20 rounded-lg p-3 border border-green-800/30">
          <p className="text-xs text-green-400/80 mb-1">Interest Earned</p>
          <p className="font-semibold text-sm text-green-400">+{stroopsToXlm(summary.interestEarned)} XLM</p>
          {xlmPriceUsd && (
            <p className="text-xs text-green-600">+${(Number(stroopsToXlm(summary.interestEarned)) * xlmPriceUsd).toFixed(2)}</p>
          )}
        </div>
      </div>

      {/* Yield Projections */}
      {projections.length > 0 && (
        <div className="mb-4">
          <h4 className="font-medium text-sm mb-3">Yield Projections</h4>
          <div className="space-y-2">
            {projections.map((proj, idx) => (
              <div key={idx} className="flex items-center justify-between p-2 bg-slate-900/30 rounded">
                <span className="text-xs text-slate-400">{proj.label}</span>
                <div className="text-right">
                  <p className="text-sm font-medium">{stroopsToXlm(proj.projectedAmount)} XLM</p>
                  <p className="text-xs text-green-400">+{stroopsToXlm(proj.projectedInterest)} interest</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Yield vs Penalty Comparison */}
      {deposit.penaltyBps > 0 && (
        <div className="bg-orange-900/20 border border-orange-800/30 rounded-lg p-3">
          <h4 className="font-medium text-sm text-orange-400 mb-2">Early Exit Analysis</h4>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-slate-400">Current Penalty ({formatBps(deposit.penaltyBps)})</span>
              <span className="text-orange-400">-{stroopsToXlm(comparison.penaltyAmount)} XLM</span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-400">Net if exit now</span>
              <span className="font-medium">{stroopsToXlm(comparison.netIfExitNow)} XLM</span>
            </div>
            <div className="flex justify-between border-t border-orange-800/30 pt-2">
              <span className="text-slate-400">Opportunity cost</span>
              <span className="text-orange-400">-{stroopsToXlm(comparison.opportunityCost)} XLM</span>
            </div>
            {comparison.breakEvenDays > 0 && (
              <p className="text-slate-500 text-xs mt-2">
                Interest will cover penalty in ~{comparison.breakEvenDays} days
              </p>
            )}
          </div>
        </div>
      )}

      {/* Additional Info */}
      <div className="mt-4 pt-4 border-t border-slate-700/60 grid grid-cols-2 gap-3 text-xs">
        <div>
          <span className="text-slate-500">APY</span>
          <p className="font-medium text-stellar-400">{summary.apy.toFixed(2)}%</p>
        </div>
        <div>
          <span className="text-slate-500">Days until unlock</span>
          <p className="font-medium">{summary.daysUntilUnlock}</p>
        </div>
        <div>
          <span className="text-slate-500">Compound frequency</span>
          <p className="font-medium">{formatCompoundFrequency(deposit.compoundFrequencySecs)}</p>
        </div>
        <div>
          <span className="text-slate-500">Token</span>
          <p className="font-medium font-mono text-xs">{isXlm ? 'XLM' : shortAddr(deposit.token)}</p>
        </div>
      </div>
    </div>
  )
}

function formatCompoundFrequency(secs: number): string {
  if (secs === 0) return 'None'
  if (secs === 60) return 'Every minute'
  if (secs === 3600) return 'Hourly'
  if (secs === 86400) return 'Daily'
  if (secs < 3600) return `Every ${secs / 60}m`
  if (secs < 86400) return `Every ${secs / 3600}h`
  return `Every ${secs / 86400}d`
}

function exportYieldReport(
  deposits: Deposit[],
  aggregate: AggregateYield,
  currentTime: number,
) {
  const timestamp = new Date().toISOString()
  const lines: string[] = []
  
  lines.push('SAFE-HAVEN YIELD REPORT')
  lines.push(`Generated: ${timestamp}`)
  lines.push('')
  
  lines.push('=== AGGREGATE SUMMARY ===')
  lines.push(`Total Principal: ${stroopsToXlm(aggregate.totalPrincipal)} XLM`)
  lines.push(`Total Current Balance: ${stroopsToXlm(aggregate.totalCurrentBalance)} XLM`)
  lines.push(`Total Interest Earned: ${stroopsToXlm(aggregate.totalInterestEarned)} XLM`)
  lines.push(`Total Projected at Unlock: ${stroopsToXlm(aggregate.totalProjectedAtUnlock)} XLM`)
  lines.push(`Total Projected Interest: ${stroopsToXlm(aggregate.totalProjectedInterest)} XLM`)
  lines.push(`Weighted APY: ${aggregate.weightedAPY.toFixed(2)}%`)
  lines.push(`Active Compounding Deposits: ${aggregate.activeCompoundingDeposits}`)
  lines.push('')
  
  lines.push('=== DEPOSIT BREAKDOWN ===')
  deposits.forEach(deposit => {
    const summary = calculateYieldSummary(deposit, currentTime)
    lines.push(`\nDeposit #${deposit.depositId}:`)
    lines.push(`  Principal: ${stroopsToXlm(summary.principal)} XLM`)
    lines.push(`  Current Balance: ${stroopsToXlm(summary.currentBalance)} XLM`)
    lines.push(`  Interest Earned: ${stroopsToXlm(summary.interestEarned)} XLM`)
    lines.push(`  Projected at Unlock: ${stroopsToXlm(summary.projectedAtUnlock)} XLM`)
    lines.push(`  Projected Interest: ${stroopsToXlm(summary.projectedInterestAtUnlock)} XLM`)
    lines.push(`  APY: ${summary.apy.toFixed(2)}%`)
    lines.push(`  Days Until Unlock: ${summary.daysUntilUnlock}`)
    lines.push(`  Compound Frequency: ${formatCompoundFrequency(deposit.compoundFrequencySecs)}`)
  })
  
  const blob = new Blob([lines.join('\n')], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `safe-haven-yield-report-${Date.now()}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        {[1, 2, 3, 4].map(i => (
          <div key={i} className="card p-4 animate-pulse">
            <div className="h-3 bg-slate-700/60 rounded w-1/2 mb-2" />
            <div className="h-7 bg-slate-700/40 rounded w-2/3" />
          </div>
        ))}
      </div>
      <div className="card p-5 animate-pulse">
        <div className="h-5 bg-slate-700/60 rounded w-1/4 mb-4" />
        <div className="space-y-3">
          {[1, 2].map(i => (
            <div key={i} className="h-16 bg-slate-700/40 rounded" />
          ))}
        </div>
      </div>
    </div>
  )
}
