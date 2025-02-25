-- PostgreSQL schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id DECIMAL(16) PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
);

CREATE TABLE user_details {
    user_id DECIMAL(16) PRIMARY KEY REFERENCES users(user_id),
    name TEXT NOT NULL,
    dob DATE NOT NULL,
    gender CHAR(1) NOT NULL,
    height_in_cm DOUBLE NOT NULL,
    weight_in_kg DOUBLE NOT NULL,
}

CREATE TABLE doctor_profile (
    doctor_profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id DECIMAL(16) REFERENCES users(user_id) NOT NULL,
    practice_permit TEXT NOT NULL,
    practice_address TEXT NOT NULL, -- Maybe separate the address into smaller units?
);

CREATE TABLE allergies (
    allergy_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id DECIMAL(16) REFERENCES users(user_id) NOT NULL,
    allergy TEXT NOT NULL,
);

CREATE TABLE device_key (
    device_id UUID  PRIMARY KEY NOT NULL,
    user_id DECIMAL(16) REFERENCES users(user_id) NOT NULL,
    public_key_pem TEXT NOT NULL,
    revoked BOOLEAN NOT NULL,
);

CREATE TABLE record (
    record_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
);

CREATE TABLE consultations (
    consultation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    doctor_id UUID REFERENCES doctor_profile(user_id) NOT NULL,
    record_id UUID REFERENCES record(record_id) NOT NULL,
);

CREATE TABLE diagnoses (
    diagnosis_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    diagnosis TEXT NOT NULL,
);

CREATE TABLE symptoms (
    symptom_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    symptom TEXT NOT NULL,
);

CREATE TABLE prescriptions (
    prescription_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    record_id UUID REFERENCES record(record_id) NOT NULL,
    drug_name TEXT NOT NULL,
    doses_in_mg INT NOT NULL,
    regiment_per_day INT NOT NULL,
    quantity_per_dose INT NOT NULL,
    instruction TEXT NOT NULL,
);
