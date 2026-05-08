mod routes;
mod middleware;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;

use storage_sqlite::migrations::run_migrations;
use storage_sqlite::pool::create_pool;
use storage_sqlite::repositories::clipboard::ClipboardRepository;
use storage_sqlite::repositories::notes::NoteRepository;
use storage_sqlite::repositories::users::UserRepository;
use crate::middleware::auth::Authenticated;
use crate::routes::auth;

struct AppState {
    notes_repo: Arc<NoteRepository>,
    clipboard_repo: Arc<ClipboardRepository>,
    pub user_repo: Arc<UserRepository>,
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

    let pool_clone = pool.clone();
    let notes_repo = Arc::new(NoteRepository::new(Arc::new(pool.clone())));
    let clipboard_repo = Arc::new(ClipboardRepository::new(Arc::new(pool)));
    let user_repo = Arc::new(UserRepository::new(Arc::new(pool_clone)));

    let app_state = web::Data::new(AppState {
        notes_repo,
        clipboard_repo,
        user_repo,
    });

    println!("Server running on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Public routes
            .route("/health", web::get().to(routes::health::health))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/refresh", web::post().to(auth::refresh)),
            )
            // Protected routes (wrapped with Authenticated middleware)
            .service(
                web::scope("")
                    .wrap(Authenticated)
                    .route("/notes", web::post().to(routes::notes::create_note))
                    .route("/notes", web::get().to(routes::notes::list_notes))
                    .route("/clipboard", web::post().to(routes::clipboard::create_clipboard))
                    .route("/clipboard", web::get().to(routes::clipboard::list_clipboard))
                    .route("/clipboard/{id}", web::delete().to(routes::clipboard::delete_clipboard)),
            )
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}