use axum::{Extension, Json, extract::State, http::StatusCode};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, query};
use tracing::{error, trace};

use crate::{
    APIResult, AppError, AppState,
    auth::AuthUser,
    protocol::{NIK_LOWERBOUND, NIK_UPPERBOUND, Nik},
    schema::UserDetail,
};

#[derive(Debug, Deserialize)]
pub struct UserDetailPayload {
    pub nik: Nik,
    pub name: String,
    pub dob: NaiveDate,
    pub gender: char,
}

pub async fn get_user_detail(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> APIResult<Json<UserDetail>> {
    let row = sqlx::query!(
        "SELECT user_id, nik, name, dob, gender FROM user_details WHERE \
         user_id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while fetching user_detail for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok(Json(UserDetail {
        user_id: row.user_id,
        nik: row.nik,
        name: row.name,
        dob: row.dob,
        gender: row.gender.chars().next().unwrap_or('U'),
    }))
}

pub async fn set_user_detail(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<UserDetailPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    trace!(
        "set_user_details\nuser_id: {}\npayload: {:?}",
        user_id, payload
    );
    if !(NIK_LOWERBOUND..=NIK_UPPERBOUND).contains(&payload.nik) {
        return Err(AppError::InvalidNik);
    }

    query!(
        "INSERT INTO user_details (user_id, nik, name, dob, gender) VALUES \
         ($1, $2, $3, $4, $5)",
        user_id,
        payload.nik,
        payload.name,
        payload.dob,
        payload.gender as i8,
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while setting user_detail for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "Successfully set user detail"})),
    ))
}
