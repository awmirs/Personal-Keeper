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

/// Fills in server-side defaults for fields that are missing or empty in
/// partially-sourced items (e.g. hand-written CSV rows): a fresh id, current
/// timestamps, the next free ordering position and sensible values for
/// optional, list and boolean fields.
fn fill_import_defaults(mut value: serde_json::Value, next_pos: &mut f64) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    if let serde_json::Value::Object(ref mut map) = value {
        map.entry("id")
            .or_insert_with(|| serde_json::json!(uuid::Uuid::now_v7().to_string()));
        map.entry("created_at").or_insert_with(|| serde_json::json!(now));
        map.entry("updated_at").or_insert_with(|| serde_json::json!(now));
        map.entry("is_favorite").or_insert_with(|| serde_json::json!(false));
        map.entry("trash_status").or_insert_with(|| serde_json::json!("Active"));
        map.entry("tags").or_insert_with(|| serde_json::json!([]));
        map.entry("description").or_insert_with(|| serde_json::json!(""));
        if !map.contains_key("position") {
            let position = *next_pos;
            *next_pos += 1.0;
            map.insert("position".to_string(), serde_json::json!(position));
        }
        // Empty CSV cells arrive as ""; normalize them for fields that are
        // not plain strings.
        for key in ["tags"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::json!([]));
            }
        }
        for key in ["is_favorite", "completed"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::json!(false));
            }
        }
        for key in ["color", "due_date"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::Value::Null);
            }
        }
    }
    value
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
