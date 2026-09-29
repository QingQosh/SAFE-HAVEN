import { beforeEach, describe, expect, it } from 'vitest'
import {
  getRecoveryState,
  RECOVERY_TIMELOCK_SECS,
  registerRecoveryContact,
  recover_account,
  verifyRecoveryCode,
} from '../lib/recovery'

const owner = 'GOWNER'
const newWallet = 'GNEW_WALLET'

describe('recovery MVP', () => {
  beforeEach(() => localStorage.clear())

  it('registers contacts and creates a seven-day recovery request', () => {
    const contact = registerRecoveryContact(owner, 'email', 'owner@example.com')
    const request = recover_account(owner, contact.id, newWallet)

    expect(getRecoveryState(owner).contacts).toHaveLength(1)
    expect(request.newWallet).toBe(newWallet)
    expect(request.unlockAt - request.initiatedAt).toBe(RECOVERY_TIMELOCK_SECS * 1000)
    expect(request.verifiedAt).toBeNull()
  })

  it('requires the registered contact and verifies the displayed code', () => {
    expect(() => recover_account(owner, 'missing', newWallet)).toThrow('registered recovery contact')
    const contact = registerRecoveryContact(owner, 'wallet', 'GCONTACT')
    const request = recover_account(owner, contact.id, newWallet)

    expect(() => verifyRecoveryCode(owner, '000000')).toThrow('Incorrect recovery code')
    const verified = verifyRecoveryCode(owner, request.verificationCode)
    expect(verified.verifiedAt).not.toBeNull()
  })

  it('prevents a second active recovery request', () => {
    const contact = registerRecoveryContact(owner, 'email', 'owner@example.com')
    recover_account(owner, contact.id, newWallet)

    expect(() => recover_account(owner, contact.id, 'GOTHER_WALLET')).toThrow('already in progress')
  })
})