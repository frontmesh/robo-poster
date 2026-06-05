# Poster Backend Documentation

**Date:** 05-06-2026
**Version:** 0.1.0 (Prototype)
**Stack:** Rust 1.75+, Axum 0.8, SQLx 0.8, PostgreSQL 16

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Axum Server                              │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Public Routes         │  Protected Routes (JWT)         │  │
│  │  - POST /auth/register │  - GET/POST/PUT/DELETE /posts   │  │
│  │  - POST /auth/login    │  - GET/POST/DELETE /accounts    │  │
│  │  - GET /health         │  - POST /posts/{id}/publish     │  │
│  │                        │  - GET /calendar                │  │
│  │                        │  - POST /ai/generate            │  │
│  │                        │  - GET /analytics/{id}          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────┐  ┌────────────┴─────────────┐  ┌──────────────┐ │
│  │ Auth      │  │ Meta API Client          │  │ Scheduler    │ │
│  │ (JWT +    │  │ (Graph API v19.0)        │  │ (60s publish │ │
│  │  HMAC)    │  │  - Create container      │  │  1h refresh) │ │
│  └───────────┘  │  - Wait for status       │  └──────────────┘ │
│                  │  - Publish               │                    │
│                  └──────────────────────────┘                    │
│                              │                                   │
│                    ┌─────────▼─────────┐                        │
│                    │   PostgreSQL      │                        │
│                    │   (users, accounts│                        │
│                    │    posts, analytics)                       │
│                    └───────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module Reference

### `main.rs` (154 lines)
**Purpose:** Server entrypoint, router assembly, health endpoint.

**Key functions:**
- `health()` — Public endpoint, checks DB connectivity, returns JSON status
- `main()` — Initializes tracing, loads config, creates DB pool, runs migrations, spawns scheduler, builds router with OpenAPI, starts server

**Router structure:**
- Public routes: `/health`, `/api/auth/*` (no auth required)
- Protected routes: All other `/api/*` routes (JWT auth middleware applied)
- Swagger UI: `/swagger-ui`
- OpenAPI spec: `/api-docs/openapi.json`

**⚠️ Known Issue:** Auth middleware is applied as a layer to the entire protected router. The health endpoint is added before the split, so it may be behind auth middleware unintentionally.

---

### `lib.rs` (14 lines)
**Purpose:** Module declarations, shared state definition.

**Exports:** `AppState`, `accounts`, `auth`, `config`, `db`, `error`, `meta`, `posts`, `premium`, `scheduler`

**`AppState` struct:**
```rust
pub struct AppState {
    pub db: sqlx::PgPool,           // PostgreSQL connection pool
    pub config: config::Config,     // Application configuration
}
```

---

### `config.rs` (31 lines)
**Purpose:** Environment variable loading into typed struct.

| Field | Env Var | Required | Default |
|-------|---------|----------|---------|
| `database_url` | `DATABASE_URL` | ✅ | — |
| `jwt_secret` | `JWT_SECRET` | ✅ | — |
| `meta_app_id` | `META_APP_ID` | ✅ | — |
| `meta_app_secret` | `META_APP_SECRET` | ✅ | — |
| `meta_redirect_uri` | `META_REDIRECT_URI` | ❌ | `http://localhost:3000/api/accounts/callback` |
| `premium_api_url` | `PREMIUM_API_URL` | ❌ | `http://localhost:3001` |
| `premium_api_key` | `PREMIUM_API_KEY` | ❌ | `""` |

**Pattern:** Required vars use `expect()` (panics at startup). Optional vars use `unwrap_or_else`/`unwrap_or_default`.

---

### `error.rs` (49 lines)
**Purpose:** Centralized error handling with HTTP status mapping.

| Variant | HTTP Status | When Used |
|---------|-------------|-----------|
| `Database(sqlx::Error)` | 500 | DB operation failure |
| `Unauthorized` | 401 | Missing/invalid JWT, wrong credentials |
| `NotFound` | 404 | Resource not found |
| `BadRequest(String)` | 400 | Invalid input (short password, bad email) |
| `Internal(String)` | 500 | Catch-all server error |
| `MetaApi(String)` | 502 | Meta Graph API errors |
| `PremiumApi(String)` | 502 | Premium service errors |

**Pattern:** Implements `IntoResponse` for Axum. All errors serialize to `{"error": "message"}` JSON.

---

### `db/mod.rs` (51 lines)
**Purpose:** Data models (SQLx `FromRow` structs).

**Tables:**
- `users` — id, email, password_hash, license_key?, created_at
- `accounts` — id, user_id (FK), provider, provider_user_id, username, access_token, refresh_token?, token_expires_at?, created_at
- `posts` — id, account_id (FK), content, media_url?, media_type?, scheduled_at?, published_at?, status, platform, platform_post_id?, created_at
- `post_analytics` — id, post_id (FK), likes, replies, reposts, impressions, fetched_at

**Relationships:** users 1→N accounts 1→N posts 1→N post_analytics (all CASCADE delete)

**Indexes:** Partial index on `posts(scheduled_at) WHERE status = 'scheduled'`

---

### `auth/mod.rs` (164 lines)
**Purpose:** User registration, login, JWT creation, password hashing.

**Public API:**

| Function | Visibility | Description |
|----------|------------|-------------|
| `register()` | Handler | Create user, return JWT |
| `login()` | Handler | Authenticate, return JWT |
| `hash_password()` | Pub | blake3 hash + base64 encode |
| `verify_password()` | Pub | Recompute hash, compare |
| `create_jwt()` | Pub | HMAC-SHA256 signed JWT |

**JWT Format:**
```
base64url(header).base64url(payload).base64url(hmac_sha256(secret, header.payload))
```

**Payload:**
```json
{"sub": "uuid", "exp": 1234567890}
```

**⚠️ Security Note:** Uses blake3 for password hashing (fast, not password-specific). Should migrate to argon2/bcrypt for production.

---

### `auth/middleware.rs` (114 lines)
**Purpose:** JWT validation middleware, `AuthUser` extractor.

**`AuthUser` struct:**
```rust
pub struct AuthUser {
    pub user_id: Uuid,
}
```

**`from_token(token, secret)` flow:**
1. Split token into 3 parts
2. Verify HMAC-SHA256 signature
3. Decode payload JSON
4. Check `exp` field (must be in future)
5. Extract `sub` field as UUID

**`auth_middleware` flow:**
1. Extract `Authorization: Bearer <token>` header
2. Read `JWT_SECRET` from env
3. Call `AuthUser::from_token()`
4. Insert `AuthUser` into request extensions

**⚠️ Issue:** Reads `JWT_SECRET` from env on every request instead of using `AppState.config`.

---

### `accounts/mod.rs` (372 lines)
**Purpose:** Meta OAuth flow, account CRUD.

**Handlers:**

| Handler | Route | Description |
|---------|-------|-------------|
| `list()` | GET /api/accounts | List user's connected accounts |
| `connect()` | POST /api/accounts/connect | Generate Meta OAuth URL |
| `callback()` | GET /api/accounts/callback | Handle OAuth redirect |
| `delete()` | DELETE /api/accounts/{id} | Disconnect account |

**OAuth Flow:**
1. `connect()` → Returns URL to `https://www.facebook.com/v19.0/dialog/oauth`
2. User authorizes on Facebook
3. Meta redirects to `callback()` with `?code=...`
4. `callback()` exchanges code for short-lived token
5. Exchanges for long-lived token (60 days)
6. Fetches Instagram business accounts + Threads profiles
7. Upserts accounts in DB

**Scopes requested:** `instagram_basic`, `instagram_content_publish`, `instagram_manage_insights`, `threads_basic`, `threads_content_publish`, `threads_manage_insights`

**⚠️ Critical Bug:** `callback()` requires `auth: AuthUser` parameter, but Meta redirects without a Bearer token. The callback will always fail with 401.

---

### `posts/mod.rs` (368 lines)
**Purpose:** Post CRUD, publishing, calendar.

**Handlers:**

| Handler | Route | Description |
|---------|-------|-------------|
| `list()` | GET /api/posts | List posts (via account JOIN for ownership) |
| `create()` | POST /api/posts | Create draft or scheduled post |
| `update()` | PUT /api/posts/{id}` | Update post content/schedule |
| `delete()` | DELETE /api/posts/{id} | Delete post |
| `publish()` | POST /api/posts/{id}/publish | Publish via Meta API |
| `calendar()` | GET /api/calendar | Posts grouped by date |

**Business Logic:**
- Status: `scheduled_at` present → "scheduled", otherwise → "draft"
- Platform default: "threads" if not specified
- Ownership: All queries JOIN on `accounts.user_id = auth.user_id`
- Publish: Validates ownership → calls MetaClient → updates status

**DTOs:**
- `CreatePostRequest` — account_id, content, media_url?, media_type?, scheduled_at?, platform?
- `PostResponse` — id, content, media_url?, scheduled_at?, published_at?, status, platform, account_id
- `CalendarDay` — date, posts

---

### `meta/mod.rs` (314 lines)
**Purpose:** Meta Graph API client for publishing to Threads/Instagram.

**Public API:**

| Method | Description |
|--------|-------------|
| `MetaClient::new(config)` | Creates client with 30s timeout |
| `publish_post(account, post)` | Full 3-step publish flow |
| `get_base_url(account)` | Platform-specific base URL |
| `get_post_insights(account, post_id)` | Fetch analytics |

**Publish Flow (3 steps):**

1. **Create container:**
   - Threads: `POST https://graph.threads.net/v1.0/{user_id}`
   - Instagram: `POST https://graph.facebook.com/v19.0/{user_id}/media`
   - Form data: `media_type`, `access_token`, `text`, `image_url`/`video_url`

2. **Wait for container:**
   - Poll `GET {base_url}/{container_id}?fields=status` every 3s
   - Max 10 retries (30s total)
   - Handles: FINISHED ✅, ERROR ❌, EXPIRED ❌

3. **Publish container:**
   - Threads: `POST {base_url}/{user_id}/threads_publish`
   - Instagram: `POST {base_url}/{user_id}/media_publish`

---

### `premium/mod.rs` (115 lines)
**Purpose:** Proxy to external premium service (AI generation, analytics).

**Handlers:**

| Handler | Route | Description |
|---------|-------|-------------|
| `generate_content()` | POST /api/ai/generate | Forward prompt to premium API |
| `get_analytics()` | GET /api/analytics/{account_id} | Forward analytics request |

**⚠️ Issues:**
- Creates new `reqwest::Client` per request (no connection pooling)
- No ownership validation on analytics (any user can query any account_id)

---

### `scheduler/mod.rs` (208 lines)
**Purpose:** Background tasks for publishing and token refresh.

**Two scheduled tasks:**

| Task | Interval | Description |
|------|----------|-------------|
| `publish_scheduled_posts` | 60 seconds | Publish due posts (LIMIT 10) |
| `refresh_expiring_tokens` | 1 hour | Refresh tokens expiring within 7 days |

**`publish_scheduled_posts` logic:**
1. Query posts with `status = 'scheduled' AND scheduled_at <= NOW() LIMIT 10`
2. For each post: load account, check token expiry
3. If token expired or account missing → mark as "failed"
4. If Meta API error → leave as "scheduled" (retry next tick)
5. If success → mark as "published"

**`refresh_expiring_tokens` logic:**
1. Query accounts with `token_expires_at < NOW() + 7 days`
2. Exchange refresh token for new long-lived token
3. Update account with new token + 59-day expiry

**⚠️ Issues:**
- Creates new `Config::from_env()` and `MetaClient` per tick
- Uses refresh token as `fb_exchange_token` (may not be correct Meta API pattern)

---

## Database Schema

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    license_key VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    media_url TEXT,
    media_type VARCHAR(50),
    scheduled_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    status VARCHAR(50) DEFAULT 'draft',
    platform VARCHAR(50) DEFAULT 'threads',
    platform_post_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE post_analytics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    likes INTEGER DEFAULT 0,
    replies INTEGER DEFAULT 0,
    reposts INTEGER DEFAULT 0,
    impressions INTEGER DEFAULT 0,
    fetched_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_accounts_user_id ON accounts(user_id);
CREATE INDEX idx_posts_account_id ON posts(account_id);
CREATE INDEX idx_posts_scheduled_at ON posts(scheduled_at) WHERE status = 'scheduled';
CREATE INDEX idx_post_analytics_post_id ON post_analytics(post_id);
```

---

## Critical Bottlenecks & Issues

### 🔴 High Priority (Security/Functionality)

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| blake3 for passwords | `auth/mod.rs` | Brute-forceable | Migrate to argon2/bcrypt |
| OAuth callback requires auth | `accounts/mod.rs:callback` | Callback always fails | Remove `auth` param, use state param for session |
| Premium analytics no ownership check | `premium/mod.rs` | Any user can query any account | Add user_id validation |
| CORS allows all origins | `main.rs` | Security risk in production | Configure allowed origins |

### 🟡 Medium Priority (Performance/Reliability)

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| JWT_SECRET read from env per request | `middleware.rs` | Performance | Use AppState.config |
| New reqwest::Client per premium request | `premium/mod.rs` | No connection pooling | Create client in AppState |
| New Config/MetaClient per scheduler tick | `scheduler/mod.rs` | Inefficiency | Use shared state |
| No DB connection pool tuning | `main.rs` | Hardcoded 5 connections | Make configurable |
| Token only in Elm model | Frontend | Lost on refresh | Add localStorage persistence |

### 🟢 Low Priority (Missing Features)

| Feature | Status | Notes |
|---------|--------|-------|
| Password reset | Not implemented | — |
| Pagination | Not implemented | Posts list grows unbounded |
| Media upload | URL-only | No direct upload |
| CAROUSEL posts | Fallback to single image | — |
| Request logging middleware | Not implemented | Listed in spec but not built |
| Analytics page | Stub | — |
| Settings page | Stub | — |

---

## Scenarios & Future Work

### Scenario 1: Production Deployment
**Current blockers:**
- CORS allows all origins
- No rate limiting
- Password hashing too fast (blake3)
- Health endpoint may be behind auth

**Needed:**
- Configure CORS for specific origins
- Add rate limiting middleware
- Migrate to argon2 for passwords
- Fix health endpoint routing

### Scenario 2: Multi-User SaaS
**Current blockers:**
- No token refresh on frontend
- No pagination
- Premium analytics has no ownership check

**Needed:**
- Frontend token persistence (localStorage)
- Pagination for posts/accounts
- Ownership validation on all premium endpoints
- License key validation (field exists but unused)

### Scenario 3: Scaling to 100+ Accounts
**Current blockers:**
- DB pool hardcoded to 5
- Scheduler processes only 10 posts per tick
- No connection pooling for external APIs

**Needed:**
- Configurable pool size
- Increase scheduler batch size or make configurable
- Share reqwest::Client across application

### Scenario 4: Adding New Platform (e.g., TikTok)
**Current architecture supports:**
- Platform-agnostic post model (platform field)
- MetaClient pattern could be replicated

**Needed:**
- New `TikTokClient` implementing similar interface
- Account connection flow for TikTok OAuth
- Platform-specific endpoint routing

---

## Quick Reference

### API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | /health | ❌ | Health check |
| POST | /api/auth/register | ❌ | Create account |
| POST | /api/auth/login | ❌ | Login |
| GET | /api/accounts | ✅ | List accounts |
| POST | /api/accounts/connect | ✅ | Get OAuth URL |
| GET | /api/accounts/callback | ✅ | OAuth callback |
| DELETE | /api/accounts/{id} | ✅ | Disconnect account |
| GET | /api/posts | ✅ | List posts |
| POST | /api/posts | ✅ | Create post |
| PUT | /api/posts/{id} | ✅ | Update post |
| DELETE | /api/posts/{id} | ✅ | Delete post |
| POST | /api/posts/{id}/publish | ✅ | Publish post |
| GET | /api/calendar | ✅ | Calendar view |
| POST | /api/ai/generate | ✅ | AI content generation |
| GET | /api/analytics/{id} | ✅ | Get analytics |

### Environment Variables

```bash
# Required
DATABASE_URL=postgres://poster:poster@localhost:5432/poster
JWT_SECRET=your-secret-key
META_APP_ID=your-meta-app-id
META_APP_SECRET=your-meta-app-secret

# Optional
META_REDIRECT_URI=http://localhost:3000/api/accounts/callback
PREMIUM_API_URL=http://localhost:3001
PREMIUM_API_KEY=your-premium-api-key
RUST_LOG=info
```

### Running

```bash
# Local development
cd backend
cp .env.example .env  # Configure env vars
cargo run

# Docker
docker compose up
# App: http://localhost:3000
# Swagger: http://localhost:3000/swagger-ui
# Health: http://localhost:3000/health
```

### Testing

```bash
cd backend
cargo test  # Runs 34 unit tests
```

---

*Generated by codebase analysis on 05-06-2026*
