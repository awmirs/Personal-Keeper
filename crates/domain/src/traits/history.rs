// crates/domain/src/traits/history.rs
// Storage-agnostic item-version history contract.

use async_trait::async_trait;

use crate::error::CoreError;
use crate::models::history::ItemVersion;

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    async fn list_versions(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<Vec<ItemVersion>, CoreError>;

    async fn get_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError>;

    async fn recent_activity(
        &self,
        user_id: &str,
        item_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ItemVersion>, CoreError>;

    async fn restore_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError>;

    async fn purge_history(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<u64, CoreError>;
}
