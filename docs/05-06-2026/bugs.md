# Known Bugs & Issues

**Date:** 05-06-2026
**Status:** Active tracking

---

## 🔴 Critical Bugs

### BUG-001: OAuth Callback Will Always Fail ✅ FIXED
**Severity:** Critical
**Location:** `backend/src/accounts/mod.rs`
**Status:** ✅ Fixed on 05-06-2026

**Description:**
The `callback()` handler required `auth: AuthUser` as a parameter, but Meta redirects to this URL without a Bearer token. The callback would always fail with 401 Unauthorized.

**Root Cause:**
OAuth redirects are browser-based (HTTP GET), not API calls with Authorization headers. The handler assumed it would receive a JWT token, which is impossible in this flow.

**Impact:**
Users could not connect Instagram or Threads accounts. The entire OAuth flow was broken.

**Fix:**
- Added `oauth_states: Mutex<HashMap<String, Uuid>>` to `AppState`
- `connect()` now generates a random state token, stores `state_token → user_id` mapping
- `callback()` now looks up the state token to identify the user instead of requiring auth
- Removed `auth: AuthUser` parameter from `callback()`
- Added validation for missing/invalid state parameters

**Files Changed:**
- `backend/src/lib.rs` — Added `oauth_states` field and `AppState::new()`
- `backend/src/accounts/mod.rs` — Updated `connect()` and `callback()` handlers
- `backend/src/main.rs` — Updated to use `AppState::new()`

---

### BUG-002: Health Endpoint Behind Auth Middleware ✅ FIXED
**Severity:** Critical
**Location:** `backend/src/main.rs`
**Status:** ✅ Fixed on 05-06-2026

**Description:**
The auth middleware was applied as a layer to the entire protected router. The health endpoint was added before the split, so it was blocked by auth middleware.

**Impact:**
Docker health checks (`curl -f http://localhost:3000/health`) and monitoring tools could not reach the health endpoint without authentication.

**Fix:**
- Split routes into three groups: public, auth, and protected
- Health endpoint in public routes (no auth)
- Auth endpoints (register, login) in auth routes (no auth)
- All other endpoints in protected routes (auth required)
- Auth middleware applied only to protected routes

**Files Changed:**
- `backend/src/main.rs` — Restructured router to separate public, auth, and protected routes

---

### BUG-003: Premium Analytics Has No Ownership Check ✅ FIXED
**Severity:** Critical
**Location:** `backend/src/premium/mod.rs:56-83`
**Status:** ✅ Fixed on 05-06-2026

**Description:**
The `get_analytics()` handler accepted any `account_id` without verifying that the account belongs to the authenticated user.

**Impact:**
Any authenticated user could request analytics for any account by guessing or enumerating account UUIDs. This was a data leakage vulnerability.

**Fix:**
- Added `auth: AuthUser` parameter to `get_analytics()`
- Added ownership check: `SELECT * FROM accounts WHERE id = $1 AND user_id = $2`
- Returns 404 if account not found or doesn't belong to user

**Files Changed:**
- `backend/src/premium/mod.rs` — Added ownership validation

**Impact:**
Any authenticated user can request analytics for any account by guessing or enumerating account UUIDs. This is a data leakage vulnerability.

**Current Code:**
```rust
pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(account_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    // No ownership check!
    let resp = client.get(format!("{}/v1/analytics/{}", ..., account_id))
```

**Fix Needed:**
Add `auth: AuthUser` parameter and verify ownership:
```rust
pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    // Verify account belongs to user
    let account = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    // ... proceed with analytics
}
```

---

## 🟡 Medium Bugs

### BUG-004: JWT Secret Read From Env Per Request
**Severity:** Medium
**Location:** `backend/src/auth/middleware.rs:66-67`
**Status:** ⚠️ Needs fix

**Description:**
The auth middleware reads `JWT_SECRET` from `std::env::var()` on every request instead of using the `AppState.config` value.

**Impact:**
- Performance: System call on every request
- Inconsistency: Could read different values if env changes during runtime

**Current Code:**
```rust
let secret = std::env::var("JWT_SECRET")
    .unwrap_or_else(|_| "dev-secret".to_string());
```

**Fix Needed:**
Pass `AppState` to middleware and use `state.config.jwt_secret`.

---

### BUG-005: CORS Allows All Origins
**Severity:** Medium
**Location:** `backend/src/main.rs:113-116`
**Status:** ⚠️ Needs fix

**Description:**
`CorsLayer::new().allow_origin(Any)` allows any domain to make requests to the API.

**Impact:**
Security risk in production. Malicious websites could make authenticated requests on behalf of users.

**Current Code:**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

**Fix Needed:**
Configure allowed origins from environment variable:
```rust
let allowed_origins = std::env::var("CORS_ORIGINS")
    .unwrap_or_else(|_| "http://localhost:3000".to_string());
```

---

### BUG-006: No Token Persistence in Frontend
**Severity:** Medium
**Location:** `frontend/src/Main.elm`
**Status:** ⚠️ Needs fix

**Description:**
Token is stored only in the Elm model (in-memory). Page refresh or browser restart loses the session.

**Impact:**
Users must re-login on every page refresh. Poor user experience.

**Fix Needed:**
Add localStorage persistence via ports:
```elm
port module Main exposing (..)
port saveToken : String -> Cmd msg
port loadToken : () -> Cmd msg
```

---

### BUG-007: Hardcoded Frontend API URL
**Severity:** Medium
**Location:** `frontend/src/Api.elm:12`
**Status:** ⚠️ Needs fix

**Description:**
`baseUrl = "http://localhost:3000/api"` is hardcoded.

**Impact:**
Cannot deploy frontend to a different domain or port.

**Fix Needed:**
Pass base URL via flags:
```elm
main : Program { apiBaseUrl : String } Model Msg
```

---

### BUG-008: blake3 For Password Hashing
**Severity:** Medium
**Location:** `backend/src/auth/mod.rs:97-101`
**Status:** ⚠️ Needs fix

**Description:**
blake3 is a fast general-purpose hash, not a password-specific KDF like argon2/bcrypt/scrypt.

**Impact:**
Vulnerable to brute-force attacks. Passwords can be cracked faster than with proper KDFs.

**Current Code:**
```rust
pub fn hash_password(password: &str) -> String {
    let hash = blake3::hash(password.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hash.as_bytes())
}
```

**Fix Needed:**
Migrate to `argon2` crate:
```rust
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt).unwrap();
    hash.to_string()
}
```

---

### BUG-009: No Connection Pooling for External APIs
**Severity:** Medium
**Location:** `backend/src/premium/mod.rs:32`, `backend/src/scheduler/mod.rs:33`
**Status:** ⚠️ Needs fix

**Description:**
New `reqwest::Client` created per request in premium module and per tick in scheduler.

**Impact:**
- Performance: No connection reuse
- Resource waste: New TCP connections for each request

**Fix Needed:**
Create shared client in `AppState`:
```rust
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
    pub oauth_states: Mutex<HashMap<String, Uuid>>,
    pub http_client: reqwest::Client,  // Add this
}
```

---

### BUG-010: Scheduler Creates New Config/Client Per Tick
**Severity:** Medium
**Location:** `backend/src/scheduler/mod.rs:33-34`
**Status:** ⚠️ Needs fix

**Description:**
`Config::from_env()` and `MetaClient::new()` are called every 60 seconds instead of using shared state.

**Impact:**
Inefficiency, potential inconsistency if env vars change.

**Current Code:**
```rust
if let Some(account) = account {
    let config = crate::config::Config::from_env();
    let meta_client = crate::meta::MetaClient::new(&config);
```

**Fix Needed:**
Pass `AppState` to scheduler and use `state.config`.

---

## 🟢 Low Priority Issues

### BUG-011: No Pagination for Posts
**Severity:** Low
**Location:** `backend/src/posts/mod.rs:39-68`
**Status:** ⚠️ Enhancement

**Description:**
Posts list returns all posts without pagination.

**Impact:**
Performance degrades with many posts. Frontend loads entire dataset.

**Fix Needed:**
Add `page` and `limit` query parameters:
```rust
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedPosts>, AppError> {
```

---

### BUG-012: No Request Logging Middleware
**Severity:** Low
**Location:** `backend/src/main.rs`
**Status:** ⚠️ Not implemented

**Description:**
No request logging middleware for tracking API usage.

**Impact:**
Difficult to debug issues in production.

**Fix Needed:**
Add `tower-http` trace layer or custom middleware.

---

### BUG-013: Analytics Page Is Stub
**Severity:** Low
**Location:** `frontend/src/Main.elm:876-881`
**Status:** ⚠️ Not implemented

**Description:**
Analytics page shows placeholder text "Analytics coming soon".

**Impact:**
Feature incomplete.

---

### BUG-014: Settings Page Is Stub
**Severity:** Low
**Location:** `frontend/src/Main.elm:883-888`
**Status:** ⚠️ Not implemented

**Description:**
Settings page shows placeholder text "Settings coming soon".

**Impact:**
Feature incomplete.

---

### BUG-015: No Media Upload
**Severity:** Low
**Location:** `frontend/src/Main.elm`, `backend/src/posts/mod.rs`
**Status:** ⚠️ Enhancement

**Description:**
Media must be provided as URL. No direct file upload.

**Impact:**
Users must host images externally.

**Fix Needed:**
Add file upload endpoint and S3/storage integration.

---

### BUG-016: CAROUSEL Posts Fall Back to Single Image
**Severity:** Low
**Location:** `backend/src/meta/mod.rs:115-119`
**Status:** ⚠️ Not implemented

**Description:**
CAROUSEL media type falls back to single image handling.

**Impact:**
Carousel posts not supported.

---

## Summary

| Severity | Fixed | Open | Total |
|----------|-------|------|-------|
| 🔴 Critical | 3 | 0 | 3 |
| 🟡 Medium | 0 | 7 | 7 |
| 🟢 Low | 0 | 6 | 6 |
| **Total** | **3** | **13** | **16** |

---

*Generated by codebase analysis on 05-06-2026*
