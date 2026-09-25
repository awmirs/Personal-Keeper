use actix_web::{web, HttpResponse};
use domain::models::contact::Contact;
use domain::traits::repository::Repository;
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateContactRequest {
    pub name: String,
    pub phones: Option<Vec<String>>,
    pub emails: Option<Vec<String>>,
    pub addresses: Option<Vec<String>>,
    pub notes: Option<String>,
}

pub async fn create_contact(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateContactRequest>,
) -> Result<HttpResponse, ApiError> {
    let next_pos = data.contact_repo.get_next_position(&user.user_id).await?;
    let mut contact = Contact {
        meta: Default::default(),
        name: body.name.clone(),
        phones: body.phones.clone().unwrap_or_default(),
        emails: body.emails.clone().unwrap_or_default(),
        addresses: body.addresses.clone().unwrap_or_default(),
        notes: body.notes.clone().unwrap_or_default(),
    };
    contact.meta.position = next_pos;
    data.contact_repo.save(&user.user_id, &contact).await?;
    Ok(HttpResponse::Created().json(&contact))
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ContactQuery {
    pub search: Option<String>,
}

pub async fn list_contacts(
    user: AuthUser,
    data: web::Data<AppState>,
    query: web::Query<ContactQuery>,
) -> Result<HttpResponse, ApiError> {
    let items = if let Some(ref q) = query.search {
        data.contact_repo.search(&user.user_id, q).await?
    } else {
        data.contact_repo.find_all(&user.user_id).await?
    };
    Ok(HttpResponse::Ok().json(&items))
}

pub async fn delete_contact(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    data.contact_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateContactRequest {
    pub name: Option<String>,
    pub phones: Option<Vec<String>>,
    pub emails: Option<Vec<String>>,
    pub addresses: Option<Vec<String>>,
    pub notes: Option<String>,
}

pub async fn update_contact(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateContactRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let existing = data.contact_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Contact not found".to_string()))?;

    let updated = Contact {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        name: body.name.clone().unwrap_or(existing.name),
        phones: body.phones.clone().unwrap_or(existing.phones),
        emails: body.emails.clone().unwrap_or(existing.emails),
        addresses: body.addresses.clone().unwrap_or(existing.addresses),
        notes: body.notes.clone().unwrap_or(existing.notes),
    };

    data.contact_repo.save(&user.user_id, &updated).await?;
    Ok(HttpResponse::Ok().json(&updated))
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

pub async fn reorder_contacts(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.contact_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
