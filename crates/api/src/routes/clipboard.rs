use actix_web::{web, HttpResponse};
use domain::models::clipboard::ClipboardItem;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;
use crate::routes::import_common::{
    fill_import_defaults, ImportSpec, ImportStrategy, ImportSummary,
};
pub use crate::routes::dto::{PositionEntry, ReorderRequest};

const CLIPBOARD_IMPORT_SPEC: ImportSpec = ImportSpec {
    string_defaults: &[],
    bool_fields: &[("persist_to_disk", true)],
    string_list_fields: &[],
    has_due_date: false,
    has_binary_fields: false,
};

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateClipboardRequest {
    pub content: String,
    #[serde(default = "default_persist")]
    pub persist_to_disk: bool,
}

fn default_persist() -> bool {
    true
}

pub async fn create_clipboard(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateClipboardRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.clipboard_repo.get_next_position(&user.user_id).await?;
    let mut item = ClipboardItem {
        meta: Default::default(),
        content: body.content.clone(),
        persist_to_disk: body.persist_to_disk,
    };
    item.meta.position = next_pos;
    data.clipboard_repo.save(&user.user_id, &item).await?;
    Ok(HttpResponse::Created().json(&item))
}

// ----- Bulk import (full-fidelity round-trip of exported data) -----

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ImportClipboardRequest {
    pub items: Vec<serde_json::Value>,
    pub strategy: ImportStrategy,
}

/// Bulk import with full fidelity: every item is deserialized into a
/// complete model (ids, tags, colors, favorites, timestamps and ordering
/// are all restored) and upserted through the repository's save(). The
/// strategy decides what happens to items whose id already exists in the
/// vault: skip keeps the existing item, replace overwrites it in place,
/// and copy re-creates it with a fresh id appended after the last item.
pub async fn import_clipboard(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ImportClipboardRequest>,
) -> Result<HttpResponse, ApiError> {
    let ImportClipboardRequest { items, strategy } = body.into_inner();
    let existing = data.clipboard_repo.find_all(&user.user_id).await?;
    let mut known_ids: std::collections::HashSet<uuid::Uuid> =
        existing.iter().map(|item| item.meta.id).collect();
    let mut next_pos = data.clipboard_repo.get_next_position(&user.user_id).await?;
    let mut summary = ImportSummary::new();

    for (index, value) in items.into_iter().enumerate() {
        let value = fill_import_defaults(value, &mut next_pos, &CLIPBOARD_IMPORT_SPEC);
        let mut item: ClipboardItem = match serde_json::from_value(value) {
            Ok(item) => item,
            Err(err) => {
                summary.failed += 1;
                summary.errors.push(format!("Item #{}: invalid data ({})", index + 1, err));
                continue;
            }
        };

        let exists = known_ids.contains(&item.meta.id);
        if exists {
            match strategy {
                ImportStrategy::Skip => {
                    summary.skipped += 1;
                    continue;
                }
                ImportStrategy::Copy => {
                    item.meta.id = uuid::Uuid::now_v7();
                    item.meta.position = next_pos;
                    next_pos += 1.0;
                }
                ImportStrategy::Replace => {}
            }
        }

        if let Err(err) = data.clipboard_repo.save(&user.user_id, &item).await {
            summary.failed += 1;
            summary.errors.push(format!("Item #{}: storage error ({})", index + 1, err));
            continue;
        }

        if exists && strategy == ImportStrategy::Replace {
            summary.replaced += 1;
        } else {
            summary.created += 1;
        }
        known_ids.insert(item.meta.id);
    }

    Ok(HttpResponse::Ok().json(&summary))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ClipboardQuery {
    pub search: Option<String>,
}

pub async fn list_clipboard(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<ClipboardQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = if let Some(ref q) = query.search {
        data.clipboard_repo.search(&user.user_id, q).await?
    } else {
        data.clipboard_repo.find_all(&user.user_id).await?
    };
    Ok(HttpResponse::Ok().json(&items))
}

pub async fn delete_clipboard(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.clipboard_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ReorderRequest / PositionEntry are shared across all vaults — see
// `crate::routes::dto`.

pub async fn reorder_clipboard(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.clipboard_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
