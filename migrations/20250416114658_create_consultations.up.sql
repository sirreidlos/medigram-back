CREATE TABLE consultations (
    consultation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    doctor_id UUID REFERENCES doctor_profiles(doctor_id) NOT NULL,
    location_id UUID REFERENCES doctor_practice_locations(location_id) NOT NULL,
    symptoms TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reminded BOOLEAN NOT NULL DEFAULT FALSE
);
