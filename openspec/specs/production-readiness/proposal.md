# Production Readiness Proposal

## Why
The project works locally but requires manual PostgreSQL setup. A new contributor or deployment needs instant setup. Additionally, there's no health check for monitoring, and logging is basic.

## What Changes
- Docker Compose with frontend build, health checks, and proper env handling
- Health check endpoint for monitoring and Docker health checks
- Structured logging with request IDs, user IDs, and timing

## Scope
- Docker Compose: backend + PostgreSQL + frontend build
- Health endpoint: GET /health
- Structured JSON logging with tracing-subscriber

## Out of Scope
- Kubernetes/Docker Swarm deployment
- CI/CD pipeline
- Monitoring stack (Prometheus/Grafana)
