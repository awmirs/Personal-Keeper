use actix_web::{web, HttpResponse, Responder};
use domain::models::bookmark::Bookmark;
use domain::traits::repository::Repository;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn create_bookmark(
    data: web::Data<AppState>,
    body: web::Json<CreateBookmarkRequest>,
) -> impl Responder {
    let bookmark = Bookmark {
        meta: Default::default(),
        url: body.url.clone(),
        title: body.title.clone().unwrap_or_default(),
        description: body.description.clone().unwrap_or_default(),
        favicon: None,
        thumbnail: None,
    };
    match data.bookmark_repo.save(&bookmark).await {
        Ok(()) => HttpResponse::Created().json(&bookmark),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
pub struct BookmarkQuery {
    pub search: Option<String>,
}

pub async fn list_bookmarks(
    data: web::Data<AppState>,
    query: web::Query<BookmarkQuery>,
) -> impl Responder {
    let result = if let Some(ref q) = query.search {
        data.bookmark_repo.search(q).await
    } else {
        data.bookmark_repo.find_all().await
    };
    match result {
        Ok(items) => HttpResponse::Ok().json(&items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn delete_bookmark(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match data.bookmark_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}