# Post Publishing Specification

## Goal
Publish posts to Threads and Instagram via the Meta API.

## Requirements
- Create post with content, optional media, platform
- Two-step publishing: create container → publish
- Support text, image, video posts
- Track post status (draft, scheduled, published)
- Store platform post ID after publishing

## API Endpoints
- `POST /api/posts` - Create post
- `GET /api/posts` - List posts
- `PUT /api/posts/:id` - Update post
- `DELETE /api/posts/:id` - Delete post
- `POST /api/posts/:id/publish` - Publish immediately

## Success Criteria
- User can create a text post
- User can publish to Threads
- Post status updates to "published"
- Platform post ID is stored
