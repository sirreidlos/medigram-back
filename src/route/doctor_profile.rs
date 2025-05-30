use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{query, query_as};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, LicensedUser},
    error::{APIResult, AppError, DatabaseError},
    schema::{DoctorPracticeLocation, DoctorProfile},
};

#[derive(Deserialize)]
pub struct DoctorProfilePayload {
    pub practice_permit: String,
    pub practice_address: String,
}

#[derive(Serialize)]
pub struct DoctorProfilePublic {
    pub doctor_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub locations: Vec<DoctorPracticeLocation>,
}

pub async fn get_doctor_profile(
    State(state): State<AppState>,
    AuthUser { .. }: AuthUser,
    Path(doctor_id): Path<Uuid>,
) -> APIResult<Json<DoctorProfilePublic>> {
    let profile = query!(
        "SELECT d.doctor_id, d.user_id, d.created_at, d.approved_at, ud.name
            FROM doctor_profiles AS d
            JOIN user_details AS ud ON ud.user_id = d.user_id
            WHERE doctor_id = $1",
        doctor_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            warn!("doctor profile for doctor {doctor_id} does not exist");
            DatabaseError::RowNotFound.into()
        }
        e => {
            error!(
                "Error while fetching doctor_profile for {}: {:?}",
                doctor_id, e
            );
            AppError::InternalError
        }
    })?;

    if profile.approved_at.is_none() {
        return Err(AppError::NotLicensed);
    }

    let locations = query_as!(
        DoctorPracticeLocation,
        "SELECT * FROM doctor_practice_locations WHERE doctor_id = $1",
        doctor_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            warn!(
                "doctor practice locations for doctor {doctor_id} does not \
                 exist"
            );
            DatabaseError::RowNotFound.into()
        }
        e => {
            error!(
                "Error while fetching doctor_practice_locations for {}: {:?}",
                doctor_id, e
            );
            AppError::InternalError
        }
    })?;

    Ok(Json(DoctorProfilePublic {
        doctor_id: profile.doctor_id,
        user_id: profile.user_id,
        name: profile.name,
        created_at: profile.created_at,
        locations,
    }))
}

pub async fn get_doctor_profile_by_user_id(
    State(state): State<AppState>,
    _: AuthUser,
    Path(user_id): Path<Uuid>,
) -> APIResult<Json<DoctorProfilePublic>> {
    let profile = query!(
        "SELECT d.doctor_id, d.user_id, d.created_at, d.approved_at, ud.name
            FROM doctor_profiles AS d
            JOIN user_details AS ud ON ud.user_id = d.user_id
            WHERE d.user_id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            warn!("doctor profile for user {user_id} does not exist");
            DatabaseError::RowNotFound.into()
        }
        e => {
            error!(
                "Error while fetching doctor_profile for {}: {:?}",
                user_id, e
            );
            AppError::InternalError
        }
    })?;

    if profile.approved_at.is_none() {
        return Err(AppError::NotLicensed);
    }

    let locations = query_as!(
        DoctorPracticeLocation,
        "SELECT * FROM doctor_practice_locations WHERE doctor_id = $1",
        profile.doctor_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            warn!("doctor profile for user {user_id} does not exist");
            DatabaseError::RowNotFound.into()
        }
        e => {
            error!(
                "Error while fetching doctor_practice_location for {}: {:?}",
                profile.doctor_id, e
            );
            AppError::InternalError
        }
    })?;

    Ok(Json(DoctorProfilePublic {
        doctor_id: profile.doctor_id,
        user_id: profile.user_id,
        name: profile.name,
        created_at: profile.created_at,
        locations,
    }))
}

pub async fn set_doctor_profile(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> APIResult<(StatusCode, Json<Value>)> {
    let profile = query_as!(
        DoctorProfile,
        "SELECT * FROM doctor_profiles WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Error while fetching doctor profile for {}: {}", user_id, e);

        AppError::InternalError
    })?;

    if profile.is_some() {
        return Err(DatabaseError::ForeignKeyViolation.into());
    }

    query!("INSERT INTO doctor_profiles (user_id) VALUES ($1)", user_id,)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!(
                "Error while inserting doctor_profile for {}:
             {:?}",
                user_id, e
            );

            match e {
                sqlx::Error::Database(db_e) => {
                    if db_e.is_foreign_key_violation() {
                        DatabaseError::ForeignKeyViolation.into()
                    } else {
                        AppError::InternalError
                    }
                }
                _ => AppError::InternalError,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(
            json!({ "message": "Successfully created a temporary profile"
            }),
        ),
    ))
}

#[derive(Deserialize)]
pub struct PracticeAddressPayload {
    practice_permit: String,
    practice_address: String,
}

pub async fn add_doctor_practice_location(
    State(state): State<AppState>,
    doctor: LicensedUser,
    Json(PracticeAddressPayload {
        practice_permit,
        practice_address,
    }): Json<PracticeAddressPayload>,
) -> APIResult<(StatusCode, Json<Value>)> {
    query!(
        "INSERT INTO doctor_practice_locations (doctor_id, practice_permit, \
         practice_address) VALUES ($1, $2, $3)",
        doctor.doctor_id,
        practice_permit,
        practice_address
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while adding a practice address for {}: {}",
            doctor.doctor_id, e
        );

        AppError::InternalError
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "Successfully submitted a practice address" })),
    ))
}

pub async fn delete_doctor_practice_location(
    State(state): State<AppState>,
    doctor: LicensedUser,
    Path(location_id): Path<Uuid>,
) -> APIResult<(StatusCode, Json<Value>)> {
    let res = query!(
        "DELETE FROM doctor_practice_locations WHERE location_id = $1 AND \
         doctor_id = $2",
        location_id,
        doctor.doctor_id,
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!(
            "Error while deleting a practice address {} for {}: {}",
            location_id, doctor.doctor_id, e
        );

        AppError::InternalError
    })?;

    if res.rows_affected() == 0 {
        return Err(DatabaseError::RowNotFound.into());
    }

    Ok((
        StatusCode::OK,
        Json(
            json!({ "message": format!("Successfully deleted practice location with id {}", location_id) }),
        ),
    ))
}
