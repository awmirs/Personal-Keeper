use actix_web::{web, HttpResponse, Responder};
use domain::models::credential::Credential;
use domain::traits::repository::Repository;
use crypto::vault::{derive_key, encrypt_bytes, decrypt_bytes};
use crypto::hash::{hash_password, verify_password};
use crate::AppState;

// ========== Unlock/Lock ==========

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UnlockRequest {
    master_password: String,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct UnlockResponse {
    status: String,
}

pub async fn vault_status(data: web::Data<AppState>) -> impl Responder {
    match data.credential_config_repo.get_master_password().await {
        Ok(Some(_)) => HttpResponse::Ok().json(serde_json::json!({ "configured": true })),
        Ok(None) => HttpResponse::Ok().json(serde_json::json!({ "configured": false })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// First-time setup or unlock. If no master password is set, this creates it.
pub async fn unlock(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<UnlockRequest>,
) -> impl Responder {
    use actix_web::HttpMessage;
    let claims = match req.extensions().get::<crypto::jwt::Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Not authenticated" })),
    };

    let config = match data.credential_config_repo.get_master_password().await {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            // First time: generate salt, hash password, store, derive key
            let salt: [u8; 32] = rand::random();
            let password_hash = match hash_password(&body.master_password) {
                Ok(h) => h,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            // Store
            if let Err(e) = data.credential_config_repo.set_master_password(&password_hash, &salt).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }));
            }
            // Derive key
            let key = derive_key(&body.master_password, &salt);
            data.master_keys.lock().unwrap().insert(claims.sub, key);
            return HttpResponse::Ok().json(UnlockResponse { status: "master_password_set".to_string() });
        }
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    // Existing master password: verify
    if !verify_password(&body.master_password, &config.0).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid master password" }));
    }

    let key = derive_key(&body.master_password, &config.1);
    data.master_keys.lock().unwrap().insert(claims.sub, key);

    HttpResponse::Ok().json(UnlockResponse { status: "unlocked".to_string() })
}

pub async fn lock(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    use actix_web::HttpMessage;
    if let Some(claims) = req.extensions().get::<crypto::jwt::Claims>() {
        data.master_keys.lock().unwrap().remove(&claims.sub);
    }
    HttpResponse::Ok().json(serde_json::json!({ "status": "locked" }))
}

// ========== CRUD (requires unlock) ==========

fn get_key(req: &actix_web::HttpRequest, data: &web::Data<AppState>) -> Option<[u8; 32]> {
    use actix_web::HttpMessage;
    let claims = req.extensions().get::<crypto::jwt::Claims>().cloned()?;
    let keys = data.master_keys.lock().unwrap();
    keys.get(&claims.sub).cloned()
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct CreateCredentialRequest {
    pub website: String,
    pub url: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub notes: Option<String>,
    pub totp_secret: Option<String>,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct UpdateCredentialRequest {
    pub website: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub notes: Option<String>,
    pub totp_secret: Option<String>,
}

pub async fn create_credential(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<CreateCredentialRequest>,
) -> impl Responder {
    let key = match get_key(&req, &data) {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Vault locked" })),
    };

    let next_pos = match data.credential_repo.get_next_position().await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    let mut cred = Credential {
        meta: Default::default(),
        website: body.website.clone(),
        url: body.url.clone().unwrap_or_default(),
        username: body.username.clone(),
        password_encrypted: body.password.as_ref().and_then(|p| encrypt_bytes(p.as_bytes(), &key).ok()),
        notes_encrypted: body.notes.as_ref().and_then(|n| encrypt_bytes(n.as_bytes(), &key).ok()),
        totp_secret_encrypted: body.totp_secret.as_ref().and_then(|t| encrypt_bytes(t.as_bytes(), &key).ok()),
    };
    cred.meta.position = next_pos;

    match data.credential_repo.save(&cred).await {
        Ok(()) => HttpResponse::Created().json(&cred),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn list_credentials(data: web::Data<AppState>) -> impl Responder {
    // Always returns without decrypted secrets
    match data.credential_repo.find_all().await {
        Ok(creds) => HttpResponse::Ok().json(&creds),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn get_credential(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let key = match get_key(&req, &data) {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Vault locked" })),
    };
    let id = path.into_inner();
    match data.credential_repo.find_by_id(&id).await {
        Ok(Some(cred)) => {
            // Construct the JSON manually, adding plain fields.
            let mut resp = serde_json::json!(cred);
            if let Some(ref enc) = cred.password_encrypted {
                if let Ok(dec) = decrypt_bytes(enc, &key) {
                    resp["password_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
                }
            }
            if let Some(ref enc) = cred.notes_encrypted {
                if let Ok(dec) = decrypt_bytes(enc, &key) {
                    resp["notes_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
                }
            }
            if let Some(ref enc) = cred.totp_secret_encrypted {
                if let Ok(dec) = decrypt_bytes(enc, &key) {
                    resp["totp_secret_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
                }
            }
            HttpResponse::Ok().json(&resp)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Credential not found" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn delete_credential(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    if get_key(&req, &data).is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Vault locked" }));
    }
    match data.credential_repo.delete(&path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn update_credential(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateCredentialRequest>,
) -> impl Responder {
    let key = match get_key(&req, &data) {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Vault locked" })),
    };
    let id = path.into_inner();
    let existing = match data.credential_repo.find_by_id(&id).await {
        Ok(Some(c)) => c,
        _ => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Credential not found" })),
    };

    let updated = Credential {
        meta: domain::models::common::ItemMetadata {
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            ..existing.meta
        },
        website: body.website.clone().unwrap_or(existing.website),
        url: body.url.clone().unwrap_or(existing.url),
        username: body.username.clone().unwrap_or(existing.username),
        password_encrypted: match &body.password {
            Some(p) if !p.is_empty() => encrypt_bytes(p.as_bytes(), &key).ok(),
            _ => existing.password_encrypted,
        },
        notes_encrypted: match &body.notes {
            Some(n) if !n.is_empty() => encrypt_bytes(n.as_bytes(), &key).ok(),
            _ => existing.notes_encrypted,
        },
        totp_secret_encrypted: match &body.totp_secret {
            Some(t) if !t.is_empty() => encrypt_bytes(t.as_bytes(), &key).ok(),
            _ => existing.totp_secret_encrypted,
        },
    };

    match data.credential_repo.save(&updated).await {
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

pub async fn reorder_credentials(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> impl Responder {
    if get_key(&req, &data).is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Vault locked" }));
    }
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    match data.credential_repo.update_positions(&positions).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}