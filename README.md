# Medigram
This is the backend application of medigram.

## Prerequisites
1. Rust (https://rustup.rs/)
2. Shuttle.rs CLI (`cargo install cargo-shuttle`)
3. SQLx CLI (`cargo install sqlx-cli`)
4. PostgreSQL

## How to run
1. Set the environment variable `DATABASE_URL` (e.g. `postgres://postgres@127.0.0.1:5432/medigram`)
2. Run `sqlx migrate run` in the project folder
3. Run `cargo shuttle run` for a local run in `http://localhost:8000`

