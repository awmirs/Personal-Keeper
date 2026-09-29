// crates/storage-sqlite/src/repositories/history.rs
// Repository for `item_versions`: the automatic item history recorded by
// the triggers created in migration V10.
//
// This implementation follows the database access pattern used by the rest
// of this crate (synchronous pool + tokio::task::spawn_blocking).

use std::time::{SystemTime, UNIX_EPOCH};

use domain::error::CoreError;
use domain::models::history::{is_valid_item_type, ItemVersion};
use domain::traits::repository::{Page, Pagination};
use crate::repositories::helpers::{decode_activity_cursor, encode_activity_cursor};
use rusqlite::{params, Connection};
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
const VERSION_COLUMNS: &str =
    "id, item_type, item_id, user_id, version, operation, data, created_at";

/// Repository over the `item_versions` history table.
pub struct HistoryRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

fn storage_err(err: rusqlite::Error) -> CoreError {
    CoreError::Storage(err.to_string())
}

fn invalid_type(item_type: &str) -> CoreError {
    CoreError::Validation(format!("Unsupported item type: {}", item_type))
}

fn version_not_found(version: i64, item_type: &str, item_id: &str) -> CoreError {
    CoreError::NotFound(format!(
        "Version {} of {} {} not found",
        version, item_type, item_id
    ))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Physical table backing an item type.
fn table_for(item_type: &str) -> Option<&'static str> {
    match item_type {
        "note" => Some("notes"),
        "clipboard" => Some("clipboard_items"),
        "todo" => Some("todos"),
        "bookmark" => Some("bookmarks"),
        "contact" => Some("contacts"),
        "credential" => Some("credentials"),
        _ => None,
    }
}

/// Columns restored from a snapshot (everything except `id`, `user_id`,
/// `created_at` and `updated_at`, which are handled explicitly).
fn restored_columns(item_type: &str) -> Option<&'static [&'static str]> {
    Some(match item_type {
        "note" => &[
            "title", "content", "is_pinned", "tags", "color_name", "color_hex",
            "is_favorite", "trash_status", "position",
        ],
        "clipboard" => &[
            "content", "persist_to_disk", "tags", "color_name", "color_hex",
            "is_favorite", "trash_status", "position",
        ],
        "todo" => &[
            "title", "description", "completed", "due_date", "tags", "color_name",
            "color_hex", "is_favorite", "trash_status", "position",
        ],
        "bookmark" => &[
            "url", "title", "description", "tags", "color_name", "color_hex",
            "is_favorite", "trash_status", "position",
        ],
        "contact" => &[
            "name", "phones", "emails", "addresses", "notes", "tags", "color_name",
            "color_hex", "is_favorite", "trash_status", "position",
        ],
        "credential" => &[
            "website", "url", "username", "password_encrypted", "notes_encrypted",
            "totp_secret_encrypted", "tags", "color_name", "color_hex",
            "is_favorite", "trash_status", "position",
        ],
        _ => return None,
    })
}

fn row_to_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<ItemVersion> {
    Ok(ItemVersion {
        id: row.get(0)?,
        item_type: row.get(1)?,
        item_id: row.get(2)?,
        user_id: row.get(3)?,
        version: row.get(4)?,
        operation: row.get(5)?,
        data: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn list_versions_sync(
    conn: &mut Connection,
    user_id: &str,
    item_type: &str,
    item_id: &str,
) -> Result<Vec<ItemVersion>, CoreError> {
    let sql = format!(
        "SELECT {} FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND user_id = ?3 ORDER BY version DESC",
        VERSION_COLUMNS
    );
    let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
    let rows = stmt
        .query_map(params![item_type, item_id, user_id], row_to_version)
        .map_err(storage_err)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(storage_err)?);
    }
    Ok(out)
}

fn get_version_sync(
    conn: &Connection,
    user_id: &str,
    item_type: &str,
    item_id: &str,
    version: i64,
) -> Result<ItemVersion, CoreError> {
    let sql = format!(
        "SELECT {} FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND user_id = ?3 AND version = ?4",
        VERSION_COLUMNS
    );
    conn.query_row(&sql, params![item_type, item_id, user_id, version], row_to_version)
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => version_not_found(version, item_type, item_id),
            other => storage_err(other),
        })
}

fn recent_activity_sync(
    conn: &mut Connection,
    user_id: &str,
    item_type: Option<&str>,
    limit: i64,
) -> Result<Vec<ItemVersion>, CoreError> {
    let limit = limit.clamp(1, 200);
    let mut out = Vec::new();
    if let Some(item_type) = item_type.filter(|t| !t.is_empty()) {
        if !is_valid_item_type(item_type) {
            return Err(invalid_type(item_type));
        }
        let sql = format!(
            "SELECT {} FROM item_versions WHERE user_id = ?1 AND item_type = ?2 ORDER BY created_at DESC, version DESC LIMIT ?3",
            VERSION_COLUMNS
        );
        let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
        let rows = stmt
            .query_map(params![user_id, item_type, limit], row_to_version)
            .map_err(storage_err)?;
        for row in rows {
            out.push(row.map_err(storage_err)?);
        }
    } else {
        let sql = format!(
            "SELECT {} FROM item_versions WHERE user_id = ?1 ORDER BY created_at DESC, version DESC LIMIT ?2",
            VERSION_COLUMNS
        );
        let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
        let rows = stmt
            .query_map(params![user_id, limit], row_to_version)
            .map_err(storage_err)?;
        for row in rows {
            out.push(row.map_err(storage_err)?);
        }
    }
    Ok(out)
}

fn restore_version_sync(
    conn: &mut Connection,
    user_id: &str,
    item_type: &str,
    item_id: &str,
    version: i64,
    table: &str,
    columns: &[&str],
) -> Result<ItemVersion, CoreError> {
    let tx = conn.transaction().map_err(storage_err)?;

    // 1. Load the requested snapshot (must belong to the user).
    let data: String = tx
        .query_row(
            "SELECT data FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND version = ?3 AND user_id = ?4",
            params![item_type, item_id, version, user_id],
            |row| row.get(0),
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => version_not_found(version, item_type, item_id),
            other => storage_err(other),
        })?;

    let now = now_secs();

    // 2. Version watermark before the restore.
    let (max_before,): (i64,) = tx
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM item_versions WHERE item_type = ?1 AND item_id = ?2",
            params![item_type, item_id],
            |row| Ok((row.get(0)?,)),
        )
        .map_err(storage_err)?;

    // 3. Apply the snapshot — update the live row, or resurrect it.
    let exists = match tx.query_row(
        &format!("SELECT 1 FROM {} WHERE id = ?1 AND user_id = ?2", table),
        params![item_id, user_id],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(_) => true,
        Err(rusqlite::Error::QueryReturnedNoRows) => false,
        Err(other) => return Err(storage_err(other)),
    };

    if exists {
        let assignments: Vec<String> = columns
            .iter()
            .map(|column| format!("{} = json_extract(?1, '$.{}')", column, column))
            .collect();
        let sql = format!(
            "UPDATE {} SET {}, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
            table,
            assignments.join(", ")
        );
        tx.execute(&sql, params![data, now, item_id, user_id])
            .map_err(storage_err)?;
    } else {
        let mut column_names: Vec<&str> = vec!["id"];
        column_names.extend(columns.iter().copied());
        column_names.push("created_at");
        let placeholders: Vec<String> = column_names
            .iter()
            .map(|column| format!("json_extract(?1, '$.{}')", column))
            .collect();
        let sql = format!(
            "INSERT INTO {} ({}, updated_at, user_id) VALUES ({}, ?2, ?3)",
            table,
            column_names.join(", "),
            placeholders.join(", ")
        );
        tx.execute(&sql, params![data, now, user_id])
            .map_err(storage_err)?;
    }

    // 4. Label the freshly recorded version as a restore.
    let (max_after,): (i64,) = tx
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM item_versions WHERE item_type = ?1 AND item_id = ?2",
            params![item_type, item_id],
            |row| Ok((row.get(0)?,)),
        )
        .map_err(storage_err)?;
    if max_after > max_before {
        tx.execute(
            "UPDATE item_versions SET operation = 'restored' WHERE item_type = ?1 AND item_id = ?2 AND version = ?3",
            params![item_type, item_id, max_after],
        )
        .map_err(storage_err)?;
    }
    tx.commit().map_err(storage_err)?;

    let target = if max_after > max_before { max_after } else { version };
    get_version_sync(conn, user_id, item_type, item_id, target)
}

fn purge_history_sync(
    conn: &mut Connection,
    user_id: &str,
    item_type: &str,
    item_id: &str,
) -> Result<u64, CoreError> {
    let purged = conn
        .execute(
            "DELETE FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND user_id = ?3",
            params![item_type, item_id, user_id],
        )
        .map_err(storage_err)?;
    Ok(purged as u64)
}

impl HistoryRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// Runs `f` on the blocking thread pool with a pooled connection.
    async fn with_conn<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self.pool.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(err) => return Err(CoreError::Storage(err.to_string())),
            };
            f(&mut conn)
        })
        .await
        .map_err(|err| CoreError::Storage(err.to_string()))?;
        result
    }

    /// All versions of one item, newest first.
    pub async fn list_versions(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<Vec<ItemVersion>, CoreError> {
        if !is_valid_item_type(item_type) {
            return Err(invalid_type(item_type));
        }
        let user_id = user_id.to_string();
        let item_type = item_type.to_string();
        let item_id = item_id.to_string();
        self.with_conn(move |conn| {
            list_versions_sync(conn, &user_id, &item_type, &item_id)
        })
        .await
    }

    /// A single version of an item.
    pub async fn get_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError> {
        if !is_valid_item_type(item_type) {
            return Err(invalid_type(item_type));
        }
        let user_id = user_id.to_string();
        let item_type = item_type.to_string();
        let item_id = item_id.to_string();
        self.with_conn(move |conn| {
            get_version_sync(conn, &user_id, &item_type, &item_id, version)
        })
        .await
    }

    /// Most recent versions across all (or one) vault type, newest first.
    pub async fn recent_activity(
        &self,
        user_id: &str,
        item_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ItemVersion>, CoreError> {
        let item_type = match item_type {
            Some(t) if !t.trim().is_empty() => {
                if !is_valid_item_type(t) {
                    return Err(invalid_type(t));
                }
                Some(t.to_string())
            }
            _ => None,
        };
        let user_id = user_id.to_string();
        self.with_conn(move |conn| {
            recent_activity_sync(conn, &user_id, item_type.as_deref(), limit)
        })
        .await
    }

    /// Restore an item to the given version. The restore itself is recorded
    /// as a new `restored` version, and restoring a hard-deleted item
    /// resurrects it.
    pub async fn restore_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError> {
        let table = match table_for(item_type) {
            Some(table) => table,
            None => return Err(invalid_type(item_type)),
        };
        let columns = match restored_columns(item_type) {
            Some(columns) => columns,
            None => return Err(invalid_type(item_type)),
        };
        let user_id = user_id.to_string();
        let item_type = item_type.to_string();
        let item_id = item_id.to_string();
        self.with_conn(move |conn| {
            restore_version_sync(conn, &user_id, &item_type, &item_id, version, table, columns)
        })
        .await
    }

    /// Delete every recorded version of an item. Returns the purged count.
    pub async fn purge_history(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<u64, CoreError> {
        if !is_valid_item_type(item_type) {
            return Err(invalid_type(item_type));
        }
        let user_id = user_id.to_string();
        let item_type = item_type.to_string();
        let item_id = item_id.to_string();
        self.with_conn(move |conn| {
            purge_history_sync(conn, &user_id, &item_type, &item_id)
        })
        .await
    }
}

#[async_trait::async_trait]
impl domain::traits::history::HistoryRepository for HistoryRepository {
    async fn list_versions_paginated(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        pagination: Pagination,
    ) -> Result<Page<ItemVersion>, CoreError> {
        if !is_valid_item_type(item_type) {
            return Err(invalid_type(item_type));
        }
        let user_id = user_id.to_string();
        let item_type = item_type.to_string();
        let item_id = item_id.to_string();
        let limit = pagination.limit as i64;
        let before_version: Option<i64> = match pagination.cursor.as_deref() {
            Some(c) => Some(
                c.parse::<i64>()
                    .map_err(|_| CoreError::Validation("Invalid cursor".to_string()))?,
            ),
            None => None,
        };
        self.with_conn(move |conn| {
            let fetch = limit.saturating_add(1);
            let mut versions = Vec::new();
            if let Some(before) = before_version {
                let sql = format!(
                    "SELECT {} FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND user_id = ?3 AND version < ?4 ORDER BY version DESC LIMIT ?5",
                    VERSION_COLUMNS
                );
                let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                let rows = stmt
                    .query_map(
                        params![item_type, item_id, user_id, before, fetch],
                        row_to_version,
                    )
                    .map_err(storage_err)?;
                for row in rows {
                    versions.push(row.map_err(storage_err)?);
                }
            } else {
                let sql = format!(
                    "SELECT {} FROM item_versions WHERE item_type = ?1 AND item_id = ?2 AND user_id = ?3 ORDER BY version DESC LIMIT ?4",
                    VERSION_COLUMNS
                );
                let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                let rows = stmt
                    .query_map(
                        params![item_type, item_id, user_id, fetch],
                        row_to_version,
                    )
                    .map_err(storage_err)?;
                for row in rows {
                    versions.push(row.map_err(storage_err)?);
                }
            }
            let has_more = versions.len() as i64 > limit;
            if has_more {
                versions.truncate(limit as usize);
            }
            let next_cursor = if has_more {
                versions.last().map(|v| v.version.to_string())
            } else {
                None
            };
            Ok(Page {
                items: versions,
                next_cursor,
            })
        })
        .await
    }

    async fn get_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError> {
        HistoryRepository::get_version(self, user_id, item_type, item_id, version).await
    }

    async fn recent_activity_paginated(
        &self,
        user_id: &str,
        item_type: Option<&str>,
        pagination: Pagination,
    ) -> Result<Page<ItemVersion>, CoreError> {
        let item_type_filter = match item_type {
            Some(t) if !t.trim().is_empty() => {
                if !is_valid_item_type(t) {
                    return Err(invalid_type(t));
                }
                Some(t.to_string())
            }
            _ => None,
        };
        let user_id = user_id.to_string();
        let limit = pagination.limit as i64;
        let cursor = pagination.cursor.clone();
        self.with_conn(move |conn| {
            let fetch = limit.saturating_add(1);
            let before: Option<(i64, i64, String)> = match cursor.as_deref() {
                Some(c) => Some(decode_activity_cursor(c).ok_or_else(|| {
                    CoreError::Validation("Invalid cursor".to_string())
                })?),
                None => None,
            };
            let mut out = Vec::new();
            match (item_type_filter.as_deref(), before) {
                (Some(t), Some((ca, ver, id))) => {
                    let sql = format!(
                        "SELECT {} FROM item_versions WHERE user_id = ?1 AND item_type = ?2 AND (created_at, version, id) < (?3, ?4, ?5) ORDER BY created_at DESC, version DESC, id DESC LIMIT ?6",
                        VERSION_COLUMNS
                    );
                    let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                    let rows = stmt
                        .query_map(params![user_id, t, ca, ver, id, fetch], row_to_version)
                        .map_err(storage_err)?;
                    for row in rows {
                        out.push(row.map_err(storage_err)?);
                    }
                }
                (Some(t), None) => {
                    let sql = format!(
                        "SELECT {} FROM item_versions WHERE user_id = ?1 AND item_type = ?2 ORDER BY created_at DESC, version DESC, id DESC LIMIT ?3",
                        VERSION_COLUMNS
                    );
                    let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                    let rows = stmt
                        .query_map(params![user_id, t, fetch], row_to_version)
                        .map_err(storage_err)?;
                    for row in rows {
                        out.push(row.map_err(storage_err)?);
                    }
                }
                (None, Some((ca, ver, id))) => {
                    let sql = format!(
                        "SELECT {} FROM item_versions WHERE user_id = ?1 AND (created_at, version, id) < (?2, ?3, ?4) ORDER BY created_at DESC, version DESC, id DESC LIMIT ?5",
                        VERSION_COLUMNS
                    );
                    let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                    let rows = stmt
                        .query_map(params![user_id, ca, ver, id, fetch], row_to_version)
                        .map_err(storage_err)?;
                    for row in rows {
                        out.push(row.map_err(storage_err)?);
                    }
                }
                (None, None) => {
                    let sql = format!(
                        "SELECT {} FROM item_versions WHERE user_id = ?1 ORDER BY created_at DESC, version DESC, id DESC LIMIT ?2",
                        VERSION_COLUMNS
                    );
                    let mut stmt = conn.prepare(&sql).map_err(storage_err)?;
                    let rows = stmt
                        .query_map(params![user_id, fetch], row_to_version)
                        .map_err(storage_err)?;
                    for row in rows {
                        out.push(row.map_err(storage_err)?);
                    }
                }
            }
            let has_more = out.len() as i64 > limit;
            if has_more {
                out.truncate(limit as usize);
            }
            let next_cursor = if has_more {
                out.last()
                    .map(|v| encode_activity_cursor(v.created_at, v.version, &v.id))
            } else {
                None
            };
            Ok(Page {
                items: out,
                next_cursor,
            })
        })
        .await
    }

    async fn restore_version(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        version: i64,
    ) -> Result<ItemVersion, CoreError> {
        HistoryRepository::restore_version(self, user_id, item_type, item_id, version).await
    }

    async fn purge_history(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<u64, CoreError> {
        HistoryRepository::purge_history(self, user_id, item_type, item_id).await
    }
}
