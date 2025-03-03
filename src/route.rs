use axum::{Extension, Json, extract::State, http::StatusCode, response::Html};
use ed25519_compact::PublicKey;
use rand::Rng;
use tracing::{error, trace};

use crate::{
    AppError, AppState,
    auth::AuthUser,
    auth::retrieve_public_key,
    canonical_json::CanonicalJson,
    model::ExampleConsentRequired,
    protocol::{Consent, ConsentError},
};

pub(crate) async fn handler(
    Extension(user): Extension<AuthUser>,
) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", user.user_id))
}

pub(crate) async fn request_nonce(
    State(state): State<AppState>,
) -> CanonicalJson<[u8; 16]> {
    let mut nonce: [u8; 16] = [0u8; 16];
    rand::rng().fill(&mut nonce);
    state.add_nonce(nonce);

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
        return Err(ConsentError::NonceConsumed.into());
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
