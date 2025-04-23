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
use medigram::schema::Allergy;

static ALLERGY_ID: &str = "f7769edf-b06b-4749-b6ff-d91efcca8403";

// .route("/allergy", get(get_allergies))
// .route("/allergy", post(add_allergy))
// .route("/allergy", delete(remove_allergy))

#[sqlx::test(fixtures("users"))]
async fn add_allergy(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::from(
            json!({
                "allergen": "beans",
                "severity": "MODERATE"
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
async fn get_allergies_empty(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy"))
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
    let allergies: Vec<Allergy> = serde_json::from_slice(&body).unwrap();

    assert!(allergies.is_empty());
}

#[sqlx::test(fixtures("users", "allergies"))]
async fn get_allergies_nonempty(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy"))
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
    // let allergies: Vec<Allergy> = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        json!([
            {
                "allergy_id": "f7769edf-b06b-4749-b6ff-d91efcca8403",
                "user_id": "d3969164-86ea-442d-a589-79de89116f9c",
                "allergen": "pollen",
                "severity": "MILD",
            }
        ])
    );
}

#[sqlx::test(fixtures("users", "allergies"))]
async fn delete_allergy_success(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy/{ALLERGY_ID}"))
        .method("DELETE")
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
}

#[sqlx::test(fixtures("users"))]
async fn delete_allergy_not_found(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy/{ALLERGY_ID}"))
        .method("DELETE")
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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("users", "allergies"))]
async fn delete_allergy_different_user(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let session_id = login_as_bob(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/allergy/{ALLERGY_ID}"))
        .method("DELETE")
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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
