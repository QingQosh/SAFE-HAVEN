import { useEffect, useState } from 'react'
import toast from 'react-hot-toast'
import { useWallet } from '../context/WalletContext'
import {
  cancelRecovery,
  getRecoveryState,
  RECOVERY_TIMELOCK_SECS,
  registerRecoveryContact,
  recover_account,
  removeRecoveryContact,
  verifyRecoveryCode,
  type RecoveryState,
} from '../lib/recovery'
import type { RecoveryContactType } from '../types'

const emptyState: RecoveryState = { contacts: [], request: null }

export function SettingsPage() {
  const { wallet } = useWallet()
  const [state, setState] = useState(emptyState)
  const [contactType, setContactType] = useState<RecoveryContactType>('email')
  const [contactValue, setContactValue] = useState('')
  const [selectedContact, setSelectedContact] = useState('')
  const [newWallet, setNewWallet] = useState('')
  const [code, setCode] = useState('')

  useEffect(() => {
    setState(wallet ? getRecoveryState(wallet.address) : emptyState)
  }, [wallet])

  if (!wallet) {
    return <div className="card p-10 text-center text-slate-400">Connect your wallet to manage account recovery.</div>
  }

  const walletAddress = wallet.address
  const request = state.request
  const selected = state.contacts.find((contact) => contact.id === request?.recoveryContactId)
  const remainingDays = request ? Math.max(0, Math.ceil((request.unlockAt - Date.now()) / 86400000)) : 0

  function refresh() {
    setState(getRecoveryState(walletAddress))
  }

  function addContact(event: React.FormEvent) {
    event.preventDefault()
    if (!contactValue.trim() || (contactType === 'email' && !contactValue.includes('@'))) {
      toast.error(contactType === 'email' ? 'Enter a valid email address' : 'Enter a wallet address')
      return
    }
    registerRecoveryContact(walletAddress, contactType, contactValue)
    setContactValue('')
    refresh()
    toast.success('Recovery contact registered')
  }

  function initiateRecovery(event: React.FormEvent) {
    event.preventDefault()
    if (!selectedContact || newWallet.trim().length < 50) {
      toast.error('Choose a contact and enter a valid Stellar wallet address')
      return
    }
    try {
      recover_account(walletAddress, selectedContact, newWallet)
      setNewWallet('')
      refresh()
      toast.success('Recovery initiated. Keep the code somewhere safe.')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not initiate recovery')
    }
  }

  function verifyCode(event: React.FormEvent) {
    event.preventDefault()
    try {
      verifyRecoveryCode(walletAddress, code)
      setCode('')
      refresh()
      toast.success('Recovery contact verified')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Verification failed')
    }
  }

  function abortRecovery() {
    cancelRecovery(walletAddress)
    refresh()
    toast.success('Recovery request cancelled')
  }

  return (
    <div className="space-y-6 max-w-3xl">
      <section className="card p-6 border-stellar-700/50">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="label">Account protection</p>
            <h2 className="text-xl font-semibold">Active guardians</h2>
            <p className="text-sm text-slate-400 mt-1">A contact can help you move your account to a new wallet.</p>
          </div>
          <span className="badge-blue">Centralized MVP</span>
        </div>
        <div className="mt-5 space-y-2">
          {state.contacts.length === 0 && <p className="text-sm text-slate-500 py-3">No recovery contacts registered yet.</p>}
          {state.contacts.map((contact) => (
            <div key={contact.id} className="flex items-center justify-between gap-3 bg-slate-800/50 border border-slate-700/60 rounded-xl p-3">
              <div className="flex items-center gap-3 min-w-0">
                <span className="text-stellar-400">{contact.type === 'email' ? '@' : '◈'}</span>
                <span className="text-sm truncate">{contact.value}</span>
                <span className="badge-green">Active</span>
              </div>
              <button className="text-xs text-slate-500 hover:text-red-400" onClick={() => { removeRecoveryContact(walletAddress, contact.id); refresh() }}>Remove</button>
            </div>
          ))}
        </div>
        <form onSubmit={addContact} className="mt-5 pt-5 border-t border-slate-800 grid sm:grid-cols-[9rem_1fr_auto] gap-3 items-end">
          <label><span className="label">Type</span><select className="input" value={contactType} onChange={(event) => setContactType(event.target.value as RecoveryContactType)}><option value="email">Email</option><option value="wallet">Wallet</option></select></label>
          <label><span className="label">Contact</span><input className="input" value={contactValue} onChange={(event) => setContactValue(event.target.value)} placeholder={contactType === 'email' ? 'you@example.com' : 'G... wallet address'} /></label>
          <button className="btn-secondary" type="submit">Add contact</button>
        </form>
      </section>

      <section className="card p-6">
        <p className="label">Wallet recovery</p>
        <h2 className="text-xl font-semibold">Initiate recovery</h2>
        <p className="text-sm text-slate-400 mt-1">A seven-day time-lock gives you time to cancel an unauthorized request.</p>
        {request ? (
          <div className="mt-5 space-y-4">
            <div className="flex flex-wrap items-center gap-2"><span className="badge-yellow">Pending · {remainingDays} day{remainingDays === 1 ? '' : 's'} remaining</span>{request.verifiedAt ? <span className="badge-green">Contact verified</span> : <span className="badge-blue">Verification needed</span>}</div>
            <div className="bg-slate-800/60 border border-yellow-700/30 rounded-xl p-4 text-sm space-y-2"><p className="text-slate-400">New wallet</p><p className="font-mono text-xs break-all">{request.newWallet}</p><p className="text-slate-400 pt-2">Contact</p><p>{selected?.value ?? 'Registered contact'}</p></div>
            {!request.verifiedAt && <form onSubmit={verifyCode} className="flex gap-3"><input className="input" value={code} onChange={(event) => setCode(event.target.value)} placeholder="6-digit verification code" inputMode="numeric" maxLength={6} /><button className="btn-primary" type="submit">Verify code</button></form>}
            <div className="bg-stellar-900/20 border border-stellar-700/40 rounded-xl p-4"><p className="text-xs uppercase tracking-wide text-stellar-300">Code sent to {selected?.value}</p><p className="font-mono text-2xl tracking-[0.3em] text-white mt-2">{request.verificationCode}</p><p className="text-xs text-slate-500 mt-2">Displayed for this frontend-only MVP. A production service must deliver this privately.</p></div>
            <button className="btn-danger" onClick={abortRecovery}>Cancel recovery</button>
          </div>
        ) : (
          <form onSubmit={initiateRecovery} className="mt-5 space-y-4">
            <label><span className="label">Recovery contact</span><select className="input" value={selectedContact} onChange={(event) => setSelectedContact(event.target.value)}><option value="">Select a registered contact</option>{state.contacts.map((contact) => <option key={contact.id} value={contact.id}>{contact.value}</option>)}</select></label>
            <label><span className="label">New wallet</span><input className="input font-mono" value={newWallet} onChange={(event) => setNewWallet(event.target.value)} placeholder="G..." /></label>
            <button className="btn-primary" type="submit" disabled={state.contacts.length === 0}>Initiate recovery</button>
          </form>
        )}
        <p className="text-xs text-slate-500 mt-5">Recovery requests unlock after {RECOVERY_TIMELOCK_SECS / 86400} days. This MVP does not change the Soroban account yet.</p>
      </section>
    </div>
  )
}