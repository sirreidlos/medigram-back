use axum::{Extension, Json, extract::State};
use serde::Serialize;
use sqlx::query_as;
use tracing::error;
use uuid::Uuid;

use crate::{APIResult, AppError, AppState, auth::AuthUser};

#[derive(Serialize)]
pub struct UserOpaque {
    user_id: Uuid,
    email: String,
}

pub async fn get_user(
    State(state): State<AppState>,
    // Extension(AuthUser { user_id }): Extension<AuthUser>,
    AuthUser { user_id }: AuthUser,
) -> APIResult<Json<UserOpaque>> {
    query_as!(
        UserOpaque,
        "SELECT user_id, email FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::RowNotFound,
        e => {
            error!("Error while fetching user for {}: {:?}", user_id, e);
            AppError::InternalError
        }
    })
}
