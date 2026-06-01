# Production Readiness Specification

## Goal
Make the project instantly runnable and production-monitorable.

## Requirements

### Docker Compose
- `docker compose up` starts the full stack
- Backend waits for PostgreSQL to be ready
- Frontend is built and served by backend
- Environment variables documented in .env.example
- Health check configured in Docker Compose

### Health Endpoint
- `GET /health` returns 200 OK with status JSON
- Response includes: status, timestamp, database connectivity
- No authentication required
- Used by Docker health check

### Structured Logging
- JSON-formatted logs for production
- Include request method, path, status code, duration
- Include user_id for authenticated requests
- Include request ID for tracing

## Success Criteria
- `docker compose up` works without manual setup
- `curl localhost:3000/health` returns healthy status
- Logs are structured JSON in production
