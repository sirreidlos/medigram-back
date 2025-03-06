use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, query, query_as};
use tracing::error;
use uuid::Uuid;

use crate::{
    APIResult, AppError, AppState, auth::AuthUser, schema::DoctorProfile,
};

#[derive(Deserialize)]
pub struct DoctorProfilePayload {
    pub practice_permit: String,
    pub practice_address: String,
}

#[derive(Deserialize)]
pub struct DoctorId {
    pub doctor_id: Uuid,
}

pub async fn get_doctor_profile(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(DoctorId { doctor_id }): Json<DoctorId>,
) -> APIResult<Json<DoctorProfile>> {
    query_as!(
        DoctorProfile,
        "SELECT * FROM doctor_profiles WHERE doctor_id = $1",
        doctor_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while fetching doctor_profile for {}: {:?}",
            doctor_id, e
        );
        AppError::InternalError
    })
}

pub async fn set_doctor_profile(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(DoctorProfilePayload {
        practice_permit,
        practice_address,
    }): Json<DoctorProfilePayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    query!(
        "INSERT INTO doctor_profiles (user_id, practice_permit, \
         practice_address, approved) VALUES ($1, $2, $3, $4)",
        user_id,
        practice_permit,
        practice_address,
        false
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while inserting doctor_profile for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Successfully submitted your application" })),
    ))
}
