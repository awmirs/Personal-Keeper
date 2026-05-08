use domain::error::CoreError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: i64,
}

pub struct UserRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl UserRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<User, CoreError> {
        let pool = Arc::clone(&self.pool);
        let username = username.to_string();
        let password_hash = password_hash.to_string();
        tokio::task::spawn_blocking(move || -> Result<User, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let id = uuid::Uuid::now_v7().to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            conn.execute(
                "INSERT INTO users (id, username, password_hash, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, username, password_hash, now, now],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(User {
                id,
                username,
                password_hash,
            })
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let username = username.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<User>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn
                .prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query_map(params![username], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                    })
                })
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(rows.next().transpose().map_err(|e| CoreError::Storage(e.to_string()))?)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    pub async fn store_refresh_token(
        &self,
        user_id: &str,
        token: &str,
        expires_at: i64,
    ) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        let token = token.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let id = uuid::Uuid::now_v7().to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            conn.execute(
                "INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, user_id, token, expires_at, now],
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    pub async fn find_by_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let token = token.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<RefreshToken>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn
                .prepare("SELECT id, user_id, token, expires_at FROM refresh_tokens WHERE token = ?1")
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query_map(params![token], |row| {
                    Ok(RefreshToken {
                        id: row.get(0)?,
                        user_id: row.get(1)?,
                        token: row.get(2)?,
                        expires_at: row.get(3)?,
                    })
                })
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(rows.next().transpose().map_err(|e| CoreError::Storage(e.to_string()))?)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    pub async fn delete_refresh_token(&self, token: &str) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let token = token.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute("DELETE FROM refresh_tokens WHERE token = ?1", params![token])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }
}