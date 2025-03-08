

# Medigram API Documentation

## Preface
Routes with authorization middleware layered on top will be marked with 🔒. Routes that require the user to be a verified will be marked with ⚕️. 

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

TODO: detail the purpose of `access-token`.

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
  "access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJlNjNhOGJlOC1iMjAwLTRhMGYtODlkMC00NDc5N2ZmMWM5ZDMiLCJleHAiOjE3NDE0NjE1NjksImlhdCI6MTc0MTQ2MDY2OX0.Q9MEqPlRCUJ9Q1Sv7TXl09KU3_4jkjd-RlevAeuwkZI",
  "session_id":"xgsY0ovfKCqpfLHfCZCSaI0AVHt2e6Xnv76VyvXsyJVsKsu89UjdDEWIU9k7IGmc",
  "token_type":"Bearer",
  "expires_in":900,
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
TODO

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

## `POST /user-measurement`
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

## `GET /user-measurement`
### Response (empty)
`200 OK`
```json
[]
```

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
