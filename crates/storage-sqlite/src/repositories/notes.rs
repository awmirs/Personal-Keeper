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

use super::helpers::{fts5_query, parse_item_metadata};

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
    async fn save(&self, user_id: &str, item: &Note) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            // V10 history triggers need UPDATE to fire on edits. INSERT OR
            // REPLACE deletes + reinserts, mislabelling every edit as
            // deleted + created in item_versions. ON CONFLICT DO UPDATE fires
            // the AFTER UPDATE OF trigger with the correct `updated` operation.
            conn.execute(
                "INSERT INTO notes (id, user_id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(id) DO UPDATE SET
                     user_id = excluded.user_id,
                     title = excluded.title,
                     content = excluded.content,
                     is_pinned = excluded.is_pinned,
                     tags = excluded.tags,
                     color_name = excluded.color_name,
                     color_hex = excluded.color_hex,
                     is_favorite = excluded.is_favorite,
                     trash_status = excluded.trash_status,
                     created_at = excluded.created_at,
                     updated_at = excluded.updated_at,
                     position = excluded.position",
                params![
                    item.meta.id.to_string(),
                    user_id,
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

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<Note>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM notes WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_note)
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

    async fn find_all(&self, user_id: &str) -> Result<Vec<Note>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM notes WHERE user_id = ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id], row_to_note)
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

    async fn delete(&self, user_id: &str, id: &str) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute("DELETE FROM notes WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<Note>, CoreError> {
        let fts = fts5_query(query);
        if fts.is_empty() {
            return Ok(Vec::new());
        }
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Note>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.content, n.is_pinned, n.tags, n.color_name, n.color_hex, n.is_favorite, n.trash_status, n.created_at, n.updated_at, n.position
                 FROM notes n
                 JOIN notes_fts ON notes_fts.rowid = n.rowid
                 WHERE n.user_id = ?1 AND notes_fts MATCH ?2
                 ORDER BY n.position ASC, n.id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id, fts], row_to_note)
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

    async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError> {
        NoteRepository::get_next_position(self, user_id).await
    }

    async fn update_positions(
        &self,
        user_id: &str,
        positions: &[(uuid::Uuid, f64)],
    ) -> Result<(), CoreError> {
        NoteRepository::update_positions(self, user_id, positions).await
    }
}
