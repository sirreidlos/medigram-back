mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use medigram::protocol::Nonce;
use medigram::route::request_nonce;
use serde_json::Value;
use serde_json::json;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use tower::{Service, ServiceExt};

use common::*;
use medigram::schema::Allergy;

static ALLERGY_ID: &str = "f7769edf-b06b-4749-b6ff-d91efcca8403";

// .route("/consultation", get(get_consultations))
// .route("/consultation", post(add_consultation))
// .route("/diagnosis/{consultation_id}", get(get_diagnoses))
// .route("/symptom/{consultation_id}", get(get_symptoms))
// .route("/prescription/{consultation_id}", get(get_prescriptions))

#[sqlx::test(fixtures("users"))]
async fn add_consultations_non_doctor(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/users/41490144-e4e1-4d1f-9eb7-f90af81c12ce/consultations"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        // even with a fake but parsable payload this should fail at the doctor
        // check anyway
        .body(Body::from(
            json!(
            {
              "consent": {
                "signer_device_id": "b896cff8-de47-451c-96c1-74086c86b9e7",
                "nonce": "XjMOZe0G6cUndk4U",
                "signature": "pCNjNI7vsUhP0TEfinN+NFOTEYLsexyVnawHx8Fx+x5VIhPho2/psGS9Ng96WGdO9mc8cNiK15Pg8KXVHdGuDQ=="
              },
              "user_id": "41676bb2-8561-47fe-9271-4c7e89defa7c",
              "location_id": "fbc0a545-f266-495d-91a1-667479a13ace",
              "diagnoses": [
                {
                  "diagnosis": "Common Cold",
                  "severity": "MILD"
                }
              ],
              "symptoms": "runny nose, coughing",
              "prescriptions": [
                {
                  "drug_name": "panadol",
                  "doses_in_mg": 100,
                  "regimen_per_day": 3,
                  "quantity_per_dose": 21,
                  "instruction": "Take after meals with a full glass of water."
                },
                {
                  "drug_name": "panadol",
                  "doses_in_mg": 100,
                  "regimen_per_day": 3,
                  "quantity_per_dose": 21,
                  "instruction": "Take after meals with a full glass of water."
                }
              ]
            }).to_string(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("users", "doctor_info"))]
async fn add_consultations_invalid_nonce(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let (session_id, _user_id) = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/users/41490144-e4e1-4d1f-9eb7-f90af81c12ce/consultations"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::from(
            json!({
              "consent": {
                "signer_device_id": "b896cff8-de47-451c-96c1-74086c86b9e7",
                "nonce": "XjMOZe0G6cUndk4U",
                "signature": "lzfJ8534rZ2f4m0CMdxE5T0emdiV3AERgxYk1q7NGUz+leM/7rgzCyVXCjjXBc8cX4P236h1bjEJ0w7oHVPzCg=="
              },
              "user_id": "41676bb2-8561-47fe-9271-4c7e89defa7c",
              "location_id": "fbc0a545-f266-495d-91a1-667479a13ace",
              "diagnoses": [
                {
                  "diagnosis": "Common Cold",
                  "severity": "MILD"
                }
              ],
              "symptoms": "runny nose, coughing",
              "prescriptions": [
                {
                  "drug_name": "panadol",
                  "doses_in_mg": 100,
                  "regimen_per_day": 3,
                  "quantity_per_dose": 21,
                  "instruction": "Take after meals with a full glass of water."
                },
                {
                  "drug_name": "panadol",
                  "doses_in_mg": 100,
                  "regimen_per_day": 3,
                  "quantity_per_dose": 21,
                  "instruction": "Take after meals with a full glass of water."
                }
              ]
            }).to_string(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::GONE);
}

#[sqlx::test(fixtures("users", "doctor_info"))]
async fn add_consultations_no_device(db_pool: Pool<Postgres>) {
    let mut app = get_app(db_pool);
    let request_nonce = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/request-nonce"))
        .body(Body::empty())
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request_nonce)
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let nonce = body.get("nonce");

    let (session_id, _user_id) = login_as_alice(&mut app).await;
    let request = Request::builder()
        .uri(format!("http://{API_ROOT_URL}/users/41490144-e4e1-4d1f-9eb7-f90af81c12ce/consultations"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {session_id}"))
        .body(Body::from(
            json!(
                    {
                      "consent": {
                        "signer_device_id": "b896cff8-de47-451c-96c1-74086c86b9e7",
                        "nonce": nonce,
                        "signature": "lzfJ8534rZ2f4m0CMdxE5T0emdiV3AERgxYk1q7NGUz+leM/7rgzCyVXCjjXBc8cX4P236h1bjEJ0w7oHVPzCg=="
                      },
                      "user_id": "41676bb2-8561-47fe-9271-4c7e89defa7c",
                      "location_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
                      "diagnoses": [
                        {
                          "diagnosis": "Common Cold",
                          "severity": "MILD"
                        }
                      ],
                      "symptoms": "runny nose, coughing",
                      "prescriptions": [
                        {
                          "drug_name": "panadol",
                          "doses_in_mg": 100,
                          "regimen_per_day": 3,
                          "quantity_per_dose": 21,
                          "instruction": "Take after meals with a full glass of water."
                        },
                        {
                          "drug_name": "panadol",
                          "doses_in_mg": 100,
                          "regimen_per_day": 3,
                          "quantity_per_dose": 21,
                          "instruction": "Take after meals with a full glass of water."
                        }
                      ]
                    }            )
            .to_string(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
