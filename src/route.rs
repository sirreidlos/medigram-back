use axum::{extract::State, response::Html};
use rand::Rng;
use tracing::trace;

use crate::{canonical_json::CanonicalJson, AppState};

pub(crate) async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

pub(crate) async fn request_nonce(State(state): State<AppState>) -> CanonicalJson<[u8; 16]> {
    let mut nonce: [u8; 16]= [0u8; 16];
    rand::rng().fill(&mut nonce);
    state.add_nonce(nonce);

    trace!("nonce requested: {:?}", nonce);
    
    CanonicalJson(nonce)
}
