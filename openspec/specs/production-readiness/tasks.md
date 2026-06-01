# Production Readiness Tasks

## Phase 1: Health Endpoint
- [ ] Add health handler to main.rs
- [ ] Add GET /health route (public, no auth)
- [ ] Test database connectivity in health check
- [ ] Return structured JSON response

## Phase 2: Docker Compose
- [ ] Update docker-compose.yml with health checks
- [ ] Add depends_on with condition: service_healthy
- [ ] Add .env.example with documented variables
- [ ] Update Dockerfile to serve frontend static files
- [ ] Add RUST_LOG environment variable

## Phase 3: Structured Logging
- [ ] Add tracing-env-filter crate
- [ ] Configure tracing-subscriber for JSON output
- [ ] Add request logging middleware
- [ ] Test log output

## Phase 4: Testing
- [ ] Test docker compose up works
- [ ] Test health endpoint returns 200
- [ ] Test logs are structured JSON
