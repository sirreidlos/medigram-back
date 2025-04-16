CREATE TABLE device_keys (
    device_id UUID  PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES users(user_id) NOT NULL,
    public_key_pem TEXT NOT NULL,
    revoked_at TIMESTAMPTZ
);
