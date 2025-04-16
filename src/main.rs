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

use medigram_back::AppState;
use moka::sync::Cache;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

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

    let app = medigram_back::app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
