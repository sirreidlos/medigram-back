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
mod canonical_json;
mod error;
mod jwt;
mod model;
mod protocol;
mod route;
mod schema;

use crate::route::{
    allergy::{add_allergy, get_allergies, remove_allergy},
    consultation::{add_consultation, get_consultations},
    doctor_profile::{get_doctor_profile, set_doctor_profile},
    purchase::{add_purchase, get_purchases},
    request_nonce,
    user::get_user,
    user_detail::{get_user_detail, set_user_detail},
    user_measurement::{add_user_measurement, get_user_measurements},
};

use axum::{
    extract::FromRef,
    routing::{delete, get, post, put},
};
use protocol::Nonce;
use std::time::Duration;
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

// 7d
const NONCE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Clone)]
struct AppState {
    nonce_cache: Cache<Nonce, ()>,
    db_pool: Pool<Postgres>,
    recognized_session_id: Cache<String, Uuid>,
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
        .route("/allergy", get(get_allergies))
        .route("/allergy", post(add_allergy))
        .route("/allergy", delete(remove_allergy))
        .route("/consultation", get(get_consultations))
        .route("/consultation", post(add_consultation))
        .route("/doctor-profile", get(get_doctor_profile))
        .route("/doctor-profile", post(set_doctor_profile))
        .route("/purchase", get(get_purchases))
        .route("/purchase", post(add_purchase))
        .route("/user", get(get_user))
        .route("/user-detail", get(get_user_detail))
        .route("/user-detail", put(set_user_detail))
        .route("/user-measurement", get(get_user_measurements))
        .route("/user-measurement", post(add_user_measurement))
        .route("/login", post(auth::login))
        .route("/register", post(auth::register))
        .route("/logout", post(auth::logout))
        .route("/request-nonce", get(request_nonce))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
