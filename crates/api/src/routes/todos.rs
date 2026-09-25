use actix_web::{web, HttpResponse};
use domain::models::todo::Todo;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
}

pub async fn create_todo(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.todo_repo.get_next_position(&user.user_id).await?;
    let mut todo = Todo {
        meta: Default::default(),
        title: body.title.clone(),
        description: body.description.clone().unwrap_or_default(),
        completed: false,
        due_date: body.due_date,
    };
    todo.meta.position = next_pos;
    data.todo_repo.save(&user.user_id, &todo).await?;
    Ok(HttpResponse::Created().json(&todo))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct TodoQuery {
    pub search: Option<String>,
}

pub async fn list_todos(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<TodoQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = if let Some(ref q) = query.search {
        data.todo_repo.search(&user.user_id, q).await?
    } else {
        data.todo_repo.find_all(&user.user_id).await?
    };
    Ok(HttpResponse::Ok().json(&items))
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
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateTodoRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let existing = data.todo_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Todo not found".to_string()))?;

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

    data.todo_repo.save(&user.user_id, &updated).await?;
    Ok(HttpResponse::Ok().json(&updated))
}

pub async fn delete_todo(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.todo_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
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
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.todo_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
