use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use base64::{Engine, engine::general_purpose};
use ed25519_compact::{KeyPair, Seed};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::AppState;
use crate::auth::jwt::create_access_token;
use crate::auth::{
    AuthError, AuthResponse, create_session_id, query_user, store_public_key,
};
use crate::error::AppError;
use crate::schema::User;

// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Find the user
    let email = payload.email;
    let password = payload.password;
    let user: User = query_user(&email, &state.db_pool).await?;

    // verify user
    let password_hash_str: String = user.password_hash;
    let password_hash: PasswordHash =
        match PasswordHash::new(&password_hash_str) {
            Ok(h) => h,
            Err(_) => {
                error!("Password hash cannot be parsed");
                debug!("Hash: {:?}", password_hash_str);
                return Err(AppError::InternalError);
            }
        };

    if Argon2::default()
        .verify_password(password.as_bytes(), &password_hash)
        .is_err()
    {
        return Err(AuthError::WrongCredentials.into());
    }

    // create tokens
    let user_id = user.user_id;
    let (access_token, expires_in) = create_access_token(&user_id.to_string())?;
    let session_id = create_session_id();
    let device_id = Uuid::new_v4();
    let key_pair = KeyPair::from_seed(Seed::generate());

    let private_key = general_purpose::STANDARD.encode(key_pair.sk.to_vec());

    store_public_key(device_id, user_id, key_pair.pk, &state.db_pool).await?;

    state
        .recognized_session_id
        .insert(session_id.clone(), user.user_id);

    info!("User {email} logged in");

    // Return the tokens
    Ok(Json(AuthResponse {
        access_token,
        session_id,
        token_type: "Bearer".to_string(),
        expires_in,
        device_id,
        private_key,
    }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let email = payload.email;
    let password = payload.password;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            error!("error occured while hasing password: {:?}", e);
            AppError::InternalError
        })?
        .to_string();

    sqlx::query!(
        "INSERT INTO users(email, password_hash) VALUES ($1, $2)",
        email,
        password_hash
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("error occured while registering email: {:?}", e);

        match e {
            sqlx::Error::Database(db_e) => {
                if db_e.is_unique_violation() {
                    AuthError::EmailUsed.into()
                } else {
                    AppError::InternalError
                }
            }
            _ => AppError::InternalError,
        }
    })?;

    info!("Successfully registered email: {}", email);

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "registration successful" })),
    ))
}
