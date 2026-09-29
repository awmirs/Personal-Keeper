// crates/domain/src/traits/history.rs
// Storage-agnostic item-version history contract.

use async_trait::async_trait;

use crate::error::CoreError;
use crate::models::history::ItemVersion;
use crate::traits::repository::{Page, Pagination};

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    /// Keyset-paginated version list for a single item, newest first.
    async fn list_versions_paginated(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        pagination: Pagination,
    ) -> Result<Page<ItemVersion>, CoreError>;

    /// Keyset-paginated recent-activity feed across one or all vaults,
    /// newest first.
    async fn recent_activity_paginated(
        &self,
        user_id: &str,
        item_type: Option<&str>,
        pagination: Pagination,
    ) -> Result<Page<ItemVersion>, CoreError>;

    /// Unbounded version list. Convenience wrapper around
    /// `list_versions_paginated` for callers that do not paginate.
    async fn list_versions(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<Vec<ItemVersion>, CoreError> {
        Ok(self
            .list_versions_paginated(user_id, item_type, item_id, Pagination::unbounded())
            .await?
            .items)
    }

    /// Unbounded recent activity, capped by `limit` for compatibility
    /// with the previous API surface.
    async fn recent_activity(
        &self,
        user_id: &str,
        item_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ItemVersion>, CoreError> {
        let bounded = limit.max(0) as u32;
        let pagination = Pagination {
            limit: bounded,
            cursor: None,
        };
        Ok(self
            .recent_activity_paginated(user_id, item_type, pagination)
            .await?
            .items)
    }

    async fn get_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError>;

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
