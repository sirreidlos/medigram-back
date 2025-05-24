CREATE TABLE diagnoses (
    diagnosis_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    diagnosis TEXT NOT NULL,
    severity TEXT NOT NULL
);
