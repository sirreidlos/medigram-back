use anyhow::Context;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use std::fs;

use sqlx::{Pool, Postgres};

pub async fn init_fixtures(pool: &Pool<Postgres>) {
    insert_user_for_login_test(pool).await
}

async fn insert_user_for_login_test(pool: &Pool<Postgres>) {
    let email = "login@example.com";
    let password = "example";

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing failed")
        .to_string();

    sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to execute insert_user_for_login_test: {e}")
        });
}
