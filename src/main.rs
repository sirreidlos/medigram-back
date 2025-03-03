// Alur Medigram
// 1. Home Page
//    - QR ID pasien (di scan oleh dokter untuk input informasi hasil
//      pemeriksaan) -> QR Generation is done by detailing the information to
//      hash, and then using the patient's seed (key) to hash it, and have it be
//      in the
//    - Purchase obat (di scan oleh apotek, hanya bisa sekali purchase)
//    - Histori / Preview singkat dokter yang pernah konsultasi
//    - Reminder untuk minum obat (nama obat, quantity, cara konsumsi)
// 2. Record Page
//    - Setiap sub-list ada: tanggal pemeriksaan, nama dokter, lokasi praktek
//      dokter, hasil diagnose (teks deskripsi, hasil scan, dll), prescription
//      (nama obat, jumlah dosis, quantity, cara konsumsi)
//    - user tidak bisa ganti atau hapus
//    - dokter yang sama tidak bisa ganti, tapi bisa duplicate (yang salah
//      dianggap misdiagnose - tidak delete, dokter buka record baru) (?)
// 3. Reminder page
//    - user bisa tambah sendiri (optional)
//    - user tidak bisa menghapus reminder dari cara konsumsi prescription (?)
// 4. Profile - Login / Regis
//    - pakai informasi KTP (NIK) dan Nomor Telp
//    - bisa tambah informasi kesehatan lain (berat badan, tinggi, alergi, dll)

mod auth;
pub mod canonical_json;
mod jwt;
mod model;
mod protocol;
mod route;
mod schema;

use crate::route::{handler, request_nonce};
use auth::auth_middleware;
use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use jwt::AuthError;
use protocol::{ConsentError, Nik, Nonce};
use route::{consent_required_example, get_user, get_user_detail};
use serde::Serialize;
use std::{collections::HashSet, time::Duration};
use tracing::info;
use uuid::Uuid;

use axum::Router;
use moka::sync::Cache;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{
    EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

enum AppError {
    InternalError,
    InvalidNik,
    RowNotFound,
    BadPayload,
    Auth(AuthError),
    Consent(ConsentError),
}

// actual decoration trait check
// pls do the check manually ty
pub type APIResult<T: Serialize> = std::result::Result<Json<T>, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error has occured",
            ),
            AppError::Auth(auth_error) => return auth_error.into_response(),
            AppError::Consent(consent_error) => {
                return consent_error.into_response();
            }
            AppError::InvalidNik => (StatusCode::BAD_REQUEST, "Invalid NIK"),
            AppError::BadPayload => {
                (StatusCode::BAD_REQUEST, "Bad Payload Data")
            }
            AppError::RowNotFound => (
                StatusCode::NOT_FOUND,
                "Resource does not exist in the database",
            ),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        Self::Auth(value)
    }
}

impl From<ConsentError> for AppError {
    fn from(value: ConsentError) -> Self {
        Self::Consent(value)
    }
}

#[derive(Clone)]
struct AppState {
    nonce_cache: Cache<Nonce, ()>,
    db_pool: Pool<Postgres>,
    recognized_session_id: Cache<String, Uuid>,
}

impl AppState {
    fn add_nonce(&self, nonce: Nonce) {
        self.nonce_cache.insert(nonce, ());
    }

    fn remove_nonce(&self, nonce: Nonce) {
        self.nonce_cache.invalidate(&nonce);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=trace", env!("CARGO_CRATE_NAME")).into()
        }))
        .with(
            fmt::layer()
                .event_format(fmt::format()) // Use the correct `Full` format
                .with_thread_names(true), // Show thread names
        )
        .init();

    let db_pool = Pool::<Postgres>::connect(
        "postgres://postgres@127.0.0.1:5432/medigram",
    )
    .await
    .expect("failed to connect to db");

    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(handler))
        .route("/example-consent", post(consent_required_example))
        .route("/get-user", get(get_user))
        .route("/get-user-detail", get(get_user_detail))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route("/test", get(get_user))
        .route("/request-nonce", get(request_nonce))
        .route("/login", post(auth::login))
        .route("/register", post(auth::register))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
