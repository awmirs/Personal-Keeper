use async_trait::async_trait;
use core::error::CoreError;
use core::models::note::Note;
use core::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct NoteRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl NoteRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Note> for NoteRepository {
    async fn save(&self, item: &Note) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        // Run blocking SQLite operation in a dedicated thread
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO notes (id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    item.meta.id.to_string(),
                    item.title,
                    item.content,
                    item.is_pinned as i32,
                    serde_json::to_string(&item.meta.tags).unwrap_or_default(),
                    item.meta.color.as_ref().map(|c| &c.name),
                    item.meta.color.as_ref().map(|c| &c.hex),
                    item.meta.is_favorite as i32,
                    format!("{:?}", item.meta.trash_status),
                    item.meta.created_at,
                    item.meta.updated_at,
                ],
            ).map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Note>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare("SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at FROM notes WHERE id = ?1")?;
            // ... mapping logic omitted for brevity, but we'll fill it in soon
            Ok(None)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    // Implement find_all, delete, search similarly
}