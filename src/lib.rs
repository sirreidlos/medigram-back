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
    allergy::{
        add_own_allergy, get_own_allergies, get_user_allergies,
        remove_own_allergy,
    },
    consultation::{
        add_user_consultation, get_doctor_consultations, get_own_consultations,
        get_user_diagnoses, get_user_prescriptions, get_user_symptoms,
    },
    doctor_profile::{get_doctor_profile, get_doctor_profile_by_user_id},
    purchase::{add_own_purchase, get_own_purchases},
    request_nonce,
    user::{get_own_info, get_user_info},
    user_detail::{get_own_details, get_user_details, set_own_details},
    user_measurement::{
        add_own_measurement, get_own_measurements, get_user_measurements,
    },
};

// 7d
pub const NONCE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
pub const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

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
        // =================== ALLERGIES ===================
        .route("/me/allergies", get(get_own_allergies))
        .route("/users/{user_id_query}/allergies", get(get_user_allergies))
        .route("/me/allergies", post(add_own_allergy))
        .route("/me/allergies/{allergy_id}", delete(remove_own_allergy))
        // =================== CONSULTATIONS ===================
        .route("/me/consultations", get(get_own_consultations))
        .route(
            "/users/{user_id_query}/consultations",
            post(add_user_consultation),
        )
        .route(
            "/doctors/{doctor_id}/users/{user_id_query}/consultations",
            get(get_doctor_consultations),
        )
        .route(
            "/users/{user_id_query}/diagnoses/{consultation_id}",
            get(get_user_diagnoses),
        )
        .route(
            "/users/{user_id_query}/symptoms/{consultation_id}",
            get(get_user_symptoms),
        )
        .route(
            "/users/{user_id_query}/prescriptions/{consultation_id}",
            get(get_user_prescriptions),
        )
        // =================== USER INFORMATION ===================
        .route("/doctors/{doctor_id}/profile", get(get_doctor_profile))
        .route(
            "/users/{user_id_query}/doctor-profile",
            get(get_doctor_profile_by_user_id),
        )
        // do we really want this? or should we go with the email approach
        // .route("/doctor-profile", post(set_doctor_profile))
        .route("/me", get(get_own_info))
        .route("/users/{user_id_query}", get(get_user_info))
        .route("/me/details", get(get_own_details))
        .route("/users/{user_id_query}/details", get(get_user_details))
        .route("/me/details", put(set_own_details))
        .route(
            "/users/{user_id_query}/measurements",
            get(get_user_measurements),
        )
        .route("/me/measurements", get(get_own_measurements))
        .route("/me/measurements", post(add_own_measurement))
        .route("/me/purchases", get(get_own_purchases))
        .route("/me/purchases", post(add_own_purchase))
        // =================== AUTH ===================
        .route("/login", post(auth::email::login))
        .route("/register", post(auth::email::register))
        .route("/logout", post(auth::logout))
        .route("/request-nonce", get(request_nonce))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
