use axum::{Json, extract::State, http::StatusCode};
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{Pool, Postgres};
use tracing::{debug, error, info};
// use tower_http::{cors::{Any, CorsLayer}, limit::RequestBodyLimitLayer};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::{
    AppError, AppState,
    jwt::{
        AuthError, AuthResponse, LoginRequest, RefreshRequest, RegisterRequest,
        create_access_token, get_token, verify_token,
    },
    schema::User,
};

/// Session ID character length
pub const SESSION_ID_LEN: usize = 64;

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
) -> Result<(StatusCode, String), AppError> {
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

    Ok((StatusCode::CREATED, "successfully registered".into()))
}

async fn query_user(
    email: &str,
    db_pool: Pool<Postgres>,
) -> Result<User, AppError> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(&db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthError::UserNotFound.into(),
            e => {
                error!("Unexpected error while querying for user: {:?}", e);
                AppError::InternalError
            }
        })
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Find the user
    let email = payload.email;
    let password = payload.password;
    let user: User = query_user(&email, state.db_pool).await?;

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
    let user_id = user.user_id.to_string();
    let (access_token, expires_in) = create_access_token(&user_id)?;
    let session_id = create_session_id();

    state
        .recognized_session_id
        .insert(session_id.clone(), user.user_id);

    // Return the tokens
    Ok(Json(AuthResponse {
        access_token,
        session_id,
        token_type: "Bearer".to_string(),
        expires_in,
    }))
}

async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<StatusCode, AppError> {
    // remove the refresh token from the whitelist
    state.recognized_session_id.remove(&payload.refresh_token);
    Ok(StatusCode::OK)
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AuthError> {
    match get_token(&headers) {
        Some(session_id)
            if state.recognized_session_id.contains_key(session_id) =>
        {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AuthError::InvalidToken),
    }
}
