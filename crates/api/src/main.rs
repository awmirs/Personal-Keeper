mod routes;
mod middleware;

use actix_web::{web, App, HttpServer};
use actix_files::Files;
use std::sync::{Arc, Mutex};

use storage_sqlite::migrations::run_migrations;
use storage_sqlite::pool::create_pool;
use storage_sqlite::repositories::bookmarks::BookmarkRepository;
use storage_sqlite::repositories::clipboard::ClipboardRepository;
use storage_sqlite::repositories::contacts::ContactRepository;
use storage_sqlite::repositories::notes::NoteRepository;
use storage_sqlite::repositories::todos::TodoRepository;
use storage_sqlite::repositories::users::UserRepository;
use storage_sqlite::repositories::credentials::CredentialRepository;
use storage_sqlite::repositories::credentials_config::CredentialConfigRepository;
use crate::middleware::auth::Authenticated;
use crate::routes::auth;

struct AppState {
    notes_repo: Arc<NoteRepository>,
    clipboard_repo: Arc<ClipboardRepository>,
    pub todo_repo: Arc<TodoRepository>,
    pub bookmark_repo: Arc<BookmarkRepository>,
    pub contact_repo: Arc<ContactRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub credential_config_repo: Arc<CredentialConfigRepository>,
    pub user_repo: Arc<UserRepository>,
    pub master_key: Arc<Mutex<Option<[u8; 32]>>>,   // derived key
}


// ---------------- Main ----------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();   // load .env if present
    // Create the data directory if it doesn't exist (optional)
    let _ = std::fs::create_dir_all("data");

    let pool = create_pool("data/personal-keeper.db")
        .expect("Failed to create SQLite pool");
    run_migrations(&pool)
        .expect("Failed to run migrations");

    let pool_clone = pool.clone();
    let notes_repo = Arc::new(NoteRepository::new(Arc::new(pool.clone())));
    let clipboard_repo = Arc::new(ClipboardRepository::new(Arc::new(pool.clone())));
    let user_repo = Arc::new(UserRepository::new(Arc::new(pool_clone)));
    let todo_repo = Arc::new(TodoRepository::new(Arc::new(pool.clone())));
    let bookmark_repo = Arc::new(BookmarkRepository::new(Arc::new(pool.clone())));
    let contact_repo = Arc::new(ContactRepository::new(Arc::new(pool.clone())));
    let credential_config_repo = Arc::new(CredentialConfigRepository::new(Arc::new(pool.clone())));
    let credential_repo = Arc::new(CredentialRepository::new(Arc::new(pool.clone())));
    let master_key = Arc::new(Mutex::new(None));


    let app_state = web::Data::new(AppState {
        notes_repo,
        clipboard_repo,
        user_repo,
        todo_repo,
        bookmark_repo,
        contact_repo,
        credential_config_repo,
        credential_repo,
        master_key
    });

    println!("Server running on http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // --- public API ---
            .route("/health", web::get().to(routes::health::health))
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/refresh", web::post().to(auth::refresh))
                    // protected sub‑scope
                    .service(
                        web::scope("")
                            .wrap(Authenticated)
                            .route("/me", web::get().to(routes::auth::me)),
                    ),
            )
            // --- protected API (all under /api) ---
            .service(
                web::scope("/api")
                    .wrap(Authenticated)
                    .route("/notes", web::post().to(routes::notes::create_note))
                    .route("/notes", web::get().to(routes::notes::list_notes))
                    .route("/notes/{id}", web::put().to(routes::notes::update_note))
                    .route("/clipboard", web::post().to(routes::clipboard::create_clipboard))
                    .route("/clipboard", web::get().to(routes::clipboard::list_clipboard))
                    .route("/clipboard/{id}", web::delete().to(routes::clipboard::delete_clipboard))
                    .route("/todos", web::post().to(routes::todos::create_todo))
                    .route("/todos", web::get().to(routes::todos::list_todos))
                    .route("/todos/{id}", web::put().to(routes::todos::update_todo))
                    .route("/todos/{id}", web::delete().to(routes::todos::delete_todo))
                    .route("/bookmarks", web::post().to(routes::bookmarks::create_bookmark))
                    .route("/bookmarks/{id}", web::put().to(routes::bookmarks::update_bookmark))
                    .route("/bookmarks", web::get().to(routes::bookmarks::list_bookmarks))
                    .route("/bookmarks/{id}", web::delete().to(routes::bookmarks::delete_bookmark))
                    .route("/contacts", web::post().to(routes::contacts::create_contact))
                    .route("/contacts/{id}", web::put().to(routes::contacts::update_contact))
                    .route("/contacts", web::get().to(routes::contacts::list_contacts))
                    .route("/contacts/{id}", web::delete().to(routes::contacts::delete_contact))
                    .route("/credentials/status", web::get().to(routes::credentials::vault_status))
                    .route("/credentials/unlock", web::post().to(routes::credentials::unlock))
                    .route("/credentials/lock", web::post().to(routes::credentials::lock))
                    .route("/credentials", web::post().to(routes::credentials::create_credential))
                    .route("/credentials", web::get().to(routes::credentials::list_credentials))
                    .route("/credentials/{id}", web::get().to(routes::credentials::get_credential))
                    .route("/credentials/{id}", web::put().to(routes::credentials::update_credential))
                    .route("/credentials/{id}", web::delete().to(routes::credentials::delete_credential)),
            )
            // --- SPA fallback (serve index.html for anything else) ---
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}