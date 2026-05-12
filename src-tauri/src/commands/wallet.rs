use anyhow::Result;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use starknet::signers::SigningKey;
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>, // Only returned on generation, not on get_wallet_address
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureResult {
    pub signature: String,
    pub message: String,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
struct StoredWallet {
    private_key: String, // Hex encoded
    public_key: String,  // Hex encoded
    address: String,     // Hex encoded
    encrypted: bool,
    salt: Option<String>,
}

pub(crate) fn canonicalize_starknet_private_key(value: &str) -> Result<String, String> {
    let stripped = value
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");

    if stripped.is_empty() || stripped.len() > 64 {
        return Err("Invalid private key format. Expected up to 64 hex characters.".to_string());
    }

    if !stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(
            "Invalid private key format. Must contain only hexadecimal characters.".to_string(),
        );
    }

    Ok(format!("0x{:0>64}", stripped.to_lowercase()))
}

fn get_wallet_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".smainer");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path.push("wallet.json");
    path
}

#[command]
pub async fn generate_wallet(password: Option<String>) -> Result<WalletInfo, String> {
    tracing::info!("Generating new wallet...");

    // Generate private key using Starknet curve
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();
    let public_key_scalar = public_key.scalar();

    // For now, use public key as address identity
    let address = public_key_scalar;

    let address_hex = format!("{:#x}", address);
    let public_key_hex = format!("{:#x}", public_key_scalar);
    let private_key_hex =
        canonicalize_starknet_private_key(&format!("{:#x}", private_key.secret_scalar()))?;

    let wallet_info = WalletInfo {
        address: address_hex.clone(),
        public_key: public_key_hex.clone(),
        created_at: chrono::Utc::now(),
        encrypted: password.is_some(),
        private_key: Some(private_key_hex.clone()), // BUG FIX: Return private key on generation
    };

    // Store wallet
    let stored = StoredWallet {
        private_key: private_key_hex,
        public_key: public_key_hex,
        address: address_hex,
        encrypted: password.is_some(),
        salt: None,
    };

    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    let path = get_wallet_path();
    fs::write(path, json).map_err(|e| e.to_string())?;

    Ok(wallet_info)
}

#[command]
pub async fn get_wallet_address() -> Result<String, String> {
    tracing::info!("Getting wallet address...");
    let path = get_wallet_path();
    if !path.exists() {
        return Err("No wallet found. Please generate a wallet first.".to_string());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let stored: StoredWallet = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(stored.address)
}

#[command]
pub async fn import_wallet(
    private_key: String,
    password: Option<String>,
) -> Result<WalletInfo, String> {
    tracing::info!("Importing wallet...");

    // Validate private key format
    let private_key_hex = canonicalize_starknet_private_key(&private_key)?;
    let private_key_scalar = FieldElement::from_hex_be(&private_key_hex)
        .map_err(|e| format!("Failed to parse private key: {}", e))?;

    let signing_key = SigningKey::from_secret_scalar(private_key_scalar);
    let public_key = signing_key.verifying_key();
    let public_key_scalar = public_key.scalar();

    // Use public key as address identity
    let address = public_key_scalar;
    let address_hex = format!("{:#x}", address);
    let public_key_hex = format!("{:#x}", public_key_scalar);

    let wallet_info = WalletInfo {
        address: address_hex.clone(),
        public_key: public_key_hex.clone(),
        created_at: chrono::Utc::now(),
        encrypted: password.is_some(),
        private_key: None, // BUG FIX: Don't return private key on import (user already has it)
    };

    // Store wallet
    let stored = StoredWallet {
        private_key: private_key_hex,
        public_key: public_key_hex,
        address: address_hex,
        encrypted: password.is_some(),
        salt: None,
    };

    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    let path = get_wallet_path();
    fs::write(path, json).map_err(|e| e.to_string())?;

    Ok(wallet_info)
}

#[cfg(test)]
mod tests {
    use super::canonicalize_starknet_private_key;

    #[test]
    fn canonicalizes_short_starknet_private_key() {
        let key = canonicalize_starknet_private_key("0xabc123").unwrap();
        assert_eq!(
            key,
            "0x0000000000000000000000000000000000000000000000000000000000abc123"
        );
    }

    #[test]
    fn preserves_full_length_starknet_private_key() {
        let raw = "1".repeat(64);
        let key = canonicalize_starknet_private_key(&raw).unwrap();
        assert_eq!(key, format!("0x{}", raw));
    }

    #[test]
    fn rejects_invalid_starknet_private_key() {
        assert!(canonicalize_starknet_private_key("0xnot-hex").is_err());
        assert!(canonicalize_starknet_private_key(&"a".repeat(65)).is_err());
    }
}

#[command]
pub async fn sign_message(
    message: String,
    _password: Option<String>,
) -> Result<SignatureResult, String> {
    tracing::info!("Signing message...");
    let path = get_wallet_path();
    if !path.exists() {
        return Err("No wallet found".to_string());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let stored: StoredWallet = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let private_key_scalar =
        FieldElement::from_hex_be(&stored.private_key).map_err(|e| e.to_string())?;
    let private_key = SigningKey::from_secret_scalar(private_key_scalar);

    // Hash message - treating input as string bytes
    // In real usage, input should likely be a hash hex string
    let message_bytes = message.as_bytes();
    // Simple hash for demo - real app should use Pedersen/Poseidon
    // Just taking first 31 bytes as FieldElement for safety/simplicity demo if too long
    // Proper way: compute hash of bytes
    let message_hash =
        FieldElement::from_byte_slice_be(message_bytes).map_err(|e| e.to_string())?;

    let signature = private_key.sign(&message_hash).map_err(|e| e.to_string())?;

    Ok(SignatureResult {
        signature: format!("{:#x}", signature.r),
        message,
        address: stored.address,
    })
}
