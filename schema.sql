-- PostgreSQL schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL
);

CREATE TABLE user_details (
    user_id UUID PRIMARY KEY REFERENCES users(user_id),
    nik BIGINT NOT NULL,
    name TEXT NOT NULL,
    dob DATE NOT NULL,
    gender CHAR NOT NULL
);

CREATE TABLE user_measurements (
    measurement_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    height_in_cm REAL NOT NULL,
    weight_in_kg REAL NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE doctor_profiles (
    doctor_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    practice_permit TEXT NOT NULL,
    practice_address TEXT NOT NULL, -- Maybe separate the address into smaller units?
    approved BOOLEAN NOT NULL,
    approved_at TIMESTAMPTZ
);

CREATE TABLE allergies (
    allergy_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    allergy TEXT NOT NULL,
    severity INT NOT NULL
);

CREATE TABLE device_keys (
    device_id UUID  PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES users(user_id) NOT NULL,
    public_key_pem TEXT NOT NULL,
    revoked_at TIMESTAMPTZ
);

CREATE TABLE medicines (
    medicine_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    dosage_form TEXT NOT NULL,
    composition_notes TEXT
);

CREATE TABLE purchases (
    purchase_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    medicine_id UUID REFERENCES medicines(medicine_id) NOT NULL,
    quantity INT NOT NULL
);

CREATE TABLE medicine_ingredients (
    medicine_ingredient_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    medicine_id UUID REFERENCES medicines(medicine_id) NOT NULL,
    ingredient TEXT NOT NULL,
    dosage_in_mg INT NOT NULL
);

CREATE TABLE consultations (
    consultation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    doctor_id UUID REFERENCES doctor_profiles(doctor_id) NOT NULL
);

CREATE TABLE diagnoses (
    diagnosis_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    diagnosis TEXT NOT NULL,
    icd_code TEXT NOT NULL,
    severity TEXT NOT NULL
);

CREATE TABLE symptoms (
    symptom_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    symptom TEXT NOT NULL
);

CREATE TABLE prescriptions (
    prescription_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    drug_name TEXT NOT NULL,
    doses_in_mg INT NOT NULL,
    regimen_per_day INT NOT NULL,
    quantity_per_dose INT NOT NULL,
    instruction TEXT NOT NULL
);
