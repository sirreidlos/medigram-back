use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{auth::AuthError, protocol::ConsentError};

/// Represents all the errors that may occur in the app
pub enum AppError {
    /// Error for any error that shouldn't be exposed to the user
    InternalError,
    /// Error for invalid `NIK` being given
    InvalidNik,
    /// Error for non-existent data
    RowNotFound,
    /// Error for when a user tries to access another user's data
    NotTheSameUser,
    /// Error for authentication-related issues
    Auth(AuthError),
    /// Error for consent-related issues
    Consent(ConsentError),
}

// actual decoration trait check
// pls do the check manually ty
pub type APIResult<T> = Result<T, AppError>;
// pub type APIResultJson<T: Serialize> = APIResult<Json<T>>;
// pub type APIResultCodeMessage = APIResult<(StatusCode, Json<Value>)>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error has occured",
            ),
            AppError::Auth(auth_error) => return auth_error.into_response(),
            AppError::Consent(consent_error) => {
                return consent_error.into_response();
            }
            AppError::InvalidNik => (StatusCode::BAD_REQUEST, "Invalid NIK"),
            AppError::RowNotFound => {
                (StatusCode::NOT_FOUND, "Row does not exist in the database")
            }
            AppError::NotTheSameUser => (
                StatusCode::FORBIDDEN,
                "You are not allowed to request for this",
            ),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub enum DatabaseError {
    RowNotFound,
    ForeignKeyViolation,
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        Self::Auth(value)
    }
}

impl From<ConsentError> for AppError {
    fn from(value: ConsentError) -> Self {
        Self::Consent(value)
    }
}
