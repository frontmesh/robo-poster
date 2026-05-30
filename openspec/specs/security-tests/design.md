# Security Fixes + Test Coverage Design

## JWT Fix

### Current (Broken)
```
token = base64(header) + "." + base64(payload)
```
No signature, no verification.

### New (Secure)
```
token = base64(header) + "." + base64(payload) + "." + base64(hmac_sha256(secret, header.payload))
```

### Dependencies
- `hmac = "0.12"` — HMAC computation
- `sha2 = "0.10"` — SHA-256 hasher
- Both are widely used, well-audited crates

### Flow
1. `create_jwt`: compute HMAC over `header_b64.payload_b64`, append as signature
2. `from_token`: split into 3 parts, recompute HMAC, compare with provided signature
3. Only proceed if signature matches

## Test Infrastructure

### Rust Dev Dependencies
```toml
[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

### Test Structure
```
backend/tests/
├── auth_tests.rs
├── meta_tests.rs
├── posts_tests.rs
└── error_tests.rs
```

## Test Plan (28 Tests)

### Auth Module (6 tests)
1. `hash_password_deterministic` — same input = same output
2. `hash_password_unique` — different inputs = different outputs
3. `verify_password_correct` — verify(hash(pw), pw) = true
4. `verify_password_wrong` — verify(hash(pw), wrong) = false
5. `verify_password_tampered` — invalid base64 returns false
6. `create_jwt_structure` — 3 dot-separated base64url segments

### Auth Middleware (6 tests)
1. `from_token_valid` — extracts correct user_id
2. `from_token_expired` — returns Unauthorized
3. `from_token_no_dots` — returns Unauthorized
4. `from_token_too_many_parts` — returns Unauthorized
5. `from_token_bad_payload` — non-JSON returns Unauthorized
6. `from_token_missing_exp` — returns Unauthorized

### Meta Client (8 tests)
1. `get_base_url_threads` — returns threads.net URL
2. `get_base_url_instagram` — returns facebook.com URL
3. `get_base_url_unknown` — defaults to threads.net
4. `create_container_text` — TEXT form data correct
5. `create_container_image` — IMAGE includes image_url
6. `create_container_video` — VIDEO includes video_url
7. `publish_endpoint_instagram` — uses /media_publish
8. `publish_endpoint_threads` — uses /threads_publish

### Posts Module (4 tests)
1. `status_draft` — no scheduled_at = draft
2. `status_scheduled` — scheduled_at present = scheduled
3. `platform_default` — no platform = threads
4. `platform_explicit` — platform passed through

### Error Module (4 tests)
1. `unauthorized_401` — maps to 401
2. `not_found_404` — maps to 404
3. `bad_request_400` — maps to 400
4. `meta_api_502` — maps to 502
