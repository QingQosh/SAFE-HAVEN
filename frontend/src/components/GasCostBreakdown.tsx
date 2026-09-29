import { useState } from 'react'
import type { GasCostBreakdown } from '../hooks/useGasEstimator'

interface GasCostBreakdownProps {
  breakdown: GasCostBreakdown
  isWarning?: boolean
  threshold?: number
}

/**
 * Component to display gas cost breakdown with educational tooltips.
 * Shows base fee, execution, storage, and total cost with USD estimate.
 */
export function GasCostBreakdown({ breakdown, isWarning = false, threshold = 10_000_000 }: GasCostBreakdownProps) {
  const [showTooltip, setShowTooltip] = useState<string | null>(null)

  const costExceedsThreshold = breakdown.totalCost > threshold
  const showWarning = isWarning || costExceedsThreshold

  return (
    <div className={`rounded-xl p-4 space-y-3 text-sm ${showWarning ? 'bg-yellow-900/30 border border-yellow-700/40' : 'bg-slate-800/60 border border-slate-700/40'}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-1">
        <h3 className={`font-semibold flex items-center gap-2 ${showWarning ? 'text-yellow-400' : 'text-slate-200'}`}>
          {showWarning && (
            <svg viewBox="0 0 24 24" fill="currentColor" className="w-4 h-4">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
            </svg>
          )}
          Gas Cost Breakdown
        </h3>
        <span className="text-slate-400 text-xs">Estimated</span>
      </div>

      {/* Cost rows */}
      <div className="space-y-2">
        {/* Base fee */}
        <CostRow
          label="Base fee"
          stroops={breakdown.baseFee}
          tooltip="One-time overhead for including this transaction on the ledger. Fixed per transaction type."
          showTooltip={showTooltip === 'base'}
          onTooltip={(show) => setShowTooltip(show ? 'base' : null)}
        />

        {/* Execution */}
        <CostRow
          label="Execution cost"
          stroops={breakdown.executionCost}
          tooltip="Cost to execute contract code. Varies based on the number of operations and complexity."
          showTooltip={showTooltip === 'execution'}
          onTooltip={(show) => setShowTooltip(show ? 'execution' : null)}
        />

        {/* Storage */}
        <CostRow
          label="Storage cost"
          stroops={breakdown.storageCost}
          tooltip="Cost to store/modify data on the ledger. Each byte added or modified incurs a fee."
          showTooltip={showTooltip === 'storage'}
          onTooltip={(show) => setShowTooltip(show ? 'storage' : null)}
        />

        {/* Divider */}
        <div className="h-px bg-slate-700/50 my-1" />

        {/* Total */}
        <div className="flex items-center justify-between pt-1">
          <span className="font-medium text-slate-300 flex items-center gap-2">
            <span>Total</span>
            <button
              onMouseEnter={() => setShowTooltip('total')}
              onMouseLeave={() => setShowTooltip(null)}
              className="text-slate-500 hover:text-slate-400 transition-colors"
              title="Total gas cost"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-3.5 h-3.5">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 16v-4M12 8h.01" strokeLinecap="round" />
              </svg>
            </button>
          </span>
          <div className="text-right">
            <div className="font-semibold text-slate-100">
              {formatStroops(breakdown.totalCost)}
            </div>
            <div className="text-xs text-slate-400">
              ~${breakdown.totalCostInUsd.toFixed(6)} USD
            </div>
          </div>
        </div>
      </div>

      {/* Warning message */}
      {costExceedsThreshold && (
        <div className="bg-yellow-900/50 border border-yellow-700/50 rounded-lg p-2.5 mt-3">
          <p className="text-yellow-300 text-xs flex items-start gap-2">
            <span className="shrink-0 mt-0.5">⚠️</span>
            <span>
              This transaction has a higher-than-typical gas cost. Consider checking if there are optimization
              opportunities or batching multiple operations.
            </span>
          </p>
        </div>
      )}

      {/* Tooltip */}
      {showTooltip === 'total' && (
        <div className="bg-slate-700 text-slate-100 text-xs rounded p-2 mt-1 border border-slate-600">
          Total fee required for this transaction. This includes all base, execution, and storage costs.
        </div>
      )}
    </div>
  )
}

interface CostRowProps {
  label: string
  stroops: number
  tooltip: string
  showTooltip: boolean
  onTooltip: (show: boolean) => void
}

function CostRow({ label, stroops, tooltip, showTooltip, onTooltip }: CostRowProps) {
  return (
    <div>
      <div className="flex items-center justify-between">
        <span className="text-slate-400 flex items-center gap-1.5">
          {label}
          <button
            onMouseEnter={() => onTooltip(true)}
            onMouseLeave={() => onTooltip(false)}
            className="text-slate-600 hover:text-slate-500 transition-colors"
            title={tooltip}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-3.5 h-3.5">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4M12 8h.01" strokeLinecap="round" />
            </svg>
          </button>
        </span>
        <span className="font-medium text-slate-200">{formatStroops(stroops)}</span>
      </div>
      {showTooltip && (
        <div className="bg-slate-700 text-slate-100 text-xs rounded p-2 mt-1 border border-slate-600">
          {tooltip}
        </div>
      )}
    </div>
  )
}

function formatStroops(stroops: number): string {
  if (stroops < 1000) {
    return `${stroops} stroops`
  } else if (stroops < 1_000_000) {
    return `${(stroops / 1000).toFixed(2)}k stroops`
  } else {
    return `${(stroops / 1_000_000).toFixed(2)}M stroops`
  }
}
