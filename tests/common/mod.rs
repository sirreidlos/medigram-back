use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
};
use http_body_util::BodyExt;
use medigram::AppState;
use moka::sync::Cache;
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use tower::{Service, ServiceExt};

pub static API_ROOT_URL: &str = "127.0.0.1:3001";

pub fn get_app(db_pool: Pool<Postgres>) -> Router {
    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    medigram::app(state)
}

pub async fn extract_session_id(res: Response<Body>) -> String {
    let login_body = res.into_body().collect().await.unwrap().to_bytes();

    let login_body: Value = serde_json::from_slice(&login_body).unwrap();
    login_body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string()
}

pub async fn login_as_alice(app: &mut Router) -> String {
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/login"))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "alice@example.com",
                "password": "test",
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = ServiceExt::<Request<Body>>::ready(app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    extract_session_id(login_response).await
}

pub async fn login_as_bob(app: &mut Router) -> String {
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/login"))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "bob@example.com",
                "password": "test",
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = ServiceExt::<Request<Body>>::ready(app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    extract_session_id(login_response).await
}
