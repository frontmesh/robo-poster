# API Documentation Proposal

## Why
The API has no documentation. Frontend integration relies on reading source code. A formal OpenAPI spec provides interactive API exploration and a contract for development.

## What Changes
- Add utoipa crate for OpenAPI generation
- Annotate all request/response types with ToSchema
- Annotate all handlers with path metadata
- Serve Swagger UI at /swagger-ui

## Scope
- OpenAPI 3.1 spec generation
- Swagger UI for interactive exploration
- All existing endpoints documented

## Out of Scope
- API versioning strategy
- Rate limiting documentation
