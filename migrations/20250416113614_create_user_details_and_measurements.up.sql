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
