/**
 * Gas optimization suggestions component.
 * Provides recommendations for batching deposits/withdrawals and other optimizations.
 */

interface GasOptimizationSuggestion {
  type: 'batch_deposits' | 'batch_withdrawals' | 'consolidate_small' | 'cancel_early'
  title: string
  description: string
  savings: string
}

export interface BatchSuggestionsProps {
  depositCount: number
  readyToWithdraw: number
  smallDepositsUnder: number
  onBatchDeposits?: () => void
  onBatchWithdraw?: () => void
}

/**
 * Component to show gas optimization suggestions and batch recommendations.
 */
export function BatchSuggestions({
  depositCount,
  readyToWithdraw,
  smallDepositsUnder,
}: BatchSuggestionsProps) {
  const suggestions: GasOptimizationSuggestion[] = []

  // Suggestion 1: Batch multiple small deposits
  if (depositCount > 2) {
    suggestions.push({
      type: 'batch_deposits',
      title: 'Batch Multiple Deposits',
      description: `You have ${depositCount} active deposits. Consider combining future deposits into fewer transactions to reduce gas fees.`,
      savings: `~${Math.min(40, depositCount * 5)}% savings`,
    })
  }

  // Suggestion 2: Batch withdrawals
  if (readyToWithdraw > 1) {
    suggestions.push({
      type: 'batch_withdrawals',
      title: 'Batch Withdrawals',
      description: `You have ${readyToWithdraw} deposits ready to withdraw. Submitting them separately will cost more gas than grouping them.`,
      savings: `~${Math.min(35, readyToWithdraw * 10)}% savings`,
    })
  }

  // Suggestion 3: Consolidate small deposits
  if (smallDepositsUnder > 0) {
    suggestions.push({
      type: 'consolidate_small',
      title: 'Consolidate Small Deposits',
      description: `You have ${smallDepositsUnder} deposits with small amounts. Consolidating them into fewer vaults can reduce ongoing storage costs.`,
      savings: `~20% storage savings`,
    })
  }

  if (suggestions.length === 0) {
    return null
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 mb-3">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5 text-green-400">
          <path d="M9 12l2 2 4-4" strokeLinecap="round" strokeLinejoin="round" />
          <circle cx="12" cy="12" r="10" />
        </svg>
        <h3 className="font-semibold text-slate-200">Gas Optimization Tips</h3>
      </div>

      {suggestions.map((suggestion) => (
        <div
          key={suggestion.type}
          className="bg-slate-800/60 border border-green-700/30 rounded-lg p-3.5 space-y-2 hover:border-green-600/50 transition-colors"
        >
          <div className="flex items-start justify-between gap-2">
            <div className="flex-1">
              <h4 className="font-medium text-slate-200">{suggestion.title}</h4>
              <p className="text-xs text-slate-400 mt-1">{suggestion.description}</p>
            </div>
            <span className="text-xs font-medium text-green-400 whitespace-nowrap ml-2">
              {suggestion.savings}
            </span>
          </div>
        </div>
      ))}

      <div className="bg-slate-800/30 rounded-lg p-3 text-xs text-slate-400 border border-slate-700/30">
        <p>
          <strong className="text-slate-300">💡 Tip:</strong> Gas fees are calculated based on contract execution and storage costs. Batching operations reduces the per-transaction overhead.
        </p>
      </div>
    </div>
  )
}

export default BatchSuggestions
