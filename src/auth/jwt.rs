use crate::auth::AuthError;
use crate::{AppState, auth::AuthResponse, error::AppError};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// JWT signing keys
pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

// in seconds
static ACCESS_TOKEN_TTL: Duration = Duration::minutes(15);
pub static SESSION_TTL: Duration = Duration::days(30);

static KEYS: Lazy<Keys> = Lazy::new(|| {
    // TODO set key properly
    let secret = "your_jwt_secret_key_here_make_this_random_and_secure";
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Subject (user ID)
    exp: i64,    // Expiration time
    iat: i64,    // Issued at
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token_id: Option<String>, // For refresh tokens, to allow revocation
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn get_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer "))
}

pub fn get_session_id(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer "))
}

pub fn verify_token(token: &str) -> Result<(), AuthError> {
    decode::<Claims>(token, &KEYS.decoding, &Validation::default())
        .map(|_| ())
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AuthError::ExpiredToken
            }
            _ => AuthError::InvalidToken,
        })
}

pub fn create_access_token(user_id: &str) -> Result<(String, i64), AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(ACCESS_TOKEN_TTL)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        refresh_token_id: None,
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    Ok((token, 15 * 60)) // 15 minutes in seconds
}

pub async fn refresh_tokens(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Check if token is blacklisted
    if state
        .recognized_session_id
        .contains_key(&payload.refresh_token)
    {
        return Err(AuthError::TokenBlacklisted.into());
    }

    // Decode and validate the refresh token
    let token_data = decode::<Claims>(
        &payload.refresh_token,
        &KEYS.decoding,
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AuthError::ExpiredToken
        }
        _ => AuthError::InvalidToken,
    })?;

    // Ensure it's a refresh token
    if token_data.claims.refresh_token_id.is_none() {
        return Err(AuthError::InvalidToken.into());
    }

    // Create new tokens
    let user_id = token_data.claims.sub;
    let (access_token, expires_in) = create_access_token(&user_id)?;
    // let refresh_token = create_refresh_token(&user_id)?;

    todo!();
    // Return the new tokens
    // Ok(Json(AuthResponse {
    //     access_token,
    //     session_id: refresh_token,
    //     token_type: "Bearer".to_string(),
    //     expires_in,
    // }))
}

pub async fn revoke_user_access(
    State(state): State<AppState>,
) -> Result<(StatusCode, String), AppError> {
    // TODO
    // - add payload for the key and user_id to revoke
    // - get all device id and remove from state cache
    //
    todo!()
}
