use anyhow::Result;
use ring::{digest, pbkdf2, rand, aead};
use base64::{Engine as _, engine::general_purpose};

pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate a secure random key
    pub fn generate_key() -> Result<Vec<u8>> {
        let rng = rand::SystemRandom::new();
        let mut key = vec![0u8; 32];
        rand::SecureRandom::fill(&rng, &mut key)?;
        Ok(key)
    }
    
    /// Derive a key from a password using PBKDF2
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
    
    /// Generate a random salt
    pub fn generate_salt() -> Result<Vec<u8>> {
        let rng = rand::SystemRandom::new();
        let mut salt = vec![0u8; 16];
        rand::SecureRandom::fill(&rng, &mut salt)?;
        Ok(salt)
    }
    
    /// Encrypt data with AES-GCM
    pub fn encrypt(data: &[u8], key: &[u8]) -> Result<String> {
        let rng = rand::SystemRandom::new();
        let mut nonce = vec![0u8; 12];
        rand::SecureRandom::fill(&rng, &mut nonce)?;
        
        let sealing_key = aead::SealingKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, key)?,
            aead::Nonce::assume_unique_for_key(nonce.clone()[..12].try_into()?),
        );
        
        let mut in_out = data.to_vec();
        let tag = aead::seal_in_place_detached(
            &sealing_key,
            aead::Aad::empty(),
            &mut in_out,
        )?;
        
        // Combine nonce + ciphertext + tag
        let mut result = nonce;
        result.extend_from_slice(&in_out);
        result.extend_from_slice(tag.as_ref());
        
        Ok(general_purpose::STANDARD.encode(result))
    }
    
    /// Decrypt data with AES-GCM
    pub fn decrypt(encrypted_data: &str, key: &[u8]) -> Result<Vec<u8>> {
        let data = general_purpose::STANDARD.decode(encrypted_data)?;
        
        if data.len() < 12 + 16 {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }
        
        let nonce = &data[0..12];
        let tag_start = data.len() - 16;
        let ciphertext = &data[12..tag_start];
        let tag = &data[tag_start..];
        
        let opening_key = aead::OpeningKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, key)?,
            aead::Nonce::try_assume_unique_for_key(nonce)?,
        );
        
        let mut in_out = ciphertext.to_vec();
        let plaintext = aead::open_in_place_detached(
            &opening_key,
            aead::Aad::empty(),
            &mut in_out,
            aead::Tag::new(tag.try_into()?)?,
        )?;
        
        Ok(plaintext.to_vec())
    }
    
    /// Hash data with SHA-256
    pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }
    
    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
    
    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        Ok(hex::decode(hex)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key1 = CryptoUtils::generate_key().unwrap();
        let key2 = CryptoUtils::generate_key().unwrap();
        
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_ne!(key1, key2);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let key = CryptoUtils::generate_key().unwrap();
        let data = b"Hello, world!";
        
        let encrypted = CryptoUtils::encrypt(data, &key).unwrap();
        let decrypted = CryptoUtils::decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }
    
    #[test]
    fn test_password_derivation() {
        let password = "test_password";
        let salt = CryptoUtils::generate_salt().unwrap();
        
        let key1 = CryptoUtils::derive_key_from_password(password, &salt).unwrap();
        let key2 = CryptoUtils::derive_key_from_password(password, &salt).unwrap();
        
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }
}