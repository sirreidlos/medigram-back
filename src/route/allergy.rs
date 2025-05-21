use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{query, query_as};
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, LicensedUser},
    error::{APIResult, AppError, DatabaseError},
    schema::{Allergy, AllergySeverity},
};

#[derive(Deserialize)]
pub struct AllergyPayload {
    pub allergen: String,
    pub severity: AllergySeverity,
}

#[derive(Deserialize)]
pub struct AllergyIDPayload {
    pub allergy_id: Uuid,
}

pub async fn get_user_allergies(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    doctor: Option<LicensedUser>,
    Path(user_id_query): Path<Uuid>,
) -> APIResult<Json<Vec<Allergy>>> {
    if user_id_query != user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

    query_as!(
        Allergy,
        "SELECT allergy_id, user_id, allergen, severity AS \"severity: \
         AllergySeverity\" FROM allergies WHERE user_id = $1",
        user_id_query
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while retrieving allergies for {}: {:?}", user_id, e);
        AppError::InternalError
    })
}

pub async fn get_own_allergies(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> APIResult<Json<Vec<Allergy>>> {
    query_as!(
        Allergy,
        "SELECT allergy_id, user_id, allergen, severity AS \"severity: \
         AllergySeverity\" FROM allergies WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while retrieving allergies for {}: {:?}", user_id, e);
        AppError::InternalError
    })
}

pub async fn add_own_allergy(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(AllergyPayload { allergen, severity }): Json<AllergyPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let _ = query!(
        "INSERT INTO allergies (user_id, allergen, severity) VALUES ($1, $2, \
         $3)",
        user_id,
        allergen,
        severity as AllergySeverity
    )
    .execute(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while adding allergy for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "allergy added" })),
    ))
}

pub async fn remove_own_allergy(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(allergy_id): Path<Uuid>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult = query!(
        "DELETE FROM allergies WHERE allergy_id = $1 AND user_id = $2",
        allergy_id,
        user_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while removing allergy {} for {}: {:?}",
            allergy_id, user_id, e
        );
        AppError::InternalError
    })?;

    if query_res.rows_affected() == 0 {
        // assume it doesnt exist
        return Err(DatabaseError::RowNotFound.into());
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "allergy removed" })),
    ))
}
