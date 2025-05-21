mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use serde_json::json;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use tower::{Service, ServiceExt};

use common::*;

// .route("/user-detail", get(get_user_detail))
// .route("/user-detail", put(set_user_detail))
// .route("/user-measurement", get(get_user_measurements))
// .route("/user-measurement", post(add_user_measurement))

#[sqlx::test(fixtures("users"))]
async fn add_measurements(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_bob(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/me/measurements"))
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
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_bob(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/me/details"))
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
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/me/measurements"))
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
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/me/details"))
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
