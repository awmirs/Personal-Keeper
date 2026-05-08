use async_trait::async_trait;
use crate::error::CoreError;

/// Generic vault provider for a specific item type.
#[async_trait]
pub trait Vault<T> {
    async fn create(&self, item: T) -> Result<T, CoreError>;
    async fn get(&self, id: &str) -> Result<T, CoreError>;
    async fn list(&self) -> Result<Vec<T>, CoreError>;
    async fn update(&self, item: T) -> Result<T, CoreError>;
    async fn delete(&self, id: &str, hard: bool) -> Result<(), CoreError>;
    async fn search(&self, query: &str) -> Result<Vec<T>, CoreError>;
}