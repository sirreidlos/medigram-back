use axum::{
    Json,
    extract::{Path, State},
};
use sqlx::query_as;
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
