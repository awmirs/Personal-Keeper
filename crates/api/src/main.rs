mod routes;
mod middleware;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;

use storage_sqlite::migrations::run_migrations;
use storage_sqlite::pool::create_pool;
use storage_sqlite::repositories::bookmarks::BookmarkRepository;
use storage_sqlite::repositories::clipboard::ClipboardRepository;
use storage_sqlite::repositories::contacts::ContactRepository;
use storage_sqlite::repositories::notes::NoteRepository;
use storage_sqlite::repositories::todos::TodoRepository;
use storage_sqlite::repositories::users::UserRepository;
use crate::middleware::auth::Authenticated;
use crate::routes::auth;

struct AppState {
    notes_repo: Arc<NoteRepository>,
    clipboard_repo: Arc<ClipboardRepository>,
    pub todo_repo: Arc<TodoRepository>,
    pub bookmark_repo: Arc<BookmarkRepository>,
    pub contact_repo: Arc<ContactRepository>,
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
    let clipboard_repo = Arc::new(ClipboardRepository::new(Arc::new(pool.clone())));
    let user_repo = Arc::new(UserRepository::new(Arc::new(pool_clone)));
    let todo_repo = Arc::new(TodoRepository::new(Arc::new(pool.clone())));
    let bookmark_repo = Arc::new(BookmarkRepository::new(Arc::new(pool.clone())));
    let contact_repo = Arc::new(ContactRepository::new(Arc::new(pool.clone())));

    let app_state = web::Data::new(AppState {
        notes_repo,
        clipboard_repo,
        user_repo,
        todo_repo,
        bookmark_repo,
        contact_repo,
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
                    .route("/clipboard/{id}", web::delete().to(routes::clipboard::delete_clipboard))
                    .route("/todos", web::post().to(routes::todos::create_todo))
                    .route("/todos", web::get().to(routes::todos::list_todos))
                    .route("/todos/{id}", web::put().to(routes::todos::update_todo))
                    .route("/todos/{id}", web::delete().to(routes::todos::delete_todo))
                    .route("/bookmarks", web::post().to(routes::bookmarks::create_bookmark))
                    .route("/bookmarks", web::get().to(routes::bookmarks::list_bookmarks))
                    .route("/bookmarks/{id}", web::delete().to(routes::bookmarks::delete_bookmark))
                    .route("/contacts", web::post().to(routes::contacts::create_contact))
                    .route("/contacts", web::get().to(routes::contacts::list_contacts))
                    .route("/contacts/{id}", web::delete().to(routes::contacts::delete_contact)),
            )
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}