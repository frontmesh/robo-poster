# Auth Specification

## Goal
Enable secure user registration and login with JWT-based authentication.

## Requirements
- Users can register with email + password
- Password must be 8+ characters
- Users can login with email + password
- JWT token issued on successful login
- Token expires after 24 hours
- Protected routes require valid JWT token
- Passwords stored as salted hashes, never plaintext

## Out of Scope
- OAuth/social login (future)
- 2FA (future)
- Password reset (future)

## API Endpoints
- `POST /api/auth/register` - Create new user account
- `POST /api/auth/login` - Authenticate and receive JWT

## Success Criteria
- User can register and receive a token
- User can login with correct credentials
- Login fails with wrong credentials (401)
- Register fails with duplicate email (400)
- Token is required for all other API endpoints
