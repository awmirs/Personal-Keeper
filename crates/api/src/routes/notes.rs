use actix_web::{web, HttpResponse};
use domain::models::note::Note;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

pub async fn create_note(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateNoteRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.notes_repo.get_next_position(&user.user_id).await?;
    let mut note = Note {
        meta: Default::default(),
        title: body.title.clone(),
        content: body.content.clone(),
        is_pinned: false,
    };
    note.meta.position = next_pos;
    data.notes_repo.save(&user.user_id, &note).await?;
    Ok(HttpResponse::Created().json(&note))
}

pub async fn list_notes(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let notes = data.notes_repo.find_all(&user.user_id).await?;
    Ok(HttpResponse::Ok().json(&notes))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

pub async fn update_note(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateNoteRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let existing = data.notes_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Note not found".to_string()))?;

    let updated = Note {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        title: body.title.clone().unwrap_or(existing.title),
        content: body.content.clone().unwrap_or(existing.content),
        is_pinned: existing.is_pinned,
    };

    data.notes_repo.save(&user.user_id, &updated).await?;
    Ok(HttpResponse::Ok().json(&updated))
}

pub async fn delete_note(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.notes_repo.delete(&user.user_id, &path.into_inner()).await?;
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

pub async fn reorder_notes(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.notes_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
