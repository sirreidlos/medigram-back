use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, Transaction, query, query_as};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, LicensedUser},
    error::{APIResult, AppError, DatabaseError},
    protocol::{Consent, ConsentError},
    route::verify_consent,
    schema::{Consultation, Diagnosis, DoctorPracticeLocation, Prescription},
};

pub async fn get_own_consultations(
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

pub async fn get_own_consultation_single(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(consultation_id): Path<Uuid>,
) -> APIResult<Json<Consultation>> {
    query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE user_id = $1 AND consultation_id = \
         $2",
        user_id,
        consultation_id
    )
    .fetch_one(&state.db_pool)
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

pub async fn get_user_consultations(
    State(state): State<AppState>,
    auth: AuthUser,
    doctor: Option<LicensedUser>,
    Path(user_id): Path<Uuid>,
) -> APIResult<Json<Vec<Consultation>>> {
    if auth.user_id != user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

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

pub async fn get_own_consultations_as_doctor(
    State(state): State<AppState>,
    doctor: Option<LicensedUser>,
) -> APIResult<Json<Vec<Consultation>>> {
    let doctor = doctor.ok_or(AppError::NotTheSameUser)?;

    query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE doctor_id = $1",
        doctor.doctor_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while retrieving consultations for doctor {}: {:?}",
            doctor.doctor_id, e
        );
        AppError::InternalError
    })
}

pub async fn get_doctor_consultations_with_user(
    State(state): State<AppState>,
    auth: AuthUser,
    doctor: Option<LicensedUser>,
    Path((doctor_id, user_id)): Path<(Uuid, Uuid)>,
) -> APIResult<Json<Vec<Consultation>>> {
    if user_id != auth.user_id && doctor.is_none() {
        return Err(AppError::NotTheSameUser);
    }

    let doctor_unwrap = doctor.unwrap();
    if doctor_unwrap.doctor_id != doctor_id {
        return Err(AppError::NotTheSameUser);
    }

    query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE user_id = $1 AND doctor_id = $2",
        user_id,
        doctor_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!(
            "Error while retrieving consultations between doctor {} and user \
             {}: {:?}",
            doctor_id, user_id, e
        );
        AppError::InternalError
    })
}

#[derive(Deserialize)]
pub struct DiagnosisPayload {
    diagnosis: String,
    severity: String,
}

#[derive(Deserialize)]
pub struct PrescriptionPayload {
    drug_name: String,
    doses_in_mg: f64,
    regimen_per_day: f64,
    quantity_per_dose: f64,
    instruction: String,
}

#[derive(Deserialize)]
pub struct ConsultationPayload {
    consent: Consent,
    user_id: Uuid,
    location_id: Uuid,
    diagnoses: Vec<DiagnosisPayload>,
    symptoms: String,
    prescriptions: Vec<PrescriptionPayload>,
}

pub async fn add_user_consultation(
    State(state): State<AppState>,
    doctor: Option<LicensedUser>,
    Path(user_id): Path<Uuid>,
    Json(ConsultationPayload {
        consent,
        user_id: _user_id,
        location_id,
        diagnoses,
        symptoms,
        prescriptions,
    }): Json<ConsultationPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    if doctor.is_none() {
        return Err(AppError::NotLicensed);
    }
    let doctor_id = doctor.unwrap().doctor_id;
    let location_query: DoctorPracticeLocation = query_as!(
        DoctorPracticeLocation,
        "SELECT * FROM doctor_practice_locations WHERE doctor_id = $1 AND \
         location_id = $2",
        doctor_id,
        location_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            warn!(
                "Row not found for doctor {} and location {}",
                doctor_id, location_id
            );
            DatabaseError::RowNotFound.into()
        }
        _ => {
            error!(
                "Error occured while retrieving location {} for doctor {}: \
                 {:?}",
                location_id, doctor_id, e
            );
            AppError::InternalError
        }
    })?;

    if location_query.approved_at.is_none() {
        return Err(AppError::LocationNotApproved);
    }

    verify_consent(consent, user_id, &state.db_pool, &state.nonce_cache)
        .await?;

    let mut tx: Transaction<Postgres> =
        state.db_pool.begin().await.map_err(|e| {
            error!("Error occured while starting a transaction: {:?}", e);
            AppError::InternalError
        })?;

    let consultation = query_as!(
        Consultation,
        "INSERT INTO consultations (doctor_id, user_id, location_id, \
         symptoms) VALUES ($1, $2, $3, $4) RETURNING consultation_id, \
         doctor_id, user_id, location_id, symptoms, created_at, reminded",
        doctor_id,
        user_id,
        location_id,
        symptoms
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
            severity,
        } = diagnosis;

        query!(
            "INSERT INTO diagnoses (consultation_id, diagnosis, severity) \
             VALUES ($1, $2, $3)",
            consultation.consultation_id,
            diagnosis,
            severity
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
            instruction,
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
        StatusCode::CREATED,
        Json(json!({ "message": "consultation record added" })),
    ))
}

pub async fn check_user(
    user_id: Uuid,
    doctor: Option<LicensedUser>,
    consultation_id: Uuid,
    db_pool: &Pool<Postgres>,
) -> APIResult<()> {
    let consultation = query_as!(
        Consultation,
        "SELECT * FROM consultations WHERE consultation_id = $1",
        consultation_id
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error occured while fetching for consultation in get_diagnoses: \
             {:?}",
            e
        );

        match e {
            sqlx::Error::RowNotFound => DatabaseError::RowNotFound.into(),
            _ => AppError::InternalError,
        }
    })?;

    if let Some(doctor) = doctor {
        if consultation.doctor_id == doctor.doctor_id {
            return Ok(());
        }
    }

    if user_id != consultation.user_id {
        return Err(AppError::NotTheSameUser);
    }

    Ok(())
}

pub async fn get_consultation_diagnoses(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    doctor: Option<LicensedUser>,
    Path(consultation_id): Path<Uuid>,
) -> APIResult<Json<Vec<Diagnosis>>> {
    check_user(user_id, doctor, consultation_id, &state.db_pool).await?;

    query_as!(
        Diagnosis,
        "SELECT * FROM diagnoses WHERE consultation_id = $1",
        consultation_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error occured while fetching for diagnoses: {:?}", e);

        AppError::InternalError
    })
}

pub async fn get_consultation_prescriptions(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    doctor: Option<LicensedUser>,
    Path(consultation_id): Path<Uuid>,
) -> APIResult<Json<Vec<Prescription>>> {
    check_user(user_id, doctor, consultation_id, &state.db_pool).await?;

    query_as!(
        Prescription,
        "SELECT * FROM prescriptions WHERE consultation_id = $1",
        consultation_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| {
        error!("Error occured while fetching for prescriptions: {:?}", e);

        AppError::InternalError
    })
}

#[derive(Deserialize)]
pub struct PrescriptionPurchasedAt {
    purchased_at: DateTime<Utc>,
}

pub async fn set_prescriptions_purchased_at(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(prescription_id): Path<Uuid>,
    Json(PrescriptionPurchasedAt { purchased_at }): Json<
        PrescriptionPurchasedAt,
    >,
) -> APIResult<(StatusCode, Json<Value>)> {
    let record = query!(
        "SELECT u.user_id FROM users as u
         JOIN consultations AS c ON c.user_id = u.user_id
         JOIN prescriptions AS p ON p.consultation_id = c.consultation_id
         WHERE p.prescription_id = $1",
        prescription_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while checking for prescription {} user {}: {}",
            prescription_id, user_id, e
        );

        AppError::InternalError
    })?;

    let Some(record) = record else {
        return Err(DatabaseError::RowNotFound.into());
    };

    if record.user_id != user_id {
        return Err(AppError::NotTheSameUser);
    }

    query!(
        "UPDATE prescriptions
         SET purchased_at = $1
         WHERE prescription_id = $2",
        purchased_at,
        prescription_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Failed to update purchased_at for prescription {}: {}",
            prescription_id, e
        );
        AppError::InternalError
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Prescription marked as purchased" })),
    ))
}

pub async fn set_reminder(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(consultation_id): Path<Uuid>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let query_res: sqlx::postgres::PgQueryResult = query!(
        "UPDATE consultations
         SET reminded = true
         WHERE consultation_id = $1 AND user_id = $2",
        consultation_id,
        user_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while updating `reminded` for consultation {} for {}: {:?}",
            consultation_id, user_id, e
        );
        AppError::InternalError
    })?;

    if query_res.rows_affected() == 0 {
        // assume it doesnt exist
        return Err(DatabaseError::RowNotFound.into());
    }

    Ok((StatusCode::OK, Json(json!({ "message": "reminded" }))))
}
