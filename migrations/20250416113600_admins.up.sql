CREATE TABLE admins (
    user_id UUID PRIMARY KEY REFERENCES users(user_id),
    promoted_by UUID NOT NULL REFERENCES admins(user_id),
    promoted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
