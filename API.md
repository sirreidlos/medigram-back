

# Medigram API Documentation

## Preface
Routes with authorization middleware layered on top will be marked with 🔒. 
Routes that allows the user of a verified practitioner to access will be marked with ⚕️ (assuming they are connected). 

Please add `Authorization: Bearer <SESSION_ID>` to the request's header.

# User Auth

## `POST /register`
Creates `user` object. The `/register` endpoint only creates the user, so a followup request will have to be done by the client for authentication through `/login`.

### Request
```json
{
  "email": "test@example.com",
  "password": "abcde"
}
```

### Response (Success)
`201 Created`
```json
{"message":"registration successful"}
```

### Response (Duplicate email)
`409 Conflict`
```json
{"error":"Email has been registered previously"}
```

## `POST /login`
Logs in to retrieve authorization information. Please immediately save the `session_id` and always attach it to every [AUTH] endpoints, `device-id` and always send it as a payload for any endpoint that requires it, `private-key` in a safe storage for signing consents. Note that `private-key` is encoded in base64.

### Request
```json
{
  "email": "test@example.com",
  "password": "abcde"
}
```

### Response (Success)
`200 OK`
```json
{
  "session_id":"xgsY0ovfKCqpfLHfCZCSaI0AVHt2e6Xnv76VyvXsyJVsKsu89UjdDEWIU9k7IGmc",
  "token_type":"Bearer",
  "device_id":"19553e8e-b9bb-4af6-b73a-448e01103125",
  "private_key": "y0eJbsKqY7so2gNwAQ0M0ZlM0... [PRIVATE KEY IN BASE64 STRING]"
}
```

### Response (User not found)
`404 Not Found`
```json
{"error":"User not found"}
```

## `POST /logout` 🔒
### Request
```json
{"device_id": "0b3158b5-0b08-4095-9773-c4618e63abbf"}
```
### Response
`200 OK`
```json
{"message":"logged out"}
```

# User Information

## `GET /user` 🔒
### Response
`200 OK`
```json
{
  "user_id":"e63a8be8-b200-4a0f-89d0-44797ff1c9d3",
  "email":"test@example.com"
}
```

## `GET /user-detail` 🔒

### Response (User data not found)
This scenario may happen when the user has just registered their email but hasn't proceeded with filling their data (e.g. closing the app after registration)

`404 Not Found`
```json
{"error":"Row does not exist in the database"}
```

### Response (Success)
`200 OK`
```json
{
  "user_id":"47945790-d358-42e2-aa88-c43f4cb28985",
  "nik":9999999999999999,
  "name":"test_user",
  "dob":"2025-03-04",
  "gender":"M"
}
```

## `PUT /user-detail` 🔒
### Request
```json
{
  "nik": 9999999999999999,
  "name": "test_user",
  "dob": "2025-03-04",
  "gender": "M"
}
```

### Response
`201 Created`
```json
{"message":"Successfully set user detail"}
```



## `POST /allergy` 🔒
### Request
```json
{
  "allergen": "beans",
  "severity": "MODERATE"
}
```
### Response
`201 Created`
```json
{"message":"allergy added"}
```

## `GET /allergy` 🔒
### Response (empty)
`200 OK`
```json
[]
```

### Response
`200 OK`
```json
[
  {
    "allergy_id":"242000eb-c263-4041-a040-b3684095694e",
    "user_id":"94f1ebd4-e817-47bc-920b-675d074d294b",
    "allergen":"beans",
    "severity":"MODERATE"
  }
]
```

## `DELETE /allergy` 🔒
### Request
```json
{
  "allergy_id":"242000eb-c263-4041-a040-b3684095694e",
}
```

### Response (Success)
`200 OK`
```json
{"message":"allergy removed"}
```

### Response (Allergy not found)
`404 Not Found`
```json
{"error":"Row does not exist in the database"}
```

## `POST /user-measurement` 🔒
### Request
```json
{
  "height_in_cm": 172.32,
  "weight_in_kg" :52.00
}
```

### Request (With time annotation in ISO 8601 format)
```json
{
  "height_in_cm": 172.32,
  "weight_in_kg" :52.00,
  "measured_at": "2025-03-08T20:11:16Z"
}
```
### Response
`201 Created`
```json
{"message":"Successfully added user measurement"}
```

## `GET /user-measurement` 🔒

### Response
```json
[
  {
    "measurement_id":"c80d7ace-f102-4754-805a-2332a865812c",
    "user_id":"47945790-d358-42e2-aa88-c43f4cb28985",
    "height_in_cm":172.32,
    "weight_in_kg":52.0,
    "measured_at":"2025-03-08T20:13:17.487958Z"
  },
  {
    "measurement_id":"d099f41a-3014-4231-a52c-1149475873c1",
    "user_id":"47945790-d358-42e2-aa88-c43f4cb28985",
    "height_in_cm":172.32,
    "weight_in_kg":52.0,
    "measured_at":"2025-03-08T20:11:16Z"
  }
]
```

### Response (empty)
`200 OK`
```json
[]
```

# Doctor
## `GET /doctor-profile/{doctor_id}` 🔒
### Request
```
GET /doctor-profile/23b41c6a-88a9-465f-abf6-4b2b318f1a0c
```

### Response
`200 OK`
```json
{
  "doctor_id":"23b41c6a-88a9-465f-abf6-4b2b318f1a0c",
  "user_id":"47945790-d358-42e2-aa88-c43f4cb28985",
  "practice_permit":"XXXXXXXXX-XXXXXXX",
  "practice_address":"Jalan Mawar No. 45, Kelurahan Sukamaju, Kecamatan Mentari, Jakarta 10110, Indonesia",
  "approved":true,
  "approved_at":"2025-03-09T03:00:09Z"
}
```

### Response (not found)
`404 Not Found`
```json
{"error":"Row does not exist in the database"}
```

# Consultation
## `POST /consultation` 🔒 (ONLY ⚕️)
Before requesting, do remember to request for a nonce through `GET /request-nonce`. After that, serialize the `device_id` and the `nonce` with a canonical JSON format, something like:
```json
"[\"862f034f-c705-48ff-bd0e-3a239c6c575e\",[59,227,41,67,102,181,171,84,15,176,74,12,137,163,111,222]]"
```
Afterwards, the patient signs it with their private key corresponding to the `device_id`.

### Request
```json
{
  "consent": {
    "signer_device_id": "862f034f-c705-48ff-bd0e-3a239c6c575e",
    "nonce": [59, 227, 41, 67, 102, 181, 171, 84, 15, 176, 74, 12, 137, 163, 111, 222],
    "signature": "lzfJ8534rZ2f4m0CMdxE5T0emdiV3AERgxYk1q7NGUz+leM/7rgzCyVXCjjXBc8cX4P236h1bjEJ0w7oHVPzCg=="
  },
  "user_id": "41676bb2-8561-47fe-9271-4c7e89defa7c",
  "diagnoses": [
    {
      "diagnosis": "Common Cold",
      "icd_code": "J00",
      "severity": "MILD"
    }
  ],
  "symptoms": ["runny nose", "coughing"],
  "prescriptions": [
    {
      "drug_name": "Paracetamol",
      "doses_in_mg": 500,
      "regimen_per_day": 3,
      "quantity_per_dose": 1,
      "instruction": "Take after meals with a full glass of water."
    }
  ]
}

```

### Response
`201 Created`
```json
{"message":"consultation record added"}
```

## `GET /consultation` 🔒⚕️

### Response
`200 OK`
```json
[
  {
    "consultation_id":"51df7e84-7d5a-492f-9eb3-ace107ca66ec",
    "doctor_id":"23b41c6a-88a9-465f-abf6-4b2b318f1a0c",
    "user_id":"41676bb2-8561-47fe-9271-4c7e89defa7c"
  }
]
```

### Response (empty)
`200 OK`
```json
[]
```

## `GET /diagnosis/{consultation_id}` 🔒⚕️

### Request
```
GET /diagnosis/51df7e84-7d5a-492f-9eb3-ace107ca66ec
```

### Response
`200 OK`
```json
[
  {
    "diagnosis_id":"f9077ba9-b329-4dbd-926e-4f574ccaf9f5",
    "consultation_id":"51df7e84-7d5a-492f-9eb3-ace107ca66ec",
    "diagnosis":"Common Cold",
    "icd_code":"J00",
    "severity":"MILD"
  }
]
```

### Response (different user)
`403 Forbidden`
```json
{"error":"You are not allowed to request for this"}
```

## `GET /symptom/{consultation_id}` 🔒⚕️

### Request
```
GET /symptom/51df7e84-7d5a-492f-9eb3-ace107ca66ec
```

### Response
`200 OK`
```json
[
  {
    "symptom_id":"b5a5840d-7168-4471-8fef-121859d472d1",
    "consultation_id":"51df7e84-7d5a-492f-9eb3-ace107ca66ec",
    "symptom":"runny nose"
  },
  {
    "symptom_id":"20337e92-f192-4ad0-8a78-391bc1e74d65",
    "consultation_id":"51df7e84-7d5a-492f-9eb3-ace107ca66ec",
    "symptom":"coughing"
  }
]
```

## `GET /prescription/{consultation_id}` 🔒⚕️

### Request
```
GET /prescription/51df7e84-7d5a-492f-9eb3-ace107ca66ec
```

### Response
`200 OK`
```json
[
  {
    "prescription_id":"e4b5ac40-d899-4f73-b52c-683b7a73639c",
    "consultation_id":"51df7e84-7d5a-492f-9eb3-ace107ca66ec",
    "drug_name":"Paracetamol",
    "doses_in_mg":500,
    "regimen_per_day":3,
    "quantity_per_dose":1,
    "instruction":"Take after meals with a full glass of water."
  }
]
```
