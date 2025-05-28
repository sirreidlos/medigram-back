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
    auth::AuthUser,
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
        "SELECT d.doctor_id, d.user_id, d.created_at, ud.name
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
        "SELECT d.doctor_id, d.user_id, d.created_at, ud.name
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

// !TODO: handle this, prob by adding a new admin role that can assign admin to
// other user or something
// pub async fn set_doctor_profile(
//     State(state): State<AppState>,
//     AuthUser { user_id, .. }: AuthUser,
//     Json(DoctorProfilePayload {
//         practice_permit,
//         practice_address,
//     }): Json<DoctorProfilePayload>,
// ) -> APIResult<(StatusCode, Json<Value>)> {
//     query!(
//         "INSERT INTO doctor_profiles (user_id, practice_permit, \
//          practice_address, approved) VALUES ($1, $2, $3, $4)",
//         user_id,
//         practice_permit,
//         practice_address,
//         false
//     )
//     .execute(&state.db_pool)
//     .await
//     .map_err(|e| {
//         error!(
//             "Error while inserting doctor_profile for {}:
//              {:?}",
//             user_id, e
//         );

//         match e {
//             sqlx::Error::Database(db_e) => {
//                 if db_e.is_foreign_key_violation() {
//                     DatabaseError::ForeignKeyViolation.into()
//                 } else {
//                     AppError::InternalError
//                 }
//             }
//             _ => AppError::InternalError,
//         }
//     })?;

//     Ok((
//         StatusCode::OK,
//         Json(json!({ "message": "Successfully submitted your application"
// })),     ))
// }
