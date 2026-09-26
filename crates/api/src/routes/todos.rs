use actix_web::{web, HttpResponse};
use domain::models::todo::Todo;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
}

pub async fn create_todo(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.todo_repo.get_next_position(&user.user_id).await?;
    let mut todo = Todo {
        meta: Default::default(),
        title: body.title.clone(),
        description: body.description.clone().unwrap_or_default(),
        completed: false,
        due_date: body.due_date,
    };
    todo.meta.position = next_pos;
    data.todo_repo.save(&user.user_id, &todo).await?;
    Ok(HttpResponse::Created().json(&todo))
}

// ----- Bulk import (full-fidelity round-trip of exported data) -----

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ImportTodosRequest {
    pub items: Vec<serde_json::Value>,
    pub strategy: ImportStrategy,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportStrategy {
    Skip,
    Replace,
    Copy,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub created: usize,
    pub replaced: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Normalizes a raw imported item so it always deserializes into the vault
/// model: fills server-side defaults for missing metadata and repairs
/// partially-sourced values — blank, non-string or invalid ids; string
/// booleans; comma-separated or JSON-text tags; blank or unparsable
/// timestamps, positions and colours; and other hand-written-file quirks.
fn fill_import_defaults(mut value: serde_json::Value, next_pos: &mut f64) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    if let serde_json::Value::Object(ref mut map) = value {
        // Ids: missing, blank, non-string or invalid values get a fresh UUID
        // so hand-written rows never collide or fail UUID parsing.
        let id_valid = match map.get("id") {
            Some(serde_json::Value::String(s)) => uuid::Uuid::parse_str(s.trim()).is_ok(),
            _ => false,
        };
        if !id_valid {
            map.insert("id".to_string(), serde_json::json!(uuid::Uuid::now_v7().to_string()));
        }
        // Timestamps: blank or unparsable values fall back to "now".
        let created_raw = map.remove("created_at").unwrap_or(serde_json::json!(now));
        map.insert("created_at".to_string(), serde_json::json!(coerce_epoch(created_raw, now)));
        let updated_raw = map.remove("updated_at").unwrap_or(serde_json::json!(now));
        map.insert("updated_at".to_string(), serde_json::json!(coerce_epoch(updated_raw, now)));
        // Trash status: blank or non-string values reset to "Active".
        let trash_raw = map.remove("trash_status").unwrap_or(serde_json::json!("Active"));
        let trash_status = match trash_raw {
            serde_json::Value::String(s) if !s.trim().is_empty() => serde_json::json!(s.trim()),
            _ => serde_json::json!("Active"),
        };
        map.insert("trash_status".to_string(), trash_status);
        map.entry("description").or_insert_with(|| serde_json::json!(""));
        // Booleans: "true"/"false"/"1"/"0"/"" and JSON bools/numbers coerce;
        // missing flags default to false.
        for key in ["is_favorite", "completed"] {
            let raw = map.remove(key).unwrap_or(serde_json::json!(false));
            map.insert(key.to_string(), serde_json::json!(coerce_bool(raw).unwrap_or(false)));
        }
        // Position: blank or unparsable values get the next free slot so the
        // imported ordering stays consistent.
        let position_raw = map.remove("position");
        let position = match position_raw {
            Some(serde_json::Value::Number(n)) => n.as_f64(),
            Some(serde_json::Value::String(s)) => s.trim().parse::<f64>().ok(),
            _ => None,
        };
        let position = match position {
            Some(position) => position,
            None => {
                let position = *next_pos;
                *next_pos += 1.0;
                position
            }
        };
        map.insert("position".to_string(), serde_json::json!(position));
        // Tags: arrays, JSON text, comma-separated or bare strings all become
        // a list of well-formed tag objects with valid ids.
        let tags = map.remove("tags").unwrap_or(serde_json::json!([]));
        map.insert("tags".to_string(), normalize_tag_list(tags));
        // Due dates: numeric strings coerce; anything unparsable becomes
        // null instead of failing the whole item.
        let due_raw = map.remove("due_date").unwrap_or(serde_json::Value::Null);
        map.insert(
            "due_date".to_string(),
            coerce_opt_epoch(due_raw)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
        );

        // Colour: blank becomes null, bare names become full labels and JSON
        // text/objects are repaired into well-formed { name, hex } labels.
        let color = map.remove("color").unwrap_or(serde_json::Value::Null);
        map.insert("color".to_string(), normalize_color(color));
    }
    value
}

/// Splits a bare comma-separated string into JSON string values.
fn split_csv_like(text: &str) -> Vec<serde_json::Value> {
    text.split(',')
        .map(|part| serde_json::json!(part.trim()))
        .collect()
}

/// Converts a raw tags value (array, JSON text, comma-separated or bare
/// string) into a list of well-formed tag objects, generating ids where
/// they are missing or invalid.
fn normalize_tag_list(value: serde_json::Value) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = match value {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                if trimmed.starts_with('[') {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if parsed.is_array() {
                            return normalize_tag_list(parsed);
                        }
                    }
                }
                split_csv_like(trimmed)
            }
        }
        _ => Vec::new(),
    };
    let tags = entries
        .into_iter()
        .filter_map(|entry| match entry {
            serde_json::Value::String(name) => {
                let name = name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(serde_json::json!({
                        "id": uuid::Uuid::now_v7().to_string(),
                        "name": name,
                    }))
                }
            }
            serde_json::Value::Object(mut obj) => {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    None
                } else {
                    let id_valid = obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| uuid::Uuid::parse_str(s).is_ok())
                        .unwrap_or(false);
                    if !id_valid {
                        obj.insert(
                            "id".to_string(),
                            serde_json::json!(uuid::Uuid::now_v7().to_string()),
                        );
                    }
                    obj.insert("name".to_string(), serde_json::json!(name));
                    Some(serde_json::Value::Object(obj))
                }
            }
            _ => None,
        })
        .collect::<Vec<serde_json::Value>>();
    serde_json::Value::Array(tags)
}

/// Coerces "true"/"false"/"1"/"0"/"" and JSON booleans/numbers to a bool.
fn coerce_bool(value: serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(parsed) => Some(parsed),
        serde_json::Value::Number(n) => Some(n.as_f64().unwrap_or(0.0) != 0.0),
        serde_json::Value::String(text) => match text.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" | "" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Coerces a raw timestamp value (number, numeric or blank string) to
/// epoch seconds, falling back to `fallback` when unparsable.
fn coerce_epoch(value: serde_json::Value, fallback: i64) -> i64 {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64).unwrap_or(fallback),
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                fallback
            } else {
                trimmed.parse::<i64>().unwrap_or(fallback)
            }
        }
        _ => fallback,
    }
}

/// Coerces a raw optional timestamp to epoch seconds; unparsable values
/// become None instead of failing the item.
fn coerce_opt_epoch(value: serde_json::Value) -> Option<i64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64),
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<i64>().ok()
            }
        }
        _ => None,
    }
}

/// Blank colours become null; bare colour names become full labels; JSON
/// text and objects are repaired into well-formed { name, hex } labels.
fn normalize_color(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Null => serde_json::Value::Null,
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                serde_json::Value::Null
            } else if trimmed.starts_with('{') {
                normalize_color(
                    serde_json::from_str::<serde_json::Value>(trimmed)
                        .unwrap_or(serde_json::Value::Null),
                )
            } else {
                serde_json::json!({ "name": trimmed, "hex": "" })
            }
        }
        serde_json::Value::Object(obj) => {
            let name = obj.get("name").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
            let hex = obj.get("hex").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");
            serde_json::json!({ "name": name, "hex": hex })
        }
        _ => serde_json::Value::Null,
    }
}

/// Bulk import with full fidelity: every item is deserialized into a
/// complete model (ids, tags, colors, favorites, timestamps and ordering
/// are all restored) and upserted through the repository's save(). The
/// strategy decides what happens to items whose id already exists in the
/// vault: skip keeps the existing item, replace overwrites it in place,
/// and copy re-creates it with a fresh id appended after the last item.
pub async fn import_todos(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ImportTodosRequest>,
) -> Result<HttpResponse, ApiError> {
    let ImportTodosRequest { items, strategy } = body.into_inner();
    let existing = data.todo_repo.find_all(&user.user_id).await?;
    let mut known_ids: std::collections::HashSet<uuid::Uuid> =
        existing.iter().map(|todo| todo.meta.id).collect();
    let mut next_pos = data.todo_repo.get_next_position(&user.user_id).await?;
    let mut summary = ImportSummary {
        created: 0,
        replaced: 0,
        skipped: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for (index, value) in items.into_iter().enumerate() {
        let value = fill_import_defaults(value, &mut next_pos);
        let mut todo: Todo = match serde_json::from_value(value) {
            Ok(todo) => todo,
            Err(err) => {
                summary.failed += 1;
                summary.errors.push(format!("Item #{}: invalid data ({})", index + 1, err));
                continue;
            }
        };

        let exists = known_ids.contains(&todo.meta.id);
        if exists {
            match strategy {
                ImportStrategy::Skip => {
                    summary.skipped += 1;
                    continue;
                }
                ImportStrategy::Copy => {
                    todo.meta.id = uuid::Uuid::now_v7();
                    todo.meta.position = next_pos;
                    next_pos += 1.0;
                }
                ImportStrategy::Replace => {}
            }
        }

        if let Err(err) = data.todo_repo.save(&user.user_id, &todo).await {
            summary.failed += 1;
            summary.errors.push(format!("Item #{}: storage error ({})", index + 1, err));
            continue;
        }

        if exists && strategy == ImportStrategy::Replace {
            summary.replaced += 1;
        } else {
            summary.created += 1;
        }
        known_ids.insert(todo.meta.id);
    }

    Ok(HttpResponse::Ok().json(&summary))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct TodoQuery {
    pub search: Option<String>,
}

pub async fn list_todos(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<TodoQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = if let Some(ref q) = query.search {
        data.todo_repo.search(&user.user_id, q).await?
    } else {
        data.todo_repo.find_all(&user.user_id).await?
    };
    Ok(HttpResponse::Ok().json(&items))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub due_date: Option<i64>,
}

pub async fn update_todo(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateTodoRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let existing = data.todo_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Todo not found".to_string()))?;

    let updated = Todo {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        title: body.title.clone().unwrap_or(existing.title),
        description: body.description.clone().unwrap_or(existing.description),
        completed: body.completed.unwrap_or(existing.completed),
        due_date: body.due_date.or(existing.due_date),
    };

    data.todo_repo.save(&user.user_id, &updated).await?;
    Ok(HttpResponse::Ok().json(&updated))
}

pub async fn delete_todo(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.todo_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ReorderRequest {
    pub positions: Vec<PositionEntry>,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct PositionEntry {
    pub id: String,
    pub position: f64,
}

pub async fn reorder_todos(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.todo_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
