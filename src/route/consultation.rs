use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction, query, query_as};
use tracing::error;
use uuid::Uuid;

use crate::{
    AppState,
    auth::AuthUser,
    error::{APIResult, AppError},
    protocol::{Consent, ConsentError},
    route::verify_consent,
    schema::{Consultation, DoctorProfile},
};

pub async fn get_consultations(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> APIResult<Json<Vec<Consultation>>> {
    query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE user_id = $1",
        user_id
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
pub struct DiagnosisPayload {
    diagnosis: String,
    icd_code: String,
    severity: String,
}

#[derive(Deserialize)]
pub struct PrescriptionPayload {
    drug_name: String,
    doses_in_mg: i32,
    regimen_per_day: i32,
    quantity_per_dose: i32,
    instruction: String,
}

#[derive(Deserialize)]
pub struct ConsultationPayload {
    consent: Consent,
    user_id: Uuid,
    diagnoses: Vec<DiagnosisPayload>,
    symptoms: Vec<String>,
    prescriptions: Vec<PrescriptionPayload>,
}

pub async fn add_consultation(
    State(state): State<AppState>,
    AuthUser {
        user_id: doctor_user_id,
        ..
    }: AuthUser,
    Json(ConsultationPayload {
        consent,
        user_id,
        diagnoses,
        symptoms,
        prescriptions,
    }): Json<ConsultationPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    verify_consent(consent, user_id, &state.db_pool, &state.nonce_cache)
        .await?;

    let doctor_profile = query_as!(
        DoctorProfile,
        "SELECT * FROM doctor_profiles WHERE user_id = $1",
        doctor_user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error occured while fetching for doctor profile: {:?}", e);

        match e {
            sqlx::Error::RowNotFound => ConsentError::NotLicensed.into(),
            _ => AppError::InternalError,
        }
    })?;

    if doctor_profile.approved_at.is_none() {
        return Err(ConsentError::NotLicensed.into());
    }

    let doctor_id = doctor_profile.doctor_id;

    let mut tx: Transaction<Postgres> =
        state.db_pool.begin().await.map_err(|e| {
            error!("Error occured while starting a transaction: {:?}", e);
            AppError::InternalError
        })?;

    let consultation = query_as!(
        Consultation,
        "INSERT INTO consultations (doctor_id, user_id) VALUES ($1, $2) \
         RETURNING consultation_id, doctor_id, user_id",
        doctor_id,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error occured while inserting into records: {:?}", e);
        AppError::InternalError
    })?;

    for diagnosis in diagnoses {
        let DiagnosisPayload {
            diagnosis,
            icd_code,
            severity,
        } = diagnosis;

        query!(
            "INSERT INTO diagnoses (consultation_id, diagnosis, icd_code, \
             severity) VALUES ($1, $2, $3, $4)",
            consultation.consultation_id,
            diagnosis,
            icd_code,
            severity
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Error occured while inserting a diagnosis: {:?}", e);
            AppError::InternalError
        })?;
    }

    for symptom in symptoms {
        query!(
            "INSERT INTO symptoms (consultation_id, symptom) VALUES ($1, $2)",
            consultation.consultation_id,
            symptom
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Error occured while inserting a diagnosis: {:?}", e);
            AppError::InternalError
        })?;
    }

    for prescription in prescriptions {
        let PrescriptionPayload {
            drug_name,
            doses_in_mg,
            regimen_per_day,
            quantity_per_dose,
            instruction,
        } = prescription;

        query!(
            "INSERT INTO prescriptions (consultation_id, drug_name, \
             doses_in_mg, regimen_per_day, quantity_per_dose, instruction) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            consultation.consultation_id,
            drug_name,
            doses_in_mg,
            regimen_per_day,
            quantity_per_dose,
            instruction
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Error occured while inserting a prescription: {:?}", e);
            AppError::InternalError
        })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Error occured while committing transaction: {:?}", e);
        AppError::InternalError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "consultation record added" })),
    ))
}
