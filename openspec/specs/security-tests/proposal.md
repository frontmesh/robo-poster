# Security Fixes + Test Coverage Proposal

## Why
The codebase has zero test coverage and a critical security vulnerability: JWT tokens are not signed. The `create_jwt` function accepts a `secret` parameter but never uses it, producing unsigned tokens that anyone can forge. Additionally, no unit tests exist to verify core business logic.

## What Changes
- Fix JWT signing with HMAC-SHA256
- Fix JWT validation to verify signatures
- Add test infrastructure (dev-dependencies, test directory)
- Add unit tests for security-critical and core logic
- Remove empty poster-api/ directory

## Scope
- JWT fix: HMAC-SHA256 signing + verification
- 28 unit tests across auth, meta, posts, and error modules
- Cleanup: remove empty poster-api/ directory

## Out of Scope
- Full integration tests (require database setup)
- Performance/load testing
- E2E testing
- Premium service tests (separate repo)
