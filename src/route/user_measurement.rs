use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, LicensedUser},
    error::{APIResult, AppError},
    schema::UserMeasurement,
};

pub async fn get_user_measurements(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    doctor: Option<LicensedUser>,
    Path(user_id_query): Path<Uuid>,
) -> APIResult<Json<Vec<UserMeasurement>>> {
    if user_id_query != user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

    sqlx::query_as!(
        UserMeasurement,
        "SELECT * FROM user_measurements WHERE user_id = $1",
        user_id_query
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while fetching user_measurements for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })
}

#[derive(Deserialize)]
pub struct UserMeasurementPayload {
    pub height_in_cm: f32,
    pub weight_in_kg: f32,
    pub measured_at: Option<DateTime<Utc>>,
}

pub async fn add_user_measurement(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(UserMeasurementPayload {
        height_in_cm,
        weight_in_kg,
        measured_at,
    }): Json<UserMeasurementPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let _ = sqlx::query!(
        "INSERT INTO user_measurements (user_id, height_in_cm, weight_in_kg, \
         measured_at) VALUES ($1, $2, $3, $4)",
        user_id,
        height_in_cm,
        weight_in_kg,
        measured_at.unwrap_or(Utc::now())
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while inserting user_measurements for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "Successfully added user measurement"})),
    ))
}
