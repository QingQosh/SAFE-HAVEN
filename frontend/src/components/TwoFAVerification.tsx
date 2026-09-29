import { useState } from 'react'
import { use2FA } from '../hooks/use2FA'

interface TwoFAVerificationProps {
  onVerified: () => void
  onCancel: () => void
}

/**
 * Modal for verifying 2FA (TOTP) before sensitive operations.
 * Supports both TOTP codes and backup codes.
 */
export function TwoFAVerification({ onVerified, onCancel }: TwoFAVerificationProps) {
  const { twoFAState, verifyCode, verifyBackupCode, updateBackupCodes } = use2FA()
  const [code, setCode] = useState('')
  const [useBackup, setUseBackup] = useState(false)
  const [error, setError] = useState('')
  const [isVerifying, setIsVerifying] = useState(false)

  if (!twoFAState.enabled || !twoFAState.secret) {
    return null
  }

  const handleVerify = async () => {
    setError('')
    setIsVerifying(true)

    try {
      if (useBackup) {
        if (!twoFAState.backupCodes) {
          setError('No backup codes available')
          return
        }
        const { valid, remaining } = verifyBackupCode(code, twoFAState.backupCodes)
        if (valid) {
          updateBackupCodes(remaining)
          onVerified()
        } else {
          setError('Invalid backup code')
        }
      } else {
        if (!code.trim()) {
          setError('Please enter your 6-digit code')
          return
        }
        if (verifyCode(code, twoFAState.secret)) {
          onVerified()
        } else {
          setError('Invalid TOTP code. Try again or use a backup code.')
        }
      }
    } catch (e) {
      setError('Verification failed. Please try again.')
      console.error('2FA verification error:', e)
    } finally {
      setIsVerifying(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-slate-900 border border-slate-700/60 rounded-2xl shadow-2xl max-w-sm w-full p-6 space-y-4">
        <div>
          <h2 className="text-xl font-bold text-white mb-1">2FA Required</h2>
          <p className="text-slate-300 text-sm">
            Enter your authentication code to proceed with this sensitive operation
          </p>
        </div>

        <div className="bg-slate-800/60 rounded-lg p-3 border border-slate-700/40">
          {!useBackup ? (
            <div className="space-y-2">
              <label className="label text-xs">
                6-digit code from your authenticator
              </label>
              <input
                type="text"
                inputMode="numeric"
                maxLength={6}
                placeholder="000000"
                value={code}
                onChange={(e) => {
                  setError('')
                  setCode(e.target.value.replace(/\D/g, ''))
                }}
                className={`input text-center text-2xl tracking-widest font-mono ${
                  error ? 'border-red-500' : ''
                }`}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <label className="label text-xs">
                Enter one backup code
              </label>
              <input
                type="text"
                placeholder="Example: ABC12345"
                value={code}
                onChange={(e) => {
                  setError('')
                  setCode(e.target.value.toUpperCase())
                }}
                className={`input font-mono ${error ? 'border-red-500' : ''}`}
              />
              <p className="text-xs text-slate-400">
                {twoFAState.backupCodes?.length || 0} backup codes remaining
              </p>
            </div>
          )}

          {error && (
            <p className="text-xs text-red-400 mt-2 flex items-start gap-1">
              <span>⚠️</span>
              <span>{error}</span>
            </p>
          )}
        </div>

        {!useBackup && twoFAState.backupCodes && twoFAState.backupCodes.length > 0 && (
          <button
            onClick={() => {
              setUseBackup(true)
              setCode('')
              setError('')
            }}
            className="text-xs text-stellar-400 hover:text-stellar-300 transition-colors"
          >
            Use backup code instead
          </button>
        )}

        {useBackup && (
          <button
            onClick={() => {
              setUseBackup(false)
              setCode('')
              setError('')
            }}
            className="text-xs text-stellar-400 hover:text-stellar-300 transition-colors"
          >
            Back to authenticator code
          </button>
        )}

        <div className="flex gap-3 pt-2">
          <button onClick={onCancel} className="btn-secondary flex-1">
            Cancel
          </button>
          <button
            onClick={handleVerify}
            disabled={isVerifying || !code.trim()}
            className="btn-primary flex-1"
          >
            {isVerifying ? (
              <span className="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin inline-block" />
            ) : (
              'Verify'
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
