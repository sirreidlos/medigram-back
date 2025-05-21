use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{query, query_as};
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::AuthUser,
    error::{APIResult, AppError},
    schema::Purchase,
};

pub async fn get_own_purchases(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> APIResult<Json<Vec<Purchase>>> {
    query_as!(
        Purchase,
        "SELECT * FROM purchases WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while retrieving purchases for {}: {:?}", user_id, e);
        AppError::InternalError
    })
}

#[derive(Deserialize)]
pub struct PurchasePayload {
    medicine_id: Uuid,
    quantity: i32,
}

pub async fn add_own_purchase(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(PurchasePayload {
        medicine_id,
        quantity,
    }): Json<PurchasePayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    query!(
        "INSERT INTO purchases (user_id, medicine_id, quantity) VALUES ($1, \
         $2, $3)",
        user_id,
        medicine_id,
        quantity
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while retrieving purchases for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "purchase added" })),
    ))
}
