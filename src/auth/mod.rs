use axum::{
    Json, RequestPartsExt,
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{self, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};
use ed25519_compact::PublicKey;
use moka::sync::Cache;
use rand::{Rng, distr::Alphanumeric, rng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, query, query_as};
use tracing::error;
use uuid::Uuid;

pub mod email;
pub mod jwt;

use crate::{
    AppState,
    error::AppError,
    schema::{DeviceKey, DoctorProfile, User},
};

/// Session ID character length
pub const SESSION_ID_LEN: usize = 64;

// Authentication response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub session_id: String,
    pub token_type: String,
    pub device_id: Uuid,
    // base 64
    pub private_key: String,
}

pub enum AuthError {
    InvalidToken,
    ExpiredToken,
    MissingCredentials,
    WrongCredentials,
    TokenCreation,
    TokenBlacklisted,
    UserNotFound,
    EmailUsed,
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
            AuthError::UserNotFound => {
                (StatusCode::NOT_FOUND, "User not found")
            }
            AuthError::EmailUsed => {
                (StatusCode::CONFLICT, "Email has been registered previously")
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Cache<String, Uuid>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let recognized_session_id = Cache::<String, Uuid>::from_ref(state);

        // get session_id
        let authorization_header = parts
            .extract::<TypedHeader<headers::Authorization<Bearer>>>()
            .await
            .map_err(|e| {
                error!("Error while extracting authorization header: {:?}", e);

                match e.reason() {
                    TypedHeaderRejectionReason::Missing => {
                        AuthError::MissingCredentials.into()
                    }
                    TypedHeaderRejectionReason::Error(_) => {
                        AppError::InternalError
                    }
                    _ => AppError::InternalError,
                }
            })?;

        let session_id = authorization_header.token();

        match recognized_session_id.get(session_id) {
            Some(user_id) => Ok(AuthUser {
                user_id,
                session_id: session_id.to_string(),
            }),
            None => Err(AuthError::InvalidToken.into()),
        }
    }
}

#[derive(Clone)]
pub struct LicensedUser {
    pub doctor_id: Uuid,
}

impl<S> OptionalFromRequestParts<S> for LicensedUser
where
    S: Send + Sync,
    Pool<Postgres>: FromRef<S>,
    Cache<String, Uuid>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let db = Pool::<Postgres>::from_ref(state);

        let auth = AuthUser::from_request_parts(parts, state).await?;
        let doctor_user_id = auth.user_id;

        let doctor_profile = match query_as!(
            DoctorProfile,
            "SELECT * FROM doctor_profiles WHERE user_id = $1",
            doctor_user_id
        )
        .fetch_one(&db)
        .await
        {
            Ok(profile) => profile,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(e) => {
                error!(
                    "Error occured while fetching for doctor profile: {:?}",
                    e
                );
                return Err(AppError::InternalError);
            }
        };

        if doctor_profile.approved_at.is_none() {
            return Ok(None);
        }

        let doctor_id = doctor_profile.doctor_id;

        Ok(Some(LicensedUser { doctor_id }))
    }
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
        "INSERT INTO device_keys (device_id, user_id, public_key_pem) VALUES \
         ($1, $2, $3)",
        device_id,
        user_id,
        public_key.to_pem(),
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

#[derive(Deserialize)]
pub struct DeviceIDPayload {
    device_id: Uuid,
}

pub async fn logout(
    State(state): State<AppState>,
    AuthUser { session_id, .. }: AuthUser,
    Json(DeviceIDPayload { device_id }): Json<DeviceIDPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // remove the refresh token from the whitelist
    state.recognized_session_id.remove(&session_id);
    query!(
        "UPDATE device_keys SET revoked_at = $1 WHERE device_id = $2",
        chrono::Utc::now(),
        device_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("error while revoking device {}: {:?}", device_id, e);
        AppError::InternalError
    })?;

    Ok((StatusCode::OK, Json(json!({ "message": "logged out" }))))
}
