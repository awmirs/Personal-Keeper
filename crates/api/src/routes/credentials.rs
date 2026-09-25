use actix_web::{web, HttpResponse};
use domain::models::credential::Credential;
use domain::traits::repository::Repository;
use crypto::vault::{derive_key, encrypt_bytes, decrypt_bytes, MasterKey};
use crypto::hash::{hash_password, verify_password};
use crate::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;

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

pub async fn vault_status(user: AuthUser, data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let configured = data.credential_config_repo.get_master_password(&user.user_id).await?.is_some();
    Ok(HttpResponse::Ok().json(serde_json::json!({ "configured": configured })))
}

pub async fn unlock(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<UnlockRequest>,
) -> Result<HttpResponse, ApiError> {
    let config = match data.credential_config_repo.get_master_password(&user.user_id).await? {
        Some(cfg) => cfg,
        None => {
            let salt: [u8; 32] = rand::random();
            let password_hash = hash_password(&body.master_password)
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            data.credential_config_repo.set_master_password(&user.user_id, &password_hash, &salt).await?;
            let key = derive_key(&body.master_password, &salt);
            data.master_keys.lock().unwrap().insert(user.user_id.clone(), key);
            return Ok(HttpResponse::Ok().json(UnlockResponse { status: "master_password_set".to_string() }));
        }
    };

    if !verify_password(&body.master_password, &config.0).unwrap_or(false) {
        return Err(ApiError::Unauthorized("Invalid master password".to_string()));
    }

    let key = derive_key(&body.master_password, &config.1);
    data.master_keys.lock().unwrap().insert(user.user_id.clone(), key);

    Ok(HttpResponse::Ok().json(UnlockResponse { status: "unlocked".to_string() }))
}

pub async fn lock(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    data.master_keys.lock().unwrap().remove(&user.user_id);
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "locked" })))
}

// ========== CRUD (requires unlock) ==========

fn get_key(user: &AuthUser, data: &web::Data<AppState>) -> Option<MasterKey> {
    let keys = data.master_keys.lock().unwrap();
    keys.get(&user.user_id).cloned()
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
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<CreateCredentialRequest>,
) -> Result<HttpResponse, ApiError> {
    let key = get_key(&user, &data).ok_or_else(|| ApiError::Unauthorized("Vault locked".to_string()))?;
    let next_pos = data.credential_repo.get_next_position(&user.user_id).await?;

    let mut cred = Credential {
        meta: Default::default(),
        website: body.website.clone(),
        url: body.url.clone().unwrap_or_default(),
        username: body.username.clone(),
        password_encrypted: body.password.as_ref().and_then(|p| encrypt_bytes(p.as_bytes(), &key.0).ok()),
        notes_encrypted: body.notes.as_ref().and_then(|n| encrypt_bytes(n.as_bytes(), &key.0).ok()),
        totp_secret_encrypted: body.totp_secret.as_ref().and_then(|t| encrypt_bytes(t.as_bytes(), &key.0).ok()),
    };
    cred.meta.position = next_pos;

    data.credential_repo.save(&user.user_id, &cred).await?;
    Ok(HttpResponse::Created().json(&cred))
}

pub async fn list_credentials(user: AuthUser, data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let creds = data.credential_repo.find_all(&user.user_id).await?;
    Ok(HttpResponse::Ok().json(&creds))
}

pub async fn get_credential(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let key = get_key(&user, &data).ok_or_else(|| ApiError::Unauthorized("Vault locked".to_string()))?;
    let id = path.into_inner();
    let cred = data.credential_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

    let mut resp = serde_json::json!(cred);
    if let Some(ref enc) = cred.password_encrypted {
        if let Ok(dec) = decrypt_bytes(enc, &key.0) {
            resp["password_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
        }
    }
    if let Some(ref enc) = cred.notes_encrypted {
        if let Ok(dec) = decrypt_bytes(enc, &key.0) {
            resp["notes_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
        }
    }
    if let Some(ref enc) = cred.totp_secret_encrypted {
        if let Ok(dec) = decrypt_bytes(enc, &key.0) {
            resp["totp_secret_plain"] = serde_json::Value::String(String::from_utf8_lossy(&dec).to_string());
        }
    }
    Ok(HttpResponse::Ok().json(&resp))
}

pub async fn delete_credential(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    if get_key(&user, &data).is_none() {
        return Err(ApiError::Unauthorized("Vault locked".to_string()));
    }
    data.credential_repo.delete(&user.user_id, &path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_credential(
    user: AuthUser,
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateCredentialRequest>,
) -> Result<HttpResponse, ApiError> {
    let key = get_key(&user, &data).ok_or_else(|| ApiError::Unauthorized("Vault locked".to_string()))?;
    let id = path.into_inner();
    let existing = data.credential_repo.find_by_id(&user.user_id, &id).await?
        .ok_or_else(|| ApiError::NotFound("Credential not found".to_string()))?;

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
            Some(p) if !p.is_empty() => encrypt_bytes(p.as_bytes(), &key.0).ok(),
            _ => existing.password_encrypted,
        },
        notes_encrypted: match &body.notes {
            Some(n) if !n.is_empty() => encrypt_bytes(n.as_bytes(), &key.0).ok(),
            _ => existing.notes_encrypted,
        },
        totp_secret_encrypted: match &body.totp_secret {
            Some(t) if !t.is_empty() => encrypt_bytes(t.as_bytes(), &key.0).ok(),
            _ => existing.totp_secret_encrypted,
        },
    };

    data.credential_repo.save(&user.user_id, &updated).await?;
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

pub async fn reorder_credentials(
    user: AuthUser,
    data: web::Data<AppState>,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, ApiError> {
    if get_key(&user, &data).is_none() {
        return Err(ApiError::Unauthorized("Vault locked".to_string()));
    }
    let positions: Vec<(uuid::Uuid, f64)> = body
        .positions
        .iter()
        .filter_map(|entry| uuid::Uuid::parse_str(&entry.id).ok().map(|id| (id, entry.position)))
        .collect();
    data.credential_repo.update_positions(&user.user_id, &positions).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}
