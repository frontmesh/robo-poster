# Meta OAuth Specification

## Goal
Enable users to connect their Instagram/Threads accounts via OAuth.

## Requirements
- Redirect to Meta OAuth dialog
- Request instagram_basic, instagram_content_publish, threads_basic, threads_content_publish scopes
- Exchange code for short-lived token
- Exchange short-lived token for long-lived token (60 days)
- Store account details in database
- Handle OAuth callback

## API Endpoints
- `POST /api/accounts/connect` - Get OAuth URL
- `GET /api/accounts/callback` - Handle OAuth callback

## Success Criteria
- User can connect Instagram account
- Token is stored and associated with user
- Account appears in accounts list
