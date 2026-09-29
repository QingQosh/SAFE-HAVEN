import { ml_kem768 } from '@noble/post-quantum/ml-kem.js'

const ENVELOPE_VERSION = 1
const IV_BYTES = 12
const LENGTH_BYTES = 2

export type QuantumSafeKeyPair = ReturnType<typeof ml_kem768.keygen>

export function generateQuantumSafeKeyPair(seed?: Uint8Array): QuantumSafeKeyPair {
  return ml_kem768.keygen(seed)
}

async function aesKey(sharedSecret: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey('raw', sharedSecret, 'AES-GCM', false, ['encrypt', 'decrypt'])
}

/** Encrypts client-only metadata for storage in deposit_quantum_safe. */
export async function encryptQuantumSafeMetadata(
  recipientPublicKey: Uint8Array,
  plaintext: Uint8Array,
): Promise<Uint8Array> {
  const { cipherText, sharedSecret } = ml_kem768.encapsulate(recipientPublicKey)
  const iv = crypto.getRandomValues(new Uint8Array(IV_BYTES))
  const ciphertext = new Uint8Array(
    await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, await aesKey(sharedSecret), plaintext),
  )
  if (cipherText.length > 0xffff) throw new Error('ML-KEM ciphertext is too large')

  const envelope = new Uint8Array(1 + LENGTH_BYTES + cipherText.length + IV_BYTES + ciphertext.length)
  const view = new DataView(envelope.buffer)
  envelope[0] = ENVELOPE_VERSION
  view.setUint16(1, cipherText.length)
  envelope.set(cipherText, 1 + LENGTH_BYTES)
  envelope.set(iv, 1 + LENGTH_BYTES + cipherText.length)
  envelope.set(ciphertext, 1 + LENGTH_BYTES + cipherText.length + IV_BYTES)
  return envelope
}

/** Decrypts an envelope returned by encryptQuantumSafeMetadata. */
export async function decryptQuantumSafeMetadata(
  recipientSecretKey: Uint8Array,
  envelope: Uint8Array,
): Promise<Uint8Array> {
  if (envelope.length < 1 + LENGTH_BYTES + IV_BYTES + 16 || envelope[0] !== ENVELOPE_VERSION) {
    throw new Error('Invalid quantum-safe metadata envelope')
  }
  const view = new DataView(envelope.buffer, envelope.byteOffset, envelope.byteLength)
  const kemLength = view.getUint16(1)
  const kemStart = 1 + LENGTH_BYTES
  const ivStart = kemStart + kemLength
  const ciphertextStart = ivStart + IV_BYTES
  if (ciphertextStart > envelope.length) throw new Error('Invalid quantum-safe metadata envelope')

  const cipherText = envelope.slice(kemStart, ivStart)
  const iv = envelope.slice(ivStart, ciphertextStart)
  const ciphertext = envelope.slice(ciphertextStart)
  const sharedSecret = ml_kem768.decapsulate(cipherText, recipientSecretKey)
  return new Uint8Array(
    await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, await aesKey(sharedSecret), ciphertext),
  )
}
