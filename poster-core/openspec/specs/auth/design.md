# Auth Design

## Architecture
- Axum handlers for register/login
- JWT tokens using base64-encoded header/payload
- Password hashing with blake3
- SQLx for PostgreSQL queries

## Database
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    license_key VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Flow
1. Register: hash password → insert user → return JWT
2. Login: fetch user → verify password → return JWT
3. Protected routes: extract token from Authorization header → validate → extract user_id

## Security
- Passwords never stored in plaintext
- JWT tokens expire after 24 hours
- No token refresh for MVP (re-login required)
