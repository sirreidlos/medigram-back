CREATE TABLE doctor_profiles (
    doctor_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
