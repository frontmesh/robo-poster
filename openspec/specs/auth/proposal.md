# Auth Proposal

## Why
Every user needs to authenticate before managing their accounts and posts. Auth is the foundation for all other features.

## What Changes
- Add users table to database
- Add register/login endpoints
- Add JWT token generation and validation
- Add password hashing

## Scope
- Email + password authentication
- JWT tokens (24h expiry)
- No OAuth yet (Meta OAuth is separate, for account linking)
