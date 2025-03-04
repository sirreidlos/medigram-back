# Medigram API Documentation

## Preface
Routes with authorization middleware layered on top will be marked with [AUTH]. Please add `Authorization: Bearer SESSION_ID` to the request's header.

# User Auth

## `POST /register`
Creates `user` object.

### Request
```json
{
  "email": "test@example.com",
  "password": "abcde"
}
```

### Response
`201 Created`
```json
{"message":"registration successful"}
```

## `POST /login`
Logs in to retrieve authorization information.

### Request
```json
{
  "email": "test@example.com",
  "password": "abcde"
}
```

### Response
`200 OK`
```json
{
      "access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI5NGYxZWJkNC1lODE3LTQ3YmMtOTIwYi02NzVkMDc0ZDI5NGIiLCJleHAiOjE3NDEwNzY5MjUsImlhdCI6MTc0MTA3NjAyNX0.WJOyZJvzV229NIEnH2NBfkZY3OLCng1rHD0zJ9xYogM",
      "session_id":"DCsjiQuLS8KEqouYmklnps4FkjIFrgQkzK4qbgEKZUCQpcypuXL3DvABoiPglIPw",
      "token_type":"Bearer",
      "expires_in":900,
      "device_id":"0f5273ae-d191-40aa-8974-c98331e3d5eb",
      "private_key": "y0eJbsKqY7so2gNwAQ0M0ZlM0... [PRIVATE KEY IN BASE64 STRING]"
}
```
## `POST /logout` [AUTH]
TODO

# User Information

## `GET /user` [AUTH]
### Response
`200 OK`
```json
{
  "user_id":"94f1ebd4-e817-47bc-920b-675d074d294b",
  "email":"test@example.com"
}
```

## `GET /user-detail` [AUTH]
### Response
`200 OK`
```json
{
  "user_id":"94f1ebd4-e817-47bc-920b-675d074d294b",
  "nik":9999999999999999,
  "name":"test_user",
  "dob":"2025-03-04",
  "gender":"M",
  "height_in_cm":15.0,
  "weight_in_kg":10.0
}
```

## `PUT /user-detail` [AUTH]
### Request
```json
{
  "nik": 9999999999999999,
  "name": "test_user",
  "dob": "2025-03-04",
  "gender": "M",
  "height_in_cm": 15,
  "weight_in_kg": 10
}
```

### Response
`201 Created`
```json
{"message":"Successfully set user detail"}
```

## `POST /allergy` [AUTH]
### Request
```json
{"allergy": "beans"}
```
### Response
`201 Created`
```json
{"message":"allergy added"}
```

## `GET /allergy` [AUTH]
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
    "allergy_id":"c4297b34-5329-4f80-aeab-0589a2e5b532",
    "user_id":"94f1ebd4-e817-47bc-920b-675d074d294b",
    "allergy":"beans"
  }
]
```

## `DELETE /allergy` [AUTH]
### Request
```json
{
  "allergy_id":"c4297b34-5329-4f80-aeab-0589a2e5b532",
  "user_id":"94f1ebd4-e817-47bc-920b-675d074d294b",
  "allergy":"beans"
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
{"error":"Resource does not exist in the database"}
```
