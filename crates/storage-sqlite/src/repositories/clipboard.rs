use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::clipboard::ClipboardItem;
use domain::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct ClipboardRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl ClipboardRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    crate::repositories::helpers::impl_position_helpers!("clipboard_items");
}

use super::helpers::parse_item_metadata;

fn row_to_clipboard(row: &rusqlite::Row) -> Result<ClipboardItem, rusqlite::Error> {
    let content: String = row.get(1)?;
    let persist_to_disk: bool = row.get::<_, i32>(2)? != 0;

    let meta = parse_item_metadata(row, 3)?;   // tags column is 3

    Ok(ClipboardItem {
        meta,
        content,
        persist_to_disk,
    })
}

#[async_trait]
impl Repository<ClipboardItem> for ClipboardRepository {
    async fn save(&self, user_id: &str, item: &ClipboardItem) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO clipboard_items
                 (id, user_id, content, persist_to_disk, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    item.meta.id.to_string(),
                    user_id,
                    item.content,
                    item.persist_to_disk as i32,
                    serde_json::to_string(&item.meta.tags).unwrap_or_default(),
                    item.meta.color.as_ref().map(|c| &c.name),
                    item.meta.color.as_ref().map(|c| &c.hex),
                    item.meta.is_favorite as i32,
                    format!("{:?}", item.meta.trash_status),
                    item.meta.created_at,
                    item.meta.updated_at,
                    item.meta.position,
                ],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<ClipboardItem>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<ClipboardItem>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, content, persist_to_disk, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM clipboard_items WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_clipboard)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(item)) => Ok(Some(item)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_all(&self, user_id: &str) -> Result<Vec<ClipboardItem>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<ClipboardItem>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, content, persist_to_disk, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM clipboard_items WHERE user_id = ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id], row_to_clipboard)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
            }
            Ok(items)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute("DELETE FROM clipboard_items WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<ClipboardItem>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<ClipboardItem>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, content, persist_to_disk, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM clipboard_items WHERE user_id = ?1 AND content LIKE ?2 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id, query], row_to_clipboard)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
            }
            Ok(items)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }
}