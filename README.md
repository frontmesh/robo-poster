# Poster

Marketing automation platform for scheduling and publishing content to Instagram and Threads.

**Version:** 0.1.0 (Prototype)
**Date:** 05-06-2026

---

## Quick Start

```bash
# Clone and configure
git clone https://github.com/frontmesh/poster.git
cd poster
cp .env.example .env
# Edit .env with your Meta app credentials

# Start with Docker
docker compose up

# Access
# App: http://localhost:3000
# Swagger UI: http://localhost:3000/swagger-ui
# Health: http://localhost:3000/health
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Poster                                 │
├─────────────────────────────────────────────────────────────────┤
│  Frontend (Elm)          │  Backend (Rust/Axum)                 │
│  - Dashboard             │  - Auth (JWT + HMAC-SHA256)          │
│  - Post Composer         │  - Meta OAuth (IG + Threads)         │
│  - Content Calendar      │  - Post Publishing (Meta Graph API)  │
│  - Account Management    │  - Scheduling (background tasks)     │
│  - AI Content Generation │  - Premium API Proxy                 │
│                          │  - OpenAPI/Swagger docs              │
├──────────────────────────┴──────────────────────────────────────┤
│  PostgreSQL (users, accounts, posts, analytics)                 │
└─────────────────────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Backend | Rust + Axum | 1.75+ / 0.8 |
| Frontend | Elm | 0.19.1 |
| Database | PostgreSQL | 16 |
| Auth | JWT (HMAC-SHA256) | — |
| API Docs | utoipa (OpenAPI 3.1) | 5.5 |
| Container | Docker + Docker Compose | — |

---

## Features

### Implemented
- ✅ User registration and login (JWT)
- ✅ Meta OAuth (Instagram + Threads account linking)
- ✅ Post creation (text, image, video)
- ✅ Post scheduling with auto-publish
- ✅ Content calendar view
- ✅ AI content generation (via premium API proxy)
- ✅ Interactive API documentation (Swagger UI)
- ✅ Health check endpoint
- ✅ Structured JSON logging
- ✅ Docker Compose deployment
- ✅ 34 unit tests

### In Progress / Planned
- 🔄 Token persistence (localStorage)
- 🔄 Pagination
- 🔄 Media upload (currently URL-only)
- 🔄 Analytics dashboard
- 🔄 Settings page
- 🔄 Rate limiting
- 🔄 Multi-language support

---

## Project Structure

```
poster/
├── backend/                    # Rust/Axum backend
│   ├── src/
│   │   ├── main.rs             # Server entrypoint
│   │   ├── lib.rs              # Module exports
│   │   ├── config.rs           # Environment config
│   │   ├── error.rs            # Error handling
│   │   ├── db/                 # Data models
│   │   ├── auth/               # Authentication
│   │   ├── accounts/           # Meta OAuth
│   │   ├── posts/              # Post CRUD + publishing
│   │   ├── meta/               # Meta Graph API client
│   │   ├── premium/            # Premium API proxy
│   │   └── scheduler/          # Background tasks
│   ├── tests/                  # 34 unit tests
│   └── migrations/             # SQL migrations
├── frontend/                   # Elm frontend
│   ├── src/
│   │   ├── Main.elm            # App logic + views
│   │   ├── Types.elm           # Type definitions
│   │   └── Api.elm             # HTTP client
│   └── public/                 # Static files + CSS
├── openspec/                   # OpenSpec specifications
├── docs/                       # Documentation
│   └── 05-06-2026/             # Timestamped docs
│       ├── backend.md          # Backend architecture
│       └── frontend.md         # Frontend architecture
├── docker-compose.yml          # Docker setup
├── Dockerfile                  # Multi-stage build
└── .env.example                # Environment variables
```

---

## API Endpoints

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

**Interactive docs:** http://localhost:3000/swagger-ui

---

## Environment Variables

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

---

## Development

### Backend

```bash
cd backend
cp .env.example .env
cargo run          # Start server
cargo test         # Run 34 tests
cargo check        # Type check
```

### Frontend

```bash
cd frontend
elm make src/Main.elm --output=public/elm.js
# Open public/index.html in browser
```

### Docker

```bash
docker compose up              # Start all services
docker compose up -d           # Start in background
docker compose down            # Stop all services
docker compose logs -f backend # Follow backend logs
```

---

## Known Issues

### 🔴 Critical
- OAuth callback requires auth header (will fail with 401)
- blake3 used for password hashing (should use argon2/bcrypt)
- CORS allows all origins (insecure for production)

### 🟡 Medium
- JWT secret read from env per request (performance)
- No token persistence in frontend (lost on refresh)
- Premium analytics has no ownership check
- Health endpoint may be behind auth middleware

### 🟢 Low
- No pagination for posts
- Analytics/Settings pages are stubs
- Hardcoded frontend API base URL
- No media upload (URL-only)

---

## Documentation

Detailed documentation is available in `docs/05-06-2026/`:

- [Backend Documentation](docs/05-06-2026/backend.md) — Architecture, modules, database, issues
- [Frontend Documentation](docs/05-06-2026/frontend.md) — Components, state management, flows

---

## Testing

```bash
# Run all tests
cd backend
cargo test

# Test output
# auth_tests: 14 tests (password hashing, JWT, middleware)
# posts_tests: 10 tests (error codes, status logic)
# meta_tests: 10 tests (URL routing, form data, endpoints)
# Total: 34 tests passing
```

---

## License

Private — All rights reserved.

---

*Last updated: 05-06-2026*
