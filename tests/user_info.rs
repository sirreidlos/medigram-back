use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use medigram::AppState;
use moka::sync::Cache;
use serde_json::Value;
use serde_json::json;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use std::time::Duration;
use tower::{Service, ServiceExt};

static API_ROOT_URL: &str = "127.0.0.1:3001";

// .route("/user-detail", get(get_user_detail))
// .route("/user-detail", put(set_user_detail))
// .route("/user-measurement", get(get_user_measurements))
// .route("/user-measurement", post(add_user_measurement))

async fn extract_session_id(res: Response<Body>) -> String {
    let login_body = res.into_body().collect().await.unwrap().to_bytes();

    let login_body: Value = serde_json::from_slice(&login_body).unwrap();
    login_body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string()
}

#[sqlx::test(fixtures("users"))]
async fn add_measurements(db_pool: Pool<Postgres>) {
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
                "email": "bob@example.com",
                "password": "test",
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let session_id = extract_session_id(login_response).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/user-measurement"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::from(
            json!({
              "height_in_cm": 123.45,
              "weight_in_kg": 13.42,
              "measured_at": "1970-01-01T00:00:00.000Z"
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
async fn set_user_detail(db_pool: Pool<Postgres>) {
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
                "email": "bob@example.com",
                "password": "test",
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let session_id = extract_session_id(login_response).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/user-detail"))
        .method("PUT")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::from(
            json!({
                "nik": 1000000000000000i64,
                "name": "alice",
                "dob": "1970-01-01",
                "gender": "F"
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

#[sqlx::test(fixtures("users", "measurements"))]
async fn get_measurements(db_pool: Pool<Postgres>) {
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

    let login_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let session_id = extract_session_id(login_response).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/user-measurement"))
        .method("GET")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::empty())
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!([{
            "measurement_id": "9440e19e-915f-44e5-9de5-27c1a29c2d98",
            "user_id": "d3969164-86ea-442d-a589-79de89116f9c",
            "height_in_cm": 123.45,
            "weight_in_kg": 67.89,
            "measured_at": "1970-01-01T00:00:00Z"
        }])
    )
}

#[sqlx::test(fixtures("users", "details"))]
async fn get_user_detail(db_pool: Pool<Postgres>) {
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

    let login_response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let session_id = extract_session_id(login_response).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/user-detail"))
        .method("GET")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::empty())
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({
            "user_id": "d3969164-86ea-442d-a589-79de89116f9c",
            "nik": 1000000000000000i64,
            "name": "alice",
            "dob": "1970-01-01",
            "gender": "F"
        })
    )
}
