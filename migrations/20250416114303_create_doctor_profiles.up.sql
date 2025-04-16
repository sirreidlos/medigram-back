CREATE TABLE doctor_profiles (
    doctor_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    practice_permit TEXT NOT NULL,
    practice_address TEXT NOT NULL, -- Maybe separate the address into smaller units?
    approved BOOLEAN NOT NULL, -- maybe just have approved_at as a signal that its done?
    approved_at TIMESTAMPTZ
);
