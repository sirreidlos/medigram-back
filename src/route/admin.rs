use axum::{
    Json, RequestPartsExt,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{self, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};
use chrono::Utc;
use moka::sync::Cache;
use serde_json::{Value, json};
use sqlx::{PgPool, Pool, Postgres};
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::AuthError,
    error::{APIResult, AppError, DatabaseError},
};

pub async fn promote_to_admin(
    State(pool): State<PgPool>,
    Path(target_user_id): Path<Uuid>,
    admin_user: AdminUser,
) -> APIResult<(StatusCode, Json<Value>)> {
    let admin_id = admin_user.user_id;

    sqlx::query!(
        r#"
        INSERT INTO admins (user_id, promoted_by, promoted_at)
        VALUES ($1, $2, $3)
        "#,
        target_user_id,
        admin_id,
        Utc::now()
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Error while promoting {target_user_id} to admin: {e:?}");

        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "User promoted to admin" })),
    ))
}

pub async fn approve_location(
    State(pool): State<PgPool>,
    Path(location_id): Path<Uuid>,
    admin_user: AdminUser,
) -> APIResult<(StatusCode, Json<Value>)> {
    let admin_id = admin_user.user_id;

    let exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM doctor_practice_locations WHERE \
         location_id = $1)",
        location_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!(
            "Error while searching for location {location_id} to approve: \
             {e:?}"
        );

        AppError::InternalError
    })?
    .unwrap_or(false);

    if !exists {
        return Err(DatabaseError::RowNotFound.into());
    }

    sqlx::query!(
        r#"
        UPDATE doctor_practice_locations
        SET approved_by = $1, approved_at = $2
        WHERE location_id = $3
        "#,
        admin_id,
        Utc::now(),
        location_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Error while trying to approve location {location_id}: {e:?}");

        AppError::InternalError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Practice location approved" })),
    ))
}

#[derive(Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
    pub session_id: String,
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Cache<String, Uuid>: FromRef<S>,
    Pool<Postgres>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let recognized_session_id = Cache::<String, Uuid>::from_ref(state);
        let pool = Pool::<Postgres>::from_ref(state);

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

        let Some(admin_id) = recognized_session_id.get(session_id) else {
            return Err(AuthError::InvalidToken.into());
        };

        let is_admin: bool = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM admins WHERE user_id = $1)",
            admin_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("Error while checking if {admin_id} is admin: {e:?}");

            AppError::InternalError
        })?
        .unwrap_or(false);

        if !is_admin {
            return Err(AppError::NotAdmin);
        }

        Ok(AdminUser {
            user_id: admin_id,
            session_id: session_id.to_string(),
        })
    }
}
