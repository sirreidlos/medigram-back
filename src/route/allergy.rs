use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{query, query_as};
use tracing::error;

use crate::{APIResult, AppError, AppState, auth::AuthUser, schema::Allergy};

#[derive(Deserialize)]
pub struct AllergyPayload {
    pub allergy: String,
}

pub async fn get_allergies(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> APIResult<Json<Vec<Allergy>>> {
    query_as!(
        Allergy,
        "SELECT * FROM allergies WHERE user_id = $1",
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
    AuthUser { user_id }: AuthUser,
    Json(AllergyPayload { allergy }): Json<AllergyPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let _ = query!(
        "INSERT INTO allergies (user_id, allergy) VALUES ($1, $2)",
        user_id,
        allergy
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
    AuthUser { user_id }: AuthUser,
    Json(Allergy { allergy_id, .. }): Json<Allergy>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult =
        query!("DELETE FROM allergies WHERE allergy_id = $1", allergy_id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                error!("Error while adding allergy for {}: {:?}", user_id, e);
                AppError::InternalError
            })?;

    if query_res.rows_affected() == 0 {
        // assume it doesnt exist
        return Err(AppError::RowNotFound);
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "allergy removed" })),
    ))
}
