CREATE TABLE prescriptions (
    prescription_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID REFERENCES consultations(consultation_id) NOT NULL,
    drug_name TEXT NOT NULL,
    doses_in_mg DOUBLE PRECISION NOT NULL,
    regimen_per_day DOUBLE PRECISION NOT NULL,
    quantity_per_dose DOUBLE PRECISION NOT NULL,
    instruction TEXT NOT NULL,
    purchased_at TIMESTAMPTZ
);
