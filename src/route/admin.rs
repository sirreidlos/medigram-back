use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;

pub async fn promote_to_admin(
    State(pool): State<PgPool>,
    Path(target_user_id): Path<Uuid>,
    auth_user: AuthUser,              
) -> Result<StatusCode, (StatusCode, String)> {
    let admin_id = auth_user.user_id;

    // 1. Check if the caller is already an admin
    let is_admin: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM admins WHERE user_id = $1)",
        admin_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .unwrap_or(false);

    if !is_admin {
        return Err((StatusCode::FORBIDDEN, "Not an admin".into()));
    }

    // 2. Insert the new admin record
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::CREATED)
}

pub async fn approve_location(
    State(pool): State<PgPool>,
    Path(location_id): Path<Uuid>,
    auth_user: AuthUser,              
) -> Result<StatusCode, (StatusCode, String)> {
    let admin_id = auth_user.user_id;

    // 1. Verify caller is admin
    let is_admin: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM admins WHERE user_id = $1)",
        admin_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .unwrap_or(false);

    if !is_admin {
        return Err((StatusCode::FORBIDDEN, "Not an admin".into()));
    }

    // 2. Check if location exists
    let exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM doctor_practice_locations WHERE location_id = $1)",
        location_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .unwrap_or(false);

    if !exists {
        return Err((StatusCode::NOT_FOUND, "Location not found".into()));
    }

    // 3. Update approval fields
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}
