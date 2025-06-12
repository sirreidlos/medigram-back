CREATE TABLE doctor_practice_locations (
    location_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    doctor_id UUID NOT NULL REFERENCES doctor_profiles(doctor_id),
    practice_permit TEXT NOT NULL,
    practice_address TEXT NOT NULL, -- Maybe separate the address into smaller units?
    approved_at TIMESTAMPTZ, -- NULL if not yet approved
    approved_by UUID REFERENCES admins(user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
