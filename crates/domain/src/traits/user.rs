// crates/domain/src/traits/user.rs
// Storage-agnostic user / refresh-token repository contract.

use async_trait::async_trait;

use crate::error::CoreError;
use crate::models::user::{RefreshToken, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<User, CoreError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, CoreError>;

    async fn store_refresh_token(
        &self,
        user_id: &str,
        token: &str,
        expires_at: i64,
    ) -> Result<(), CoreError>;

    async fn find_by_refresh_token(
        &self,
        token: &str,
    ) -> Result<Option<RefreshToken>, CoreError>;

    async fn delete_refresh_token(&self, token: &str) -> Result<(), CoreError>;
}
