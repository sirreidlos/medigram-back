use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use sqlx::query_as;
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, LicensedUser},
    error::{APIResult, AppError, DatabaseError},
};

#[derive(Serialize)]
pub struct UserOpaque {
    user_id: Uuid,
    email: String,
}

// TODO: is this meant for doctors to see the patient info?
// in that case, change the UserOpaque to give the name and NIK
// and also make sure that only licensed users can request for this
// perhaps also make sure that they're connected?
pub async fn get_user(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    doctor: Option<LicensedUser>,
    Path(user_id_query): Path<Uuid>,
) -> APIResult<Json<UserOpaque>> {
    if user_id_query != user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

    query_as!(
        UserOpaque,
        "SELECT user_id, email FROM users WHERE user_id = $1",
        user_id_query
    )
    .fetch_one(&state.db_pool)
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
