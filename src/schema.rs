//! Mapping of SQL to struct.
//!
//! The structs here simply models the `schema.sql` 1-to-1. It should be mainly
//! used for querying with sqlx. Mapping it to the business logic struct (e.g.
//! `FullConsultationRecord`) will be done elsewhere.

use crate::protocol::Nik;
use chrono::{DateTime, NaiveDate, Utc};
use ed25519_compact::PublicKey;
use uuid::Uuid;

struct User {
    user_id: Nik,
    email: String,
    password_hash: String,
}

struct UserDetail {
    user_id: Nik,
    name: String,
    dob: NaiveDate,
    gender: char,
    height_in_cm: f64,
    weight_in_kg: f64,
}

struct Allergy {
    allergy_id: Uuid,
    user_id: Nik,
    allergy: String,
}

// TODO map device_id to public_key in an lru cache
struct DeviceKey {
    device_id: Uuid,
    user_id: Nik,
    public_key: PublicKey,
    revoked: bool,
}

struct Record {
    record_id: Uuid,
    user_id: Uuid,
}

struct Consultation {
    consultation_id: Uuid,
    doctor_id: Uuid,
    record_id: Uuid,
    diagnoses: Vec<Diagnosis>,
    symptoms: Vec<Symptom>,
}

struct Diagnosis {
    diagnosis_id: Uuid,
    consultation_id: Uuid,
    diagnosis: String,
}

struct Symptom {
    symptom_id: Uuid,
    consultation_id: Uuid,
    symptom: String,
}

struct Prescription {
    prescription_id: Uuid,
    record_id: Uuid,
    drug_name: String,
    doses_in_mg: u64,
    regiment_per_day: u8,
    quantity_per_dose: u8,
    instruction: String,
}
