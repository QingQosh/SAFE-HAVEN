import { useState, useEffect } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import { TxStatusBadge } from './TxStatusBadge'
import { SubmitTimeoutBanner } from './SubmitTimeoutBanner'
import { useSubmitTimeout, TimeoutError } from '../hooks/useSubmitTimeout'
import { buildDepositByLedger, submitTx, getTokenDecimals, getTokenMetadata, getLedgerSequence } from '../lib/stellar'
import { 
  baseUnitsToAmount, 
  formatBps, 
  isValidContractAddress, 
  validateTokenAddress, 
  amountToBaseUnits 
} from '../lib/format'
import type { TxStatus } from '../types'
import type { ContractInfo } from '../App'
import { CONFIG } from '../config'

interface LockByLedgerFormProps {
  contractInfo: ContractInfo
  onSuccess: () => void
}

const MIN_LOCK_LEDGERS = 12

export function LockByLedgerForm({ contractInfo, onSuccess }: LockByLedgerFormProps) {
  const { wallet, isRestoringSession, signTransaction } = useWallet()

  const [tokenAddress, setTokenAddress] = useState<string>(CONFIG.NATIVE_TOKEN)
  const [amount, setAmount] = useState('')
  const [unlockLedger, setUnlockLedger] = useState('')
  const [penaltyBps, setPenaltyBps] = useState('0')

  const [currentLedger, setCurrentLedger] = useState<number>(0)
  const [ledgerLoading, setLedgerLoading] = useState(false)

  const [txStatus, setTxStatus] = useState<TxStatus>('idle')
  const [txHash, setTxHash] = useState<string | undefined>()
  const [txError, setTxError] = useState<string | undefined>()

  // Whether the last submission attempt timed out (drives the banner retry UI).
  const [timedOut, setTimedOut] = useState(false)

  // Submission timeout — 2-minute countdown with 30-second warning.
  const submitTimeout = useSubmitTimeout({
    onTimeout: (elapsedMs) => {
      console.warn(`[LockByLedgerForm] Submission timed out after ${elapsedMs}ms`)
    },
    onWarning: () => {
      toast('Submission is taking longer than expected…', { icon: '⚠️', duration: 4000 })
    },
  })

  // Token decimals state — defaults to 7 (XLM) but updates when token changes
  const [tokenDecimals, setTokenDecimals] = useState<number>(7)
  const [decimalsLoading, setDecimalsLoading] = useState(false)

  // Token metadata state for verification
  const [tokenMetadata, setTokenMetadata] = useState<{ name: string; symbol: string } | null>(null)
  const [tokenAddressError, setTokenAddressError] = useState<string>('')

  // Fetch current ledger sequence on component mount
  useEffect(() => {
    const fetchLedger = async () => {
      setLedgerLoading(true)
      const ledger = await getLedgerSequence()
      setCurrentLedger(ledger)
      setLedgerLoading(false)
    }
    fetchLedger()
  }, [])

  // Fetch token decimals when token address changes
  useEffect(() => {
    if (!tokenAddress || tokenAddress === CONFIG.NATIVE_TOKEN) {
      setTokenDecimals(7)
      setDecimalsLoading(false)
      setTokenMetadata(null)
      setTokenAddressError('')
      return
    }

    // Validate format first
    const validation = validateTokenAddress(tokenAddress)
    if (!validation.valid) {
      setTokenAddressError(validation.message)
      setTokenDecimals(7)
      setTokenMetadata(null)
      setDecimalsLoading(false)
      return
    }

    setTokenAddressError('')
    setDecimalsLoading(true)

    // Fetch both decimals and metadata in parallel
    Promise.all([getTokenDecimals(tokenAddress), getTokenMetadata(tokenAddress)])
      .then(([decimals, metadata]) => {
        if (decimals !== null) {
          setTokenDecimals(decimals)
        } else {
          setTokenDecimals(7)
        }
        if (metadata) {
          setTokenMetadata(metadata)
        }
        setDecimalsLoading(false)
      })
      .catch(() => {
        setTokenDecimals(7)
        setTokenMetadata(null)
        setDecimalsLoading(false)
      })
  }, [tokenAddress])

  // Derived validation
  const amountNum = parseFloat(amount)
  const penaltyBpsNum = parseInt(penaltyBps, 10)
  const unlockLedgerNum = parseInt(unlockLedger, 10)
  const amountInBaseUnits = amount ? amountToBaseUnits(amount, tokenDecimals) : 0n

  const ledgerGap = currentLedger > 0 && unlockLedgerNum ? unlockLedgerNum - currentLedger : 0
  const estimatedSecondsRemaining = ledgerGap * 5 // 5 seconds per ledger

  const errors = {
    amount:
      !amount
        ? ''
        : isNaN(amountNum) || amountNum <= 0
          ? 'Amount must be > 0'
          : amountInBaseUnits > contractInfo.maxDeposit
            ? `Max: ${baseUnitsToAmount(contractInfo.maxDeposit, tokenDecimals)} tokens`
            : '',
    unlockLedger:
      !unlockLedger
        ? ''
        : isNaN(unlockLedgerNum)
          ? 'Must be a valid number'
          : currentLedger === 0
            ? 'Loading current ledger…'
            : unlockLedgerNum <= currentLedger
              ? 'Must be in the future'
              : ledgerGap < MIN_LOCK_LEDGERS
                ? `Minimum gap: ${MIN_LOCK_LEDGERS} ledgers (current: ${ledgerGap})`
                : '',
    penalty: isNaN(penaltyBpsNum) || penaltyBpsNum < 0 || penaltyBpsNum > 10_000 ? '0–10000 only' : '',
  }

  const isValid =
    amount &&
    unlockLedger &&
    !errors.amount &&
    !errors.unlockLedger &&
    !errors.penalty &&
    !contractInfo.paused &&
    !decimalsLoading &&
    currentLedger > 0

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!wallet || !isValid) return

    setTxStatus('signing')
    setTxError(undefined)
    setTxHash(undefined)
    setTimedOut(false)

    // Start the 2-minute countdown.
    const timeoutRace = submitTimeout.start()

    try {
      const amountBaseUnits = amountToBaseUnits(amount, tokenDecimals)
      const xdr = await buildDepositByLedger(
        wallet.address,
        tokenAddress,
        amountBaseUnits,
        unlockLedgerNum,
        penaltyBpsNum,
      )
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)

      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setTxStatus('submitting')

        // Race the submission against the timeout.
        const result = await Promise.race([submitTx(sigResult.xdr), timeoutRace.then(() => null)])

        if (result === null) {
          // Safety net — TimeoutError is normally thrown, but handle null too.
          setTxStatus('error')
          setTimedOut(true)
          setTxError('Submission timed out. Please try again.')
          return
        }

        submitTimeout.cancel()

        if (result.success) {
          setTxStatus('success')
          setTxHash(result.txHash)
          toast.success('Deposit successful! Your tokens are locked by ledger.')
          setAmount('')
          setUnlockLedger('')
          setPenaltyBps('0')
          setTimeout(onSuccess, 1500)
        } else {
          setTxStatus('error')
          setTxError(result.error)
          toast.error(result.error ?? 'Deposit failed')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        submitTimeout.cancel()
        setTxStatus('idle')
        return
      } else {
        // Signing error: already toasted, but still reset state
        submitTimeout.cancel()
        setTxStatus('idle')
      }
    } catch (e) {
      submitTimeout.cancel()
      if (e instanceof TimeoutError) {
        setTxStatus('error')
        setTimedOut(true)
        setTxError('Submission timed out. Please try again.')
        toast.error('Deposit submission timed out. Your wallet wasn\'t charged — please try again.')
      } else {
        const msg = e instanceof Error ? e.message : 'Unexpected error'
        setTxStatus('error')
        setTxError(msg)
        toast.error(msg)
      }
    }
  }

  const isPending = txStatus === 'signing' || txStatus === 'submitting' || txStatus === 'confirming'

  // Retry the last submission with the same form data (no re-entry needed).
  function handleTimeoutRetry() {
    setTimedOut(false)
    setTxStatus('idle')
    setTxError(undefined)
  }

  if (!wallet && !isRestoringSession) {
    return (
      <div className="card p-10 text-center">
        <p className="text-slate-400">Connect your wallet to deposit tokens.</p>
      </div>
    )
  }

  return (
    <div className="max-w-lg">
      <div className="card p-6">
        <h2 className="font-semibold text-lg mb-1">Lock tokens until a ledger sequence</h2>
        <p className="text-sm text-slate-400 mb-6">
          Tokens will be transferred to the contract and locked until a specific ledger number is reached.
          Stellar produces roughly one ledger every 5 seconds.
        </p>

        {contractInfo.paused && (
          <div className="mb-5 p-3 rounded-xl bg-red-900/30 border border-red-700/40 text-red-400 text-sm">
            ⚠️ Contract is currently paused. Deposits are disabled.
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-5" noValidate>
          {/* Token */}
          <div>
            <label className="label">Token contract address</label>
            <div className="relative">
              <input
                className={`input ${
                  tokenAddressError
                    ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500'
                    : tokenAddress && isValidContractAddress(tokenAddress) && !decimalsLoading
                      ? 'border-green-500/50 focus:ring-green-500/30 focus:border-green-500'
                      : ''
                }`}
                type="text"
                value={tokenAddress}
                onChange={(e) => setTokenAddress(e.target.value.trim())}
                placeholder="CDLZFC3…"
                disabled={isPending}
              />
              {decimalsLoading && (
                <span className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 text-xs">
                  Verifying…
                </span>
              )}
              {!decimalsLoading && tokenAddress && isValidContractAddress(tokenAddress) && !tokenAddressError && (
                <span className="absolute right-4 top-1/2 -translate-y-1/2 text-green-400 text-xs">
                  ✓
                </span>
              )}
            </div>
            {tokenAddressError ? (
              <p className="text-xs text-red-400 mt-1">{tokenAddressError}</p>
            ) : (
              <p className="text-xs text-slate-500 mt-1">
                {tokenAddress === CONFIG.NATIVE_TOKEN
                  ? 'Native XLM token (7 decimals)'
                  : tokenMetadata
                    ? `${tokenMetadata.symbol} - ${tokenMetadata.name} (${tokenDecimals} decimals)`
                    : `Custom token (${tokenDecimals} decimals${decimalsLoading ? ', verifying…' : ''})`}
              </p>
            )}
          </div>

          {/* Amount */}
          <div>
            <label className="label">Amount</label>
            <div className="relative">
              <input
                className={`input pr-14 ${errors.amount ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
                type="number"
                min="0"
                step={`0.${'0'.repeat(Math.max(0, tokenDecimals - 1))}1`}
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0"
                disabled={isPending || decimalsLoading}
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 text-sm font-medium pointer-events-none">
                {tokenAddress === CONFIG.NATIVE_TOKEN ? 'XLM' : 'tokens'}
              </span>
            </div>
            {errors.amount && <p className="text-xs text-red-400 mt-1">{errors.amount}</p>}
          </div>

          {/* Current Ledger Display */}
          <div>
            <label className="label">
              Current ledger
              {ledgerLoading && <span className="ml-1 text-slate-500 normal-case text-xs">loading…</span>}
            </label>
            <div className="input bg-slate-800/40 border-slate-700/50 text-slate-300 cursor-default">
              {currentLedger > 0 ? currentLedger.toLocaleString() : '—'}
            </div>
            <p className="text-xs text-slate-500 mt-1">Automatically fetched from the Stellar network</p>
          </div>

          {/* Unlock Ledger */}
          <div>
            <label className="label">
              Unlock at ledger number
              <span className="ml-1 text-slate-500 normal-case">
                — min {currentLedger + MIN_LOCK_LEDGERS}
              </span>
            </label>
            <input
              className={`input ${errors.unlockLedger ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
              type="number"
              min={currentLedger + MIN_LOCK_LEDGERS}
              value={unlockLedger}
              onChange={(e) => setUnlockLedger(e.target.value)}
              placeholder={currentLedger > 0 ? (currentLedger + MIN_LOCK_LEDGERS).toString() : 'Enter ledger number'}
              disabled={isPending || currentLedger === 0}
            />
            {errors.unlockLedger && <p className="text-xs text-red-400 mt-1">{errors.unlockLedger}</p>}
          </div>

          {/* Estimated Unlock Time */}
          {unlockLedger && currentLedger > 0 && !errors.unlockLedger && (
            <div className="bg-slate-800/60 rounded-xl p-4 text-sm">
              <p className="text-slate-400 text-xs uppercase tracking-wide font-medium mb-2">Estimated unlock time</p>
              <div className="flex justify-between">
                <span className="text-slate-400">Ledger gap</span>
                <span className="text-slate-200 font-medium">
                  {ledgerGap} ledger{ledgerGap !== 1 ? 's' : ''}
                </span>
              </div>
              <div className="flex justify-between mt-2">
                <span className="text-slate-400">Time (approx.)</span>
                <span className="text-slate-200 font-medium">{formatEstimatedTime(estimatedSecondsRemaining)}</span>
              </div>
              <p className="text-xs text-slate-500 mt-3">
                Estimate: (unlock_ledger - current) × 5 seconds. Actual times may vary.
              </p>
            </div>
          )}

          {/* Penalty BPS */}
          <div>
            <label className="label">
              Early exit penalty (basis points)
              <span className="ml-1 text-slate-500 normal-case">— 0 = no penalty, 10000 = 100%</span>
            </label>
            <div className="relative">
              <input
                className={`input pr-20 ${errors.penalty ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
                type="number"
                min="0"
                max="10000"
                step="1"
                value={penaltyBps}
                onChange={(e) => setPenaltyBps(e.target.value)}
                disabled={isPending}
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 text-sm pointer-events-none">
                {isNaN(penaltyBpsNum) ? '—' : formatBps(penaltyBpsNum)}
              </span>
            </div>
            {errors.penalty && <p className="text-xs text-red-400 mt-1">{errors.penalty}</p>}
          </div>

          {/* Summary */}
          {amount && unlockLedger && !errors.amount && !errors.unlockLedger && (
            <div className="bg-slate-800/60 rounded-xl p-4 text-sm space-y-1.5">
              <p className="text-slate-400 text-xs uppercase tracking-wide font-medium mb-2">Summary</p>
              <Row label="Locking" value={`${amount} ${tokenAddress === CONFIG.NATIVE_TOKEN ? 'XLM' : 'tokens'}`} />
              <Row label="Unlock at ledger" value={unlockLedger} />
              <Row label="Gap (ledgers)" value={`${ledgerGap}`} />
              <Row label="Estimated time" value={formatEstimatedTime(estimatedSecondsRemaining)} />
              {penaltyBpsNum > 0 && <Row label="Early exit penalty" value={formatBps(penaltyBpsNum)} accent="orange" />}
            </div>
          )}

          <TxStatusBadge status={txStatus} txHash={txHash} error={txError} />

          <SubmitTimeoutBanner
            secondsRemaining={submitTimeout.secondsRemaining}
            isWarning={submitTimeout.isWarning}
            timedOut={timedOut}
            onRetry={handleTimeoutRetry}
            onDismiss={() => setTimedOut(false)}
          />

          <button type="submit" className="btn-primary w-full" disabled={!isValid || isPending}>
            {isPending ? (
              <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
                <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
              </svg>
            )}
            {isPending ? 'Processing…' : 'Lock Tokens'}
          </button>
        </form>
      </div>
    </div>
  )
}

function formatEstimatedTime(seconds: number): string {
  if (seconds <= 0) return 'Unlocked'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  const parts: string[] = []
  if (d > 0) parts.push(`${d}d`)
  if (h > 0) parts.push(`${h}h`)
  if (m > 0) parts.push(`${m}m`)
  if (d === 0) parts.push(`${s}s`)
  return parts.join(' ')
}

function Row({ label, value, accent }: { label: string; value: string; accent?: string }) {
  return (
    <div className="flex justify-between gap-2">
      <span className="text-slate-500">{label}</span>
      <span className={`font-medium ${accent === 'orange' ? 'text-orange-400' : 'text-slate-200'}`}>{value}</span>
    </div>
  )
}
