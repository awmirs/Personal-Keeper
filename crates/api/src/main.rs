use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use domain::models::note::Note;
use domain::traits::repository::Repository;
use storage_sqlite::migrations::run_migrations;
use storage_sqlite::pool::create_pool;
use storage_sqlite::repositories::notes::NoteRepository;

struct AppState {
    notes_repo: Arc<NoteRepository>,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create the data directory if it doesn't exist (optional)
    let _ = std::fs::create_dir_all("data");

    let pool = create_pool("data/personal-keeper.db")
        .expect("Failed to create SQLite pool");
    run_migrations(&pool)
        .expect("Failed to run migrations");

    let notes_repo = Arc::new(NoteRepository::new(Arc::new(pool)));

    let app_state = web::Data::new(AppState { notes_repo });

    println!("Server running on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .route("/notes", web::post().to(create_note))
            .route("/notes", web::get().to(list_notes))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}