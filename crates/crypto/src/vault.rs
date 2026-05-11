use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use domain::models::credential::EncryptedData;

/// Derive a 256-bit key from a master password using Argon2id.
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Argon2 key derivation failed");
    key
}

/// Encrypt plaintext bytes with AES-256-GCM.
pub fn encrypt_bytes(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedData, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

/// Decrypt bytes from an EncryptedData.
pub fn decrypt_bytes(data: &EncryptedData, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&data.nonce);
    cipher
        .decrypt(nonce, data.ciphertext.as_ref())
        .map_err(|e| e.to_string())
}