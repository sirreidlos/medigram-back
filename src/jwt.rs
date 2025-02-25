use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
// use tower_http::{cors::{Any, CorsLayer}, limit::RequestBodyLimitLayer};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: String,
    username: String,
    password_hash: String,
}

// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// Authentication response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub struct AuthUser {
    pub user_id: String,
}

// JWT signing keys
pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

// in seconds
static ACCESS_TOKEN_TTL: Duration = Duration::minutes(15);
pub static REFRESH_TOKEN_TTL: Duration = Duration::days(30);

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = "your_jwt_secret_key_here_make_this_random_and_secure";
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Subject (user ID)
    exp: usize,  // Expiration time
    iat: usize,  // Issued at
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

pub enum AuthError {
    InvalidToken,
    ExpiredToken,
    MissingCredentials,
    WrongCredentials,
    TokenCreation,
    TokenBlacklisted,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token")
            }
            AuthError::ExpiredToken => {
                (StatusCode::UNAUTHORIZED, "Token expired")
            }
            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid username or password")
            }
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
            }
            AuthError::TokenBlacklisted => {
                (StatusCode::UNAUTHORIZED, "Token has been revoked")
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub fn create_access_token(user_id: &str) -> Result<(String, i64), AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(ACCESS_TOKEN_TTL)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
        refresh_token_id: None,
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    Ok((token, 15 * 60)) // 15 minutes in seconds
}

pub fn create_refresh_token(user_id: &str) -> Result<String, AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(REFRESH_TOKEN_TTL)
        .expect("valid timestamp")
        .timestamp();

    let refresh_token_id = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
        refresh_token_id: Some(refresh_token_id),
    };

    encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Check if token is blacklisted
    if state.blacklist.contains_key(&payload.refresh_token) {
        return Err(AuthError::TokenBlacklisted);
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
        return Err(AuthError::InvalidToken);
    }

    // Create new tokens
    let user_id = token_data.claims.sub;
    let (access_token, expires_in) = create_access_token(&user_id)?;
    let refresh_token = create_refresh_token(&user_id)?;

    // Return the new tokens
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in,
    }))
}
