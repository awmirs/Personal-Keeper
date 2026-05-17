use async_trait::async_trait;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;
use domain::error::CoreError;
use domain::models::note::Note;
use domain::traits::repository::Repository;

pub struct NoteRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl NoteRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    // Generated position helpers
    crate::repositories::helpers::impl_position_helpers!("notes");
}

use super::helpers::parse_item_metadata;

fn row_to_note(row: &rusqlite::Row) -> Result<Note, rusqlite::Error> {
    let title: String = row.get(1)?;
    let content: String = row.get(2)?;
    let is_pinned: bool = row.get::<_, i32>(3)? != 0;

    let meta = parse_item_metadata(row, 4)?; // tags is at column 4

    Ok(Note {
        meta,
        title,
        content,
        is_pinned,
    })
}

#[async_trait]
impl Repository<Note> for NoteRepository {
    async fn save(&self, item: &Note) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO notes (id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                    item.meta.position,
                ],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
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
            let mut stmt = conn.prepare(
                "SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM notes WHERE id = ?1"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id], row_to_note)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(note)) => Ok(Some(note)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_all(&self) -> Result<Vec<Note>, CoreError> {
        let pool = Arc::clone(&self.pool);
        tokio::task::spawn_blocking(move || -> Result<Vec<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM notes ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map([], row_to_note)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut notes = Vec::new();
            for row in rows {
                notes.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
            }
            Ok(notes)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn delete(&self, id: &str) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, query: &str) -> Result<Vec<Note>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        tokio::task::spawn_blocking(move || -> Result<Vec<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM notes WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![query], row_to_note)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut notes = Vec::new();
            for row in rows {
                notes.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
            }
            Ok(notes)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }
}