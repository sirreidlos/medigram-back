CREATE TABLE purchases (
    purchase_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    medicine_id UUID REFERENCES medicines(medicine_id) NOT NULL,
    quantity INT NOT NULL,
    prescription_id UUID REFERENCES prescriptions(prescription_id)
);
