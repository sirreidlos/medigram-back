use axum::{Extension, Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::error;

use crate::{
    APIResult, AppError, AppState, auth::AuthUser, schema::UserMeasurement,
};

pub async fn get_user_measurements(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
) -> APIResult<Json<Vec<UserMeasurement>>> {
    sqlx::query_as!(
        UserMeasurement,
        "SELECT * FROM user_measurements WHERE user_id = $1",
        user_id
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
    Extension(AuthUser { user_id }): Extension<AuthUser>,
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
