use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{query, query_as};
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::AuthUser,
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

pub async fn health_check() -> String {
    "It works!".to_owned()
}

pub async fn get_allergies(
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

pub async fn add_allergy(
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

pub async fn remove_allergy(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(AllergyIDPayload { allergy_id }): Json<AllergyIDPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult =
        query!("DELETE FROM allergies WHERE allergy_id = $1", allergy_id)
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
