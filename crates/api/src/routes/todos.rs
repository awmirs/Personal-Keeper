use actix_web::{web, HttpResponse, Responder};
use domain::models::todo::Todo;
use domain::traits::repository::Repository;
use crate::AppState;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
}

pub async fn create_todo(
    data: web::Data<AppState>,
    body: web::Json<CreateTodoRequest>,
) -> impl Responder {
    let next_pos = match data.todo_repo.get_next_position().await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };
    let mut todo = Todo {
        meta: Default::default(),
        title: body.title.clone(),
        description: body.description.clone().unwrap_or_default(),
        completed: false,
        due_date: body.due_date,
    };
    todo.meta.position = next_pos;
    match data.todo_repo.save(&todo).await {
        Ok(()) => HttpResponse::Created().json(&todo),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct TodoQuery {
    pub search: Option<String>,
}

pub async fn list_todos(
    data: web::Data<AppState>,
    query: web::Query<TodoQuery>,
) -> impl Responder {
    let result = if let Some(ref q) = query.search {
        data.todo_repo.search(q).await
    } else {
        data.todo_repo.find_all().await
    };
    match result {
        Ok(items) => HttpResponse::Ok().json(&items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub due_date: Option<i64>,
}

pub async fn update_todo(
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateTodoRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let existing = match data.todo_repo.find_by_id(&id).await {
        Ok(Some(t)) => t,
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Todo not found" })),
    };

    let updated = Todo {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        title: body.title.clone().unwrap_or(existing.title),
        description: body.description.clone().unwrap_or(existing.description),
        completed: body.completed.unwrap_or(existing.completed),
        due_date: body.due_date.or(existing.due_date),
    };

    match data.todo_repo.save(&updated).await {
        Ok(()) => HttpResponse::Ok().json(&updated),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn delete_todo(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match data.todo_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ReorderRequest {
    pub positions: Vec<PositionEntry>,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct PositionEntry {
    pub id: String,
    pub position: f64,
}

pub async fn reorder_todos(
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> impl Responder {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    match data.todo_repo.update_positions(&positions).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}