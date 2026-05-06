use serde::{Deserialize, Serialize};
use super::common::ItemMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    #[serde(flatten)]
    pub meta: ItemMetadata,
    pub website: String,
    pub url: String,
    pub username: String,
    // The password is stored encrypted; only hold the ciphertext + IV
    pub password_encrypted: Option<EncryptedData>,
    pub notes_encrypted: Option<EncryptedData>,
    pub totp_secret_encrypted: Option<EncryptedData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,          // 12 bytes for AES-256-GCM
}