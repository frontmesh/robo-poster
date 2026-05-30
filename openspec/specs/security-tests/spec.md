# Security Fixes + Test Coverage Specification

## Goal
Fix critical security vulnerabilities and establish test coverage for core business logic.

## Requirements

### JWT Security
- Tokens must be signed with HMAC-SHA256 using the application secret
- Tokens must have 3 parts: header.payload.signature (standard JWT format)
- `AuthUser::from_token` must verify the signature before trusting the payload
- Unsigned or tampered tokens must be rejected with Unauthorized

### Test Coverage
- All pure functions in auth module must have unit tests
- Meta API client helpers must have unit tests
- Error module status code mapping must be verified
- Post status logic must be tested

### Cleanup
- Remove empty poster-api/ directory from repository

## Success Criteria
- JWT tokens are signed and verifiable
- Forged/unsigned tokens are rejected
- 28 unit tests pass
- No regressions in existing functionality
