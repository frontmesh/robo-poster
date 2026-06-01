# Production Readiness Design

## Docker Compose

### Services
```yaml
services:
  db:
    image: postgres:16-alpine
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U poster"]
      interval: 5s
      timeout: 5s
      retries: 5

  backend:
    build: .
    ports: ["3000:3000"]
    environment:
      DATABASE_URL: postgres://poster:poster@db:5432/poster
      JWT_SECRET: ${JWT_SECRET:-change-me-in-production}
      RUST_LOG: info
    depends_on:
      db:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Frontend Serving
Build frontend in Docker, copy static files to backend static directory.

## Health Endpoint

### Implementation
```rust
async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let status = if db_ok { "healthy" } else { "degraded" };

    Json(json!({
        "status": status,
        "timestamp": Utc::now().to_rfc3339(),
        "database": if db_ok { "connected" } else { "disconnected" }
    }))
}
```

### Route
- `GET /health` — public, no auth required
- Returns 200 if healthy, 503 if degraded

## Structured Logging

### Configuration
```rust
tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    )
    .json()
    .init();
```

### Request Logging Middleware
Logs method, path, status, duration for every request.
