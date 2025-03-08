pub mod allergy;
pub mod consultation;
pub mod doctor_profile;
pub mod purchase;
pub mod user;
pub mod user_detail;
pub mod user_measurement;

use axum::{Json, extract::State, http::StatusCode, response::Html};
use chrono::{DateTime, Utc};
use ed25519_compact::PublicKey;
use moka::sync::Cache;
use rand::Rng;
use sqlx::{Pool, Postgres, query_as};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    AppState, NONCE_TTL,
    auth::{AuthUser, retrieve_public_key},
    canonical_json::CanonicalJson,
    error::AppError,
    model::ExampleConsentRequired,
    protocol::{Consent, ConsentError, Nonce},
    schema::DeviceKey,
};

pub async fn handler(user: AuthUser) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", user.user_id))
}

pub async fn request_nonce(
    State(state): State<AppState>,
) -> CanonicalJson<Nonce> {
    let mut nonce: Nonce = [0u8; 16];
    rand::rng().fill(&mut nonce);
    state.nonce_cache.insert(nonce, ());

    trace!("nonce requested: {:?}", nonce);

    CanonicalJson(nonce)
}

pub async fn consent_required_example(
    State(state): State<AppState>,
    Json(payload): Json<ExampleConsentRequired>,
) -> Result<StatusCode, AppError> {
    let consent: Consent = payload.consent;
    let nonce = consent.nonce;

    if !state.nonce_cache.contains_key(&nonce) {
        return Err(ConsentError::NonceGone.into());
    }

    state.nonce_cache.remove(&nonce);

    let device_id = consent.signer_device_id;
    let key_info = retrieve_public_key(device_id, &state.db_pool).await?;
    let public_key =
        PublicKey::from_pem(&key_info.public_key_pem).map_err(|e| {
            error!("error while converting pem to pk: {:?}", e);
            AppError::InternalError
        })?;

    let is_valid = consent.verify(&public_key);

    if is_valid {
        Ok(StatusCode::OK)
    } else {
        Err(ConsentError::NonConsent.into())
    }
}

fn key_expired(key_revoked_time: Option<DateTime<Utc>>) -> bool {
    if let Some(t) = key_revoked_time {
        if Utc::now() - NONCE_TTL > t {
            return true;
        }
    }

    false
}

pub async fn verify_consent(
    consent: Consent,
    signer: Uuid,
    db_pool: &Pool<Postgres>,
    nonce_cache: &Cache<Nonce, ()>,
) -> Result<(), AppError> {
    let nonce = consent.nonce;
    if !nonce_cache.contains_key(&nonce) {
        return Err(ConsentError::NonceGone.into());
    }
    nonce_cache.remove(&nonce);

    let device_id = consent.signer_device_id;
    let device_key = query_as!(
        DeviceKey,
        "SELECT * FROM device_keys WHERE device_id = $1",
        device_id
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => ConsentError::DeviceNotFound.into(),
        e => {
            error!(
                "Error while fetching device key for {}: {:?}",
                device_id, e
            );
            AppError::InternalError
        }
    })?;

    if key_expired(device_key.revoked_at) {
        return Err(ConsentError::KeyExpired.into());
    }

    if device_key.user_id != signer {
        return Err(ConsentError::UserDeviceMismatch.into());
    }

    let pk = PublicKey::from_pem(&device_key.public_key_pem).map_err(|e| {
        error!(
            "Error occured while parsing pem to pk: pem: {}\nerror: {:?}",
            device_key.public_key_pem, e
        );
        AppError::InternalError
    })?;

    if !consent.verify(&pk) {
        return Err(ConsentError::NonConsent.into());
    }

    Ok(())
}
