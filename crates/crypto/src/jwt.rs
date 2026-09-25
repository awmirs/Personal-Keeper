use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;
use std::sync::OnceLock;

fn jwt_secret() -> &'static str {
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET.get_or_init(|| {
        match env::var("JWT_SECRET") {
            Ok(val) if !val.trim().is_empty() => val,
            _ => {
                #[cfg(not(debug_assertions))]
                panic!("FATAL: JWT_SECRET environment variable must be set in production mode!");
                #[cfg(debug_assertions)]
                "dev-secret-not-for-production".to_string()
            }
        }
    })
}
const ACCESS_TOKEN_MINUTES: u64 = 15;
const REFRESH_TOKEN_DAYS: u64 = 7;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id
    pub exp: usize,         // expiry (unix timestamp)
    pub iat: usize,         // issued at
    pub token_type: String, // "access" or "refresh"
}

pub fn create_access_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + (ACCESS_TOKEN_MINUTES * 60) as usize,
        iat: now,
        token_type: "access".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_ref()),
    )
}

pub fn create_refresh_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + (REFRESH_TOKEN_DAYS * 24 * 60 * 60) as usize,
        iat: now,
        token_type: "refresh".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_ref()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}