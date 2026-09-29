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

/// Converts a raw user query into FTS5 MATCH syntax.
///
/// Strategy: split on whitespace, keep only Unicode alphanumerics plus
/// `_`, `-` and `.` in each token, wrap every token in double quotes
/// (escaping embedded `"` by doubling) and append `*` for prefix
/// matching. Tokens are joined with a space, which FTS5 treats as an
/// implicit AND.
///
/// Examples:
///   `hello world`   → `"hello"* "world"*`
///   `"unbalanced`   → `"unbalanced"*`
///   `a:b (c) -d`    → `"a"* "b"* "c"* "d"*`
///   `!!!`           → `` (caller must short-circuit on empty result)
pub fn fts5_query(raw: &str) -> String {
    let mut out = String::new();
    for token in raw.split_whitespace() {
        let cleaned: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push('"');
        for ch in cleaned.chars() {
            if ch == '"' {
                out.push('"');
                out.push('"');
            } else {
                out.push(ch);
            }
        }
        out.push('"');
        out.push('*');
    }
    out
}

/// Macro to generate `get_next_position` and `update_positions` methods
/// on a repository struct. Call inside an `impl` block with the table name.
///
/// The `$table` argument doubles as the counter entity name used by
/// `position_counters`; both must be stable per vault.
macro_rules! impl_position_helpers {
    ($table:expr) => {
        /// Atomically reserve the next ordering slot for `user_id`.
        ///
        /// A single INSERT … ON CONFLICT DO UPDATE … RETURNING against
        /// `position_counters` is the whole operation: on first use for a
        /// (user, entity) pair the row is created with the base table's
        /// current MAX(position) + 1000, on every later call the counter
        /// is bumped by 1000 in the same statement. No read-then-write
        /// race, no extra round trips.
        pub async fn get_next_position(&self, user_id: &str) -> Result<f64, CoreError> {
            let pool = Arc::clone(&self.pool);
            let entity = $table.to_string();
            let sql = format!(
                "INSERT INTO position_counters (user_id, entity, next_position) \
                 VALUES (?1, ?2, COALESCE((SELECT MAX(position) FROM {table} WHERE user_id = ?1), 0.0) + 1000.0) \
                 ON CONFLICT(user_id, entity) DO UPDATE SET next_position = position_counters.next_position + 1000.0 \
                 RETURNING next_position",
                table = $table,
            );
            let user_id = user_id.to_string();
            tokio::task::spawn_blocking(move || {
                let conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
                let pos: f64 = conn
                    .query_row(&sql, rusqlite::params![user_id, entity], |row| row.get(0))
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(pos)
            })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
        }

        /// Apply a batch of (id, position) updates in one transaction.
        ///
        /// Uses a single `UPDATE … FROM (VALUES …)` per chunk of 400 rows
        /// instead of one UPDATE per row. Chunking keeps the parameter
        /// count under SQLite's 999 limit: 1 (updated_at) + 2 per entry
        /// (id, position) + 1 (user_id) = 802 max per statement.
        pub async fn update_positions(
            &self,
            user_id: &str,
            positions: &[(uuid::Uuid, f64)],
        ) -> Result<(), CoreError> {
            if positions.is_empty() {
                return Ok(());
            }
            let pool = Arc::clone(&self.pool);
            let user_id = user_id.to_string();
            let updates: Vec<(String, f64)> = positions
                .iter()
                .map(|(id, pos)| (id.to_string(), *pos))
                .collect();
            tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(|e| CoreError::Storage(e.to_string()))?;
                let tx = conn.transaction().map_err(|e| CoreError::Storage(e.to_string()))?;
                let now = chrono::Utc::now().timestamp();
                for chunk in updates.chunks(400) {
                    let placeholders: Vec<String> = (0..chunk.len())
                        .map(|i| format!("(?{}, ?{})", i * 2 + 2, i * 2 + 3))
                        .collect();
                    let user_id_param = chunk.len() * 2 + 2;
                    let sql = format!(
                        "UPDATE {table} SET position = v.position, updated_at = ?1 \
                         FROM (VALUES {placeholders}) AS v(id, position) \
                         WHERE {table}.id = v.id AND {table}.user_id = ?{uidx}",
                        table = $table,
                        placeholders = placeholders.join(", "),
                        uidx = user_id_param,
                    );
                    let mut params: Vec<rusqlite::types::Value> =
                        Vec::with_capacity(chunk.len() * 2 + 2);
                    params.push(rusqlite::types::Value::Integer(now));
                    for (id, pos) in chunk {
                        params.push(rusqlite::types::Value::Text(id.clone()));
                        params.push(rusqlite::types::Value::Real(*pos));
                    }
                    params.push(rusqlite::types::Value::Text(user_id.clone()));
                    tx.execute(&sql, rusqlite::params_from_iter(params.iter()))
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