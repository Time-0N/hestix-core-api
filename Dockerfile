# Required Environment Variables (loaded via --env-file or container envs):
# - DATABASE_URL
# - PORT
# - DB_MAX_CONNECTIONS
# - KEYCLOAK_BASE_URL
# - KEYCLOAK_REALM
# - KEYCLOAK_CLIENT_ID
# - KEYCLOAK_CLIENT_SECRET
# - KEYCLOAK_ALLOWED_AUDIENCES
# - KEYCLOAK_REDIRECT_URI

# Build Stage
FROM rustlang/rust:nightly-slim as builder
LABEL authors="Time_ON"

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev

RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/hestix-core-api ./app

EXPOSE 3000

CMD ["./app"]