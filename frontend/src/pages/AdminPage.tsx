import { useState } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import { TxStatusBadge } from '../components/TxStatusBadge'
import {
  buildPause,
  buildUnpause,
  buildEmergencyWithdraw,
  buildTransferAdmin,
  buildAcceptAdmin,
  buildCancelTransferAdmin,
  buildRenounceAdmin,
  submitTx,
} from '../lib/stellar'
import { shortAddr, explorerAddrUrl, isValidStellarAddress } from '../lib/format'
import type { TxStatus } from '../types'
import type { ContractInfo } from '../App'

interface AdminPageProps {
  contractInfo: ContractInfo
  onContractInfoRefresh: () => void
}

export function AdminPage({ contractInfo, onContractInfoRefresh }: AdminPageProps) {
  const { wallet, isRestoringSession, signTransaction } = useWallet()

  // Pause/unpause
  const [pauseTxStatus, setPauseTxStatus] = useState<TxStatus>('idle')
  const [pauseTxHash,   setPauseTxHash]   = useState<string | undefined>()
  const [pauseTxError,  setPauseTxError]  = useState<string | undefined>()

  // Emergency withdraw
  const [emrgDepositor,      setEmrgDepositor]      = useState('')
  const [emrgDepositId,      setEmrgDepositId]      = useState('')
  const [emrgDepositorError, setEmrgDepositorError] = useState('')
  const [emrgTxStatus,       setEmrgTxStatus]       = useState<TxStatus>('idle')
  const [emrgTxHash,         setEmrgTxHash]         = useState<string | undefined>()
  const [emrgTxError,        setEmrgTxError]        = useState<string | undefined>()

  // Transfer admin
  const [transferNewAdmin,  setTransferNewAdmin]  = useState('')
  const [transferTxStatus,  setTransferTxStatus]  = useState<TxStatus>('idle')
  const [transferTxHash,    setTransferTxHash]    = useState<string | undefined>()
  const [transferTxError,   setTransferTxError]   = useState<string | undefined>()

  // Accept admin
  const [acceptTxStatus, setAcceptTxStatus] = useState<TxStatus>('idle')
  const [acceptTxHash,   setAcceptTxHash]   = useState<string | undefined>()
  const [acceptTxError,  setAcceptTxError]  = useState<string | undefined>()

  // Cancel transfer admin
  const [cancelTxStatus, setCancelTxStatus] = useState<TxStatus>('idle')
  const [cancelTxHash,   setCancelTxHash]   = useState<string | undefined>()
  const [cancelTxError,  setCancelTxError]  = useState<string | undefined>()

  // Renounce admin
  const [renounceConfirmText, setRenounceConfirmText] = useState('')
  const [renounceTxStatus,    setRenounceTxStatus]    = useState<TxStatus>('idle')
  const [renounceTxHash,      setRenounceTxHash]      = useState<string | undefined>()
  const [renounceTxError,     setRenounceTxError]     = useState<string | undefined>()

  // 2FA states
  const [show2FA, setShow2FA] = useState(false)
  const [pendingAction, setPendingAction] = useState<'pause' | 'unpause' | 'emergency' | null>(null)

  const isAdmin = wallet?.address === contractInfo.admin
  const pausePending = pauseTxStatus === 'signing' || pauseTxStatus === 'submitting' || pauseTxStatus === 'confirming'
  const emrgPending  = emrgTxStatus  === 'signing' || emrgTxStatus  === 'submitting' || emrgTxStatus  === 'confirming'
  const transferPending = transferTxStatus === 'signing' || transferTxStatus === 'submitting' || transferTxStatus === 'confirming'
  const acceptPending = acceptTxStatus === 'signing' || acceptTxStatus === 'submitting' || acceptTxStatus === 'confirming'
  const cancelPending = cancelTxStatus === 'signing' || cancelTxStatus === 'submitting' || cancelTxStatus === 'confirming'
  const renouncePending = renounceTxStatus === 'signing' || renounceTxStatus === 'submitting' || renounceTxStatus === 'confirming'

  // Validate emergency withdraw depositor address
  const emrgDepositorIsValid = emrgDepositor === '' || isValidStellarAddress(emrgDepositor)
  
  const handleEmrgDepositorChange = (value: string) => {
    const trimmed = value.trim()
    setEmrgDepositor(trimmed)
    if (trimmed && !isValidStellarAddress(trimmed)) {
      setEmrgDepositorError('Invalid address format. Must be a G-address or C-address.')
    } else {
      setEmrgDepositorError('')
    }
  }

  async function handleTogglePause() {
    if (!wallet || pausePending) return
    
    setPauseTxStatus('signing')
    setPauseTxError(undefined)
    setPauseTxHash(undefined)

    try {
      const xdr = contractInfo.paused
        ? await buildUnpause(wallet.address)
        : await buildPause(wallet.address)

      if (!xdr) throw new Error('Failed to build transaction')
      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setPauseTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setPauseTxStatus('success')
          setPauseTxHash(result.txHash)
          toast.success(contractInfo.paused ? 'Contract unpaused.' : 'Contract paused.')
          // Refresh contract info to get latest pause state
          setTimeout(onContractInfoRefresh, 1500)
          // Reset state after success
          setTimeout(() => {
            setPauseTxStatus('idle')
            setPauseTxHash(undefined)
            setPauseTxError(undefined)
          }, 3000)
        } else {
          setPauseTxStatus('error')
          setPauseTxError(result.error)
          toast.error(result.error ?? 'Transaction failed')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setPauseTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setPauseTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setPauseTxStatus('error')
      setPauseTxError(msg)
      toast.error(msg)
    }
  }

  async function handleEmergencyWithdraw(e: React.FormEvent) {
    e.preventDefault()
    if (!wallet || !emrgDepositor || !emrgDepositId || !emrgDepositorIsValid) return

    // Check if 2FA is required
    if (twoFAState.enabled) {
      setPendingAction('emergency')
      setShow2FA(true)
      return
    }

    // Proceed without 2FA
    await executeEmergencyWithdraw()
  }

  async function executeEmergencyWithdraw() {
    if (!wallet || !emrgDepositor || !emrgDepositId) return

    setEmrgTxStatus('signing')
    setEmrgTxError(undefined)
    setEmrgTxHash(undefined)

    try {
      const xdr = await buildEmergencyWithdraw(wallet.address, emrgDepositor, parseInt(emrgDepositId, 10))
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setEmrgTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setEmrgTxStatus('success')
          setEmrgTxHash(result.txHash)
          toast.success('Emergency withdrawal successful. Funds returned to depositor.')
          setEmrgDepositor('')
          setEmrgDepositId('')
          setEmrgDepositorError('')
        } else {
          setEmrgTxStatus('error')
          setEmrgTxError(result.error)
          toast.error(result.error ?? 'Emergency withdrawal failed')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setEmrgTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setEmrgTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setEmrgTxStatus('error')
      setEmrgTxError(msg)
      toast.error(msg)
    }
  }

  async function handleTransferAdmin(e: React.FormEvent) {
    e.preventDefault()
    if (!wallet || !transferNewAdmin) return

    setTransferTxStatus('signing')
    setTransferTxError(undefined)
    setTransferTxHash(undefined)

    try {
      const xdr = await buildTransferAdmin(wallet.address, transferNewAdmin)
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setTransferTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setTransferTxStatus('success')
          setTransferTxHash(result.txHash)
          toast.success('Admin transfer initiated. Awaiting acceptance from new admin.')
          setTransferNewAdmin('')
          setTimeout(onContractInfoRefresh, 1500)
        } else {
          setTransferTxStatus('error')
          setTransferTxError(result.error)
          toast.error(result.error ?? 'Transfer admin failed')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setTransferTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setTransferTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setTransferTxStatus('error')
      setTransferTxError(msg)
      toast.error(msg)
    }
  }

  async function handleAcceptAdmin() {
    if (!wallet) return

    setAcceptTxStatus('signing')
    setAcceptTxError(undefined)
    setAcceptTxHash(undefined)

    try {
      const xdr = await buildAcceptAdmin(wallet.address)
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setAcceptTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setAcceptTxStatus('success')
          setAcceptTxHash(result.txHash)
          toast.success('Admin transfer accepted!')
          setTimeout(onContractInfoRefresh, 1500)
        } else {
          setAcceptTxStatus('error')
          setAcceptTxError(result.error)
          toast.error(result.error ?? 'Failed to accept admin')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setAcceptTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setAcceptTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setAcceptTxStatus('error')
      setAcceptTxError(msg)
      toast.error(msg)
    }
  }

  async function handleCancelTransfer() {
    if (!wallet) return

    setCancelTxStatus('signing')
    setCancelTxError(undefined)
    setCancelTxHash(undefined)

    try {
      const xdr = await buildCancelTransferAdmin(wallet.address)
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setCancelTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setCancelTxStatus('success')
          setCancelTxHash(result.txHash)
          toast.success('Admin transfer cancelled.')
          setTimeout(onContractInfoRefresh, 1500)
        } else {
          setCancelTxStatus('error')
          setCancelTxError(result.error)
          toast.error(result.error ?? 'Failed to cancel transfer')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setCancelTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setCancelTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setCancelTxStatus('error')
      setCancelTxError(msg)
      toast.error(msg)
    }
  }

  async function handleRenounceAdmin() {
    if (!wallet) return

    setRenounceTxStatus('signing')
    setRenounceTxError(undefined)
    setRenounceTxHash(undefined)

    try {
      const xdr = await buildRenounceAdmin(wallet.address)
      if (!xdr) throw new Error('Failed to build transaction')

      const sigResult = await signTransaction(xdr)
      
      // Handle the three signing outcomes
      if (sigResult.signed) {
        // Success: proceed with submission
        setRenounceTxStatus('submitting')
        const result = await submitTx(sigResult.xdr)
        if (result.success) {
          setRenounceTxStatus('success')
          setRenounceTxHash(result.txHash)
          toast.success('Admin renounced. Contract is now fully trustless.')
          setRenounceConfirmText('')
          setTimeout(onContractInfoRefresh, 1500)
        } else {
          setRenounceTxStatus('error')
          setRenounceTxError(result.error)
          toast.error(result.error ?? 'Failed to renounce admin')
        }
      } else if (sigResult.rejected) {
        // User rejected: silently reset state
        setRenounceTxStatus('idle')
      } else {
        // Signing error: already toasted, but still reset state
        setRenounceTxStatus('idle')
      }
      return
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Unexpected error'
      setRenounceTxStatus('error')
      setRenounceTxError(msg)
      toast.error(msg)
    }
  }

  if (!wallet && !isRestoringSession) {
    return (
      <div className="card p-10 text-center max-w-lg">
        <p className="text-slate-400">Connect your wallet to access admin controls.</p>
      </div>
    )
  }

  // Check if admin has been renounced (fully trustless)
  if (contractInfo.admin === null) {
    return (
      <div className="card p-10 text-center max-w-lg">
        <div className="w-12 h-12 rounded-xl bg-green-900/30 border border-green-700/40 flex items-center justify-center mx-auto mb-4">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-6 h-6 text-green-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <p className="font-medium text-green-400">Fully Trustless</p>
        <p className="text-sm text-slate-400 mt-1">
          This contract is fully trustless — admin has been permanently renounced.
        </p>
      </div>
    )
  }

  if (!isAdmin) {
    return (
      <div className="card p-10 text-center max-w-lg">
        <div className="w-12 h-12 rounded-xl bg-red-900/30 border border-red-700/40 flex items-center justify-center mx-auto mb-4">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-6 h-6 text-red-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
        </div>
        <p className="font-medium text-red-400">Not authorized</p>
        <p className="text-sm text-slate-400 mt-1">
          Connected address is not the contract admin.
        </p>
        {contractInfo.admin && (
          <p className="text-xs text-slate-500 mt-3 font-mono">
            Admin:{' '}
            <a
              href={explorerAddrUrl(contractInfo.admin)}
              target="_blank"
              rel="noopener noreferrer"
              className="text-stellar-400 hover:text-stellar-300"
            >
              {shortAddr(contractInfo.admin)}
            </a>
          </p>
        )}
      </div>
    )
  }

  return (
    <div className="max-w-lg mx-auto space-y-4 md:space-y-5">
      {/* Contract status card */}
      <div className="card p-4 md:p-6">
        <h2 className="font-semibold text-base md:text-lg mb-3 md:mb-4">Contract Status</h2>
        <div className="grid grid-cols-2 gap-y-2 md:gap-y-3 text-xs md:text-sm mb-4">
          <span className="text-slate-400">Admin</span>
          <a
            href={explorerAddrUrl(contractInfo.admin!)}
            target="_blank"
            rel="noopener noreferrer"
            className="font-mono text-stellar-400 hover:text-stellar-300 truncate text-xs"
          >
            {shortAddr(contractInfo.admin!)}
          </a>

          <span className="text-slate-400">Status</span>
          <span>
            {contractInfo.paused
              ? <span className="badge-red text-xs">Paused</span>
              : <span className="badge-green text-xs">Active</span>
            }
          </span>

          <span className="text-slate-400">Depositors</span>
          <span className="text-slate-200 text-xs md:text-sm">{contractInfo.depositorCount}</span>

          {contractInfo.feeRecipient && (
            <>
              <span className="text-slate-400 text-xs md:text-sm">Fee recipient</span>
              <a
                href={explorerAddrUrl(contractInfo.feeRecipient)}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-stellar-400 hover:text-stellar-300 truncate text-xs"
              >
                {shortAddr(contractInfo.feeRecipient)}
              </a>
            </>
          )}

          {contractInfo.pendingAdmin && (
            <>
              <span className="text-slate-400 text-xs md:text-sm">Pending admin</span>
              <a
                href={explorerAddrUrl(contractInfo.pendingAdmin)}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-blue-400 hover:text-blue-300 truncate text-xs"
              >
                {shortAddr(contractInfo.pendingAdmin)}
              </a>
            </>
          )}
        </div>

        {contractInfo.pendingAdmin && (
          <div className="mt-3 md:mt-4 mb-3 md:mb-4">
            <button
              onClick={handleCancelTransfer}
              className="btn-secondary w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto"
              disabled={cancelPending}
            >
              {cancelPending
                ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
                : 'Cancel transfer'
              }
            </button>
          </div>
        )}

        <TxStatusBadge status={pauseTxStatus} txHash={pauseTxHash} error={pauseTxError} />

        <div className="mt-3 md:mt-4">
          <button
            className={`w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto ${contractInfo.paused ? 'btn-primary' : 'btn-danger'}`}
            onClick={handleTogglePause}
            disabled={pausePending}
          >
            {pausePending
              ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
              : contractInfo.paused ? 'Unpause' : 'Pause'
            }
          </button>
        </div>
      </div>

      {/* Emergency withdrawal */}
      <div className="card p-4 md:p-6">
        <h2 className="font-semibold text-base md:text-lg mb-1">Emergency Withdrawal</h2>
        <p className="text-xs md:text-sm text-slate-400 mb-4">
          Returns funds to depositor, bypassing time lock.
        </p>

        <form onSubmit={handleEmergencyWithdraw} className="space-y-3 md:space-y-4">
          <div>
            <label className="label">Depositor address</label>
            <input
              className={`input ${emrgDepositorError ? 'border-red-500/50' : ''}`}
              type="text"
              value={emrgDepositor}
              onChange={(e) => handleEmrgDepositorChange(e.target.value)}
              placeholder="G... or C..."
              disabled={emrgPending}
            />
            {emrgDepositorError && (
              <p className="text-xs text-red-400 mt-1">{emrgDepositorError}</p>
            )}
          </div>

          <div>
            <label className="label">Deposit ID</label>
            <input
              className="input"
              type="number"
              min="0"
              value={emrgDepositId}
              onChange={(e) => setEmrgDepositId(e.target.value)}
              placeholder="0"
              disabled={emrgPending}
            />
          </div>

          <TxStatusBadge status={emrgTxStatus} txHash={emrgTxHash} error={emrgTxError} />

          <button
            type="submit"
            className="btn-danger w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto"
            disabled={!emrgDepositor || !emrgDepositId || !emrgDepositorIsValid || emrgPending}
          >
            {emrgPending
              ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
              : 'Emergency withdraw'
            }
          </button>
        </form>
      </div>

      {/* Transfer admin */}
      <div className="card p-4 md:p-6">
        <h2 className="font-semibold text-base md:text-lg mb-1">Transfer Admin</h2>
        <p className="text-xs md:text-sm text-slate-400 mb-4">
          Two-step transfer. New admin must accept to complete.
        </p>

        {contractInfo.pendingAdmin ? (
          <div className="space-y-3 md:space-y-4">
            <div className="rounded-lg bg-blue-900/20 border border-blue-700/40 p-3">
              <p className="text-xs text-blue-300 mb-2">Pending admin</p>
              <a
                href={explorerAddrUrl(contractInfo.pendingAdmin)}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-stellar-400 hover:text-stellar-300 break-all text-xs"
              >
                {contractInfo.pendingAdmin}
              </a>
            </div>

            <TxStatusBadge status={cancelTxStatus} txHash={cancelTxHash} error={cancelTxError} />

            <button
              onClick={handleCancelTransfer}
              className="btn-secondary w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto"
              disabled={cancelPending}
            >
              {cancelPending
                ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
                : 'Cancel'
              }
            </button>
          </div>
        ) : (
          <form onSubmit={handleTransferAdmin} className="space-y-3 md:space-y-4">
            <div>
              <label className="label">New admin</label>
              <input
                className="input"
                type="text"
                value={transferNewAdmin}
                onChange={(e) => setTransferNewAdmin(e.target.value.trim())}
                placeholder="G... or C..."
                disabled={transferPending}
              />
            </div>

            <TxStatusBadge status={transferTxStatus} txHash={transferTxHash} error={transferTxError} />

            <button
              type="submit"
              className="btn-primary w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto"
              disabled={!transferNewAdmin || transferPending}
            >
              {transferPending
                ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
                : 'Initiate transfer'
              }
            </button>
          </form>
        )}

        {contractInfo.pendingAdmin && wallet?.address === contractInfo.pendingAdmin && (
          <div className="mt-4 md:mt-6 border-t border-slate-700/40 pt-4 md:pt-6">
            <p className="text-xs md:text-sm text-slate-400 mb-3 md:mb-4">
              You are the pending admin. Accept to complete.
            </p>

            <TxStatusBadge status={acceptTxStatus} txHash={acceptTxHash} error={acceptTxError} />

            <button
              onClick={handleAcceptAdmin}
              className="btn-primary w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto mt-3"
              disabled={acceptPending}
            >
              {acceptPending
                ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
                : 'Accept admin role'
              }
            </button>
          </div>
        )}
      </div>

      {/* Renounce admin */}
      <div className="card p-4 md:p-6">
        <div className="rounded-lg bg-red-900/20 border border-red-700/40 p-3 mb-4">
          <div className="flex items-center gap-2 md:gap-3">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-4 h-4 md:w-5 md:h-5 text-red-400 flex-shrink-0">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
            </svg>
            <div className="min-w-0">
              <p className="font-semibold text-red-400 text-sm md:text-base">Renounce Admin</p>
              <p className="text-xs text-red-300 mt-0.5">Permanent. Cannot undo.</p>
            </div>
          </div>
        </div>

        <p className="text-xs md:text-sm text-slate-400 mb-4">
          Make contract fully trustless. No one can pause, emergency withdraw, or change config.
        </p>

        <form
          onSubmit={(e) => {
            e.preventDefault()
            if (renounceConfirmText === 'RENOUNCE') {
              handleRenounceAdmin()
            }
          }}
          className="space-y-3 md:space-y-4"
        >
          <div>
            <label className="label">Type "RENOUNCE"</label>
            <input
              className="input"
              type="text"
              value={renounceConfirmText}
              onChange={(e) => setRenounceConfirmText(e.target.value)}
              placeholder="RENOUNCE"
              disabled={renouncePending}
            />
          </div>

          <TxStatusBadge status={renounceTxStatus} txHash={renounceTxHash} error={renounceTxError} />

          <button
            type="submit"
            className="btn-danger w-full text-xs md:text-sm py-2.5 min-h-10 md:min-h-auto"
            disabled={renounceConfirmText !== 'RENOUNCE' || renouncePending}
          >
            {renouncePending
              ? <span className="w-4 h-4 border-2 border-current/30 border-t-current rounded-full animate-spin" />
              : 'Renounce admin'
            }
          </button>
        </form>
      </div>
    </div>
  )
}
