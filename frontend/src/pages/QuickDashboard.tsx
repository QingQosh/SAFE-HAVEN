import { useState, useEffect } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import { useDeposits } from '../hooks/useDeposits'
import { QuickWithdraw } from '../components/QuickWithdraw'
import { DepositWizard } from '../components/DepositWizard'
import { buildDeposit, buildWithdraw, buildCancelDeposit, submitTx, getTimeRemaining } from '../lib/stellar'
import { amountToBaseUnits } from '../lib/format'
import { CONFIG } from '../config'
import type { VaultEntry } from '../types'

type DashboardView = 'deposits' | 'newdeposit'

interface EnrichedDeposit extends VaultEntry {
  depositId: number
  timeRemaining: number
}

export function QuickDashboard() {
  const { wallet, isRestoringSession, signTransaction } = useWallet()
  const { deposits, loading: depositsLoading, refresh } = useDeposits(wallet?.address ?? null)
  const [view, setView] = useState<DashboardView>('deposits')
  const [enrichedDeposits, setEnrichedDeposits] = useState<EnrichedDeposit[]>([])
  const [withdrawing, setWithdrawing] = useState(false)
  const [depositError, setDepositError] = useState<string>()

  // Enrich deposits with time remaining
  useEffect(() => {
    if (!deposits.length) {
      setEnrichedDeposits([])
      return
    }

    const enrichAsync = async () => {
      const enriched = await Promise.all(
        deposits.map(async (deposit, idx) => {
          try {
            const timeRemaining = await getTimeRemaining(wallet?.address ?? '', idx)
            return { ...deposit, depositId: idx, timeRemaining }
          } catch {
            return { ...deposit, depositId: idx, timeRemaining: 0 }
          }
        })
      )
      setEnrichedDeposits(enriched)
    }

    void enrichAsync()
  }, [deposits, wallet?.address])

  async function handleNewDeposit(params: { amount: string; unlockTimestamp: number; penaltyBps: number }) {
    if (!wallet) return
    setDepositError(undefined)

    try {
      // Build and submit deposit transaction
      const amountBaseUnits = amountToBaseUnits(params.amount, 7) // XLM has 7 decimals
      const xdr = await buildDeposit(
        wallet.address,
        CONFIG.NATIVE_TOKEN,
        amountBaseUnits,
        params.unlockTimestamp,
        params.penaltyBps
      )
      
      if (!xdr) {
        throw new Error('Failed to build deposit transaction')
      }

      const sigResult = await signTransaction(xdr)
      if (!sigResult.signed) {
        const errorMsg = sigResult.rejected ? 'User rejected the transaction' : sigResult.error || 'Failed to sign transaction'
        throw new Error(errorMsg)
      }

      const result = await submitTx(sigResult.xdr)
      if (!result.success) {
        throw new Error(result.error || 'Transaction failed')
      }

      toast.success('Deposit locked! 🔒')
      setView('deposits')
      await refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Deposit failed'
      setDepositError(msg)
      toast.error(msg)
    }
  }

  async function handleWithdraw(depositId: number) {
    if (!wallet) return
    setWithdrawing(true)

    try {
      const xdr = await buildWithdraw(wallet.address, depositId)
      if (!xdr) throw new Error('Failed to build withdrawal')

      const sigResult = await signTransaction(xdr)
      if (!sigResult.signed) {
        const errorMsg = sigResult.rejected ? 'User rejected the transaction' : sigResult.error || 'Failed to sign'
        throw new Error(errorMsg)
      }

      const result = await submitTx(sigResult.xdr)
      if (!result.success) throw new Error(result.error || 'Withdrawal failed')

      toast.success('Withdrawn! 🎉')
      await refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Withdrawal failed'
      toast.error(msg)
    } finally {
      setWithdrawing(false)
    }
  }

  async function handleCancel(depositId: number) {
    if (!wallet) return
    setWithdrawing(true)

    try {
      const xdr = await buildCancelDeposit(wallet.address, depositId)
      if (!xdr) throw new Error('Failed to build cancellation')

      const sigResult = await signTransaction(xdr)
      if (!sigResult.signed) {
        const errorMsg = sigResult.rejected ? 'User rejected the transaction' : sigResult.error || 'Failed to sign'
        throw new Error(errorMsg)
      }

      const result = await submitTx(sigResult.xdr)
      if (!result.success) throw new Error(result.error || 'Cancellation failed')

      toast.success('Deposit cancelled.')
      await refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Cancellation failed'
      toast.error(msg)
    } finally {
      setWithdrawing(false)
    }
  }

  if (!wallet && !isRestoringSession) {
    return (
      <div className="card p-10 text-center">
        <p className="text-slate-400 mb-4">Connect your wallet to get started.</p>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto">
      {/* View Switcher */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setView('deposits')}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            view === 'deposits'
              ? 'bg-stellar-600 text-white'
              : 'bg-slate-800/40 border border-slate-700/40 text-slate-300 hover:border-slate-600'
          }`}
        >
          My Vaults {enrichedDeposits.length > 0 && `(${enrichedDeposits.length})`}
        </button>
        <button
          onClick={() => setView('newdeposit')}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            view === 'newdeposit'
              ? 'bg-stellar-600 text-white'
              : 'bg-slate-800/40 border border-slate-700/40 text-slate-300 hover:border-slate-600'
          }`}
        >
          + New Deposit
        </button>
      </div>

      {/* Deposits View */}
      {view === 'deposits' && (
        <div>
          {depositsLoading ? (
            <div className="text-center py-8 text-slate-400">Loading your vaults…</div>
          ) : enrichedDeposits.length === 0 ? (
            <div className="card p-8 text-center">
              <p className="text-slate-400 mb-4">No vaults yet. Start by creating one!</p>
              <button
                onClick={() => setView('newdeposit')}
                className="btn-primary px-4 py-2"
              >
                Create Vault
              </button>
            </div>
          ) : (
            <div className="grid gap-4">
              {enrichedDeposits.map((deposit) => (
                <QuickWithdraw
                  key={deposit.depositId}
                  deposit={deposit}
                  onWithdraw={handleWithdraw}
                  onCancel={handleCancel}
                  isLoading={withdrawing}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {/* New Deposit View */}
      {view === 'newdeposit' && (
        <DepositWizard
          contractInfo={{
            admin: null,
            pendingAdmin: null,
            paused: false,
            maxDeposit: BigInt('1e15'),
            maxLockSecs: 5 * 365 * 24 * 60 * 60,
            version: null,
            depositorCount: 0,
            feeRecipient: null,
            loading: false,
          } as any}
          onSubmit={handleNewDeposit}
          isSubmitting={withdrawing}
          error={depositError}
        />
      )}
    </div>
  )
}
