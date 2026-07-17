use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, ensure, Result};
use rand::RngCore;

/// 96-bit nonce as recommended by NIST SP 800-38D for AES-GCM.
const NONCE_LEN: usize = 12;

/// Encrypt `plaintext` and return `nonce || ciphertext`.
pub fn encrypt(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow!("AES-256-GCM encryption failed: {e}"))?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt data previously produced by [`encrypt`].
pub fn decrypt(key_bytes: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        data.len() > NONCE_LEN,
        "encrypted payload is too short ({} bytes)",
        data.len()
    );

    let (nonce_slice, ciphertext) = data.split_at(NONCE_LEN);
    let nonce_arr: [u8; NONCE_LEN] = nonce_slice
        .try_into()
        .expect("split_at guarantees exactly NONCE_LEN bytes");
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));
    let nonce = Nonce::from(nonce_arr);

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| anyhow!("AES-256-GCM decryption failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = [0xABu8; 32];
        let plaintext = b"{\"plate\":\"LT893DK\",\"owner\":\"test\"}";

        let encrypted = encrypt(&key, plaintext).unwrap();

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len());

        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = [0xABu8; 32];
        let wrong_key = [0xCDu8; 32];
        let plaintext = b"secret vehicle data";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let result = decrypt(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_rejects_short_payload() {
        let key = [0xABu8; 32];
        let result = decrypt(&key, &[0u8; 12]);
        assert!(result.is_err());
    }
}
