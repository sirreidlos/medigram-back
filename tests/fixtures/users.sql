INSERT INTO users (user_id, email, password_hash)
VALUES
    ('d3969164-86ea-442d-a589-79de89116f9c', 'alice@example.com', '$argon2id$v=19$m=19456,t=2,p=1$IICbY2zraHSN1biU03ZTYA$YcdL6uN+9Tzj+b11aDyazK+R7yQE6ZF8HNC2xdzdYSQ'), -- password is `test`
    ('41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'bob@example.com', '$argon2id$v=19$m=19456,t=2,p=1$bkbxfmRSRD42B5miOJeRLw$GPcVK9JjMtug0eYXxusv8yntpW+utzy5haQZ7+4UMu0'), -- password is `test`
    ('080d497e-696b-423d-80f1-331014fb4bf4', 'admin@example.com',  '$argon2id$v=19$m=19456,t=2,p=1$7HFaoR1g/kX71TGNtxH0WQ$E5s/CXf/Xd59xhFuGPOgn5jkTybDEgqL4CBJzAygAsM');

INSERT INTO admins (user_id, promoted_by, promoted_at)
VALUES
    ('080d497e-696b-423d-80f1-331014fb4bf4', '080d497e-696b-423d-80f1-331014fb4bf4', '1970-01-01 00:00:00+00');
