use actix_web::{web, HttpMessage, HttpResponse, Responder};
use crypto::hash::{hash_password, verify_password};
use crypto::jwt::{create_access_token, create_refresh_token, verify_token};
use crypto::jwt::Claims;

use crate::AppState;

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Serialize)]
pub struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

pub async fn register(
    data: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> impl Responder {
    if body.username.is_empty() || body.password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Username must not be empty and password must be at least 8 characters" }));
    }

    let password_hash = match hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match data.user_repo.create_user(&body.username, &password_hash).await {
        Ok(user) => {
            let access = create_access_token(&user.id).unwrap();
            let refresh = create_refresh_token(&user.id).unwrap();
            // Store refresh token
            let expires_at = chrono::Utc::now().timestamp() + 7 * 24 * 3600;
            let _ = data.user_repo.store_refresh_token(&user.id, &refresh, expires_at).await;
            HttpResponse::Created().json(AuthResponse {
                access_token: access,
                refresh_token: refresh,
            })
        }
        Err(e) => HttpResponse::Conflict()
            .json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    data: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let user = match data.user_repo.find_by_username(&body.username).await {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials" })),
    };

    match verify_password(&body.password, &user.password_hash) {
        Ok(true) => {
            let access = create_access_token(&user.id).unwrap();
            let refresh = create_refresh_token(&user.id).unwrap();
            let expires_at = chrono::Utc::now().timestamp() + 7 * 24 * 3600;
            let _ = data.user_repo.store_refresh_token(&user.id, &refresh, expires_at).await;
            HttpResponse::Ok().json(AuthResponse {
                access_token: access,
                refresh_token: refresh,
            })
        }
        _ => HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials" })),
    }
}

#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

pub async fn refresh(
    data: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> impl Responder {
    let claims = match verify_token(&body.refresh_token) {
        Ok(c) if c.token_type == "refresh" => c,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid refresh token" })),
    };

    // Check DB for token existence (optional but secure)
    let stored = data.user_repo.find_by_refresh_token(&body.refresh_token).await;
    if stored.is_err() || stored.unwrap().is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Token not recognized" }));
    }

    // Rotate: delete old, issue new
    let _ = data.user_repo.delete_refresh_token(&body.refresh_token).await;

    let access = create_access_token(&claims.sub).unwrap();
    let refresh = create_refresh_token(&claims.sub).unwrap();
    let expires_at = chrono::Utc::now().timestamp() + 7 * 24 * 3600;
    let _ = data.user_repo.store_refresh_token(&claims.sub, &refresh, expires_at).await;

    HttpResponse::Ok().json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
    })
}

pub async fn me(
    _data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let claims = req.extensions().get::<Claims>().cloned();
    match claims {
        Some(c) => HttpResponse::Ok().json(serde_json::json!({
            "user_id": c.sub,
            "username": ""  // We don't store username in claims; could look it up later
        })),
        None => HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Not authenticated" })),
    }
}