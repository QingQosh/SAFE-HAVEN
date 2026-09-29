import { useState } from 'react'
import toast from 'react-hot-toast'
import { use2FA } from '../hooks/use2FA'
import { TwoFASetup } from './TwoFASetup'

/**
 * Component for managing 2FA settings.
 * Allows enabling, disabling, and managing 2FA preferences.
 */
export function TwoFASettings() {
  const { twoFAState, disable2FA } = use2FA()
  const [showSetup, setShowSetup] = useState(false)
  const [showDisableConfirm, setShowDisableConfirm] = useState(false)

  const handleDisable2FA = () => {
    disable2FA()
    setShowDisableConfirm(false)
    toast.success('2FA has been disabled')
  }

  if (showSetup) {
    return (
      <TwoFASetup
        onComplete={() => {
          setShowSetup(false)
          toast.success('2FA setup complete! Your account is now protected.')
        }}
        onCancel={() => setShowSetup(false)}
      />
    )
  }

  return (
    <div className="card p-6">
      <h3 className="font-semibold text-lg mb-4">Two-Factor Authentication</h3>

      {!twoFAState.enabled ? (
        <div className="space-y-4">
          <p className="text-sm text-slate-300">
            Add an extra layer of security to sensitive operations like withdrawals and admin transfers.
          </p>
          <div className="bg-slate-800/40 rounded-lg p-3 space-y-2 text-sm text-slate-400">
            <p>
              <strong className="text-slate-300">✓ Protected operations:</strong>
              <br />
              • Withdrawals
              <br />
              • Cancel deposits
              <br />
              • Admin transfers
              <br />
              • Contract pause/unpause
            </p>
          </div>
          <button onClick={() => setShowSetup(true)} className="btn-primary w-full">
            Enable 2FA
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="flex items-center justify-between p-3 bg-green-900/30 rounded-lg border border-green-700/40">
            <div>
              <p className="text-sm font-medium text-green-400">✓ 2FA Enabled</p>
              <p className="text-xs text-green-300 mt-0.5">
                {twoFAState.backupCodes?.length || 0} backup codes remaining
              </p>
            </div>
            <svg viewBox="0 0 24 24" fill="currentColor" className="w-5 h-5 text-green-400">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
            </svg>
          </div>

          <div className="bg-slate-800/40 rounded-lg p-3 space-y-2 text-xs text-slate-400">
            <p>
              <strong className="text-slate-300">Recovery codes:</strong>
              <br />
              Use backup codes if you lose access to your authenticator app. Each code can only be used once.
            </p>
          </div>

          <button
            onClick={() => setShowSetup(true)}
            className="btn-secondary w-full text-sm"
          >
            Reconfigure 2FA
          </button>

          <button
            onClick={() => setShowDisableConfirm(true)}
            className="btn-danger w-full text-sm"
          >
            Disable 2FA
          </button>

          {showDisableConfirm && (
            <div className="bg-red-900/30 border border-red-700/40 rounded-lg p-3 space-y-3">
              <p className="text-sm text-red-300">
                Are you sure? Disabling 2FA will remove the extra security layer from your sensitive operations.
              </p>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowDisableConfirm(false)}
                  className="btn-secondary flex-1 text-sm"
                >
                  Cancel
                </button>
                <button
                  onClick={handleDisable2FA}
                  className="btn-danger flex-1 text-sm"
                >
                  Confirm Disable
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
