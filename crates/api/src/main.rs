mod routes;
mod middleware;
mod docs;
mod error;

use actix_web::{web, App, HttpServer};
use actix_files::Files;
use std::sync::Arc;

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
use crate::routes::credentials::VaultSession;

struct AppState {
    notes_repo: Arc<dyn domain::traits::repository::Repository<domain::models::note::Note>>,
    clipboard_repo: Arc<dyn domain::traits::repository::Repository<domain::models::clipboard::ClipboardItem>>,
    pub todo_repo: Arc<dyn domain::traits::repository::Repository<domain::models::todo::Todo>>,
    pub bookmark_repo: Arc<dyn domain::traits::repository::Repository<domain::models::bookmark::Bookmark>>,
    pub contact_repo: Arc<dyn domain::traits::repository::Repository<domain::models::contact::Contact>>,
    pub credential_repo: Arc<dyn domain::traits::repository::Repository<domain::models::credential::Credential>>,
    pub credential_config_repo: Arc<dyn domain::traits::credential_config::CredentialConfigRepository>,
    pub user_repo: Arc<dyn domain::traits::user::UserRepository>,
    /// Derived vault keys, keyed by user ID. Async mutex because the
    /// guarded map is touched from async handlers and a `std::sync::Mutex`
    /// would block the Actix worker if the critical section ever awaited.
    pub master_keys: Arc<tokio::sync::Mutex<std::collections::HashMap<String, VaultSession>>>,
    /// Idle timeout after which a user's unlocked vault re-locks itself.
    /// Read once from `VAULT_AUTO_LOCK_SECS`; default 15 minutes.
    pub vault_auto_lock: std::time::Duration,
    pub history_repo: Arc<dyn domain::traits::history::HistoryRepository>,
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
    let notes_repo: Arc<dyn domain::traits::repository::Repository<domain::models::note::Note>> =
        Arc::new(NoteRepository::new(Arc::new(pool.clone())));
    let clipboard_repo: Arc<dyn domain::traits::repository::Repository<domain::models::clipboard::ClipboardItem>> =
        Arc::new(ClipboardRepository::new(Arc::new(pool.clone())));
    let user_repo: Arc<dyn domain::traits::user::UserRepository> =
        Arc::new(UserRepository::new(Arc::new(pool_clone)));
    let todo_repo: Arc<dyn domain::traits::repository::Repository<domain::models::todo::Todo>> =
        Arc::new(TodoRepository::new(Arc::new(pool.clone())));
    let bookmark_repo: Arc<dyn domain::traits::repository::Repository<domain::models::bookmark::Bookmark>> =
        Arc::new(BookmarkRepository::new(Arc::new(pool.clone())));
    let contact_repo: Arc<dyn domain::traits::repository::Repository<domain::models::contact::Contact>> =
        Arc::new(ContactRepository::new(Arc::new(pool.clone())));
    let credential_config_repo: Arc<dyn domain::traits::credential_config::CredentialConfigRepository> =
        Arc::new(CredentialConfigRepository::new(Arc::new(pool.clone())));
    let credential_repo: Arc<dyn domain::traits::repository::Repository<domain::models::credential::Credential>> =
        Arc::new(CredentialRepository::new(Arc::new(pool.clone())));
    // Idle timeout for unlocked vault sessions. A missing or unparsable
    // value silently falls back to the default so a misconfiguration can
    // never prevent the server from starting.
    let vault_auto_lock = std::env::var("VAULT_AUTO_LOCK_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or_else(|| std::time::Duration::from_secs(900));

    let master_keys = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    let history_repo: Arc<dyn domain::traits::history::HistoryRepository> =
        Arc::new(storage_sqlite::repositories::history::HistoryRepository::new(Arc::new(pool.clone())));

    let app_state = web::Data::new(AppState {
        notes_repo,
        clipboard_repo,
        user_repo,
        todo_repo,
        bookmark_repo,
        contact_repo,
        credential_config_repo,
        credential_repo,
        master_keys,
        vault_auto_lock,
        history_repo,
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
                    .app_data(web::JsonConfig::default().limit(64 * 1024 * 1024))
                    .route("/search", web::get().to(routes::search::search))
                    .configure(crate::routes::history::configure_history)
                    .route("/notes", web::post().to(routes::notes::create_note))
                    .route("/notes", web::get().to(routes::notes::list_notes))
                    .route("/notes/reorder", web::put().to(routes::notes::reorder_notes))
                    .route("/notes/import", web::post().to(routes::notes::import_notes))
                    .route("/notes/{id}", web::put().to(routes::notes::update_note))
                    .route("/notes/{id}", web::delete().to(routes::notes::delete_note))
                    .route("/clipboard", web::post().to(routes::clipboard::create_clipboard))
                    .route("/clipboard", web::get().to(routes::clipboard::list_clipboard))
                    .route("/clipboard/reorder", web::put().to(routes::clipboard::reorder_clipboard))
                    .route("/clipboard/import", web::post().to(routes::clipboard::import_clipboard))
                    .route("/clipboard/{id}", web::delete().to(routes::clipboard::delete_clipboard))
                    .route("/todos", web::post().to(routes::todos::create_todo))
                    .route("/todos", web::get().to(routes::todos::list_todos))
                    .route("/todos/reorder", web::put().to(routes::todos::reorder_todos))
                    .route("/todos/import", web::post().to(routes::todos::import_todos))
                    .route("/todos/{id}", web::put().to(routes::todos::update_todo))
                    .route("/todos/{id}", web::delete().to(routes::todos::delete_todo))
                    .route("/bookmarks", web::post().to(routes::bookmarks::create_bookmark))
                    .route("/bookmarks", web::get().to(routes::bookmarks::list_bookmarks))
                    .route("/bookmarks/reorder", web::put().to(routes::bookmarks::reorder_bookmarks))
                    .route("/bookmarks/import", web::post().to(routes::bookmarks::import_bookmarks))
                    .route("/bookmarks/{id}", web::put().to(routes::bookmarks::update_bookmark))
                    .route("/bookmarks/{id}", web::delete().to(routes::bookmarks::delete_bookmark))
                    .route("/contacts", web::post().to(routes::contacts::create_contact))
                    .route("/contacts", web::get().to(routes::contacts::list_contacts))
                    .route("/contacts/reorder", web::put().to(routes::contacts::reorder_contacts))
                    .route("/contacts/import", web::post().to(routes::contacts::import_contacts))
                    .route("/contacts/{id}", web::put().to(routes::contacts::update_contact))
                    .route("/contacts/{id}", web::delete().to(routes::contacts::delete_contact))
                    .route("/credentials/status", web::get().to(routes::credentials::vault_status))
                    .route("/credentials/unlock", web::post().to(routes::credentials::unlock))
                    .route("/credentials/lock", web::post().to(routes::credentials::lock))
                    .route("/credentials", web::post().to(routes::credentials::create_credential))
                    .route("/credentials", web::get().to(routes::credentials::list_credentials))
                    .route("/credentials/reorder", web::put().to(routes::credentials::reorder_credentials))
                    .route("/credentials/{id}", web::get().to(routes::credentials::get_credential))
                    .route("/credentials/{id}", web::put().to(routes::credentials::update_credential))
                    .route("/credentials/{id}", web::delete().to(routes::credentials::delete_credential)),
            )
            // --- SPA fallback (serve index.html for anything else) ---
            .configure(|cfg| {
                #[cfg(feature = "swagger")]
                cfg.service(docs::swagger_ui_service());

                #[cfg(not(feature = "swagger"))]
                let _ = &cfg;
            })
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}