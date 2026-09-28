use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::credential::{Credential, EncryptedData};
use domain::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct CredentialRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl CredentialRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    crate::repositories::helpers::impl_position_helpers!("credentials");
}

/// Serialise an optional EncryptedData to JSON string.
fn enc_to_json(data: &Option<EncryptedData>) -> Option<String> {
    data.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default())
}

/// Deserialise a JSON string to Option<EncryptedData>.
fn json_to_enc(s: &Option<String>) -> Option<EncryptedData> {
    s.as_ref().and_then(|js| serde_json::from_str(js).ok())
}

use super::helpers::{fts5_query, parse_item_metadata};

fn row_to_credential(row: &rusqlite::Row) -> Result<Credential, rusqlite::Error> {
    let website: String = row.get(1)?;
    let url: String = row.get(2)?;
    let username: String = row.get(3)?;
    let password_json: Option<String> = row.get(4)?;
    let notes_json: Option<String> = row.get(5)?;
    let totp_json: Option<String> = row.get(6)?;

    let meta = parse_item_metadata(row, 7)?;   // tags column is 7

    Ok(Credential {
        meta,
        website,
        url,
        username,
        password_encrypted: json_to_enc(&password_json),
        notes_encrypted: json_to_enc(&notes_json),
        totp_secret_encrypted: json_to_enc(&totp_json),
    })
}

#[async_trait]
impl Repository<Credential> for CredentialRepository {
    async fn save(&self, user_id: &str, item: &Credential) -> Result<(), CoreError> {
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
                "INSERT INTO credentials
                 (id, user_id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                 ON CONFLICT(id) DO UPDATE SET
                     user_id = excluded.user_id,
                     website = excluded.website,
                     url = excluded.url,
                     username = excluded.username,
                     password_encrypted = excluded.password_encrypted,
                     notes_encrypted = excluded.notes_encrypted,
                     totp_secret_encrypted = excluded.totp_secret_encrypted,
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
                    item.website,
                    item.url,
                    item.username,
                    enc_to_json(&item.password_encrypted),
                    enc_to_json(&item.notes_encrypted),
                    enc_to_json(&item.totp_secret_encrypted),
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

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<Credential>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM credentials WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_credential)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(c)) => Ok(Some(c)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_all(&self, user_id: &str) -> Result<Vec<Credential>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM credentials WHERE user_id = ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id], row_to_credential)
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
            conn.execute("DELETE FROM credentials WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<Credential>, CoreError> {
        let fts = fts5_query(query);
        if fts.is_empty() {
            return Ok(Vec::new());
        }
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT cr.id, cr.website, cr.url, cr.username, cr.password_encrypted, cr.notes_encrypted, cr.totp_secret_encrypted, cr.tags, cr.color_name, cr.color_hex, cr.is_favorite, cr.trash_status, cr.created_at, cr.updated_at, cr.position
                 FROM credentials cr
                 JOIN credentials_fts ON credentials_fts.rowid = cr.rowid
                 WHERE cr.user_id = ?1 AND credentials_fts MATCH ?2
                 ORDER BY cr.position ASC, cr.id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id, fts], row_to_credential)
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

    async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError> {
        CredentialRepository::get_next_position(self, user_id).await
    }

    async fn update_positions(
        &self,
        user_id: &str,
        positions: &[(uuid::Uuid, f64)],
    ) -> Result<(), CoreError> {
        CredentialRepository::update_positions(self, user_id, positions).await
    }
}
