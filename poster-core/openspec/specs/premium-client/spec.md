# Premium Client Specification

## Goal
Integrate with poster-api for premium features (AI generation, analytics).

## Requirements
- HTTP client to call poster-api endpoints
- Pass license key for authentication
- Proxy AI generation requests
- Proxy analytics requests

## API Endpoints
- `POST /api/ai/generate` - Generate content via AI
- `GET /api/analytics/:account_id` - Get analytics

## Success Criteria
- AI content generation works when premium API is available
- Graceful fallback when premium API is unavailable
- Analytics data is displayed in dashboard
