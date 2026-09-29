# ADR-008 - Quantum-Safe Deposit Authorization and Metadata

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-09-25 |
| Deciders | SAFE-HAVEN core contributors |
|

## Decision

New quantum-safe deposits use ML-DSA-44, the NIST FIPS 204 security-category-2
parameter set, through the pure-Rust `ml-dsa` crate. A depositor registers one
encoded 1312-byte public key with `register_quantum_safe_key`. The contract
verifies the 2420-byte ML-DSA signature over canonical XDR containing:

- the depositor and token addresses;
- amount, unlock time, and penalty;
- the current per-depositor deposit ID; and
- the encrypted metadata bytes.

The deposit ID is part of the signed payload, so a signature cannot be replayed
for a later deposit. `quantum_safe_message` returns the exact bytes a client
must sign. `deposit_quantum_safe` still requires Stellar account authorization
because the token contract requires authorization for the transfer from the
depositor. ML-DSA therefore protects deposit intent and provides migration-ready
post-quantum authorization; it does not replace Stellar's account signature or
make an existing classical Stellar key quantum-safe.

Sensitive metadata is client-side ciphertext. The contract stores it as opaque
bytes under a separate key and never attempts to decrypt it. Clients should use
NIST-standard ML-KEM (FIPS 203), with authenticated encryption such as
AES-256-GCM or ChaCha20-Poly1305 for the payload. Plaintext must never be sent
to the contract.

## Backward compatibility and migration

Existing `VaultEntry`, `LedgerVaultEntry`, and `MultiTokenVaultEntry` storage
layouts are unchanged. Existing deposits continue to use their current
withdrawal and cancellation paths. Migration is opt-in:

1. Generate an ML-DSA-44 key pair and an ML-KEM recipient key pair off-chain.
2. Register the ML-DSA public key with `register_quantum_safe_key`.
3. Encrypt any client metadata off-chain and retain the KEM secret key securely.
4. Sign `quantum_safe_message` and submit `deposit_quantum_safe`.
5. Move funds from a legacy deposit with the normal withdrawal path, then create
   a new quantum-safe deposit.

A future contract upgrade may add an explicit account/key rotation flow. No
forced migration is performed by this release.

## Security properties and assumptions

- ML-DSA-44 verification is performed on-chain using the FIPS 204 algorithm;
  signatures and public keys are length- and encoding-checked.
- Signed fields are domain-separated by the contract entry point's XDR shape and
  include the next deposit ID, preventing cross-deposit replay.
- Quantum resistance assumes the ML-DSA and ML-KEM standards and implementations
  remain secure, the client protects private keys, and ciphertext is
  authenticated before decryption.
- The contract cannot provide confidentiality for ledger-visible fields such as
  token, amount, depositor, or lock time. Only client metadata can be encrypted.
- ML-DSA verification increases CPU/instruction cost substantially compared with
  Stellar native authorization. It is intentionally opt-in and is not enabled
  for legacy deposits.

## Alternatives rejected

- A hash or commitment was rejected because it is not signature verification.
- Replacing `Address::require_auth` was rejected because Soroban token transfers
  still require classical Stellar authorization.
- Adding fields to `VaultEntry` was rejected because it would break decoding of
  existing persistent entries; encrypted metadata uses separate storage keys.
