use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::contact::Contact;
use domain::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct ContactRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl ContactRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    crate::repositories::helpers::impl_position_helpers!("contacts");
}

use super::helpers::parse_item_metadata;

fn row_to_contact(row: &rusqlite::Row) -> Result<Contact, rusqlite::Error> {
    let name: String = row.get(1)?;
    let phones_json: String = row.get(2)?;
    let emails_json: String = row.get(3)?;
    let addresses_json: String = row.get(4)?;
    let notes: String = row.get(5)?;

    let phones: Vec<String> = serde_json::from_str(&phones_json).unwrap_or_default();
    let emails: Vec<String> = serde_json::from_str(&emails_json).unwrap_or_default();
    let addresses: Vec<String> = serde_json::from_str(&addresses_json).unwrap_or_default();

    let meta = parse_item_metadata(row, 6)?;   // tags column is 6

    Ok(Contact {
        meta,
        name,
        phones,
        emails,
        addresses,
        notes,
    })
}

#[async_trait]
impl Repository<Contact> for ContactRepository {
    async fn save(&self, user_id: &str, item: &Contact) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO contacts
                 (id, user_id, name, phones, emails, addresses, notes, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    item.meta.id.to_string(),
                    user_id,
                    item.name,
                    serde_json::to_string(&item.phones).unwrap_or_default(),
                    serde_json::to_string(&item.emails).unwrap_or_default(),
                    serde_json::to_string(&item.addresses).unwrap_or_default(),
                    item.notes,
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

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<Contact>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Contact>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, name, phones, emails, addresses, notes, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM contacts WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_contact)
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

    async fn find_all(&self, user_id: &str) -> Result<Vec<Contact>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Contact>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, name, phones, emails, addresses, notes, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM contacts WHERE user_id = ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id], row_to_contact)
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
            conn.execute("DELETE FROM contacts WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<Contact>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Contact>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, name, phones, emails, addresses, notes, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM contacts WHERE user_id = ?1 AND (name LIKE ?2 OR notes LIKE ?2) ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id, query], row_to_contact)
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