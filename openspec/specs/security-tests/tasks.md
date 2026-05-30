# Security Fixes + Test Coverage Tasks

## Phase 1: Cleanup
- [ ] Remove empty poster-api/ directory

## Phase 2: JWT Fix
- [ ] Add hmac + sha2 crates to Cargo.toml
- [ ] Fix create_jwt to compute HMAC signature
- [ ] Fix from_token to verify signature
- [ ] Update token format to 3 parts

## Phase 3: Test Infrastructure
- [ ] Add dev-dependencies to Cargo.toml
- [ ] Create tests/ directory structure

## Phase 4: Auth Tests (6 tests)
- [ ] hash_password_deterministic
- [ ] hash_password_unique
- [ ] verify_password_correct
- [ ] verify_password_wrong
- [ ] verify_password_tampered
- [ ] create_jwt_structure

## Phase 5: Middleware Tests (6 tests)
- [ ] from_token_valid
- [ ] from_token_expired
- [ ] from_token_no_dots
- [ ] from_token_too_many_parts
- [ ] from_token_bad_payload
- [ ] from_token_missing_exp

## Phase 6: Meta Client Tests (8 tests)
- [ ] get_base_url_threads
- [ ] get_base_url_instagram
- [ ] get_base_url_unknown
- [ ] create_container_text
- [ ] create_container_image
- [ ] create_container_video
- [ ] publish_endpoint_instagram
- [ ] publish_endpoint_threads

## Phase 7: Posts + Error Tests (8 tests)
- [ ] status_draft
- [ ] status_scheduled
- [ ] platform_default
- [ ] platform_explicit
- [ ] unauthorized_401
- [ ] not_found_404
- [ ] bad_request_400
- [ ] meta_api_502
