use actix_web::{web, HttpResponse, Responder};
use domain::models::clipboard::ClipboardItem;
use domain::traits::repository::Repository;
use crate::AppState;

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
    data: web::Data<AppState>,
    body: web::Json<CreateClipboardRequest>,
) -> impl Responder {
    let item = ClipboardItem {
        meta: Default::default(),
        content: body.content.clone(),
        persist_to_disk: body.persist_to_disk,
    };
    match data.clipboard_repo.save(&item).await {
        Ok(()) => HttpResponse::Created().json(&item),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
pub struct ClipboardQuery {
    pub search: Option<String>,
}

pub async fn list_clipboard(
    data: web::Data<AppState>,
    query: web::Query<ClipboardQuery>,
) -> impl Responder {
    let result = if let Some(ref q) = query.search {
        data.clipboard_repo.search(q).await
    } else {
        data.clipboard_repo.find_all().await
    };
    match result {
        Ok(items) => HttpResponse::Ok().json(&items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn delete_clipboard(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match data.clipboard_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}