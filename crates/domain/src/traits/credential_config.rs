// crates/domain/src/traits/credential_config.rs
// Storage-agnostic master-password configuration contract for the
// Credentials vault (salt + Argon2 hash, one row per user).

use async_trait::async_trait;

use crate::error::CoreError;

#[async_trait]
pub trait CredentialConfigRepository: Send + Sync {
    async fn set_master_password(
        &self,
        user_id: &str,
        password_hash: &str,
        salt: &[u8],
    ) -> Result<(), CoreError>;

    async fn get_master_password(
        &self,
        user_id: &str,
    ) -> Result<Option<(String, Vec<u8>)>, CoreError>;
}
