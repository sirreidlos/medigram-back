use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub struct CanonicalJson<T>(pub T);

impl<T> IntoResponse for CanonicalJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json_canonicalizer::to_string(&self.0) {
            Ok(json) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                json,
            )
                .into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
