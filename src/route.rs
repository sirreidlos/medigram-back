use axum::{Extension, extract::State, response::Html};
use rand::Rng;
use tracing::trace;

use crate::{AppState, auth::AuthUser, canonical_json::CanonicalJson};

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
