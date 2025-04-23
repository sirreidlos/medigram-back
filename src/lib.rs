pub mod auth;
pub mod canonical_json;
pub mod error;
pub mod model;
pub mod protocol;
pub mod route;
pub mod schema;

use axum::{
    Router,
    extract::FromRef,
    routing::{delete, get, post, put},
};
use protocol::Nonce;

use std::time::Duration;
use uuid::Uuid;

use moka::sync::Cache;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use route::{
    allergy::{add_allergy, get_allergies, remove_allergy},
    consultation::{
        add_consultation, get_consultations, get_diagnoses, get_prescriptions,
        get_symptoms,
    },
    doctor_profile::get_doctor_profile,
    purchase::{add_purchase, get_purchases},
    request_nonce,
    user::get_user,
    user_detail::{get_user_detail, set_user_detail},
    user_measurement::{add_user_measurement, get_user_measurements},
};

// 7d
const NONCE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Clone)]
pub struct AppState {
    pub nonce_cache: Cache<Nonce, ()>,
    pub db_pool: Pool<Postgres>,
    pub recognized_session_id: Cache<String, Uuid>,
}

impl FromRef<AppState> for Cache<String, Uuid> {
    fn from_ref(input: &AppState) -> Self {
        input.recognized_session_id.clone()
    }
}

impl FromRef<AppState> for Pool<Postgres> {
    fn from_ref(input: &AppState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<AppState> for Cache<Nonce, ()> {
    fn from_ref(input: &AppState) -> Self {
        input.nonce_cache.clone()
    }
}

pub async fn health_check() -> String {
    "It works!".to_owned()
}

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(health_check))
        .route("/allergy", get(get_allergies))
        .route("/allergy", post(add_allergy))
        .route("/allergy/{allergy_id}", delete(remove_allergy))
        .route("/consultation", get(get_consultations))
        .route("/consultation", post(add_consultation))
        .route("/doctor-profile/{doctor_id}", get(get_doctor_profile))
        // do we really want this? or should we go with the email approach
        // .route("/doctor-profile", post(set_doctor_profile))
        .route("/purchase", get(get_purchases))
        .route("/purchase", post(add_purchase))
        .route("/user", get(get_user))
        .route("/user-detail", get(get_user_detail))
        .route("/user-detail", put(set_user_detail))
        .route("/user-measurement", get(get_user_measurements))
        .route("/user-measurement", post(add_user_measurement))
        .route("/diagnosis/{consultation_id}", get(get_diagnoses))
        .route("/symptom/{consultation_id}", get(get_symptoms))
        .route("/prescription/{consultation_id}", get(get_prescriptions))
        .route("/login", post(auth::email::login))
        .route("/register", post(auth::email::register))
        .route("/logout", post(auth::logout))
        .route("/request-nonce", get(request_nonce))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
