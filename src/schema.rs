//! Mapping of SQL to struct.
//!
//! The structs here simply models the `schema.sql` 1-to-1. It should be mainly
//! used for querying with sqlx. Mapping it to the business logic struct (e.g.
//! `FullConsultationRecord`) will be done elsewhere.

use crate::protocol::Nik;
use chrono::{DateTime, NaiveDate, Utc};
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
}

#[derive(Serialize)]
pub struct UserMeasurement {
    pub measurement_id: Uuid,
    pub user_id: Uuid,
    pub height_in_cm: f32,
    pub weight_in_kg: f32,
    pub measured_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DoctorProfile {
    pub doctor_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
}

#[derive(Serialize)]
pub struct DoctorPracticeLocation {
    pub location_id: Uuid,
    pub doctor_id: Uuid,
    pub practice_permit: String,
    pub practice_address: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "allergy_severity", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllergySeverity {
    Mild,
    Moderate,
    Severe,
    AnaphylacticShock,
}

#[derive(Serialize, Deserialize)]
pub struct Allergy {
    pub allergy_id: Uuid,
    pub user_id: Uuid,
    pub allergen: String,
    pub severity: AllergySeverity,
}

// TODO map device_id to public_key in an lru cache
// for now its fine not to have a cache, reconsider this if you're scaling up
#[derive(Serialize)]
pub struct DeviceKey {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub public_key_pem: String,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct Medicine {
    pub medicine_id: Uuid,
    pub name: String,
    pub dosage_form: String,
    pub composition_notes: String,
}

pub struct MedicineIngredient {
    medicine_ingredient_id: Uuid,
    medicine_id: Uuid,
    ingredient: String,
    dosage_in_mg: i32,
}

#[derive(Serialize)]
pub struct Purchase {
    pub purchase_id: Uuid,
    pub user_id: Uuid,
    pub medicine_id: Uuid,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct Consultation {
    pub consultation_id: Uuid,
    pub doctor_id: Uuid,
    pub user_id: Uuid,
    pub location_id: Uuid,
    pub symptoms: String,
    pub created_at: DateTime<Utc>,
    pub reminded: bool,
}

#[derive(Serialize)]
pub struct Diagnosis {
    pub diagnosis_id: Uuid,
    pub consultation_id: Uuid,
    pub diagnosis: String,
    pub severity: String,
}

#[derive(Serialize)]
pub struct Prescription {
    pub prescription_id: Uuid,
    pub consultation_id: Uuid,
    pub drug_name: String,
    pub doses_in_mg: f64,
    pub regimen_per_day: f64,
    pub quantity_per_dose: f64,
    pub instruction: String,
    pub purchased_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct MedicalCondition {
    pub condition_id: Uuid,
    pub user_id: Uuid,
    pub condition: String,
}
