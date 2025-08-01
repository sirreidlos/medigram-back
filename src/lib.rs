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
    routing::{delete, get, patch, post, put},
};
use protocol::Nonce;

use std::time::Duration;
use uuid::Uuid;

use moka::sync::Cache;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use route::{
    admin::{approve_location, promote_to_admin},
    allergy::{
        add_own_allergy, get_own_allergies, get_user_allergies,
        remove_own_allergy,
    },
    consultation::{
        add_user_consultation, get_consultation_diagnoses,
        get_consultation_prescriptions, get_doctor_consultations_with_user,
        get_own_consultation_single, get_own_consultations,
        get_own_consultations_as_doctor, get_user_consultations,
        set_prescriptions_purchased_at, set_reminder,
    },
    doctor_profile::{
        add_doctor_practice_location, delete_doctor_practice_location,
        get_doctor_profile, get_doctor_profile_by_user_id, set_doctor_profile,
    },
    medical_condition::{
        delete_own_conditions, get_own_conditions, get_user_conditions,
        post_own_conditions,
    },
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
        .route("/users/{user_id}/allergies", get(get_user_allergies))
        .route("/me/allergies", post(add_own_allergy))
        .route("/me/allergies/{allergy_id}", delete(remove_own_allergy))
        // =================== CONSULTATIONS ===================
        .route("/me/consultations", get(get_own_consultations))
        .route(
            "/me/consultations/{consultation_id}",
            get(get_own_consultation_single),
        )
        .route(
            "/users/{user_id}/consultations",
            get(get_user_consultations),
        )
        // is this necessary?
        .route(
            "/users/{user_id}/consultations",
            post(add_user_consultation),
        )
        .route(
            "/doctor/consultations",
            get(get_own_consultations_as_doctor),
        )
        .route(
            "/doctors/{doctor_id}/users/{user_id}/consultations",
            get(get_doctor_consultations_with_user),
        )
        .route(
            "/consultations/{consultation_id}/diagnoses",
            get(get_consultation_diagnoses),
        )
        .route(
            "/consultations/{consultation_id}/prescriptions",
            get(get_consultation_prescriptions),
        )
        .route(
            "/prescriptions/{prescription_id}/purchase",
            patch(set_prescriptions_purchased_at),
        )
        .route(
            "/consultations/{consultation_id}/reminder",
            put(set_reminder),
        )
        // =================== USER INFORMATION ===================
        .route("/me", get(get_own_info))
        .route("/users/{user_id}", get(get_user_info))
        .route("/me/details", get(get_own_details))
        .route("/users/{user_id}/details", get(get_user_details))
        .route("/me/details", put(set_own_details))
        .route("/users/{user_id}/measurements", get(get_user_measurements))
        .route("/me/measurements", get(get_own_measurements))
        .route("/me/measurements", post(add_own_measurement))
        .route("/me/purchases", get(get_own_purchases))
        .route("/me/purchases", post(add_own_purchase))
        // =================== DOCTOR PROFILES ===================
        .route("/doctors/{doctor_id}/profile", get(get_doctor_profile))
        .route(
            "/users/{user_id}/doctor-profile",
            get(get_doctor_profile_by_user_id),
        )
        .route("/me/doctor-profile", post(set_doctor_profile))
        .route(
            "/doctor/practice-location",
            post(add_doctor_practice_location),
        )
        .route(
            "/doctor/practice-location/{location_id}",
            delete(delete_doctor_practice_location),
        )
        // =================== MEDICAL CONDITIONS ===================
        .route("/me/medical-conditions", get(get_own_conditions))
        .route("/me/medical-conditions", post(post_own_conditions))
        .route(
            "/me/medical-conditions/{condition_id}",
            delete(delete_own_conditions),
        )
        .route(
            "/users/{user_id}/medical-conditions",
            get(get_user_conditions),
        )
        // =================== AUTH ===================
        .route("/login", post(auth::email::login))
        .route("/register", post(auth::email::register))
        .route("/logout", post(auth::logout))
        .route("/request-nonce", get(request_nonce))
        // =================== ADMIN ===================
        .route("/users/{user_id}/promote-to-admin", post(promote_to_admin))
        .route(
            "/doctor/practice-location/{location_id}/approve",
            post(approve_location),
        )
        // =================== STATIC FOR DOCS ===================
        .nest_service("/static/api", ServeDir::new("./static/api"))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
