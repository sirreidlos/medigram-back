--  Manually populate?
CREATE TABLE medicines (
    medicine_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    dosage_form TEXT NOT NULL,
    composition_notes TEXT
);

CREATE TABLE medicine_ingredients (
    medicine_ingredient_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    medicine_id UUID REFERENCES medicines(medicine_id) NOT NULL,
    ingredient TEXT NOT NULL,
    dosage_in_mg INT NOT NULL
);
