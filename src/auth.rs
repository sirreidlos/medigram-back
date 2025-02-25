use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
// use tower_http::{cors::{Any, CorsLayer}, limit::RequestBodyLimitLayer};
use uuid::Uuid;

use crate::{
    AppState,
    jwt::{
        AuthError, AuthResponse, LoginRequest, RefreshRequest,
        create_access_token, create_refresh_token,
    },
};

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Find the user
    // sqlx::query_as!()
    todo!();

    // check against hash

    // Create tokens
    let (access_token, expires_in) = create_access_token(&user.id)?;
    let refresh_token = create_refresh_token(&user.id)?;

    // Return the tokens
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in,
    }))
}

async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<StatusCode, AuthError> {
    // Blacklist the refresh token
    state.blacklist.insert(payload.refresh_token, ());
    // {
    //     let mut token_store = state.token_store.lock().unwrap();
    //     token_store.blacklist_token(payload.refresh_token);
    // }

    Ok(StatusCode::OK)
}
