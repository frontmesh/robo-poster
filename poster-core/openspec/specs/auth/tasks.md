# Auth Tasks

## Phase 1: Database
- [x] Create users table migration
- [x] Add SQLx model for User

## Phase 2: Backend
- [x] Implement password hashing (blake3)
- [x] Implement register endpoint
- [x] Implement login endpoint
- [x] Add JWT token generation
- [x] Add error handling for auth
- [x] Add JWT validation middleware
- [x] Add password validation (8+ chars)
- [x] Handle duplicate email error
- [x] Apply auth middleware to protected routes

## Phase 3: Frontend
- [x] Add login page UI
- [x] Add register page UI
- [x] Store token in model
- [x] Add auth state management
- [x] Wire up login/register API calls
- [x] Add logout functionality
- [x] Pass token to API requests

## Phase 4: Testing
- [ ] Test register flow
- [ ] Test login flow
- [ ] Test invalid credentials
- [ ] Test duplicate email
