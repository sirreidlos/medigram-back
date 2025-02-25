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

mod protocol;
mod schema;

use axum::{Router, response::Html, routing::get};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                                                                    .unwrap_or_else(|_| {
                    format!("{}=trace", env!("CARGO_CRATE_NAME")).into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
