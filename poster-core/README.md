# Poster

Marketing automation for Threads & Instagram.

## Features

- Post scheduling and publishing
- Multi-account management
- AI content generation
- Analytics dashboard
- Content calendar

## Tech Stack

- **Backend:** Rust + Axum
- **Frontend:** Elm
- **Database:** PostgreSQL
- **Specs:** OpenSpec

## Quick Start

```bash
# Start database
docker compose up -d db

# Run backend
cd backend
cp .env.example .env
cargo run

# Build frontend
cd frontend
elm make src/Main.elm --output=public/elm.js
```

## Docker

```bash
docker compose up
```

## License

MIT
