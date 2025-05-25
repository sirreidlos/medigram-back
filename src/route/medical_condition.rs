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
    schema::MedicalCondition,
};

pub async fn get_user_conditions(
    State(state): State<AppState>,
    auth: AuthUser,
    doctor: Option<LicensedUser>,
    Path(user_id): Path<Uuid>,
) -> APIResult<Json<Vec<MedicalCondition>>> {
    if user_id != auth.user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

    query_as!(
        MedicalCondition,
        "SELECT * FROM medical_conditions WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => DatabaseError::RowNotFound.into(),
        e => {
            error!("Error while fetching user for {}: {:?}", user_id, e);
            AppError::InternalError
        }
    })
}

pub async fn get_own_conditions(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> APIResult<Json<Vec<MedicalCondition>>> {
    query_as!(
        MedicalCondition,
        "SELECT * FROM medical_conditions WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => DatabaseError::RowNotFound.into(),
        e => {
            error!("Error while fetching user for {}: {:?}", user_id, e);
            AppError::InternalError
        }
    })
}

#[derive(Deserialize)]
pub struct MedicalConditionPayload {
    pub condition: String,
}

pub async fn post_own_conditions(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(MedicalConditionPayload { condition }): Json<MedicalConditionPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let _ = query!(
        "INSERT INTO medical_conditions (user_id, condition) VALUES ($1, $2)",
        user_id,
        condition
    )
    .execute(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while adding medical condition for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "medical condition added" })),
    ))
}

pub async fn delete_own_conditions(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(condition_id): Path<Uuid>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult = query!(
        "DELETE FROM medical_conditions WHERE condition_id = $1 AND user_id = \
         $2",
        condition_id,
        user_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while removing medical condition {} for {}: {:?}",
            condition_id, user_id, e
        );
        AppError::InternalError
    })?;

    if query_res.rows_affected() == 0 {
        // assume it doesnt exist
        return Err(DatabaseError::RowNotFound.into());
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "medical condition removed" })),
    ))
}
