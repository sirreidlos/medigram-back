CREATE TABLE medical_conditions (
    condition_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    condition TEXT NOT NULL
);
