use actix_web::{web, HttpResponse, Responder};
use domain::models::note::Note;
use domain::traits::repository::Repository;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

pub async fn create_note(
    data: web::Data<AppState>,
    body: web::Json<CreateNoteRequest>,
) -> impl Responder {
    let note = Note {
        meta: Default::default(),
        title: body.title.clone(),
        content: body.content.clone(),
        is_pinned: false,
    };
    match data.notes_repo.save(&note).await {
        Ok(()) => HttpResponse::Created().json(&note),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn list_notes(data: web::Data<AppState>) -> impl Responder {
    match data.notes_repo.find_all().await {
        Ok(notes) => HttpResponse::Ok().json(&notes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

pub async fn update_note(
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateNoteRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let existing = match data.notes_repo.find_by_id(&id).await {
        Ok(Some(n)) => n,
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Note not found" })),
    };

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

    match data.notes_repo.save(&updated).await {
        Ok(()) => HttpResponse::Ok().json(&updated),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}