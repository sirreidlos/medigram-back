mod fixtures;

use anyhow::Context;
use axum::body::Body;
use axum::http::Request;
use medigram::{AppState, app, auth::email::RegisterRequest};
use moka::sync::Cache;
use reqwest::{Client, StatusCode};
use serde_json::json;
use sqlx::Pool;
use sqlx::migrate::Migrator;
use sqlx::postgres::Postgres;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, sync::OnceCell};
use tower::{Service, ServiceExt};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
static DB_POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
static SERVER_STARTED: OnceCell<()> = OnceCell::const_new();
static FIXTURES_INTIALIZED: OnceCell<()> = OnceCell::const_new();
static API_ROOT_URL: &str = "127.0.0.1:3001";

async fn run_server() -> anyhow::Result<()> {
    let db_pool = get_db_pool().await;

    MIGRATOR
        .undo(&db_pool, 0)
        .await
        .context("Failed to undo migrations")?;
    MIGRATOR
        .run(&db_pool)
        .await
        .context("Failed to run migrations")?;

    let state = AppState {
        nonce_cache: Cache::builder()
            .time_to_live(Duration::from_secs(7 * 24 * 60 * 60))
            .build(),
        db_pool,
        recognized_session_id: Cache::builder()
            .time_to_live(Duration::from_secs(30 * 24 * 60 * 60))
            .build(),
    };

    let app = app(state);
    let listener = TcpListener::bind(API_ROOT_URL).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_server_once() {
    SERVER_STARTED
        .get_or_init(|| async {
            tokio::spawn(async {
                if let Err(err) = run_server().await {
                    eprintln!("Server crashed: {err:?}");
                    std::process::exit(1);
                }
            });

            let client = reqwest::Client::new();
            let mut retries = 10;
            loop {
                if retries == 0 {
                    panic!("Server failed to start in time");
                }

                match client
                    .get(format!("http://{API_ROOT_URL}/request-nonce"))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => break,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        retries -= 1;
                    }
                }
            }
        })
        .await;
}

async fn get_db_pool() -> Pool<Postgres> {
    DB_POOL
        .get_or_init(|| async {
            dotenvy::dotenv().ok();
            let db_url = dotenvy::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL must be set");

            Pool::<Postgres>::connect(&db_url)
                .await
                .expect("failed to connect to db")
        })
        .await
        .clone()
}

async fn init_fixtures(pool: &Pool<Postgres>) {
    FIXTURES_INTIALIZED
        .get_or_init(|| async {
            fixtures::init_fixtures(pool).await;
        })
        .await;
}

#[tokio::test]
async fn account_register() {
    start_server_once().await;

    let client = Client::new();

    let res = client
        .post(format!("http://{API_ROOT_URL}/register"))
        .json(&json!({
            "email": "test@example.com",
            "password": "test",
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn account_login() {
    start_server_once().await;
    init_fixtures(&get_db_pool().await).await;

    let client = Client::new();

    let res = client
        .post(format!("http://{API_ROOT_URL}/login"))
        .json(&json!({
            "email": "login@example.com",
            "password": "example",
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(res.status(), reqwest::StatusCode::OK);
}
