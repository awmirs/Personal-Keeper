use actix_web::{web, HttpResponse, Responder};
use domain::models::contact::Contact;
use domain::traits::repository::Repository;
use crate::AppState;

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
    data: web::Data<AppState>,
    body: web::Json<CreateContactRequest>,
) -> impl Responder {
    let next_pos = match data.contact_repo.get_next_position().await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };
    let mut contact = Contact {
        meta: Default::default(),
        name: body.name.clone(),
        phones: body.phones.clone().unwrap_or_default(),
        emails: body.emails.clone().unwrap_or_default(),
        addresses: body.addresses.clone().unwrap_or_default(),
        notes: body.notes.clone().unwrap_or_default(),
    };
    contact.meta.position = next_pos;
    match data.contact_repo.save(&contact).await {
        Ok(()) => HttpResponse::Created().json(&contact),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ContactQuery {
    pub search: Option<String>,
}

pub async fn list_contacts(
    data: web::Data<AppState>,
    query: web::Query<ContactQuery>,
) -> impl Responder {
    let result = if let Some(ref q) = query.search {
        data.contact_repo.search(q).await
    } else {
        data.contact_repo.find_all().await
    };
    match result {
        Ok(items) => HttpResponse::Ok().json(&items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn delete_contact(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match data.contact_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
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
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateContactRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let existing = match data.contact_repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Contact not found" })),
    };

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

    match data.contact_repo.save(&updated).await {
        Ok(()) => HttpResponse::Ok().json(&updated),
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

pub async fn reorder_contacts(
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> impl Responder {
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    match data.contact_repo.update_positions(&positions).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}