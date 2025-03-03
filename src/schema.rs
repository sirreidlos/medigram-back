//! Mapping of SQL to struct.
//!
//! The structs here simply models the `schema.sql` 1-to-1. It should be mainly
//! used for querying with sqlx. Mapping it to the business logic struct (e.g.
//! `FullConsultationRecord`) will be done elsewhere.

use crate::protocol::Nik;
use chrono::{DateTime, NaiveDate, Utc};
use ed25519_compact::PublicKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: String,
}

#[derive(Serialize)]
pub struct UserDetail {
    pub user_id: Uuid,
    pub nik: Nik,
    pub name: String,
    pub dob: NaiveDate,
    pub gender: char,
    pub height_in_cm: f32,
    pub weight_in_kg: f32,
}

#[derive(Serialize)]
pub struct DoctorProfile {
    pub doctor_profile_id: Uuid,
    pub user_id: Uuid,
    pub practice_permit: String,
    pub practice_address: String,
    pub approved: bool,
    pub approved_time: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct Allergy {
    pub allergy_id: Uuid,
    pub user_id: Uuid,
    pub allergy: String,
}

// TODO map device_id to public_key in an lru cache
// for now its fine not to have a cache, reconsider this if you're scaling up
#[derive(Serialize)]
pub struct DeviceKey {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub public_key_pem: String,
    pub revoked: bool,
}

#[derive(Serialize)]
pub struct Record {
    pub record_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Serialize)]
pub struct Consultation {
    pub consultation_id: Uuid,
    pub doctor_id: Uuid,
    pub record_id: Uuid,
    pub diagnoses: Vec<Diagnosis>,
    pub symptoms: Vec<Symptom>,
}

#[derive(Serialize)]
pub struct Diagnosis {
    pub diagnosis_id: Uuid,
    pub consultation_id: Uuid,
    pub diagnosis: String,
}

#[derive(Serialize)]
pub struct Symptom {
    pub symptom_id: Uuid,
    pub consultation_id: Uuid,
    pub symptom: String,
}

#[derive(Serialize)]
pub struct Prescription {
    pub prescription_id: Uuid,
    pub record_id: Uuid,
    pub drug_name: String,
    pub doses_in_mg: u64,
    pub regiment_per_day: u8,
    pub quantity_per_dose: u8,
    pub instruction: String,
}
