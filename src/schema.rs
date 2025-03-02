//! Mapping of SQL to struct.
//!
//! The structs here simply models the `schema.sql` 1-to-1. It should be mainly
//! used for querying with sqlx. Mapping it to the business logic struct (e.g.
//! `FullConsultationRecord`) will be done elsewhere.

use crate::protocol::Nik;
use chrono::{DateTime, NaiveDate, Utc};
use ed25519_compact::PublicKey;
use uuid::Uuid;

pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: String,
}

pub struct UserDetail {
    user_id: Uuid,
    nik: Nik,
    name: String,
    dob: NaiveDate,
    gender: char,
    height_in_cm: f64,
    weight_in_kg: f64,
}

pub struct Allergy {
    allergy_id: Uuid,
    user_id: Uuid,
    allergy: String,
}

// TODO map device_id to public_key in an lru cache
pub struct DeviceKey {
    device_id: Uuid,
    user_id: Uuid,
    public_key: PublicKey,
    revoked: bool,
}

pub struct Record {
    record_id: Uuid,
    user_id: Uuid,
}

pub struct Consultation {
    consultation_id: Uuid,
    doctor_id: Uuid,
    record_id: Uuid,
    diagnoses: Vec<Diagnosis>,
    symptoms: Vec<Symptom>,
}

pub struct Diagnosis {
    diagnosis_id: Uuid,
    consultation_id: Uuid,
    diagnosis: String,
}

pub struct Symptom {
    symptom_id: Uuid,
    consultation_id: Uuid,
    symptom: String,
}

pub struct Prescription {
    prescription_id: Uuid,
    record_id: Uuid,
    drug_name: String,
    doses_in_mg: u64,
    regiment_per_day: u8,
    quantity_per_dose: u8,
    instruction: String,
}
