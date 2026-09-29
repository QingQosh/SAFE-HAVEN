import { describe, expect, it } from 'vitest'
import {
  decryptQuantumSafeMetadata,
  encryptQuantumSafeMetadata,
  generateQuantumSafeKeyPair,
} from '../lib/quantumSafe'

describe('quantum-safe metadata encryption', () => {
  it('round-trips metadata through ML-KEM-768 and AES-GCM', async () => {
    const keys = generateQuantumSafeKeyPair()
    const plaintext = new TextEncoder().encode('private beneficiary note')

    const envelope = await encryptQuantumSafeMetadata(keys.publicKey, plaintext)
    const decrypted = await decryptQuantumSafeMetadata(keys.secretKey, envelope)

    expect(new TextDecoder().decode(decrypted)).toBe('private beneficiary note')
  })

  it('rejects authenticated ciphertext tampering', async () => {
    const keys = generateQuantumSafeKeyPair()
    const envelope = await encryptQuantumSafeMetadata(
      keys.publicKey,
      new TextEncoder().encode('private note'),
    )
    envelope[envelope.length - 1] ^= 1

    await expect(decryptQuantumSafeMetadata(keys.secretKey, envelope)).rejects.toThrow()
  })
})
