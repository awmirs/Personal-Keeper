use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::bookmark::Bookmark;
use domain::traits::repository::{Pagination, Page, Repository};
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
    crate::repositories::helpers::impl_position_helpers!("bookmarks");
}

use super::helpers::{decode_cursor, encode_cursor, fts5_query, parse_item_metadata};

fn row_to_bookmark(row: &rusqlite::Row) -> Result<Bookmark, rusqlite::Error> {
    let url: String = row.get(1)?;
    let title: String = row.get(2)?;
    let description: String = row.get(3)?;
    let favicon: Option<Vec<u8>> = row.get(4)?;
    let thumbnail: Option<Vec<u8>> = row.get(5)?;

    let meta = parse_item_metadata(row, 6)?;   // tags column is 6

    Ok(Bookmark {
        meta,
        url,
        title,
        description,
        favicon,
        thumbnail,
    })
}

#[async_trait]
impl Repository<Bookmark> for BookmarkRepository {
    async fn save(&self, user_id: &str, item: &Bookmark) -> Result<(), CoreError> {
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
                "INSERT INTO bookmarks
                 (id, user_id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(id) DO UPDATE SET
                     user_id = excluded.user_id,
                     url = excluded.url,
                     title = excluded.title,
                     description = excluded.description,
                     favicon = excluded.favicon,
                     thumbnail = excluded.thumbnail,
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
                    item.meta.position,
                ],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<Bookmark>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM bookmarks WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_bookmark)
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

        async fn find_all_paginated(
        &self,
        user_id: &str,
        pagination: Pagination,
    ) -> Result<Page<Bookmark>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        let limit = pagination.limit as i64;
        let cursor = pagination.cursor.clone();
        tokio::task::spawn_blocking(move || -> Result<Page<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let fetch = limit.saturating_add(1);
            let mut items: Vec<Bookmark> = Vec::new();
            match cursor {
                Some(c) => {
                    let (pos, id) = decode_cursor(&c)
                        .ok_or_else(|| CoreError::Validation("Invalid cursor".to_string()))?;
                    let mut stmt = conn.prepare(
                        "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                         FROM bookmarks WHERE user_id = ?1 AND (position, id) > (?2, ?3) ORDER BY position ASC, id ASC LIMIT ?4"
                    )
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    let rows = stmt.query_map(params![user_id, pos, id, fetch], row_to_bookmark)
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    for row in rows {
                        items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
                    }
                }
                None => {
                    let mut stmt = conn.prepare(
                        "SELECT id, url, title, description, favicon, thumbnail, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                         FROM bookmarks WHERE user_id = ?1 ORDER BY position ASC, id ASC LIMIT ?2"
                    )
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    let rows = stmt.query_map(params![user_id, fetch], row_to_bookmark)
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    for row in rows {
                        items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
                    }
                }
            }
            let has_more = items.len() as i64 > limit;
            if has_more {
                items.truncate(limit as usize);
            }
            let next_cursor = if has_more {
                items.last().map(|it| encode_cursor(it.meta.position, &it.meta.id.to_string()))
            } else {
                None
            };
            Ok(Page { items, next_cursor })
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
            conn.execute("DELETE FROM bookmarks WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

        async fn search_paginated(
        &self,
        user_id: &str,
        query: &str,
        pagination: Pagination,
    ) -> Result<Page<Bookmark>, CoreError> {
        let fts = fts5_query(query);
        if fts.is_empty() {
            return Ok(Page::empty());
        }
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        let limit = pagination.limit as i64;
        let cursor = pagination.cursor.clone();
        tokio::task::spawn_blocking(move || -> Result<Page<Bookmark>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let fetch = limit.saturating_add(1);
            let mut items: Vec<Bookmark> = Vec::new();
            match cursor {
                Some(c) => {
                    let (pos, id) = decode_cursor(&c)
                        .ok_or_else(|| CoreError::Validation("Invalid cursor".to_string()))?;
                    let mut stmt = conn.prepare(
                        "SELECT b.id, b.url, b.title, b.description, b.favicon, b.thumbnail, b.tags, b.color_name, b.color_hex, b.is_favorite, b.trash_status, b.created_at, b.updated_at, b.position
                         FROM bookmarks b
                         JOIN bookmarks_fts ON bookmarks_fts.rowid = b.rowid
                         WHERE b.user_id = ?1 AND bookmarks_fts MATCH ?2 AND (b.position, b.id) > (?3, ?4)
                         ORDER BY b.position ASC, b.id ASC LIMIT ?5"
                    )
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    let rows = stmt.query_map(params![user_id, fts, pos, id, fetch], row_to_bookmark)
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    for row in rows {
                        items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
                    }
                }
                None => {
                    let mut stmt = conn.prepare(
                        "SELECT b.id, b.url, b.title, b.description, b.favicon, b.thumbnail, b.tags, b.color_name, b.color_hex, b.is_favorite, b.trash_status, b.created_at, b.updated_at, b.position
                         FROM bookmarks b
                         JOIN bookmarks_fts ON bookmarks_fts.rowid = b.rowid
                         WHERE b.user_id = ?1 AND bookmarks_fts MATCH ?2
                         ORDER BY b.position ASC, b.id ASC LIMIT ?3"
                    )
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    let rows = stmt.query_map(params![user_id, fts, fetch], row_to_bookmark)
                        .map_err(|e| CoreError::Storage(e.to_string()))?;
                    for row in rows {
                        items.push(row.map_err(|e| CoreError::Storage(e.to_string()))?);
                    }
                }
            }
            let has_more = items.len() as i64 > limit;
            if has_more {
                items.truncate(limit as usize);
            }
            let next_cursor = if has_more {
                items.last().map(|it| encode_cursor(it.meta.position, &it.meta.id.to_string()))
            } else {
                None
            };
            Ok(Page { items, next_cursor })
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError> {
        BookmarkRepository::get_next_position(self, user_id).await
    }

    async fn update_positions(
        &self,
        user_id: &str,
        positions: &[(uuid::Uuid, f64)],
    ) -> Result<(), CoreError> {
        BookmarkRepository::update_positions(self, user_id, positions).await
    }
}
