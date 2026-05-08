use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use core::models::note::Note;
use core::models::common::ItemMetadata;
use std::sync::Mutex;
use std::collections::HashMap;

struct AppState {
    notes: Mutex<HashMap<String, Note>>,
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
    let mut notes = data.notes.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let note = Note {
        meta: ItemMetadata {
            id: uuid::Uuid::parse_str(&id).unwrap(),
            ..Default::default()
        },
        title: body.title.clone(),
        content: body.content.clone(),
        is_pinned: false,
    };
    notes.insert(id.clone(), note);
    HttpResponse::Created().json(notes.get(&id).unwrap())
}

async fn list_notes(data: web::Data<AppState>) -> impl Responder {
    let notes = data.notes.lock().unwrap();
    let list: Vec<&Note> = notes.values().collect();
    HttpResponse::Ok().json(&list)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        notes: Mutex::new(HashMap::new()),
    });

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