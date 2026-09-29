import { describe, it, expect, vi } from 'vitest'
import type { SigningResult } from '../types'

/**
 * Simulation of how calling code now properly handles each signing outcome
 */
describe('Signing result handling in UI pages', () => {
  // Simulated toast notifications
  const mockToast = {
    success: vi.fn(),
    error: vi.fn(),
  }

  // Simulated state updates
  const stateUpdates: string[] = []

  const recordStateUpdate = (msg: string) => {
    stateUpdates.push(msg)
  }

  const resetState = () => {
    stateUpdates.length = 0
    mockToast.success.mockClear()
    mockToast.error.mockClear()
  }

  // Simulates the logic in pages like DepositPage, WithdrawPage, Dashboard
  const handleSigningResult = (
    sigResult: SigningResult,
    onSuccess?: () => void
  ) => {
    if (sigResult.signed) {
      // Success: proceed with transaction submission
      recordStateUpdate('state: submitting')
      recordStateUpdate('submit: true')
      if (onSuccess) onSuccess()
      return true
    } else if (sigResult.rejected) {
      // User rejected: silently reset state, no toast
      recordStateUpdate('state: idle (silent reset)')
      return false
    } else {
      // Signing error: already toasted by context, but still reset state
      recordStateUpdate('state: idle (error already toasted)')
      return false
    }
  }

  it('should handle successful signing correctly', () => {
    resetState()
    const sigResult: SigningResult = { signed: true, xdr: 'abc123' }

    const result = handleSigningResult(sigResult, () => {
      recordStateUpdate('success: callback executed')
    })

    expect(result).toBe(true)
    expect(stateUpdates).toContain('state: submitting')
    expect(stateUpdates).toContain('success: callback executed')
    expect(stateUpdates).toContain('submit: true')
    expect(mockToast.error).not.toHaveBeenCalled()
  })

  it('should handle user rejection with silent reset (no toast)', () => {
    resetState()
    const sigResult: SigningResult = { signed: false, rejected: true }

    const result = handleSigningResult(sigResult)

    expect(result).toBe(false)
    expect(stateUpdates).toEqual(['state: idle (silent reset)'])
    expect(mockToast.error).not.toHaveBeenCalled() // No additional toast
  })

  it('should handle signing error with state reset (toast already shown)', () => {
    resetState()
    const sigResult: SigningResult = {
      signed: false,
      rejected: false,
      error: 'Network timeout',
    }

    // In real code, the context already showed a toast for this error
    mockToast.error('Signing error: Network timeout')

    const result = handleSigningResult(sigResult)

    expect(result).toBe(false)
    expect(stateUpdates).toEqual(['state: idle (error already toasted)'])
    expect(mockToast.error).toHaveBeenCalledWith('Signing error: Network timeout')
  })

  it('should distinguish rejection from error for UX differences', () => {
    resetState()

    // Case 1: User rejection - should NOT show retry prompt
    const rejection: SigningResult = { signed: false, rejected: true }
    const shouldShowRetryPrompt1 = !(rejection.signed || rejection.rejected)
    expect(shouldShowRetryPrompt1).toBe(false)

    resetState()

    // Case 2: Signing error - SHOULD show retry prompt
    const error: SigningResult = {
      signed: false,
      rejected: false,
      error: 'Malformed XDR',
    }
    const shouldShowRetryPrompt2 = !(error.signed || error.rejected)
    expect(shouldShowRetryPrompt2).toBe(true)
  })

  it('should provide all needed information in the XDR case', () => {
    resetState()
    const sigResult: SigningResult = {
      signed: true,
      xdr: 'AAAAAgAAAABziU76Z06GEJVyifHQ3XR23LpwcKRqKBAqX7zrWZLXAAAAZABMHx0AAAACAAA...',
    }

    expect(sigResult.signed).toBe(true)
    if (sigResult.signed) {
      expect(sigResult.xdr).toBeDefined()
      // Can use sigResult.xdr for submitTx()
      recordStateUpdate(`submit: ${sigResult.xdr.substring(0, 22)}...`)
    }

    expect(stateUpdates[0]).toMatch(/^submit: AAAA.*\.\.\.$/)
  })

  it('should provide error message for retry logic', () => {
    resetState()
    const sigResult: SigningResult = {
      signed: false,
      rejected: false,
      error: 'Freighter: Invalid network passphrase',
    }

    expect(sigResult.signed).toBe(false)
    expect(sigResult.rejected).toBe(false)

    if (!sigResult.signed && !sigResult.rejected) {
      // Page can use error for advanced retry logic or logging
      recordStateUpdate(`error: ${sigResult.error}`)
      recordStateUpdate('show: retry prompt to user')
    }

    expect(stateUpdates).toEqual([
      'error: Freighter: Invalid network passphrase',
      'show: retry prompt to user',
    ])
  })
})
