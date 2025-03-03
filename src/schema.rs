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
    pub user_id: Uuid,
    pub nik: Nik,
    pub name: String,
    pub dob: NaiveDate,
    pub gender: char,
    pub height_in_cm: f64,
    pub weight_in_kg: f64,
}

pub struct Allergy {
    pub allergy_id: Uuid,
    pub user_id: Uuid,
    pub allergy: String,
}

// TODO map device_id to public_key in an lru cache
// for now its fine not to have a cache, reconsider this if you're scaling up
pub struct DeviceKey {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub public_key_pem: String,
    pub revoked: bool,
}

pub struct Record {
    pub record_id: Uuid,
    pub user_id: Uuid,
}

pub struct Consultation {
    pub consultation_id: Uuid,
    pub doctor_id: Uuid,
    pub record_id: Uuid,
    pub diagnoses: Vec<Diagnosis>,
    pub symptoms: Vec<Symptom>,
}

pub struct Diagnosis {
    pub diagnosis_id: Uuid,
    pub consultation_id: Uuid,
    pub diagnosis: String,
}

pub struct Symptom {
    pub symptom_id: Uuid,
    pub consultation_id: Uuid,
    pub symptom: String,
}

pub struct Prescription {
    pub prescription_id: Uuid,
    pub record_id: Uuid,
    pub drug_name: String,
    pub doses_in_mg: u64,
    pub regiment_per_day: u8,
    pub quantity_per_dose: u8,
    pub instruction: String,
}
