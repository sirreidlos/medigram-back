CREATE TYPE allergy_severity AS ENUM ('MILD', 'MODERATE', 'SEVERE', 'ANAPHYLACTIC_SHOCK');

CREATE TABLE allergies (
    allergy_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    allergen TEXT NOT NULL,
    severity allergy_severity NOT NULL
);
