# Hestix Core API

The **Hestix Core API** serves as the central backend of the Hestix system, handling authentication, data synchronization, and secure communication with other modules in the ecosystem.

## 🧰 Tech Stack

- **[Rust](https://www.rust-lang.org/):** High-performance systems programming language powering the entire backend.
- **[Axum](https://docs.rs/axum/latest/axum/):** Web framework built on Tokio, providing ergonomic and modular routing for HTTP APIs.
- **[Keycloak](https://www.keycloak.org/):** Identity and access management platform used for OAuth2 & OpenID Connect authentication.
- **[PostgreSQL](https://www.postgresql.org/):** Reliable, scalable relational database for persistent data storage.
- **[SQLx](https://docs.rs/sqlx/):** Compile-time safe and asynchronous ORM for interacting with PostgreSQL.

## 📁 Project Structure

```
├── src
│   ├── main.rs               # Entrypoint
│   ├── router.rs             # Route definitions
│   ├── state.rs              # Shared application state
│   ├── models/               # Domain models (e.g. UserEntity)
│   ├── handlers/             # HTTP request handlers
│   ├── repositories/         # Database interaction logic
│   ├── services/             # Business logic (e.g. AuthService, UserService)
│   └── security/             # Keycloak integration, extractors, JWT validation
├── migrations/               # SQLx migration scripts
├── Cargo.toml
```

## 🚀 Running Locally

1. **Set up `.env`:**
   ```env
   DATABASE_URL=postgres://user:password@url/database
   KEYCLOAK_BASE_URL=http://url:8080
   KEYCLOAK_REALM=your_keycloak_realm
   KEYCLOAK_CLIENT_ID=core-api
   KEYCLOAK_CLIENT_SECRET=your-client-secret
   ```

2. **Run migrations:**
   ```bash
   sqlx migrate run
   ```

3. **Start the server:**
   ```bash
   cargo run
   ```

## 📌 Notes

- SQLx uses compile-time checks. Run `cargo sqlx prepare` after migration changes if using offline mode.
- Caching is handled with [moka](https://docs.rs/moka/), a high-perf async cache.
- All user identity checks are JWT-based and delegated to Keycloak.