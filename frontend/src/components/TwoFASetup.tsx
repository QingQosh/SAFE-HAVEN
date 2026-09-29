import { useState } from 'react'
import QRCode from 'qrcode.react'
import { use2FA } from '../hooks/use2FA'

interface TwoFASetupProps {
  onComplete: () => void
  onCancel: () => void
}

/**
 * Component for setting up 2FA (TOTP).
 * Shows QR code, backup codes, and verification.
 */
export function TwoFASetup({ onComplete, onCancel }: TwoFASetupProps) {
  const { generateSecret, verifyCode, enable2FA } = use2FA()
  const [step, setStep] = useState<'generate' | 'verify' | 'backup'>('generate')
  const [otpauthUrl, setOtpauthUrl] = useState<string>('')
  const [secret, setSecret] = useState<string>('')
  const [backupCodes, setBackupCodes] = useState<string[]>([])
  const [verificationCode, setVerificationCode] = useState('')
  const [verifyError, setVerifyError] = useState('')
  const [backupCopied, setBackupCopied] = useState(false)

  const handleGenerate = () => {
    const { secret: sec, otpauthUrl: url, backupCodes: codes } = generateSecret()
    setSecret(sec)
    setOtpauthUrl(url)
    setBackupCodes(codes)
    setStep('verify')
  }

  const handleVerify = () => {
    setVerifyError('')
    if (!verificationCode.trim()) {
      setVerifyError('Please enter your TOTP code')
      return
    }

    if (verifyCode(verificationCode, secret)) {
      enable2FA(secret, backupCodes)
      setStep('backup')
    } else {
      setVerifyError('Invalid code. Please try again.')
      setVerificationCode('')
    }
  }

  const handleCopyBackupCodes = () => {
    const text = backupCodes.join('\n')
    navigator.clipboard.writeText(text).then(() => {
      setBackupCopied(true)
      setTimeout(() => setBackupCopied(false), 2000)
    })
  }

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-slate-900 border border-slate-700/60 rounded-2xl shadow-2xl max-w-md w-full p-8 space-y-6">
        {step === 'generate' && (
          <>
            <div className="text-center">
              <h2 className="text-2xl font-bold text-white mb-2">Set Up 2FA</h2>
              <p className="text-slate-300 text-sm">
                Time-based one-time password (TOTP) authentication
              </p>
            </div>

            <div className="bg-slate-800/60 rounded-lg p-4 space-y-2 text-sm text-slate-300">
              <p>
                <strong className="text-slate-200">1. Get an authenticator app:</strong>
              </p>
              <ul className="list-disc list-inside text-xs text-slate-400 space-y-1">
                <li>Google Authenticator</li>
                <li>Microsoft Authenticator</li>
                <li>Authy</li>
                <li>Any TOTP-compatible app</li>
              </ul>
              <p className="mt-2">
                <strong className="text-slate-200">2. Scan the QR code</strong> on the next step
              </p>
            </div>

            <div className="flex gap-3">
              <button onClick={onCancel} className="btn-secondary flex-1">
                Cancel
              </button>
              <button onClick={handleGenerate} className="btn-primary flex-1">
                Next
              </button>
            </div>
          </>
        )}

        {step === 'verify' && (
          <>
            <div className="text-center">
              <h2 className="text-2xl font-bold text-white mb-2">Scan QR Code</h2>
              <p className="text-slate-300 text-sm">
                Open your authenticator app and scan this code
              </p>
            </div>

            <div className="bg-white p-4 rounded-lg flex justify-center">
              {otpauthUrl && (
                <QRCode
                  value={otpauthUrl}
                  size={200}
                  level="H"
                  includeMargin={true}
                  quietZone={10}
                />
              )}
            </div>

            <div className="space-y-2">
              <label className="label">Enter 6-digit code from your app</label>
              <input
                type="text"
                inputMode="numeric"
                maxLength={6}
                placeholder="000000"
                value={verificationCode}
                onChange={(e) => {
                  setVerifyError('')
                  setVerificationCode(e.target.value.replace(/\D/g, ''))
                }}
                className={`input text-center text-2xl tracking-widest font-mono ${
                  verifyError ? 'border-red-500 focus:ring-red-500/50' : ''
                }`}
              />
              {verifyError && <p className="text-xs text-red-400">{verifyError}</p>}
            </div>

            <div className="flex gap-3">
              <button onClick={() => setStep('generate')} className="btn-secondary flex-1">
                Back
              </button>
              <button onClick={handleVerify} className="btn-primary flex-1">
                Verify & Continue
              </button>
            </div>
          </>
        )}

        {step === 'backup' && (
          <>
            <div className="text-center">
              <h2 className="text-2xl font-bold text-white mb-2">Save Backup Codes</h2>
              <p className="text-slate-300 text-sm">
                Store these codes in a safe place. Use them if you lose access to your authenticator.
              </p>
            </div>

            <div className="bg-slate-800/60 rounded-lg p-4 space-y-2 max-h-48 overflow-y-auto">
              {backupCodes.map((code, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between text-sm font-mono bg-slate-700/40 px-2 py-1.5 rounded border border-slate-600/30"
                >
                  <span className="text-slate-400">{idx + 1}.</span>
                  <span className="text-slate-200">{code}</span>
                  <span className="text-xs text-slate-500">Copy</span>
                </div>
              ))}
            </div>

            <button
              onClick={handleCopyBackupCodes}
              className="btn-secondary w-full text-sm"
            >
              {backupCopied ? '✓ Copied to clipboard' : 'Copy all codes'}
            </button>

            <label className="flex items-start gap-2 cursor-pointer">
              <input
                type="checkbox"
                className="mt-1"
                id="backup-saved"
              />
              <span className="text-xs text-slate-400">
                I have saved my backup codes in a safe place
              </span>
            </label>

            <button
              onClick={onComplete}
              className="btn-primary w-full"
              disabled={!(document.getElementById('backup-saved') as HTMLInputElement)?.checked}
            >
              Setup Complete
            </button>
          </>
        )}
      </div>
    </div>
  )
}
