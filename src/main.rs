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

use medigram::{AppState, NONCE_TTL, SESSION_TTL};
use moka::sync::Cache;
use sqlx::Pool;
use sqlx::postgres::Postgres;
use std::time::Duration;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres@127.0.0.1:5432/medigram"
    )]
    db_pool: Pool<Postgres>,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("migration failed");

    let state = AppState {
        nonce_cache: Cache::builder().time_to_live(NONCE_TTL).build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(SESSION_TTL)
            .build(),
    };

    let app = medigram::app(state);

    Ok(app.into())
}
