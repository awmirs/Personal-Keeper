use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use domain::models::clipboard::ClipboardItem;
use domain::models::note::Note;
use domain::traits::repository::Repository;
use std::sync::Arc;

use storage_sqlite::migrations::run_migrations;
use storage_sqlite::pool::create_pool;
use storage_sqlite::repositories::clipboard::ClipboardRepository;
use storage_sqlite::repositories::notes::NoteRepository;

struct AppState {
    notes_repo: Arc<NoteRepository>,
    clipboard_repo: Arc<ClipboardRepository>,
}

// ---------------- Health ----------------
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

// ---------------- Notes ----------------
#[derive(serde::Deserialize)]
struct CreateNoteRequest {
    title: String,
    content: String,
}

async fn create_note(
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

async fn list_notes(data: web::Data<AppState>) -> impl Responder {
    match data.notes_repo.find_all().await {
        Ok(notes) => HttpResponse::Ok().json(&notes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ---------------- Clipboard ----------------
#[derive(serde::Deserialize)]
struct CreateClipboardRequest {
    content: String,
    #[serde(default = "default_persist")]
    persist_to_disk: bool,
}

fn default_persist() -> bool {
    true
}

async fn create_clipboard(
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
struct ClipboardQuery {
    search: Option<String>,
}

async fn list_clipboard(
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

async fn delete_clipboard(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match data.clipboard_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ---------------- Main ----------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create the data directory if it doesn't exist (optional)
    let _ = std::fs::create_dir_all("data");

    let pool = create_pool("data/personal-keeper.db")
        .expect("Failed to create SQLite pool");
    run_migrations(&pool)
        .expect("Failed to run migrations");

    let notes_repo = Arc::new(NoteRepository::new(Arc::new(pool.clone())));
    let clipboard_repo = Arc::new(ClipboardRepository::new(Arc::new(pool)));

    let app_state = web::Data::new(AppState {
        notes_repo,
        clipboard_repo,
    });

    println!("Server running on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Health
            .route("/health", web::get().to(health))
            // Notes
            .route("/notes", web::post().to(create_note))
            .route("/notes", web::get().to(list_notes))
            // Clipboard
            .route("/clipboard", web::post().to(create_clipboard))
            .route("/clipboard", web::get().to(list_clipboard))
            .route("/clipboard/{id}", web::delete().to(delete_clipboard))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}