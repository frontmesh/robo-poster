# Poster Cloudflare Rewrite Prompt

We are building a new Cloudflare-native rewrite of the existing Poster project.

## Goal

Create a deployable Cloudflare Workers social post scheduler inspired by the current Poster prototype and by the architecture of `git@github.com:1e3pm/post-scheduler.git`, but do not blindly copy code. Treat the old Poster repo as product/reference context and build a clean implementation.

## Tech Choices

- Backend: ReScript for core app/domain logic.
- Worker runtime: Cloudflare Workers.
- Thin TypeScript wrappers are allowed for Worker entrypoints, Durable Object classes, bindings, and awkward Cloudflare interop.
- Database: Cloudflare D1.
- Scheduler: Durable Object alarms.
- Deployment: Wrangler with Workers Assets if/when frontend is included.

## Product Scope

Build a multi-platform social scheduler with:

- user signup/login using JWT
- secure password hashing, not plain SHA-256
- provider credential management
- connected channels/accounts
- posts that can target one or more channels
- per-destination publish status
- scheduled publishing via Durable Object alarms
- initial providers: start with Meta/Instagram/Threads if easiest from old Poster, then add others incrementally

## Data Model

Use a model closer to `post-scheduler`:

- users
- providers
- channels
- posts
- post_destinations
- oauth_states
- optionally files/post_files/file_chunks later

## Implementation Constraints

- Keep Cloudflare-specific surfaces thin and explicit.
- Put durable domain logic in ReScript modules.
- Use TypeScript only where it reduces interop friction.
- Do not hardcode reusable values.
- Do not keep obsolete code paths or unused fallback branches.
- Avoid nested ternaries.
- Use D1 migrations as the source of truth for schema.
- Secrets must be Wrangler secrets, not checked into config.
- Include local dev and deploy instructions from the start.

## Planning First

Before writing files, inspect the new folder, check available tooling, and produce a concrete implementation plan covering:

- project structure
- package/build tooling
- ReScript + TypeScript interop boundary
- D1 schema
- Worker routes
- Durable Object alarm flow
- auth/session design
- first provider implementation
- test strategy
- deployment steps

Do not implement until the plan is clear.
