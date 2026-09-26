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

// ----- Bulk import (full-fidelity round-trip of exported data) -----

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct ImportContactsRequest {
    pub items: Vec<serde_json::Value>,
    pub strategy: ImportStrategy,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportStrategy {
    Skip,
    Replace,
    Copy,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub created: usize,
    pub replaced: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Fills in server-side defaults for fields that are missing or empty in
/// partially-sourced items (e.g. hand-written CSV rows): a fresh id, current
/// timestamps, the next free ordering position and sensible values for
/// optional, list and boolean fields.
fn fill_import_defaults(mut value: serde_json::Value, next_pos: &mut f64) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    if let serde_json::Value::Object(ref mut map) = value {
        map.entry("id")
            .or_insert_with(|| serde_json::json!(uuid::Uuid::now_v7().to_string()));
        map.entry("created_at").or_insert_with(|| serde_json::json!(now));
        map.entry("updated_at").or_insert_with(|| serde_json::json!(now));
        map.entry("is_favorite").or_insert_with(|| serde_json::json!(false));
        map.entry("trash_status").or_insert_with(|| serde_json::json!("Active"));
        map.entry("tags").or_insert_with(|| serde_json::json!([]));
        map.entry("phones").or_insert_with(|| serde_json::json!([]));
        map.entry("emails").or_insert_with(|| serde_json::json!([]));
        map.entry("addresses").or_insert_with(|| serde_json::json!([]));
        map.entry("notes").or_insert_with(|| serde_json::json!(""));
        if !map.contains_key("position") {
            let position = *next_pos;
            *next_pos += 1.0;
            map.insert("position".to_string(), serde_json::json!(position));
        }
        // Empty CSV cells arrive as ""; normalize them for fields that are
        // not plain strings.
        for key in ["tags", "phones", "emails", "addresses"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::json!([]));
            }
        }
        for key in ["is_favorite"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::json!(false));
            }
        }
        for key in ["color"] {
            if map.get(key) == Some(&serde_json::Value::String(String::new())) {
                map.insert(key.to_string(), serde_json::Value::Null);
            }
        }
    }
    value
}

/// Bulk import with full fidelity: every item is deserialized into a
/// complete model (ids, tags, colors, favorites, timestamps and ordering
/// are all restored) and upserted through the repository's save(). The
/// strategy decides what happens to items whose id already exists in the
/// vault: skip keeps the existing item, replace overwrites it in place,
/// and copy re-creates it with a fresh id appended after the last item.
pub async fn import_contacts(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ImportContactsRequest>,
) -> Result<HttpResponse, ApiError> {
    let ImportContactsRequest { items, strategy } = body.into_inner();
    let existing = data.contact_repo.find_all(&user.user_id).await?;
    let mut known_ids: std::collections::HashSet<uuid::Uuid> =
        existing.iter().map(|contact| contact.meta.id).collect();
    let mut next_pos = data.contact_repo.get_next_position(&user.user_id).await?;
    let mut summary = ImportSummary {
        created: 0,
        replaced: 0,
        skipped: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for (index, value) in items.into_iter().enumerate() {
        let value = fill_import_defaults(value, &mut next_pos);
        let mut contact: Contact = match serde_json::from_value(value) {
            Ok(contact) => contact,
            Err(err) => {
                summary.failed += 1;
                summary.errors.push(format!("Item #{}: invalid data ({})", index + 1, err));
                continue;
            }
        };

        let exists = known_ids.contains(&contact.meta.id);
        if exists {
            match strategy {
                ImportStrategy::Skip => {
                    summary.skipped += 1;
                    continue;
                }
                ImportStrategy::Copy => {
                    contact.meta.id = uuid::Uuid::now_v7();
                    contact.meta.position = next_pos;
                    next_pos += 1.0;
                }
                ImportStrategy::Replace => {}
            }
        }

        if let Err(err) = data.contact_repo.save(&user.user_id, &contact).await {
            summary.failed += 1;
            summary.errors.push(format!("Item #{}: storage error ({})", index + 1, err));
            continue;
        }

        if exists && strategy == ImportStrategy::Replace {
            summary.replaced += 1;
        } else {
            summary.created += 1;
        }
        known_ids.insert(contact.meta.id);
    }

    Ok(HttpResponse::Ok().json(&summary))
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
