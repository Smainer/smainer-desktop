use anyhow::Result;
use ring::{digest, pbkdf2, rand, aead};
use ring::rand::SecureRandom;
use base64::{Engine as _, engine::general_purpose};

#[allow(dead_code)]
pub struct CryptoUtils;

#[allow(dead_code)]
impl CryptoUtils {
    /// Generate a secure random 32-byte key
    pub fn generate_key() -> Result<Vec<u8>> {
        let rng = rand::SystemRandom::new();
        let mut key = vec![0u8; 32];
        rng.fill(&mut key).map_err(|_| anyhow::anyhow!("Failed to generate key"))?;
        Ok(key)
    }

    /// Derive a key from a password using PBKDF2-HMAC-SHA256
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        let mut key = vec![0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(100_000).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );
        Ok(key)
    }

    /// Generate a random 16-byte salt
    pub fn generate_salt() -> Result<Vec<u8>> {
        let rng = rand::SystemRandom::new();
        let mut salt = vec![0u8; 16];
        rng.fill(&mut salt).map_err(|_| anyhow::anyhow!("Failed to generate salt"))?;
        Ok(salt)
    }

    /// Encrypt data with AES-256-GCM.
    /// Output layout: base64( nonce[12] || ciphertext || tag[16] )
    pub fn encrypt(data: &[u8], key: &[u8]) -> Result<String> {
        let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes).map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;

        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;
        let less_safe = aead::LessSafeKey::new(unbound);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.to_vec();
        less_safe
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        // Prepend nonce so decrypt can reconstruct it
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);

        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypt data produced by `encrypt`.
    pub fn decrypt(encrypted_data: &str, key: &[u8]) -> Result<Vec<u8>> {
        let data = general_purpose::STANDARD.decode(encrypted_data)?;

        // Minimum: 12 (nonce) + 16 (GCM tag) = 28 bytes
        if data.len() < 28 {
            return Err(anyhow::anyhow!("Invalid encrypted data: too short"));
        }

        let (nonce_bytes, ciphertext_and_tag) = data.split_at(12);
        let nonce_arr: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|_| anyhow::anyhow!("Failed to create decryption key"))?;
        let less_safe = aead::LessSafeKey::new(unbound);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_arr);

        let mut in_out = ciphertext_and_tag.to_vec();
        let plaintext = less_safe
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Decryption failed: invalid key or corrupt data"))?;

        Ok(plaintext.to_vec())
    }

    /// SHA-256 hash
    pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }

    /// Bytes to lowercase hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    /// Hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        Ok(hex::decode(hex)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let k1 = CryptoUtils::generate_key().unwrap();
        let k2 = CryptoUtils::generate_key().unwrap();
        assert_eq!(k1.len(), 32);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = CryptoUtils::generate_key().unwrap();
        let data = b"Hello, Smainer!";
        let enc = CryptoUtils::encrypt(data, &key).unwrap();
        let dec = CryptoUtils::decrypt(&enc, &key).unwrap();
        assert_eq!(data.as_ref(), dec.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = CryptoUtils::generate_key().unwrap();
        let key2 = CryptoUtils::generate_key().unwrap();
        let enc = CryptoUtils::encrypt(b"secret", &key1).unwrap();
        assert!(CryptoUtils::decrypt(&enc, &key2).is_err());
    }

    #[test]
    fn test_password_derivation_deterministic() {
        let salt = CryptoUtils::generate_salt().unwrap();
        let k1 = CryptoUtils::derive_key_from_password("pw", &salt).unwrap();
        let k2 = CryptoUtils::derive_key_from_password("pw", &salt).unwrap();
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 32);
    }
}
