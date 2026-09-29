import { useWallet } from '../context/WalletContext'
import { useContractLogs } from '../context/ContractLogsContext'
import { useDeposits } from '../hooks/useDeposits'
import type { ContractInfo } from '../App'
import { DepositCard } from '../components/DepositCard'
import { TxStatusBadge } from '../components/TxStatusBadge'
import { WithdrawalConfirmation } from '../components/WithdrawalConfirmation'
import { buildWithdraw, buildCancelDeposit, submitTx } from '../lib/stellar'
import { shortAddr } from '../lib/format'
import type { TxStatus, Deposit } from '../types'
import { useState } from 'react'
import toast from 'react-hot-toast'

interface DashboardProps {
  contractInfo: ContractInfo
}

export function Dashboard({ contractInfo }: DashboardProps) {
  const { wallet, isRestoringSession, signTransaction } = useWallet()
  const { deposits, loading, error, refresh, pollRemoveDeposit } = useDeposits(wallet?.address ?? null)
  const { addLog, updateLog } = useContractLogs()
  const [txStatus, setTxStatus] = useState<TxStatus>('idle')
  const [txHash,   setTxHash]   = useState<string | undefined>()
  const [txError,  setTxError]  = useState<string | undefined>()
  const [pendingId, setPendingId] = useState<number | null>(null)

  // Confirmation modal state
  const [showConfirmation, setShowConfirmation] = useState(false)
  const [pendingDeposit, setPendingDeposit] = useState<Deposit | null>(null)
  const [pendingAction, setPendingAction] = useState<'withdraw' | 'cancel' | null>(null)

  async function handleWithdraw(depositId: number) {
    const deposit = deposits.find(d => d.depositId === depositId)
    if (!deposit) return
    
    setPendingDeposit(deposit)
    setPendingAction('withdraw')
    setShowConfirmation(true)
  }

  async function handleCancel(depositId: number) {
    const deposit = deposits.find(d => d.depositId === depositId)
    if (!deposit) return
    
    setPendingDeposit(deposit)
    setPendingAction('cancel')
    setShowConfirmation(true)
  }

  async function handleConfirmWithdrawal() {
    if (!wallet || !pendingDeposit || !pendingAction) return

    setShowConfirmation(false)
    const depositId = pendingDeposit.depositId
    setPendingId(depositId)
    setTxStatus('signing')
    setTxError(undefined)
    setTxHash(undefined)

    // Add pending log entry
    const logId = addLog({
      operation: pendingAction === 'withdraw' ? 'withdraw' : 'cancel_deposit',
      status: 'pending',
      initiator: wallet.address,
      parameters: { depositId },
    })

    try {
      const xdr = pendingAction === 'withdraw'
        ? await buildWithdraw(wallet.address, depositId)
        : await buildCancelDeposit(wallet.address, depositId)
        
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)

        if (result.success) {
          setTxStatus('success')
          setTxHash(result.txHash)
          updateLog(logId, {
            status: 'success',
            txHash: result.txHash,
          })
          toast.success(pendingAction === 'withdraw' ? 'Withdrawal successful!' : 'Deposit cancelled.')
          // Poll for individual deposit removal instead of full refresh
          await pollRemoveDeposit(depositId)
        } else {
          setTxStatus('error')
          setTxError(result.error)
          updateLog(logId, {
            status: 'error',
            errorMessage: result.error,
          })
          toast.error(result.error ?? `${pendingAction === 'withdraw' ? 'Withdrawal' : 'Cancel'} failed`)
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setTxStatus('idle')
        updateLog(logId, {
          status: 'error',
          errorMessage: 'User rejected the transaction',
        })
      } else {
        // Signing error: already toasted, but still reset state
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
    } finally {
      setPendingId(null)
      setPendingDeposit(null)
      setPendingAction(null)
    }
  }

  function handleCancelConfirmation() {
    setShowConfirmation(false)
    setPendingDeposit(null)
    setPendingAction(null)
  }

  // Old withdraw handler removed - now using confirmation modal
  // Old cancel handler removed - now using confirmation modal

  if (!wallet && !isRestoringSession) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-center">
        <div className="w-16 h-16 rounded-2xl bg-stellar-900/40 border border-stellar-700/40 flex items-center justify-center mb-4">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-8 h-8 text-stellar-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9m18-3H3m7.5 3h3" />
          </svg>
        </div>
        <h2 className="text-xl font-semibold mb-2">Connect your wallet</h2>
        <p className="text-slate-400 text-sm max-w-xs">Connect your Freighter wallet to view and manage your vault deposits.</p>
      </div>
    )
  }

  // Show loading skeleton while restoring session
  if (isRestoringSession) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
        </div>
        <div>
          <div className="flex items-center justify-between mb-4">
            <h2 className="font-semibold text-lg">Your Vaults</h2>
            <div className="w-16 h-9 bg-slate-700/40 rounded-lg animate-pulse" />
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {[1, 2].map((i) => (
              <div key={i} className="card p-5 h-36 animate-pulse">
                <div className="flex gap-3">
                  <div className="w-10 h-10 rounded-full bg-slate-700/60" />
                  <div className="flex-1 space-y-2">
                    <div className="h-4 bg-slate-700/60 rounded w-1/2" />
                    <div className="h-3 bg-slate-700/40 rounded w-1/3" />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <>
      {/* Withdrawal Confirmation Modal */}
      {showConfirmation && pendingDeposit && (
        <WithdrawalConfirmation
          isOpen={showConfirmation}
          deposit={pendingDeposit}
          recipient={wallet?.address}
          estimatedGas="~0.0001 XLM"
          onConfirm={handleConfirmWithdrawal}
          onCancel={handleCancelConfirmation}
        />
      )}

      {/* Main content */}
      <div className="space-y-4 md:space-y-6">
        {/* Stats row */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-2 md:gap-3">
          {loading && deposits.length === 0 ? (
            // Loading skeleton for stats
            <>
              <StatCardSkeleton />
              <StatCardSkeleton />
              <StatCardSkeleton />
              <StatCardSkeleton />
            </>
          ) : (
            <>
              <StatCard label="Your Deposits" value={String(deposits.length)} />
              <StatCard label="Unlocked" value={String(deposits.filter(d => d.timeRemaining === 0 && d.unlockVerified).length)} accent="green" />
              <StatCard label="Locked" value={String(deposits.filter(d => d.timeRemaining === null || d.timeRemaining > 0 || !d.unlockVerified).length)} accent="yellow" />
              <StatCard label="Total Depositors" value={contractInfo.depositorCount > 0 ? String(contractInfo.depositorCount) : '–'} />
            </>
          )}
        </div>

        {/* Tx status */}
        <TxStatusBadge status={txStatus} txHash={txHash} error={txError} />

        {/* Deposits */}
        <div>
          <div className="flex items-center justify-between mb-3 md:mb-4 gap-2">
            <h2 className="font-semibold text-lg">Your Vaults</h2>
            <button onClick={refresh} className="btn-secondary text-xs px-3 py-1.5 h-10 md:h-auto" disabled={loading}>
              {loading ? (
                <span className="w-3 h-3 border-2 border-current/30 border-t-current rounded-full animate-spin" />
              ) : (
                <svg viewBox="0 0 20 20" fill="currentColor" className="w-3.5 h-3.5">
                  <path fillRule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clipRule="evenodd" />
                </svg>
              )}
              <span className="hidden sm:inline">Refresh</span>
            </button>
          </div>

          {loading && deposits.length === 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 md:gap-4">
              {[1, 2].map((i) => (
                <div key={i} className="card p-4 md:p-5 h-32 animate-pulse">
                  <div className="flex gap-2 md:gap-3">
                    <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-slate-700/60" />
                    <div className="flex-1 space-y-2">
                      <div className="h-3 md:h-4 bg-slate-700/60 rounded w-1/2" />
                      <div className="h-2 md:h-3 bg-slate-700/40 rounded w-1/3" />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : error ? (
            <div className="card p-4 md:p-6 text-center text-red-400 text-sm">
              <p className="font-medium">Failed to load deposits</p>
              <p className="text-xs text-slate-500 mt-1">{error}</p>
            </div>
          ) : deposits.length === 0 ? (
            <div className="card p-6 md:p-10 text-center">
              <p className="text-slate-400 text-sm md:text-base">No active vaults for</p>
              {wallet && <p className="font-mono text-xs text-stellar-400 mt-1">{shortAddr(wallet.address)}</p>}
              <p className="text-slate-500 text-xs md:text-sm mt-3">Use the Deposit tab to lock your first tokens.</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 md:gap-4">
              {deposits.map((d) => (
                <DepositCard
                  key={d.depositId}
                  deposit={d}
                  onWithdraw={handleWithdraw}
                  onCancel={handleCancel}
                  txPending={pendingId === d.depositId}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  )
}

function StatCard({ label, value, accent }: { label: string; value: string; accent?: 'green' | 'yellow' }) {
  const valueClass =
    accent === 'green'  ? 'text-green-400' :
    accent === 'yellow' ? 'text-yellow-400' :
    'text-slate-100'

  return (
    <div className="card p-3 md:p-4">
      <p className="text-xs text-slate-500 uppercase tracking-wide">{label}</p>
      <p className={`text-lg md:text-2xl font-bold mt-1 ${valueClass}`}>{value}</p>
    </div>
  )
}

function StatCardSkeleton() {
  return (
    <div className="card p-3 md:p-4 animate-pulse">
      <div className="h-2 md:h-3 bg-slate-700/60 rounded w-1/2" />
      <div className="h-5 md:h-7 bg-slate-700/40 rounded w-2/3 mt-2" />
    </div>
  )
}
