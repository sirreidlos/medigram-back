INSERT INTO users (user_id, email, password_hash)
VALUES
    ('d3969164-86ea-442d-a589-79de89116f9c', 'alice@example.com', '$argon2id$v=19$m=19456,t=2,p=1$IICbY2zraHSN1biU03ZTYA$YcdL6uN+9Tzj+b11aDyazK+R7yQE6ZF8HNC2xdzdYSQ'), -- password is `test`
    ('41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'bob@example.com', '$argon2id$v=19$m=19456,t=2,p=1$bkbxfmRSRD42B5miOJeRLw$GPcVK9JjMtug0eYXxusv8yntpW+utzy5haQZ7+4UMu0'), -- password is `test`
    ('080d497e-696b-423d-80f1-331014fb4bf4', 'admin@example.com',  '$argon2id$v=19$m=19456,t=2,p=1$7HFaoR1g/kX71TGNtxH0WQ$E5s/CXf/Xd59xhFuGPOgn5jkTybDEgqL4CBJzAygAsM');

INSERT INTO admins (user_id, promoted_by, promoted_at)
VALUES
    ('080d497e-696b-423d-80f1-331014fb4bf4', '080d497e-696b-423d-80f1-331014fb4bf4', '1970-01-01 00:00:00+00');

INSERT INTO user_details (user_id, nik, name, dob, gender)
VALUES
    ('d3969164-86ea-442d-a589-79de89116f9c', 1000000000000000, 'Alice', '1970-01-01', 'F'),
    ('41490144-e4e1-4d1f-9eb7-f90af81c12ce', 1000000000000001, 'Bob', '1970-01-02', 'M');

INSERT INTO user_measurements (measurement_id, user_id, height_in_cm, weight_in_kg, measured_at)
VALUES
    ('9440e19e-915f-44e5-9de5-27c1a29c2d98', 'd3969164-86ea-442d-a589-79de89116f9c', 123.45, 67.89, '1970-01-01 00:00:00+00'),
    ('8a4f27d6-bcff-414f-b63c-2627b8da2a55', 'd3969164-86ea-442d-a589-79de89116f9c', 125.03, 68.12, '1970-02-01 00:00:00+00'),
    ('893d929e-091c-47bd-8ae4-5388c761aba1', 'd3969164-86ea-442d-a589-79de89116f9c', 126.45, 63.33, '1970-03-01 00:00:00+00'),
    ('d38c1ca7-b7bc-437d-add3-afc0d34cf8ba', '41490144-e4e1-4d1f-9eb7-f90af81c12ce', 126.45, 63.33, '1970-03-01 00:00:00+00');

INSERT INTO allergies (allergy_id, user_id, allergen, severity)
VALUES
    ('f7769edf-b06b-4749-b6ff-d91efcca8403', 'd3969164-86ea-442d-a589-79de89116f9c', 'pollen', 'MILD'),
    ('ec9045b1-67b8-4de5-9e43-67178760a622', 'd3969164-86ea-442d-a589-79de89116f9c', 'peanuts', 'SEVERE'),
    ('51f3a760-7151-4e4b-8ebd-fc6505fd04bf', '41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'peanuts', 'SEVERE');

INSERT INTO doctor_profiles (doctor_id, user_id, created_at, approved_at, approved_by)
VALUES
    ('a5ca9dee-89b4-4228-aff5-506b995f3b42', 'd3969164-86ea-442d-a589-79de89116f9c', '1970-03-01 00:00:00+00', '1970-03-01 00:00:00+00', '080d497e-696b-423d-80f1-331014fb4bf4');

INSERT INTO doctor_practice_locations (location_id, doctor_id, practice_permit, practice_address, approved_at, approved_by, created_at)
VALUES
    ('fbc0a545-f266-495d-91a1-667479a13ace', 'a5ca9dee-89b4-4228-aff5-506b995f3b42', '420/SIP-001/Dinkes/I/2025', 'Jl. Raya Kb. Jeruk No.27, RT.1/RW.9, Kemanggisan, Kec. Palmerah, Kota Jakarta Barat, Daerah Khusus Ibukota Jakarta 11530', '1970-03-04 00:00:00+00', '080d497e-696b-423d-80f1-331014fb4bf4', '1970-03-01 00:00:00+00');

INSERT INTO consultations (consultation_id, user_id, doctor_id, location_id, symptoms, created_at, reminded)
VALUES
    ('052059a6-abd6-4fe7-bf04-a0b39b5af792', '41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'a5ca9dee-89b4-4228-aff5-506b995f3b42', 'fbc0a545-f266-495d-91a1-667479a13ace', 'increased thirst, frequent urination, headache, dizziness', '2025-05-24T05:49:16+0000', false),
    ('8be82641-92d8-4ab1-8018-dfd9d664410c', '41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'a5ca9dee-89b4-4228-aff5-506b995f3b42', 'fbc0a545-f266-495d-91a1-667479a13ace', 'sore throat', '2025-05-24T05:49:55+0000', false);

INSERT INTO diagnoses (diagnosis_id, consultation_id, diagnosis, severity)
VALUES
    ('9eedff3a-ea54-4fc6-a0ed-2a45886e3570', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Type 2 diabetes mellitus without complications', 'moderate'),
    ('7ccaa2dd-6819-4f10-9b72-d212a3e256e7', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Essential (primary) hypertension', 'mild'),
    ('e185312b-aa90-4859-bded-36dca51f7e9b', '8be82641-92d8-4ab1-8018-dfd9d664410c', 'Acute bronchitis', 'moderate');


INSERT INTO prescriptions (prescription_id, consultation_id, drug_name, doses_in_mg, regimen_per_day, quantity_per_dose, instruction, purchased_at)
VALUES
    ('0a4ac6fe-42f4-4398-80e9-a6f718c01273', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Amoxicillin', 500, 3, 1, 'Take one capsule every 8 hours after meals.', '2025-05-20 10:30:00+00'),
    ('93ffcb11-6d32-47fd-9416-ecc34106944c', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Paracetamol', 650, 4, 1, 'Take one tablet every 6 hours as needed for pain.', '2025-05-22 14:45:00+00'),
    ('633344c3-ba0a-4fa1-8934-0bbd1e6b2ff3', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Atorvastatin', 10, 1, 1, 'Take one tablet daily at bedtime.', NULL),
    ('3c23c3ea-e43b-446a-9186-874be3edb30d', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Lisinopril', 20, 1, 1, 'Take once daily in the morning with water.', '2025-05-21 09:00:00+00'),
    ('cd8b1af2-f309-45cf-8d37-1c45366185cd', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Metformin', 500, 2, 1, 'Take after breakfast and dinner.', NULL),
    ('e06eda28-2946-4aa2-b7d5-24ae6c55c634', '052059a6-abd6-4fe7-bf04-a0b39b5af792', 'Ibuprofen', 400, 3, 1, 'Take every 8 hours with food for inflammation.', '2025-05-24 17:20:00+00'),
    ('af55575b-6e89-431a-8ebf-63b2b19640e6', '8be82641-92d8-4ab1-8018-dfd9d664410c', 'Cetirizine', 10, 1, 1, 'Take once daily for allergy symptoms.', '2025-05-19 08:15:00+00'),
    ('be8f88e7-9099-4d57-9509-1afaf6965ba4', '8be82641-92d8-4ab1-8018-dfd9d664410c', 'Omeprazole', 20, 1, 1, 'Take 30 minutes before breakfast.', '2025-05-18 07:45:00+00'),
    ('1247f844-7327-4604-9962-ad735fb9d557', '8be82641-92d8-4ab1-8018-dfd9d664410c', 'Azithromycin', 250, 1, 2, 'Take two tablets daily for 3 days.', '2025-05-23 11:00:00+00'),
    ('7b018e91-cc6d-44c5-a15f-38b7cf700af3', '8be82641-92d8-4ab1-8018-dfd9d664410c', 'Diazepam', 5, 2, 1, 'Take one tablet in the morning and one at night.', NULL);


INSERT INTO medical_conditions (condition_id, user_id, condition)
VALUES
    ('0c29c9ab-a1aa-48eb-b43f-b447d70b6428' ,'d3969164-86ea-442d-a589-79de89116f9c', 'Hypertension'),
    ('aec6b9e5-69c6-4d7a-afdd-fa87cb151f7d' ,'d3969164-86ea-442d-a589-79de89116f9c', 'Type 2 Diabetes'),
    ('45c67bd2-837f-43fa-bd4c-9f51eec2d8df' ,'d3969164-86ea-442d-a589-79de89116f9c', 'Asthma'),
    ('51779557-1cb9-42e8-8748-060294bba4e8' ,'41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'Chronic Migraine'),
    ('75aa86a8-e96d-49d0-8345-641209fc63d9' ,'41490144-e4e1-4d1f-9eb7-f90af81c12ce', 'Anxiety Disorder');
