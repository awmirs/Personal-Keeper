use actix_web::{web, HttpResponse};
use domain::models::bookmark::Bookmark;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn create_bookmark(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateBookmarkRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.bookmark_repo.get_next_position(&user.user_id).await?;
    let mut bookmark = Bookmark {
        meta: Default::default(),
        url: body.url.clone(),
        title: body.title.clone().unwrap_or_default(),
        description: body.description.clone().unwrap_or_default(),
        favicon: None,
        thumbnail: None,
    };
    bookmark.meta.position = next_pos;
    data.bookmark_repo.save(&user.user_id, &bookmark).await?;
    Ok(HttpResponse::Created().json(&bookmark))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct BookmarkQuery {
    pub search: Option<String>,
}

pub async fn list_bookmarks(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<BookmarkQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = if let Some(ref q) = query.search {
        data.bookmark_repo.search(&user.user_id, q).await?
    } else {
        data.bookmark_repo.find_all(&user.user_id).await?
    };
    Ok(HttpResponse::Ok().json(&items))
}

pub async fn delete_bookmark(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.bookmark_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateBookmarkRequest {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn update_bookmark(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateBookmarkRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let existing = data.bookmark_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Bookmark not found".to_string()))?;

    let updated = Bookmark {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        url: body.url.clone().unwrap_or(existing.url),
        title: body.title.clone().unwrap_or(existing.title),
        description: body.description.clone().unwrap_or(existing.description),
        favicon: existing.favicon,
        thumbnail: existing.thumbnail,
    };

    data.bookmark_repo.save(&user.user_id, &updated).await?;
    Ok(HttpResponse::Ok().json(&updated))
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

pub async fn reorder_bookmarks(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.bookmark_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
