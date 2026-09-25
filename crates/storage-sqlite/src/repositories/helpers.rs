use domain::models::common::{ColorLabel, ItemMetadata, Tag, TrashStatus};
use rusqlite::Row;

/// `id` is always read from column 0.
/// `tags_start` is the column where the `tags` JSON field begins.
/// The following nine columns are expected to be:
///   tags, color_name, color_hex, is_favorite, trash_status,
///   created_at, updated_at, position
pub fn parse_item_metadata(row: &Row, tags_start: usize) -> Result<ItemMetadata, rusqlite::Error> {
    let id: String = row.get(0)?;
    let tags_json: String = row.get(tags_start)?;
    let color_name: Option<String> = row.get(tags_start + 1)?;
    let color_hex: Option<String> = row.get(tags_start + 2)?;
    let is_favorite: bool = row.get::<_, i32>(tags_start + 3)? != 0;
    let trash_status_str: String = row.get(tags_start + 4)?;
    let created_at: i64 = row.get(tags_start + 5)?;
    let updated_at: i64 = row.get(tags_start + 6)?;
    let position: f64 = row.get(tags_start + 7)?;

    let tags: Vec<Tag> = serde_json::from_str(&tags_json).unwrap_or_default();
    let color = match (color_name, color_hex) {
        (Some(name), Some(hex)) => Some(ColorLabel { name, hex }),
        _ => None,
    };
    let trash_status = match trash_status_str.as_str() {
        "Trashed" => TrashStatus::Trashed,
        "Deleted" => TrashStatus::Deleted,
        _ => TrashStatus::Active,
    };

    Ok(ItemMetadata {
        id: uuid::Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidQuery)?,
        created_at,
        updated_at,
        tags,
        color,
        is_favorite,
        trash_status,
        position,
    })
}

/// Macro to generate `get_next_position` and `update_positions` methods
/// on a repository struct. Call inside an `impl` block with the table name.
macro_rules! impl_position_helpers {
    ($table:expr) => {
        pub async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError> {
            let pool = Arc::clone(&self.pool);
            let sql = format!(
                "SELECT COALESCE(MAX(position), 0.0) + 1000.0 FROM {} WHERE user_id = ?1",
                $table
            );
            let user_id = user_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
                let mut stmt = conn.prepare(&sql)
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                let pos: f64 = stmt.query_row(rusqlite::params![user_id], |row| row.get(0))
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(pos)
            })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
        }

        pub async fn update_positions(
            &self,
            user_id: &str,
            positions: &[(uuid::Uuid, f64)],
        ) -> Result<(), CoreError> {
            let pool = Arc::clone(&self.pool);
            let sql = format!(
                "UPDATE {} SET position = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
                $table
            );
            let user_id = user_id.to_string();
            let updates: Vec<(String, f64)> = positions
                .iter()
                .map(|(id, pos)| (id.to_string(), *pos))
                .collect();
            tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
                let tx = conn.transaction().map_err(|e| CoreError::Storage(e.to_string()))?;
                for (id, pos) in &updates {
                    tx.execute(
                        &sql,
                        rusqlite::params![pos, chrono::Utc::now().timestamp(), id, user_id],
                    )
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                }
                tx.commit().map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
        }
    };
}
pub(crate) use impl_position_helpers;