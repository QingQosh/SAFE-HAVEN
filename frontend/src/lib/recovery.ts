import type { RecoveryContact, RecoveryRequest, RecoveryContactType } from '../types'

export const RECOVERY_TIMELOCK_SECS = 7 * 24 * 60 * 60

export interface RecoveryState {
  contacts: RecoveryContact[]
  request: RecoveryRequest | null
}

const emptyState = (): RecoveryState => ({ contacts: [], request: null })

function storageKey(walletAddress: string) {
  return `safe_haven_recovery_${walletAddress}`
}

function readState(walletAddress: string): RecoveryState {
  try {
    const raw = localStorage.getItem(storageKey(walletAddress))
    if (!raw) return emptyState()
    const parsed = JSON.parse(raw) as Partial<RecoveryState>
    return {
      contacts: Array.isArray(parsed.contacts) ? parsed.contacts : [],
      request: parsed.request ?? null,
    }
  } catch {
    return emptyState()
  }
}

function writeState(walletAddress: string, state: RecoveryState) {
  localStorage.setItem(storageKey(walletAddress), JSON.stringify(state))
}

function makeId() {
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`
}

function makeVerificationCode() {
  return String(Math.floor(100000 + Math.random() * 900000))
}

export function getRecoveryState(walletAddress: string): RecoveryState {
  return readState(walletAddress)
}

export function registerRecoveryContact(
  walletAddress: string,
  type: RecoveryContactType,
  value: string,
): RecoveryContact {
  const state = readState(walletAddress)
  const contact: RecoveryContact = { id: makeId(), type, value: value.trim(), addedAt: Date.now() }
  writeState(walletAddress, { ...state, contacts: [...state.contacts, contact] })
  return contact
}

/** Frontend-only MVP for the future recover_account contract/API call. */
export function recover_account(
  walletAddress: string,
  recoveryContactId: string,
  newWallet: string,
): RecoveryRequest {
  const state = readState(walletAddress)
  if (!state.contacts.some((contact) => contact.id === recoveryContactId)) {
    throw new Error('Choose a registered recovery contact')
  }
  if (state.request && state.request.unlockAt > Date.now()) {
    throw new Error('A recovery request is already in progress')
  }

  const now = Date.now()
  const request: RecoveryRequest = {
    recoveryContactId,
    newWallet: newWallet.trim(),
    verificationCode: makeVerificationCode(),
    initiatedAt: now,
    unlockAt: now + RECOVERY_TIMELOCK_SECS * 1000,
    verifiedAt: null,
  }
  writeState(walletAddress, { ...state, request })
  return request
}

export function verifyRecoveryCode(walletAddress: string, code: string): RecoveryRequest {
  const state = readState(walletAddress)
  if (!state.request) throw new Error('No recovery request is active')
  if (state.request.verificationCode !== code.trim()) throw new Error('Incorrect recovery code')
  const request = { ...state.request, verifiedAt: Date.now() }
  writeState(walletAddress, { ...state, request })
  return request
}

export function cancelRecovery(walletAddress: string) {
  const state = readState(walletAddress)
  writeState(walletAddress, { ...state, request: null })
}

export function removeRecoveryContact(walletAddress: string, contactId: string) {
  const state = readState(walletAddress)
  writeState(walletAddress, {
    ...state,
    contacts: state.contacts.filter((contact) => contact.id !== contactId),
  })
}