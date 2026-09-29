// crates/domain/src/traits/repository.rs
// Storage-agnostic repository contract shared by every vault item type.

use async_trait::async_trait;
use crate::error::CoreError;

/// Pagination parameters for list and search queries.
///
/// `limit` is the maximum number of items to return per page. `cursor`
/// is an opaque token produced by a previous response; when present,
/// results begin strictly after the cursor position in the query's
/// sort order.
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: u32,
    pub cursor: Option<String>,
}

impl Pagination {
    /// Default page size when the caller does not specify one.
    pub const DEFAULT_LIMIT: u32 = 50;

    /// Hard upper bound on caller-supplied `limit` values. Internal
    /// callers can bypass this via `unbounded()`.
    pub const MAX_LIMIT: u32 = 1000;

    /// Builds a bounded `Pagination` from optional query parameters.
    pub fn new(limit: Option<u32>, cursor: Option<String>) -> Self {
        let limit = limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .clamp(1, Self::MAX_LIMIT);
        Self { limit, cursor }
    }

    /// Pagination that returns every item in one page. Used by handlers
    /// that genuinely need the full vault contents (bulk import,
    /// cross-vault search), not by the paginated list endpoints.
    pub fn unbounded() -> Self {
        Self {
            limit: u32::MAX,
            cursor: None,
        }
    }
}

/// One page of results plus the cursor for the next page (if any).
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl<T> Page<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
        }
    }
}

#[async_trait]
pub trait Repository<T: Send + Sync + 'static>: Send + Sync {
    async fn save(&self, user_id: &str, item: &T) -> Result<(), CoreError>;
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<T>, CoreError>;
    async fn delete(&self, user_id: &str, id: &str) -> Result<(), CoreError>;

    /// Keyset-paginated list in the vault's natural order.
    async fn find_all_paginated(
        &self,
        user_id: &str,
        pagination: Pagination,
    ) -> Result<Page<T>, CoreError>;

    /// Keyset-paginated search in the vault's natural order.
    async fn search_paginated(
        &self,
        user_id: &str,
        query: &str,
        pagination: Pagination,
    ) -> Result<Page<T>, CoreError>;

    /// Unbounded list. Convenience wrapper around `find_all_paginated`
    /// for handlers that need every row.
    async fn find_all(&self, user_id: &str) -> Result<Vec<T>, CoreError> {
        Ok(self
            .find_all_paginated(user_id, Pagination::unbounded())
            .await?
            .items)
    }

    /// Unbounded search. Convenience wrapper around `search_paginated`.
    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<T>, CoreError> {
        Ok(self
            .search_paginated(user_id, query, Pagination::unbounded())
            .await?
            .items)
    }

    async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError>;
    async fn update_positions(
        &self,
        user_id: &str,
        positions: &[(uuid::Uuid, f64)],
    ) -> Result<(), CoreError>;
}
