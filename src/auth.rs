use axum::{
    Json, extract::State, handler::HandlerWithoutStateExt, http::StatusCode,
};
use base64::Engine;
use base64::engine::general_purpose;
use ed25519_compact::{KeyPair, PublicKey, Seed};
use rand::{Rng, distr::Alphanumeric, rng};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use tracing::{debug, error, info};
// use tower_http::{cors::{Any, CorsLayer}, limit::RequestBodyLimitLayer};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use uuid::Uuid;

use crate::{
    AppError, AppState,
    jwt::{
        AuthError, AuthResponse, LoginRequest, RefreshRequest, RegisterRequest,
        create_access_token, get_session_id, get_token, verify_token,
    },
    schema::{DeviceKey, User},
};

/// Session ID character length
pub const SESSION_ID_LEN: usize = 64;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Generates a [`SESSION_ID_LEN`] characters long string for `session_id`
fn create_session_id() -> String {
    let session_id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(SESSION_ID_LEN)
        .map(char::from)
        .collect();
    session_id
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
        error!("registering email: {:?}", e);
        AppError::InternalError
    })?;

    info!("Successfully registered email: {}", email);

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "registration successful" })),
    ))
}

async fn query_user(
    email: &str,
    db_pool: &Pool<Postgres>,
) -> Result<User, AppError> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthError::UserNotFound.into(),
            e => {
                error!("Unexpected error while querying for user: {:?}", e);
                AppError::InternalError
            }
        })
}

async fn store_public_key(
    device_id: Uuid,
    user_id: Uuid,
    public_key: PublicKey,
    db_pool: &Pool<Postgres>,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO device_keys (device_id, user_id, public_key_pem, \
         revoked) VALUES ($1, $2, $3, $4)",
        device_id,
        user_id,
        public_key.to_pem(),
        false
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        error!("inserting public key: {:?}", e);
        AppError::InternalError
    })?;

    Ok(())
}

pub async fn retrieve_public_key(
    device_id: Uuid,
    db_pool: &Pool<Postgres>,
) -> Result<Json<DeviceKey>, AppError> {
    sqlx::query_as!(
        DeviceKey,
        "SELECT * FROM device_keys WHERE device_id = $1",
        device_id
    )
    .fetch_one(db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("inserting public key: {:?}", e);
        AppError::InternalError
    })
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

async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // remove the refresh token from the whitelist
    state.recognized_session_id.remove(&payload.refresh_token);
    Ok((StatusCode::OK, Json(json!({ "message": "logged out" }))))
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AuthError> {
    match get_session_id(&headers) {
        Some(session_id) => {
            if let Some(user_id) = state.recognized_session_id.get(session_id) {
                let data = AuthUser { user_id };
                request.extensions_mut().insert(data);
                let response = next.run(request).await;

                Ok(response)
            } else {
                Err(AuthError::InvalidToken)
            }
        }
        _ => Err(AuthError::InvalidToken),
    }
}
