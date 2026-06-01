# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm install -g elm@0.19.1-6
RUN elm make src/Main.elm --output=public/elm.js

# Stage 2: Build backend
FROM rust:1.75 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

# Stage 3: Final image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/target/release/poster-core /usr/local/bin/poster-core
COPY --from=frontend-builder /app/public/ /app/static/
ENV STATIC_DIR=/app/static
EXPOSE 3000
CMD ["poster-core"]
