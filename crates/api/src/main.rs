mod routes;
mod middleware;
mod docs;
mod error;
mod state;

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
use crate::state::VaultSession;

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
            .configure(routes::auth::configure_auth)
            // --- protected API (all under /api) ---
            .service(
                web::scope("/api")
                    .wrap(Authenticated)
                    .app_data(web::JsonConfig::default().limit(64 * 1024 * 1024))
                    .configure(routes::search::configure_search)
                    .configure(crate::routes::history::configure_history)
                    .configure(routes::notes::configure_notes)
                    .configure(routes::clipboard::configure_clipboard)
                    .configure(routes::todos::configure_todos)
                    .configure(routes::bookmarks::configure_bookmarks)
                    .configure(routes::contacts::configure_contacts)
                    .configure(routes::credentials::configure_credentials),
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