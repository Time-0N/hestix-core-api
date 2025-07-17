# Hestix Core API

The **Hestix Core API** is the central backend for the Hestix ecosystem. It handles authentication (via Keycloak), user syncing, and exposes a clean, type‑safe HTTP API over Rust/Axum.

## 🧰 Tech Stack


- **[Rust](https://www.rust-lang.org/)** — the language
- **[Axum](https://docs.rs/axum/latest/axum/)** — HTTP framework
- **[Tokio](https://tokio.rs/)** — async runtime
- **[SQLx](https://docs.rs/sqlx/)** — compile‑time checked Postgres client
- **PostgreSQL** — relational datastore
- **Keycloak** — OAuth2/OpenID Connect server
- **[Moka](https://docs.rs/moka/)** — in‑memory async cache for user lookups
- **[Anyhow](https://docs.rs/anyhow/)** — ergonomic error handling
- **[tracing](https://docs.rs/tracing/)** + **[tower-http](https://docs.rs/tower-http/)** — structured logging & HTTP request traces

## 📁 Project Structure

```
├── Cargo.toml  
├── migrations/               # sqlx migration scripts  
└── src/  
    ├── main.rs               # tiny entrypoint: calls bootstrap::run()  
    ├── bootstrap.rs          # loads config, builds pool, clients, services, router, and serves  
    ├── config.rs             # typed `.env` parsing (DATABASE_URL, LISTEN_PORT, LOG_FILTER, KEYCLOAK_*)  
    ├── app_state.rs          # composes cache, repos, services, and Keycloak client into shared state  
    ├── routes/               # all route definitions & nesting  
    │   ├── mod.rs            # `pub fn create_router(state: AppState) -> Router`  
    │   ├── auth_routes.rs    # `/api/auth/*`  
    │   └── user_routes.rs    # `/api/user/*`  
    ├── handlers/             # HTTP handlers (thin, call services)  
    ├── models/               # your domain models (e.g. UserEntity)  
    ├── repositories/         # trait + Postgres impl for data access  
    ├── services/             # business logic (AuthService, UserService, KeycloakService)  
    ├── security/             # Keycloak client, JWT extractors & validators  
    └── cache/                 # e.g. `resolver.rs` for cache + repo lookup  
```

## 🚀 Running Locally

1. **Set up `.env`:**
   ```env
   DATABASE_URL=postgres://user:password@localhost/hestix
   DB_MAX_CONNECTIONS=5
   PORT=3000
   LOG_FILTER=info

   KEYCLOAK_URL=http://localhost:8080
   KEYCLOAK_REALM=myrealm
   KEYCLOAK_CLIENT_ID=core-api
   KEYCLOAK_CLIENT_SECRET=supersecret
   KEYCLOAK_REDIRECT_URI=http://localhost:0069
   ```

2. **Run migrations:**
   ```bash
   sqlx migrate run
   ```

3. **Start the server:**
   ```bash
   cargo run
   ```
   The API will listen on `127.0.0.1:$PORT` (default port 3000).

## 📌 Notes

- SQLx uses compile-time checks. Run `cargo sqlx prepare` after migration changes if using offline mode.
- Caching is handled with [moka](https://docs.rs/moka/), a high-perf async cache.
- All user identity checks are JWT-based and delegated to Keycloak.