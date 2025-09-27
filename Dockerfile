# Build Stage - Optimized with dependency caching
FROM rust:latest as builder
LABEL authors="Time_ON"
WORKDIR /app

# Accept DATABASE_URL for SQLx compile-time checks (from internet DB)
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Install build dependencies first
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs and build dependencies (this will be cached)
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src target/release/deps/hestix*

# Now copy everything and build
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/hestix-core-api ./app

COPY --from=builder /app/migrations ./migrations

EXPOSE 3000

CMD ["./app"]