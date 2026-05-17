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

use super::helpers::parse_item_metadata;

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
    async fn save(&self, item: &Credential) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO credentials
                 (id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    item.meta.id.to_string(),
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

    async fn find_by_id(&self, id: &str) -> Result<Option<Credential>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM credentials WHERE id = ?1"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id], row_to_credential)
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

    async fn find_all(&self) -> Result<Vec<Credential>, CoreError> {
        let pool = Arc::clone(&self.pool);
        tokio::task::spawn_blocking(move || -> Result<Vec<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM credentials ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map([], row_to_credential)
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
            conn.execute("DELETE FROM credentials WHERE id = ?1", params![id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, query: &str) -> Result<Vec<Credential>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        tokio::task::spawn_blocking(move || -> Result<Vec<Credential>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM credentials WHERE website LIKE ?1 OR url LIKE ?1 OR username LIKE ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![query], row_to_credential)
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