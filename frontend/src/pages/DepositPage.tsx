import { useState, useEffect } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import { useContractLogs } from '../context/ContractLogsContext'
import { useDeposits } from '../hooks/useDeposits'
import { TxStatusBadge } from '../components/TxStatusBadge'
import { LockByLedgerForm } from '../components/LockByLedgerForm'
import { SubmitTimeoutBanner } from '../components/SubmitTimeoutBanner'
import { DuplicateDepositWarning, isDuplicateWarningsDisabled, isDepositDismissed, findDuplicateDeposits } from '../components/DuplicateDepositWarning'
import { useSubmitTimeout, TimeoutError } from '../hooks/useSubmitTimeout'
import { buildDeposit, submitTx, getTokenDecimals, getTokenMetadata } from '../lib/stellar'
import { formatBps, formatDuration, dateTimeLocalToUnixSeconds, getMinDateTimeLocal, formatUnlockTimestampWithTimezone, getTimezoneOffsetString, amountToBaseUnits, baseUnitsToAmount, isValidContractAddress, validateTokenAddress } from '../lib/format'
import type { TxStatus } from '../types'
import type { ContractInfo } from '../App'
import { CONFIG } from '../config'
import { Address, nativeToScVal } from '@stellar/stellar-sdk'

type DepositTab = 'timestamp' | 'ledger'

interface DepositPageProps {
  contractInfo: ContractInfo
  onSuccess: () => void
}

export function DepositPage({ contractInfo, onSuccess }: DepositPageProps) {
  const { wallet, isRestoringSession, signTransaction } = useWallet()
  const { addLog, updateLog } = useContractLogs()
  const { deposits, loading: depositsLoading } = useDeposits(wallet?.address ?? null)

  // Deposit tab state
  const [depositTab, setDepositTab] = useState<DepositTab>('timestamp')

  const [tokenAddress, setTokenAddress] = useState<string>(CONFIG.NATIVE_TOKEN)
  const [amount,       setAmount]       = useState('')
  const [unlockDate,   setUnlockDate]   = useState('')
  const [penaltyBps,   setPenaltyBps]   = useState('0')

  const [txStatus, setTxStatus] = useState<TxStatus>('idle')
  const [txHash,   setTxHash]   = useState<string | undefined>()
  const [txError,  setTxError]  = useState<string | undefined>()

  // Duplicate warning modal state
  const [showDuplicateWarning, setShowDuplicateWarning] = useState(false)
  const [duplicateDeposits, setDuplicateDeposits] = useState<typeof deposits>([])

  // Whether the last submission attempt timed out (drives the banner retry UI).
  const [timedOut, setTimedOut] = useState(false)

  // Submission timeout — 2-minute countdown with 30-second warning.
  const submitTimeout = useSubmitTimeout({
    onTimeout: (elapsedMs) => {
      console.warn(`[DepositPage] Submission timed out after ${elapsedMs}ms`)
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
    Promise.all([getTokenDecimals(tokenAddress), getTokenMetadata(tokenAddress)]).then(
      ([decimals, metadata]) => {
        if (decimals !== null) {
          setTokenDecimals(decimals)
        } else {
          setTokenDecimals(7)
        }
        if (metadata) {
          setTokenMetadata(metadata)
        }
        setDecimalsLoading(false)
      },
    ).catch(() => {
      setTokenDecimals(7)
      setTokenMetadata(null)
      setDecimalsLoading(false)
    })
  }, [tokenAddress])

  // Derived validation
  const amountNum       = parseFloat(amount)
  const penaltyBpsNum   = parseInt(penaltyBps, 10)
  const unlockTimestamp = unlockDate ? dateTimeLocalToUnixSeconds(unlockDate) : 0
  const nowSecs         = Math.floor(Date.now() / 1000)
  const lockDuration    = unlockTimestamp - nowSecs

  // Convert amount to base units using the token's decimal precision
  const amountInBaseUnits = amount ? amountToBaseUnits(amount, tokenDecimals) : 0n

  const errors = {
    amount:    !amount ? '' : isNaN(amountNum) || amountNum <= 0 ? 'Amount must be > 0' :
               amountInBaseUnits > contractInfo.maxDeposit ? `Max: ${baseUnitsToAmount(contractInfo.maxDeposit, tokenDecimals)} tokens` : '',
    unlock:    !unlockDate ? '' : unlockTimestamp <= nowSecs ? 'Must be in the future' :
               lockDuration < CONFIG.MIN_LOCK_DURATION_SECS ? `Minimum lock: ${formatDuration(CONFIG.MIN_LOCK_DURATION_SECS)}` :
               lockDuration > contractInfo.maxLockSecs ? `Max lock: ${formatDuration(contractInfo.maxLockSecs)}` : '',
    penalty:   isNaN(penaltyBpsNum) || penaltyBpsNum < 0 || penaltyBpsNum > 10_000 ? '0–10000 only' : '',
  }
  const isValid = amount && unlockDate && !errors.amount && !errors.unlock && !errors.penalty && !contractInfo.paused && !decimalsLoading

  // Estimate gas when inputs change
  useEffect(() => {
    if (!wallet || !isValid) {
      setGasEstimate(null)
      return
    }

    const estimateAsync = async () => {
      setIsEstimating(true)
      try {
        const amountStroops = xlmToStroops(amount)
        const args = [
          new Address(wallet.address).toScVal(),
          new Address(tokenAddress).toScVal(),
          nativeToScVal(amountStroops, { type: 'i128' }),
          nativeToScVal(unlockTimestamp, { type: 'u64' }),
          nativeToScVal(penaltyBpsNum, { type: 'u32' }),
        ]
        const result = await estimateGas(wallet.address, 'deposit', args)
        setGasEstimate(result)
      } catch (e) {
        console.error('Gas estimation failed:', e)
        setGasEstimate({ success: false })
      } finally {
        setIsEstimating(false)
      }
    }

    // Debounce estimation
    const timer = setTimeout(() => void estimateAsync(), 500)
    return () => clearTimeout(timer)
  }, [wallet, amount, tokenAddress, unlockTimestamp, penaltyBpsNum, isValid, estimateGas])

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!wallet || !isValid) return

    // Check for duplicates before proceeding
    const amountBaseUnits = amountToBaseUnits(amount, tokenDecimals)
    const pendingDeposit = {
      token: tokenAddress,
      amount: amountBaseUnits,
      unlockTime: unlockTimestamp,
      penaltyBps: penaltyBpsNum,
    }

    // Skip duplicate check if globally disabled or this specific deposit was dismissed
    const skipCheck = isDuplicateWarningsDisabled() || isDepositDismissed(pendingDeposit)
    
    if (!skipCheck && !depositsLoading) {
      const duplicates = findDuplicateDeposits(deposits, pendingDeposit)
      
      if (duplicates.length > 0) {
        setDuplicateDeposits(duplicates)
        setShowDuplicateWarning(true)
        return // Stop submission, wait for user decision
      }
    }

    // Proceed with submission
    await executeDeposit()
  }

  async function executeDeposit() {
    if (!wallet) return

    setTxStatus('signing')
    setTxError(undefined)
    setTxHash(undefined)
    setTimedOut(false)

    // Start the 2-minute countdown. If it fires before we call
    // submitTimeout.cancel(), the race below will throw a TimeoutError.
    const timeoutRace = submitTimeout.start()

    // Add pending log entry
    const logId = addLog({
      operation: 'deposit',
      status: 'pending',
      initiator: wallet.address,
      parameters: {
        token: tokenAddress,
        amount: amount,
        unlockTime: unlockTimestamp,
        penaltyBps: penaltyBpsNum,
      },
    })

    try {
      const amountBaseUnits = amountToBaseUnits(amount, tokenDecimals)
      const xdr = await buildDeposit(wallet.address, tokenAddress, amountBaseUnits, unlockTimestamp, penaltyBpsNum)
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setTxStatus('submitting')

        // Race the submission against the timeout.
        const result = await Promise.race([submitTx(sigResult.xdr), timeoutRace.then(() => null)])
        
        if (result === null) {
          // The timeout won the race (timeoutRace resolved — but we treat
          // null as a sentinel for "timeout fired" if TimeoutError wasn't thrown).
          // In practice TimeoutError is thrown, so this path is a safety net.
          setTxStatus('error')
          setTimedOut(true)
          setTxError('Submission timed out. Please try again.')
          return
        }

        submitTimeout.cancel()

        if (result.success) {
          setTxStatus('success')
          setTxHash(result.txHash)
          updateLog(logId, {
            status: 'success',
            txHash: result.txHash,
          })
          toast.success('Deposit successful! Your tokens are locked.')
          setAmount('')
          setUnlockDate('')
          setPenaltyBps('0')
          setTimeout(onSuccess, 1500)
        } else {
          setTxStatus('error')
          setTxError(result.error)
          updateLog(logId, {
            status: 'error',
            errorMessage: result.error,
          })
          toast.error(result.error ?? 'Deposit failed')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        submitTimeout.cancel()
        setTxStatus('idle')
        updateLog(logId, {
          status: 'error',
          errorMessage: 'User rejected the transaction',
        })
        return
      } else {
        // Signing error: already toasted, but still reset state
        submitTimeout.cancel()
        setTxStatus('idle')
        updateLog(logId, {
          status: 'error',
          errorMessage: sigResult.error,
        })
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setTxStatus('error')
      setTxError(msg)
      updateLog(logId, {
        status: 'error',
        errorMessage: msg,
      })
      toast.error(msg)
    }
  }

  function handleDuplicateProceed(_suppressFuture: boolean) {
    setShowDuplicateWarning(false)
    // Proceed with deposit submission
    void executeDeposit()
  }

  function handleDuplicateModify() {
    setShowDuplicateWarning(false)
    // Keep form open for user to modify
  }

  function handleDuplicateCancel() {
    setShowDuplicateWarning(false)
    // Just close the modal, stay on form
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
    <>
      {/* Duplicate Warning Modal */}
      <DuplicateDepositWarning
        isOpen={showDuplicateWarning}
        pendingDeposit={{
          token: tokenAddress,
          amount: amountToBaseUnits(amount || '0', tokenDecimals),
          unlockTime: unlockTimestamp,
          penaltyBps: penaltyBpsNum,
        }}
        duplicates={duplicateDeposits}
        onProceed={handleDuplicateProceed}
        onModify={handleDuplicateModify}
        onCancel={handleDuplicateCancel}
      />

      {/* Main Content */}
      <div className="max-w-lg mx-auto">
      {/* Tab Navigation */}
      <div className="mb-6 flex gap-2">
        <button
          onClick={() => setDepositTab('timestamp')}
          className={[
            'flex-1 px-3 md:px-4 py-2 rounded-lg font-medium transition-all text-xs md:text-sm min-h-10',
            depositTab === 'timestamp'
              ? 'bg-stellar-600 text-white shadow-sm'
              : 'bg-slate-900/60 text-slate-400 hover:text-slate-200 border border-slate-700/60 hover:border-slate-600',
          ].join(' ')}
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4 inline mr-2">
            <path d="M5.5 13a3.5 3.5 0 01-.369-6.98 4 4 0 117.753-1 4.5 4.5 0 11-4.384 5.98z" />
          </svg>
          <span className="hidden sm:inline">By Timestamp</span>
          <span className="sm:hidden">Time</span>
        </button>
        <button
          onClick={() => setDepositTab('ledger')}
          className={[
            'flex-1 px-3 md:px-4 py-2 rounded-lg font-medium transition-all text-xs md:text-sm min-h-10',
            depositTab === 'ledger'
              ? 'bg-stellar-600 text-white shadow-sm'
              : 'bg-slate-900/60 text-slate-400 hover:text-slate-200 border border-slate-700/60 hover:border-slate-600',
          ].join(' ')}
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4 inline mr-2">
            <path fillRule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clipRule="evenodd" />
          </svg>
          <span className="hidden sm:inline">By Ledger</span>
          <span className="sm:hidden">Ledger</span>
        </button>
      </div>

      {/* Timestamp-based Deposit Form */}
      {depositTab === 'timestamp' && (
        <div className="card p-4 md:p-6">
          <h2 className="font-semibold text-base md:text-lg mb-1">Lock tokens in a vault</h2>
          <p className="text-xs md:text-sm text-slate-400 mb-4 md:mb-6">
            Tokens will be locked until your chosen date.
          </p>

          {contractInfo.paused && (
            <div className="mb-4 md:mb-5 p-3 rounded-lg md:rounded-xl bg-red-900/30 border border-red-700/40 text-red-400 text-xs md:text-sm">
              ⚠️ Contract is paused. Deposits disabled.
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4 md:space-y-5" noValidate>
            {/* Token */}
            <div>
              <label className="label">Token address</label>
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
                  <span className="absolute right-3 md:right-4 top-1/2 -translate-y-1/2 text-slate-400 text-xs">
                    Verifying…
                  </span>
                )}
                {!decimalsLoading && tokenAddress && isValidContractAddress(tokenAddress) && !tokenAddressError && (
                  <span className="absolute right-3 md:right-4 top-1/2 -translate-y-1/2 text-green-400 text-xs">
                    ✓
                  </span>
                )}
              </div>
              {tokenAddressError ? (
                <p className="text-xs text-red-400 mt-1">{tokenAddressError}</p>
              ) : (
                <p className="text-xs text-slate-500 mt-1 leading-snug">
                  {tokenAddress === CONFIG.NATIVE_TOKEN
                    ? 'XLM (7 decimals)'
                    : tokenMetadata
                      ? `${tokenMetadata.symbol} (${tokenDecimals} decimals)`
                      : `Custom token (${tokenDecimals} decimals${decimalsLoading ? ', verifying…' : ''})`}
                </p>
              )}
            </div>

            {/* Amount */}
            <div>
              <label className="label">Amount</label>
              <div className="relative">
                <input
                  className={`input pr-12 md:pr-14 ${errors.amount ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
                  type="number"
                  min="0"
                  step={`0.${'0'.repeat(Math.max(0, tokenDecimals - 1))}1`}
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0"
                  disabled={isPending || decimalsLoading}
                />
                <span className="absolute right-3 md:right-4 top-1/2 -translate-y-1/2 text-slate-500 text-xs md:text-sm font-medium pointer-events-none">
                  {tokenAddress === CONFIG.NATIVE_TOKEN ? 'XLM' : 'tokens'}
                </span>
              </div>
              {errors.amount && <p className="text-xs text-red-400 mt-1">{errors.amount}</p>}
            </div>

            {/* Unlock date */}
            <div>
              <label className="label text-xs">
                Unlock date & time
              </label>
              <p className="text-xs text-slate-500 mb-1.5">{getTimezoneOffsetString()}</p>
              <input
                className={`input ${errors.unlock ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
                type="datetime-local"
                value={unlockDate}
                onChange={(e) => setUnlockDate(e.target.value)}
                min={getMinDateTimeLocal()}
                disabled={isPending}
              />
              {errors.unlock && <p className="text-xs text-red-400 mt-1">{errors.unlock}</p>}
            </div>

            {/* Penalty BPS */}
            <div>
              <label className="label text-xs">
                Early exit penalty (bps)
              </label>
              <p className="text-xs text-slate-500 mb-1.5">0–10000 (0% = no penalty)</p>
              <div className="relative">
                <input
                  className={`input pr-12 ${errors.penalty ? 'border-red-500 focus:ring-red-500/50 focus:border-red-500' : ''}`}
                  type="number"
                  min="0"
                  max="10000"
                  step="1"
                  value={penaltyBps}
                  onChange={(e) => setPenaltyBps(e.target.value)}
                  disabled={isPending}
                />
                <span className="absolute right-3 md:right-4 top-1/2 -translate-y-1/2 text-slate-500 text-xs font-medium pointer-events-none">
                  {isNaN(penaltyBpsNum) ? '—' : formatBps(penaltyBpsNum)}
                </span>
              </div>
              {errors.penalty && <p className="text-xs text-red-400 mt-1">{errors.penalty}</p>}
            </div>

            {/* Summary */}
            {amount && unlockDate && !errors.amount && !errors.unlock && (
              <div className="bg-slate-800/60 rounded-lg md:rounded-xl p-3 md:p-4 text-xs md:text-sm space-y-1">
                <p className="text-slate-400 text-xs uppercase tracking-wide font-medium mb-2">Summary</p>
                <Row label="Locking" value={`${amount} XLM`} />
                {(() => {
                  const ts = formatUnlockTimestampWithTimezone(unlockTimestamp)
                  return (
                    <>
                      <Row label="Unlock (local)" value={ts.local} />
                      <Row label="Unlock (UTC)" value={ts.utc} />
                    </>
                  )
                })()}
                {penaltyBpsNum > 0 && <Row label="Early penalty" value={formatBps(penaltyBpsNum)} accent="orange" />}
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

            <button
              type="submit"
              className="btn-primary w-full text-sm md:text-base py-2.5 min-h-11 md:min-h-auto"
              disabled={!isValid || isPending}
            >
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
      )}

      {/* Ledger-based Deposit Form */}
      {depositTab === 'ledger' && <LockByLedgerForm contractInfo={contractInfo} onSuccess={onSuccess} />}
      </div>
    </>
  )
}

function Row({ label, value, accent }: { label: string; value: string; accent?: string }) {
  return (
    <div className="flex justify-between gap-2">
      <span className="text-slate-500">{label}</span>
      <span className={`font-medium ${accent === 'orange' ? 'text-orange-400' : 'text-slate-200'}`}>{value}</span>
    </div>
  )
}
