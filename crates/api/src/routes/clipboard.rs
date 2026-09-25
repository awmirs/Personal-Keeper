use actix_web::{web, HttpResponse};
use domain::models::clipboard::ClipboardItem;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

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
