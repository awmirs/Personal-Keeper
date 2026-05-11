use domain::error::CoreError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct CredentialConfigRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl CredentialConfigRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// Set the master password hash and salt (only once if not exists).
    pub async fn set_master_password(
        &self,
        password_hash: &str,
        salt: &[u8],
    ) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let password_hash = password_hash.to_string();
        let salt = salt.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO credentials_config (id, password_hash, salt) VALUES ('master', ?1, ?2)",
                params![password_hash, salt],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    /// Get the master password hash and salt.
    pub async fn get_master_password(
        &self,
    ) -> Result<Option<(String, Vec<u8>)>, CoreError> {
        let pool = Arc::clone(&self.pool);
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn
                .prepare("SELECT password_hash, salt FROM credentials_config WHERE id = 'master'")
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query_map([], |row| {
                    let hash: String = row.get(0)?;
                    let salt: Vec<u8> = row.get(1)?;
                    Ok((hash, salt))
                })
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(data)) => Ok(Some(data)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }
}