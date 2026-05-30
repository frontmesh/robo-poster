# Scheduling Specification

## Goal
Schedule posts for future publishing.

## Requirements
- User can set scheduled_at time on posts
- Background task checks for due posts every minute
- Due posts are published automatically
- Token refresh for expiring tokens

## Implementation
- Tokio background task with 60-second interval
- Query posts where scheduled_at <= NOW() AND status = 'scheduled'
- Publish each due post via Meta API
- Refresh tokens expiring within 7 days

## Success Criteria
- User can schedule a post for future time
- Post is published at scheduled time
- Tokens are refreshed before expiry
