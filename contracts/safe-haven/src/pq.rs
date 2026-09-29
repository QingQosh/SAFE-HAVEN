extern crate alloc;

use alloc::vec::Vec;

use ml_dsa::{MlDsa44, Signature, Verifier, VerifyingKey};
use soroban_sdk::Bytes;

pub const ML_DSA_PUBLIC_KEY_BYTES: u32 = 1_312;
pub const ML_DSA_SIGNATURE_BYTES: u32 = 2_420;

fn copy_bytes(bytes: &Bytes) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() as usize);
    for index in 0..bytes.len() {
        output.push(bytes.get(index).unwrap_or(0));
    }
    output
}

pub fn has_valid_lengths(public_key: &Bytes, signature: &Bytes) -> bool {
    public_key.len() == ML_DSA_PUBLIC_KEY_BYTES && signature.len() == ML_DSA_SIGNATURE_BYTES
}

pub fn is_valid_public_key(public_key: &Bytes) -> bool {
    if public_key.len() != ML_DSA_PUBLIC_KEY_BYTES {
        return false;
    }
    let public_key = copy_bytes(public_key);
    VerifyingKey::<MlDsa44>::try_from(public_key.as_slice()).is_ok()
}

pub fn verify(public_key: &Bytes, signature: &Bytes, message: &Bytes) -> bool {
    if !has_valid_lengths(public_key, signature) {
        return false;
    }

    let public_key = copy_bytes(public_key);
    let signature = copy_bytes(signature);
    let message = copy_bytes(message);

    let Ok(verifying_key) = VerifyingKey::<MlDsa44>::try_from(public_key.as_slice()) else {
        return false;
    };
    let Ok(signature) = Signature::<MlDsa44>::try_from(signature.as_slice()) else {
        return false;
    };

    verifying_key.verify(message.as_slice(), &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ml_dsa::{Keypair, MlDsa44, Seed, SignatureEncoding, Signer, SigningKey};
    use soroban_sdk::Env;

    #[test]
    fn verifies_ml_dsa_signature_and_rejects_tampering() {
        let env = Env::default();
        let signing_key = SigningKey::<MlDsa44>::from_seed(&Seed::default());
        let verifying_key = signing_key.verifying_key();
        let message = b"safe-haven quantum-safe deposit";
        let signature = signing_key.sign(message);
        let public_key = verifying_key.to_bytes();
        let signature = signature.to_bytes();
        let public_key = Bytes::from_slice(&env, public_key.as_ref());
        let signature = Bytes::from_slice(&env, signature.as_ref());
        let message_bytes = Bytes::from_slice(&env, message);

        assert!(verify(&public_key, &signature, &message_bytes));

        let tampered_message = Bytes::from_slice(&env, b"tampered deposit");
        assert!(!verify(&public_key, &signature, &tampered_message));
    }
}
