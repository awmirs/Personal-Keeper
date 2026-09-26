// crates/api/src/routes/history.rs
// REST API for the item version history (see migration V10).

use std::collections::BTreeSet;

use actix_web::{web, HttpResponse};
use domain::models::history::{is_valid_item_type, ItemVersion};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::middleware::auth::AuthUser;
use crate::AppState;

/// Credential fields that are never exposed through the history API.
const SECRET_FIELDS: &[&str] = &["password_encrypted", "notes_encrypted", "totp_secret_encrypted"];
const HIDDEN_VALUE: &str = "[hidden]";
/// Technical fields excluded from field-level diffs (pure noise).
const DIFF_EXCLUDED_FIELDS: &[&str] = &["id", "user_id", "created_at", "updated_at"];

#[derive(serde::Serialize)]
pub struct ItemVersionDto {
    pub id: String,
    pub item_type: String,
    pub item_id: String,
    pub version: i64,
    pub operation: String,
    pub created_at: i64,
    pub title: Option<String>,
    pub data: Option<Value>,
}

#[derive(serde::Serialize)]
pub struct FieldDiffDto {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

#[derive(serde::Serialize)]
pub struct VersionDiffDto {
    pub item_type: String,
    pub item_id: String,
    pub version_a: i64,
    pub version_b: i64,
    pub operation_a: String,
    pub operation_b: String,
    pub created_at_a: i64,
    pub created_at_b: i64,
    pub fields: Vec<FieldDiffDto>,
}

#[derive(serde::Serialize)]
pub struct ActivityEntryDto {
    pub id: String,
    pub item_type: String,
    pub item_id: String,
    pub version: i64,
    pub operation: String,
    pub created_at: i64,
    pub title: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RecentQuery {
    pub limit: Option<i64>,
    pub item_type: Option<String>,
}

fn parse_snapshot(version: &ItemVersion) -> Result<Value, ApiError> {
    serde_json::from_str(&version.data).map_err(|err| {
        ApiError::Internal(format!(
            "Corrupt history snapshot for {} {} version {}: {}",
            version.item_type, version.item_id, version.version, err
        ))
    })
}

fn mask_secret(field: &str, value: &mut Value) {
    if SECRET_FIELDS.contains(&field) {
        if let Value::String(text) = value {
            if !text.is_empty() {
                *text = HIDDEN_VALUE.to_string();
            }
        }
    }
}

fn snapshot_title(item_type: &str, data: &Value) -> Option<String> {
    let object = data.as_object()?;
    let text = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };
    match item_type {
        "note" | "todo" => text("title"),
        "bookmark" => text("title").or_else(|| text("url")),
        "clipboard" => {
            let content = object.get("content").and_then(Value::as_str)?;
            let first = content.lines().next().unwrap_or("").trim().to_string();
            if first.chars().count() > 80 {
                let mut truncated: String = first.chars().take(80).collect();
                truncated.push('\u{2026}');
                Some(truncated)
            } else {
                Some(first)
            }
        }
        "contact" => text("name"),
        "credential" => text("website").or_else(|| text("username")),
        _ => None,
    }
}

fn version_to_dto(version: &ItemVersion, include_data: bool) -> Result<ItemVersionDto, ApiError> {
    let data = parse_snapshot(version)?;
    let title = snapshot_title(&version.item_type, &data);
    let data = if include_data {
        let mut value = data;
        if let Some(map) = value.as_object_mut() {
            for field in SECRET_FIELDS {
                let masked = match map.get(*field) {
                    Some(Value::String(text)) if !text.is_empty() => {
                        Some(Value::String(HIDDEN_VALUE.to_string()))
                    }
                    _ => None,
                };
                if let Some(masked) = masked {
                    map.insert((*field).to_string(), masked);
                }
            }
        }
        Some(value)
    } else {
        None
    };
    Ok(ItemVersionDto {
        id: version.id.clone(),
        item_type: version.item_type.clone(),
        item_id: version.item_id.clone(),
        version: version.version,
        operation: version.operation.clone(),
        created_at: version.created_at,
        title,
        data,
    })
}

fn diff_snapshots(a: &ItemVersion, b: &ItemVersion) -> Result<VersionDiffDto, ApiError> {
    let data_a = parse_snapshot(a)?;
    let data_b = parse_snapshot(b)?;
    let mut fields = Vec::new();
    if let (Some(object_a), Some(object_b)) = (data_a.as_object(), data_b.as_object()) {
        let keys: BTreeSet<&String> = object_a.keys().chain(object_b.keys()).collect();
        for key in keys {
            if DIFF_EXCLUDED_FIELDS.contains(&key.as_str()) {
                continue;
            }
            let value_a = object_a.get(key);
            let value_b = object_b.get(key);
            if value_a == value_b {
                continue;
            }
            let mut old_value = value_a.cloned();
            let mut new_value = value_b.cloned();
            if let Some(value) = old_value.as_mut() {
                mask_secret(key, value);
            }
            if let Some(value) = new_value.as_mut() {
                mask_secret(key, value);
            }
            fields.push(FieldDiffDto {
                field: key.clone(),
                old_value,
                new_value,
            });
        }
    }
    Ok(VersionDiffDto {
        item_type: a.item_type.clone(),
        item_id: a.item_id.clone(),
        version_a: a.version,
        version_b: b.version,
        operation_a: a.operation.clone(),
        operation_b: b.operation.clone(),
        created_at_a: a.created_at,
        created_at_b: b.created_at,
        fields,
    })
}

fn bad_type(item_type: &str) -> ApiError {
    ApiError::BadRequest(format!("Unsupported item type: {}", item_type))
}

pub async fn list_item_versions(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (item_type, item_id) = path.into_inner();
    if !is_valid_item_type(&item_type) {
        return Err(bad_type(&item_type));
    }
    let versions = data
        .history_repo
        .list_versions(&user.user_id, &item_type, &item_id)
        .await?;
    let dtos = versions
        .iter()
        .map(|version| version_to_dto(version, true))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HttpResponse::Ok().json(dtos))
}

pub async fn get_item_version(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<(String, String, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (item_type, item_id, version) = path.into_inner();
    if !is_valid_item_type(&item_type) {
        return Err(bad_type(&item_type));
    }
    let found = data
        .history_repo
        .get_version(&user.user_id, &item_type, &item_id, version)
        .await?;
    let dto = version_to_dto(&found, true)?;
    Ok(HttpResponse::Ok().json(dto))
}

pub async fn diff_item_versions(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<(String, String, i64, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (item_type, item_id, version_a, version_b) = path.into_inner();
    if !is_valid_item_type(&item_type) {
        return Err(bad_type(&item_type));
    }
    let a = data
        .history_repo
        .get_version(&user.user_id, &item_type, &item_id, version_a)
        .await?;
    let b = data
        .history_repo
        .get_version(&user.user_id, &item_type, &item_id, version_b)
        .await?;
    let dto = diff_snapshots(&a, &b)?;
    Ok(HttpResponse::Ok().json(dto))
}

pub async fn restore_item_version(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<(String, String, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (item_type, item_id, version) = path.into_inner();
    if !is_valid_item_type(&item_type) {
        return Err(bad_type(&item_type));
    }
    let restored = data
        .history_repo
        .restore_version(&user.user_id, &item_type, &item_id, version)
        .await?;
    let dto = version_to_dto(&restored, true)?;
    Ok(HttpResponse::Ok().json(dto))
}

pub async fn purge_item_history(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (item_type, item_id) = path.into_inner();
    if !is_valid_item_type(&item_type) {
        return Err(bad_type(&item_type));
    }
    let purged = data
        .history_repo
        .purge_history(&user.user_id, &item_type, &item_id)
        .await?;
    Ok(HttpResponse::Ok().json(json!({ "purged": purged })))
}

pub async fn recent_activity(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<RecentQuery>,
) -> Result<HttpResponse, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let item_type = query
        .item_type
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    let entries = data
        .history_repo
        .recent_activity(&user.user_id, item_type, limit)
        .await?;
    let dtos = entries
        .iter()
        .map(|entry| -> Result<ActivityEntryDto, ApiError> {
            let snapshot = parse_snapshot(entry)?;
            Ok(ActivityEntryDto {
                id: entry.id.clone(),
                item_type: entry.item_type.clone(),
                item_id: entry.item_id.clone(),
                version: entry.version,
                operation: entry.operation.clone(),
                created_at: entry.created_at,
                title: snapshot_title(&entry.item_type, &snapshot),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HttpResponse::Ok().json(dtos))
}

/// Scope registration: mounted inside the authenticated `/api` scope in
/// main.rs via `.configure(crate::routes::history::configure_history)`;
/// the effective paths are `/api/history/*` and every endpoint requires
/// authentication, like the rest of the API.
pub fn configure_history(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/history")
            .service(
                web::resource("/recent").route(web::get().to(recent_activity)),
            )
            .service(
                web::resource("/{item_type}/{item_id}")
                    .route(web::get().to(list_item_versions))
                    .route(web::delete().to(purge_item_history)),
            )
            .service(
                web::resource("/{item_type}/{item_id}/versions/{version}")
                    .route(web::get().to(get_item_version)),
            )
            .service(
                web::resource("/{item_type}/{item_id}/versions/{version}/restore")
                    .route(web::post().to(restore_item_version)),
            )
            .service(
                web::resource("/{item_type}/{item_id}/diff/{version_a}/{version_b}")
                    .route(web::get().to(diff_item_versions)),
            ),
    );
}
