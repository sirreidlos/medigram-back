use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use medigram::AppState;
use moka::sync::Cache;
use serde_json::json;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use std::time::Duration;
use tower::{Service, ServiceExt};

static API_ROOT_URL: &str = "127.0.0.1:3001";

#[sqlx::test(migrations = "./migrations")]
async fn account_creation(db_pool: Pool<Postgres>) {
    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    let mut app = medigram::app(state);

    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/register"))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "test",
            })
            .to_string(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(fixtures("users"))]
async fn account_register_email_used(db_pool: Pool<Postgres>) {
    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    let mut app = medigram::app(state);

    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/register"))
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

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures("users"))]
async fn account_login(db_pool: Pool<Postgres>) {
    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    let mut app = medigram::app(state);

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

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
