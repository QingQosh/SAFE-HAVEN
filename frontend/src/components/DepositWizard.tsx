import { useState } from 'react'
import { formatBps, formatDuration, amountToBaseUnits, baseUnitsToAmount } from '../lib/format'
import type { ContractInfo } from '../App'

interface DepositWizardProps {
  contractInfo: ContractInfo
  onSubmit: (params: {
    amount: string
    unlockTimestamp: number
    penaltyBps: number
  }) => Promise<void>
  isSubmitting?: boolean
  error?: string
}

type WizardStep = 'amount' | 'duration' | 'penalty' | 'review'

const LOCK_PRESETS = [
  { label: '1 Week', days: 7, secondsOffset: 7 * 24 * 60 * 60 },
  { label: '1 Month', days: 30, secondsOffset: 30 * 24 * 60 * 60 },
  { label: '3 Months', days: 90, secondsOffset: 90 * 24 * 60 * 60 },
  { label: '6 Months', days: 180, secondsOffset: 180 * 24 * 60 * 60 },
  { label: '1 Year', days: 365, secondsOffset: 365 * 24 * 60 * 60 },
]

const PENALTY_PRESETS = [
  { label: 'No penalty', bps: 0, color: 'slate' },
  { label: '5% penalty', bps: 500, color: 'orange' },
  { label: '10% penalty', bps: 1000, color: 'amber' },
  { label: '25% penalty', bps: 2500, color: 'red' },
]

export function DepositWizard({ contractInfo, onSubmit, isSubmitting, error }: DepositWizardProps) {
  const [step, setStep] = useState<WizardStep>('amount')
  const [amount, setAmount] = useState('')
  const [selectedPreset, setSelectedPreset] = useState<typeof LOCK_PRESETS[0] | null>(null)
  const [customDays, setCustomDays] = useState('')
  const [penaltyBps, setPenaltyBps] = useState(0)

  const nowSecs = Math.floor(Date.now() / 1000)
  const lockDurationSecs = selectedPreset?.secondsOffset || (customDays ? parseInt(customDays, 10) * 24 * 60 * 60 : 0)
  const unlockTimestamp = nowSecs + lockDurationSecs

  const amountNum = parseFloat(amount)
  const isValidAmount = amount && !isNaN(amountNum) && amountNum > 0

  const isValidDuration = lockDurationSecs > 0 && 
                         lockDurationSecs >= 60 && 
                         lockDurationSecs <= contractInfo.maxLockSecs

  const canProceedAmount = isValidAmount
  const canProceedDuration = isValidDuration
  const canSubmit = isValidAmount && isValidDuration

  function handleNextFromAmount() {
    if (canProceedAmount) setStep('duration')
  }

  function handleNextFromDuration() {
    if (canProceedDuration) setStep('penalty')
  }

  function handleNextFromPenalty() {
    setStep('review')
  }

  function handleBack() {
    if (step === 'duration') setStep('amount')
    else if (step === 'penalty') setStep('duration')
    else if (step === 'review') setStep('penalty')
  }

  async function handleFinalSubmit() {
    if (!canSubmit) return
    await onSubmit({
      amount,
      unlockTimestamp,
      penaltyBps,
    })
  }

  return (
    <div className="card p-4 md:p-6 max-w-lg mx-auto">
      {/* Progress indicator */}
      <div className="flex gap-2 mb-6 justify-center">
        {['amount', 'duration', 'penalty', 'review'].map((s, i) => (
          <div
            key={s}
            className={`h-1 flex-1 rounded-full transition-colors ${
              s === step
                ? 'bg-stellar-600'
                : ['amount', 'duration', 'penalty', 'review'].indexOf(s) < ['amount', 'duration', 'penalty', 'review'].indexOf(step)
                  ? 'bg-green-600'
                  : 'bg-slate-700'
            }`}
          />
        ))}
      </div>

      {/* Step: Amount */}
      {step === 'amount' && (
        <div className="space-y-4">
          <div>
            <h2 className="font-semibold text-lg mb-1">How much are you locking?</h2>
            <p className="text-sm text-slate-400 mb-4">XLM (Stellar native token)</p>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-300 mb-2">Amount</label>
            <div className="relative">
              <input
                type="number"
                min="0"
                step="0.0000001"
                placeholder="Enter amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="w-full input pr-12"
                disabled={isSubmitting}
                autoFocus
              />
              <span className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 font-medium text-sm">
                XLM
              </span>
            </div>
            {amount && !isValidAmount && (
              <p className="text-xs text-red-400 mt-1">Amount must be greater than 0</p>
            )}
            {amount && amountNum > baseUnitsToAmount(contractInfo.maxDeposit, 7) && (
              <p className="text-xs text-red-400 mt-1">
                Max deposit: {baseUnitsToAmount(contractInfo.maxDeposit, 7)} XLM
              </p>
            )}
          </div>

          <div className="bg-slate-800/40 border border-slate-700/40 rounded-lg p-3 text-xs text-slate-400">
            <p className="font-medium mb-1">💡 Tip</p>
            <p>Lock funds to enforce HODL discipline or time-gate access for escrow.</p>
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleNextFromAmount}
              disabled={!canProceedAmount || isSubmitting}
              className="flex-1 btn-primary py-2 text-sm"
            >
              Next →
            </button>
          </div>
        </div>
      )}

      {/* Step: Duration */}
      {step === 'duration' && (
        <div className="space-y-4">
          <div>
            <h2 className="font-semibold text-lg mb-1">How long should it be locked?</h2>
            <p className="text-sm text-slate-400 mb-4">{amount} XLM will be locked</p>
          </div>

          <div>
            <p className="text-xs font-medium text-slate-300 mb-2">Quick presets</p>
            <div className="grid grid-cols-2 gap-2">
              {LOCK_PRESETS.map((preset) => (
                <button
                  key={preset.label}
                  onClick={() => setSelectedPreset(preset)}
                  className={`p-3 rounded-lg border text-sm font-medium transition-all ${
                    selectedPreset?.label === preset.label
                      ? 'bg-stellar-600 border-stellar-500 text-white'
                      : 'bg-slate-800/40 border-slate-700/40 text-slate-300 hover:border-slate-600'
                  }`}
                  disabled={isSubmitting}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>

          <div>
            <p className="text-xs font-medium text-slate-300 mb-2">Or enter custom days</p>
            <input
              type="number"
              min="1"
              placeholder="E.g., 60"
              value={customDays}
              onChange={(e) => {
                setCustomDays(e.target.value)
                setSelectedPreset(null)
              }}
              className="w-full input"
              disabled={isSubmitting}
            />
            {customDays && !isValidDuration && (
              <p className="text-xs text-red-400 mt-1">
                Lock must be 1 minute – {Math.floor(contractInfo.maxLockSecs / (24 * 60 * 60))} days
              </p>
            )}
          </div>

          {isValidDuration && (
            <div className="bg-slate-800/40 border border-slate-700/40 rounded-lg p-3 text-xs">
              <p className="font-medium text-slate-300 mb-1">🔒 Lock summary</p>
              <p className="text-slate-400">
                Your funds will unlock on{' '}
                <span className="font-mono text-slate-200">
                  {new Date(unlockTimestamp * 1000).toLocaleDateString()} at{' '}
                  {new Date(unlockTimestamp * 1000).toLocaleTimeString()}
                </span>
              </p>
            </div>
          )}

          <div className="flex gap-2">
            <button
              onClick={handleBack}
              disabled={isSubmitting}
              className="px-4 py-2 bg-slate-800/40 border border-slate-700/40 rounded-lg hover:bg-slate-800/60 transition-colors text-sm font-medium"
            >
              ← Back
            </button>
            <button
              onClick={handleNextFromDuration}
              disabled={!canProceedDuration || isSubmitting}
              className="flex-1 btn-primary py-2 text-sm"
            >
              Next →
            </button>
          </div>
        </div>
      )}

      {/* Step: Penalty */}
      {step === 'penalty' && (
        <div className="space-y-4">
          <div>
            <h2 className="font-semibold text-lg mb-1">Set early exit penalty</h2>
            <p className="text-sm text-slate-400 mb-4">Optional: penalize yourself for breaking the lock</p>
          </div>

          <div className="space-y-2">
            {PENALTY_PRESETS.map((preset) => (
              <button
                key={preset.label}
                onClick={() => setPenaltyBps(preset.bps)}
                className={`w-full p-3 rounded-lg border text-left text-sm font-medium transition-all ${
                  penaltyBps === preset.bps
                    ? `bg-stellar-600 border-stellar-500 text-white`
                    : `bg-slate-800/40 border-slate-700/40 text-slate-300 hover:border-slate-600`
                }`}
                disabled={isSubmitting}
              >
                <div className="flex justify-between">
                  <span>{preset.label}</span>
                  {penaltyBps === preset.bps && <span>✓</span>}
                </div>
              </button>
            ))}
          </div>

          {penaltyBps > 0 && (
            <div className="bg-amber-900/30 border border-amber-700/40 rounded-lg p-3 text-xs">
              <p className="font-medium text-amber-300 mb-1">⚠️ Early exit penalty</p>
              <p className="text-amber-200">
                If you withdraw before {formatDuration(lockDurationSecs)}, you'll forfeit {formatBps(penaltyBps)}.
              </p>
            </div>
          )}

          <div className="flex gap-2">
            <button
              onClick={handleBack}
              disabled={isSubmitting}
              className="px-4 py-2 bg-slate-800/40 border border-slate-700/40 rounded-lg hover:bg-slate-800/60 transition-colors text-sm font-medium"
            >
              ← Back
            </button>
            <button
              onClick={handleNextFromPenalty}
              disabled={isSubmitting}
              className="flex-1 btn-primary py-2 text-sm"
            >
              Review →
            </button>
          </div>
        </div>
      )}

      {/* Step: Review */}
      {step === 'review' && (
        <div className="space-y-4">
          <div>
            <h2 className="font-semibold text-lg mb-1">Confirm your vault</h2>
            <p className="text-sm text-slate-400 mb-4">Review before locking</p>
          </div>

          <div className="bg-slate-800/60 border border-slate-700/40 rounded-lg p-4 space-y-3">
            <div className="flex justify-between items-center pb-3 border-b border-slate-700/40">
              <span className="text-sm text-slate-400">Amount</span>
              <span className="font-semibold text-lg text-slate-100">{amount} XLM</span>
            </div>

            <div className="flex justify-between items-center pb-3 border-b border-slate-700/40">
              <span className="text-sm text-slate-400">Lock duration</span>
              <span className="font-semibold text-slate-100">{formatDuration(lockDurationSecs)}</span>
            </div>

            <div className="flex justify-between items-center pb-3 border-b border-slate-700/40">
              <span className="text-sm text-slate-400">Unlock date</span>
              <span className="font-mono text-sm text-slate-100">
                {new Date(unlockTimestamp * 1000).toLocaleDateString()}
              </span>
            </div>

            {penaltyBps > 0 && (
              <div className="flex justify-between items-center">
                <span className="text-sm text-slate-400">Early exit penalty</span>
                <span className="font-semibold text-orange-400">{formatBps(penaltyBps)}</span>
              </div>
            )}
          </div>

          {error && (
            <div className="bg-red-900/30 border border-red-700/40 rounded-lg p-3 text-xs text-red-400">
              {error}
            </div>
          )}

          <div className="flex gap-2">
            <button
              onClick={handleBack}
              disabled={isSubmitting}
              className="px-4 py-2 bg-slate-800/40 border border-slate-700/40 rounded-lg hover:bg-slate-800/60 transition-colors text-sm font-medium"
            >
              ← Back
            </button>
            <button
              onClick={handleFinalSubmit}
              disabled={!canSubmit || isSubmitting}
              className="flex-1 btn-primary py-2 text-sm"
            >
              {isSubmitting ? 'Locking…' : '🔒 Lock Now'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
