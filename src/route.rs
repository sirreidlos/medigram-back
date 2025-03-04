use axum::{Extension, Json, extract::State, http::StatusCode, response::Html};
use chrono::NaiveDate;
use ed25519_compact::PublicKey;
use moka::sync::Cache;
use num_traits::ToPrimitive;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, Type, query, query_as};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    APIResult, AppError, AppState,
    auth::{AuthUser, retrieve_public_key},
    canonical_json::CanonicalJson,
    model::ExampleConsentRequired,
    protocol::{
        Consent, ConsentError, NIK_LOWERBOUND, NIK_UPPERBOUND, Nik, Nonce,
    },
    schema::{
        Allergy, Consultation, DeviceKey, DoctorProfile, Record, User,
        UserDetail,
    },
};

pub async fn handler(Extension(user): Extension<AuthUser>) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", user.user_id))
}

pub async fn request_nonce(
    State(state): State<AppState>,
) -> CanonicalJson<Nonce> {
    let mut nonce: Nonce = [0u8; 16];
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

#[derive(Debug, Deserialize)]
pub struct UserDetailPayload {
    pub nik: Nik,
    pub name: String,
    pub dob: NaiveDate,
    pub gender: char,
    pub height_in_cm: f32,
    pub weight_in_kg: f32,
}

#[derive(Deserialize)]
pub struct DoctorProfilePayload {
    pub practice_permit: String,
    pub practice_address: String,
}

#[derive(Deserialize)]
pub struct AllergyPayload {
    pub allergy: String,
}

#[derive(Deserialize)]
pub struct DoctorId {
    pub doctor_id: Uuid,
}

#[derive(Serialize)]
pub struct UserOpaque {
    user_id: Uuid,
    email: String,
}

pub async fn get_user(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
) -> APIResult<Json<UserOpaque>> {
    query_as!(
        UserOpaque,
        "SELECT user_id, email FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::RowNotFound,
        e => {
            error!("Error while fetching user for {}: {:?}", user_id, e);
            AppError::InternalError
        }
    })
}

pub async fn get_user_detail(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
) -> APIResult<Json<UserDetail>> {
    let row = sqlx::query!(
        "SELECT user_id, nik, name, dob, gender, height_in_cm, weight_in_kg \
         FROM user_details WHERE user_id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while fetching user_detail for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok(Json(UserDetail {
        user_id: row.user_id,
        nik: row.nik,
        name: row.name,
        dob: row.dob,
        gender: row.gender.chars().next().unwrap_or('U'),
        height_in_cm: row.height_in_cm,
        weight_in_kg: row.weight_in_kg,
    }))
}

pub async fn set_user_detail(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(payload): Json<UserDetailPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    trace!(
        "set_user_details\nuser_id: {}\npayload: {:?}",
        user_id, payload
    );
    if !(NIK_LOWERBOUND..=NIK_UPPERBOUND).contains(&payload.nik) {
        return Err(AppError::InvalidNik);
    }

    query!(
        "INSERT INTO user_details (user_id, nik, name, dob, gender, \
         height_in_cm, weight_in_kg) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id,
        payload.nik,
        payload.name,
        payload.dob,
        payload.gender as i8,
        payload.height_in_cm,
        payload.weight_in_kg
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while setting user_detail for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "Successfully set user detail"})),
    ))
}

pub async fn get_doctor_profile(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(DoctorId { doctor_id }): Json<DoctorId>,
) -> APIResult<Json<DoctorProfile>> {
    query_as!(
        DoctorProfile,
        "SELECT * FROM doctor_profiles WHERE doctor_profile_id = $1",
        doctor_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while fetching doctor_profile for {}: {:?}",
            doctor_id, e
        );
        AppError::InternalError
    })
}

pub async fn set_doctor_profile(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(DoctorProfilePayload {
        practice_permit,
        practice_address,
    }): Json<DoctorProfilePayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    query!(
        "INSERT INTO doctor_profiles (user_id, practice_permit, \
         practice_address, approved) VALUES ($1, $2, $3, $4)",
        user_id,
        practice_permit,
        practice_address,
        false
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while inserting doctor_profile for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Successfully submitted your application" })),
    ))
}

pub async fn get_allergies(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
) -> APIResult<Json<Vec<Allergy>>> {
    query_as!(
        Allergy,
        "SELECT * FROM allergies WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while retrieving allergies for {}: {:?}", user_id, e);
        AppError::InternalError
    })
}

pub async fn add_allergy(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(AllergyPayload { allergy }): Json<AllergyPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let _ = query!(
        "INSERT INTO allergies (user_id, allergy) VALUES ($1, $2)",
        user_id,
        allergy
    )
    .execute(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error while adding allergy for {}: {:?}", user_id, e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "allergy added" })),
    ))
}

#[axum::debug_handler]
pub async fn remove_allergy(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(Allergy { allergy_id, .. }): Json<Allergy>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult =
        query!("DELETE FROM allergies WHERE allergy_id = $1", allergy_id)
            .execute(&state.db_pool)
            .await
            .map_err(|e| {
                error!("Error while adding allergy for {}: {:?}", user_id, e);
                AppError::InternalError
            })?;

    if query_res.rows_affected() == 0 {
        // assume it doesnt exist
        return Err(AppError::RowNotFound);
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "allergy removed" })),
    ))
}

pub async fn get_records(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
) -> APIResult<Json<Vec<Record>>> {
    query_as!(Record, "SELECT * FROM records WHERE user_id = $1", user_id)
        .fetch_all(&state.db_pool)
        .await
        .map(Json)
        .map_err(|e| {
            error!("Error while retrieving records for {}: {:?}", user_id, e);
            AppError::InternalError
        })
}

#[derive(Deserialize)]
pub struct RecordIDPayload {
    record_id: Uuid,
}

pub async fn get_consultations(
    State(state): State<AppState>,
    Extension(AuthUser { user_id }): Extension<AuthUser>,
    Json(RecordIDPayload { record_id }): Json<RecordIDPayload>,
) -> APIResult<Json<Vec<Consultation>>> {
    query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE record_id = $1",
        record_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while retrieving consultations for {}: {:?}",
            user_id, e
        );
        AppError::InternalError
    })
}

#[derive(Deserialize)]
pub struct PrescriptionPayload {
    drug_name: String,
    doses_in_mg: u32,
    regiment_per_day: u32,
    quantity_per_dose: u32,
    instruction: String,
}

#[derive(Deserialize)]
pub struct ConsultationPayload {
    consent: Consent,
    user_id: Uuid,
    diagnoses: Vec<String>,
    symptoms: Vec<String>,
    prescriptions: Vec<PrescriptionPayload>,
}

async fn verify_consent(
    consent: Consent,
    signer: Uuid,
    db_pool: &Pool<Postgres>,
    nonce_cache: &Cache<Nonce, ()>,
) -> Result<(), AppError> {
    let nonce = consent.nonce;
    if !nonce_cache.contains_key(&nonce) {
        return Err(ConsentError::NonceConsumed.into());
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

pub async fn add_consultation(
    State(state): State<AppState>,
    Extension(AuthUser { user_id: doctor_id }): Extension<AuthUser>,
    Json(ConsultationPayload {
        consent,
        user_id,
        diagnoses,
        symptoms,
        prescriptions,
    }): Json<ConsultationPayload>,
) -> APIResult<(StatusCode, String)> {
    let _ =
        verify_consent(consent, user_id, &state.db_pool, &state.nonce_cache)
            .await?;

    let record = query_as!(
        Record,
        "INSERT INTO records (user_id) VALUES ($1) RETURNING record_id, \
         user_id",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error occured while inserting into records: {:?}", e);
        AppError::InternalError
    })?;

    todo!()
}
