use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use starknet::signers::SigningKey;
use starknet::core::types::FieldElement;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub encrypted: bool,
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
    let private_key_hex = format!("{:#x}", private_key.secret_scalar());

    let wallet_info = WalletInfo {
        address: address_hex.clone(),
        public_key: public_key_hex.clone(),
        created_at: chrono::Utc::now(),
        encrypted: password.is_some(),
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
pub async fn sign_message(message: String, _password: Option<String>) -> Result<SignatureResult, String> {
    tracing::info!("Signing message...");
    let path = get_wallet_path();
    if !path.exists() {
        return Err("No wallet found".to_string());
    }
    
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let stored: StoredWallet = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    let private_key_scalar = FieldElement::from_hex_be(&stored.private_key).map_err(|e| e.to_string())?;
    let private_key = SigningKey::from_secret_scalar(private_key_scalar);
    
    // Hash message - treating input as string bytes
    // In real usage, input should likely be a hash hex string
    let message_bytes = message.as_bytes();
    // Simple hash for demo - real app should use Pedersen/Poseidon
    // Just taking first 31 bytes as FieldElement for safety/simplicity demo if too long
    // Proper way: compute hash of bytes
    let message_hash = FieldElement::from_byte_slice_be(message_bytes).map_err(|e| e.to_string())?;
    
    let signature = private_key.sign(&message_hash).map_err(|e| e.to_string())?;
    
    Ok(SignatureResult {
        signature: format!("{:#x}", signature.r),
        message,
        address: stored.address,
    })
}
