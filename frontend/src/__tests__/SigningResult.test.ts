import { describe, it, expect } from 'vitest'
import type { SigningResult } from '../types'

/**
 * Test that the SigningResult discriminated union properly distinguishes
 * between user rejection, network errors, and signing failures
 */
describe('SigningResult discriminated union', () => {
  it('should handle successful signing', () => {
    const result: SigningResult = { signed: true, xdr: 'AAAAAgAAAAA...' }
    
    expect(result.signed).toBe(true)
    if (result.signed) {
      expect(result.xdr).toBeDefined()
      expect(typeof result.xdr).toBe('string')
    }
  })

  it('should handle user rejection', () => {
    const result: SigningResult = { signed: false, rejected: true }
    
    expect(result.signed).toBe(false)
    expect(result.rejected).toBe(true)
    
    // Verify error field doesn't exist on rejection
    expect('error' in result).toBe(false)
  })

  it('should handle signing error', () => {
    const result: SigningResult = { signed: false, rejected: false, error: 'Network timeout' }
    
    expect(result.signed).toBe(false)
    expect(result.rejected).toBe(false)
    expect(result.error).toBeDefined()
    expect(result.error).toBe('Network timeout')
  })

  it('should allow callers to distinguish between rejection and error', () => {
    const rejection: SigningResult = { signed: false, rejected: true }
    const error: SigningResult = { signed: false, rejected: false, error: 'Failed to sign' }
    
    // For rejection: silently reset state (no toast needed)
    const shouldShowRetryPrompt = (result: SigningResult) => {
      return !result.signed && !result.rejected
    }

    expect(shouldShowRetryPrompt(rejection)).toBe(false)
    expect(shouldShowRetryPrompt(error)).toBe(true)
  })

  it('should be narrowable with type guards', () => {
    const results: SigningResult[] = [
      { signed: true, xdr: 'test1' },
      { signed: false, rejected: true },
      { signed: false, rejected: false, error: 'test error' },
    ]

    const signed = results.filter((r) => r.signed)
    const rejected = results.filter((r) => !r.signed && r.rejected)
    const errors = results.filter((r) => !r.signed && !r.rejected)

    expect(signed).toHaveLength(1)
    expect(rejected).toHaveLength(1)
    expect(errors).toHaveLength(1)
  })
})
