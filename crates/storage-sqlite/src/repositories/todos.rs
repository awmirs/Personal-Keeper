use async_trait::async_trait;
use domain::error::CoreError;
use domain::models::todo::Todo;
use domain::traits::repository::Repository;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

pub struct TodoRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl TodoRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    crate::repositories::helpers::impl_position_helpers!("todos");
}

use super::helpers::parse_item_metadata;

fn row_to_todo(row: &rusqlite::Row) -> Result<Todo, rusqlite::Error> {
    let title: String = row.get(1)?;
    let description: String = row.get(2)?;
    let completed: bool = row.get::<_, i32>(3)? != 0;
    let due_date: Option<i64> = row.get(4)?;

    let meta = parse_item_metadata(row, 5)?;   // tags column is 5

    Ok(Todo {
        meta,
        title,
        description,
        completed,
        due_date,
    })
}

#[async_trait]
impl Repository<Todo> for TodoRepository {
    async fn save(&self, user_id: &str, item: &Todo) -> Result<(), CoreError> {
        let pool = Arc::clone(&self.pool);
        let item = item.clone();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO todos
                 (id, user_id, title, description, completed, due_date, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    item.meta.id.to_string(),
                    user_id,
                    item.title,
                    item.description,
                    item.completed as i32,
                    item.due_date,
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

    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<Todo>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let id = id.to_string();
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<Todo>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, description, completed, due_date, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM todos WHERE id = ?1 AND user_id = ?2"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut rows = stmt.query_map(params![id, user_id], row_to_todo)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            match rows.next() {
                Some(Ok(todo)) => Ok(Some(todo)),
                Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
                None => Ok(None),
            }
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn find_all(&self, user_id: &str) -> Result<Vec<Todo>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Todo>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, description, completed, due_date, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM todos WHERE user_id = ?1 ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id], row_to_todo)
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
            conn.execute("DELETE FROM todos WHERE id = ?1 AND user_id = ?2", params![id, user_id])
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            Ok(())
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    async fn search(&self, user_id: &str, query: &str) -> Result<Vec<Todo>, CoreError> {
        let pool = Arc::clone(&self.pool);
        let query = format!("%{}%", query);
        let user_id = user_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<Todo>, CoreError> {
            let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, title, description, completed, due_date, tags, color_name, color_hex, is_favorite, trash_status, created_at, updated_at, position
                 FROM todos WHERE user_id = ?1 AND (title LIKE ?2 OR description LIKE ?2) ORDER BY position ASC, id ASC"
            )
                .map_err(|e| CoreError::Storage(e.to_string()))?;
            let rows = stmt.query_map(params![user_id, query], row_to_todo)
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