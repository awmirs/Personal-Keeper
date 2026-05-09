use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::bookmark::Bookmark;
use domain::models::common::{ColorLabel, ItemMetadata, Tag, TrashStatus};
use domain::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct BookmarkRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl BookmarkRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
}

fn row_to_bookmark(row: &rusqlite::Row) -> Result<Bookmark, rusqlite::Error> {
    let id: String = row.get(0)?;
    let url: String = row.get(1)?;
    let title: String = row.get(2)?;
    let description: String = row.get(3)?;
    let favicon: Option<Vec<u8>> = row.get(4)?;
    let thumbnail: Option<Vec<u8>> = row.get(5)?;
    let tags_json: String = row.get(6)?;
    let color_name: Option<String> = row.get(7)?;
    let color_hex: Option<String> = row.get(8)?;
    let is_favorite: bool = row.get::<_, i32>(9)? != 0;
    let trash_status_str: String = row.get(10)?;
    let created_at: i64 = row.get(11)?;
    let updated_at: i64 = row.get(12)?;

    let tags: Vec<Tag> = serde_json::from_str(&tags_json).unwrap_or_default();
    let color = match (color_name, color_hex) {
        (Some(name), Some(hex)) => Some(ColorLabel { name, hex }),
        _ => None,
    };
    let trash_status = match trash_status_str.as_str() {
        "Trashed" => TrashStatus::Trashed,
        "Deleted" => TrashStatus::Deleted,
        _ => TrashStatus::Active,
    };

    Ok(Bookmark {
        meta: ItemMetadata {
            id: uuid::Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidQuery)?,
            created_at,
            updated_at,
            tags,
            color,
            is_favorite,
            trash_status,
        },
        url,
        title,
        description,
        favicon,
        thumbnail,
    })
}

#[async_trait]
impl Repository<Bookmark> for BookmarkRepository {
    async fn save(&self, item: &Bookmark) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO bookmarks
                 (id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    item.meta.id.to_string(),
                    item.url,
                    item.title,
                    item.description,
                    item.favicon,
                    item.thumbnail,
                    serde_json::to_string(&item.meta.tags).unwrap_or_default(),
                    item.meta.color.as_ref().map(|c| &c.name),
                    item.meta.color.as_ref().map(|c| &c.hex),
                    item.meta.is_favorite as i32,
                    format!("{:?}", item.meta.trash_status),
                    item.meta.created_at,
                    item.meta.updated_at,
                ],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Bookmark>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at
                 FROM bookmarks WHERE id = ?1"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id], row_to_bookmark)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(b)) => Ok(Some(b)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_all(&self) -> Result<Vec<Bookmark>, CoreError> {
        let pool = Arc::clone(&self.pool);
        tokio::task::spawn_blocking(move || -> Result<Vec<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at
                 FROM bookmarks ORDER BY updated_at DESC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map([], row_to_bookmark)
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

    async fn delete(&self, id: &str) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, query: &str) -> Result<Vec<Bookmark>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        tokio::task::spawn_blocking(move || -> Result<Vec<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at
                 FROM bookmarks WHERE url LIKE ?1 OR title LIKE ?1 OR description LIKE ?1 ORDER BY updated_at DESC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![query], row_to_bookmark)
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