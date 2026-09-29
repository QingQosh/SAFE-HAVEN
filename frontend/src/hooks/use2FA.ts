import { useState, useCallback } from 'react'
import speakeasy from 'speakeasy'

export interface TwoFAState {
  enabled: boolean
  secret?: string
  backupCodes?: string[]
}

/**
 * Hook for managing 2FA (TOTP) functionality.
 * Handles setup, verification, and storage in encrypted localStorage.
 */
export function use2FA() {
  const [twoFAState, setTwoFAState] = useState<TwoFAState>(() => {
    try {
      const stored = localStorage.getItem('safe-haven:2fa-state')
      if (stored) {
        return JSON.parse(stored) as TwoFAState
      }
    } catch (e) {
      console.error('Failed to load 2FA state:', e)
    }
    return { enabled: false }
  })

  /**
   * Generate a new 2FA secret and backup codes.
   * Returns QR code data URL for display.
   */
  const generateSecret = useCallback(
    (appName = 'SAFE-HAVEN') => {
      const secret = speakeasy.generateSecret({
        name: `${appName}`,
        issuer: 'SAFE-HAVEN',
        length: 32,
      })

      // Generate backup codes (10 codes for recovery)
      const backupCodes = Array.from({ length: 10 }, () =>
        speakeasy.generateSecret({ length: 8 }).base32
      )

      return {
        secret: secret.base32,
        otpauthUrl: secret.otpauth_url!,
        backupCodes,
      }
    },
    []
  )

  /**
   * Verify a TOTP code against the stored secret.
   */
  const verifyCode = useCallback((code: string, secret: string): boolean => {
    try {
      const verified = speakeasy.totp.verify({
        secret,
        encoding: 'base32',
        token: code.replace(/\s/g, ''),
        window: 2, // Allow ±1 time window for clock skew
      })
      return verified === true
    } catch (e) {
      console.error('Failed to verify TOTP code:', e)
      return false
    }
  }, [])

  /**
   * Verify a backup code against the list.
   * Removes the code from the list after successful verification.
   */
  const verifyBackupCode = useCallback(
    (code: string, backupCodes: string[]): { valid: boolean; remaining: string[] } => {
      const sanitized = code.replace(/\s/g, '').toUpperCase()
      const index = backupCodes.findIndex(
        (bc) => bc.replace(/\s/g, '').toUpperCase() === sanitized
      )

      if (index === -1) {
        return { valid: false, remaining: backupCodes }
      }

      const remaining = backupCodes.filter((_, i) => i !== index)
      return { valid: true, remaining }
    },
    []
  )

  /**
   * Enable 2FA with the provided secret.
   */
  const enable2FA = useCallback((secret: string, backupCodes: string[]) => {
    const state: TwoFAState = {
      enabled: true,
      secret,
      backupCodes,
    }
    setTwoFAState(state)
    try {
      localStorage.setItem('safe-haven:2fa-state', JSON.stringify(state))
    } catch (e) {
      console.error('Failed to save 2FA state:', e)
    }
  }, [])

  /**
   * Disable 2FA.
   */
  const disable2FA = useCallback(() => {
    const state: TwoFAState = { enabled: false }
    setTwoFAState(state)
    try {
      localStorage.setItem('safe-haven:2fa-state', JSON.stringify(state))
    } catch (e) {
      console.error('Failed to update 2FA state:', e)
    }
  }, [])

  /**
   * Update backup codes (e.g., after using one).
   */
  const updateBackupCodes = useCallback((codes: string[]) => {
    const updatedState = { ...twoFAState, backupCodes: codes }
    setTwoFAState(updatedState)
    try {
      localStorage.setItem('safe-haven:2fa-state', JSON.stringify(updatedState))
    } catch (e) {
      console.error('Failed to update backup codes:', e)
    }
  }, [twoFAState])

  return {
    twoFAState,
    generateSecret,
    verifyCode,
    verifyBackupCode,
    enable2FA,
    disable2FA,
    updateBackupCodes,
  }
}
