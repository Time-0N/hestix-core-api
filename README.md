# Hestix Core API — ZITADEL (OIDC + PKCE)

The **Hestix Core API** is the central backend for the Hestix ecosystem. It handles authentication via **ZITADEL** (OIDC + PKCE), user syncing, and exposes a clean, type‑safe HTTP API built on **Rust/Axum**.

## 🧰 Tech Stack
- **Rust**
- **Axum** (HTTP framework)
- **Tokio** (async runtime)
- **SQLx** (compile‑time checked Postgres client)
- **PostgreSQL** (relational datastore)
- **ZITADEL** (OIDC: Authorization Code + PKCE)
- **Moka** (in‑memory async cache)
- **anyhow**, **tracing**, **tower-http**

## 📁 Project Structure (high‑level)
```text
├── Cargo.toml
├── migrations/                        # SQLx migrations
└── src/
    ├── main.rs                        # tiny entrypoint (bootstrap::run)
    ├── bootstrap.rs                   # config, db pool, services, router, server
    ├── config.rs                      # typed .env parsing
    ├── app_state.rs                   # composes repos/services/providers into shared state
    ├── routes/                        # routing (auth, user, …)
    ├── handlers/                      # HTTP handlers (thin; call services)
    ├── services/                      # business logic (AuthService, UserService)
    ├── repositories/                  # trait + Postgres impl for data access
    ├── models/                        # domain models (UserEntity, …)
    ├── middleware/security/extractor.rs  # auth extractor (reads cookies/headers, validates JWT)
    ├── oidc/                          # generic OIDC layer: discovery, jwks, errors, traits
    │   ├── claims.rs
    │   ├── discovery.rs
    │   ├── jwk.rs
    │   ├── error.rs
    │   └── provider.rs                # OidcProvider, RoleMapper, OidcAdminApi traits
    └── providers/
        └── zitadel/                   # ZITADEL implementation of the OIDC traits
            ├── provider.rs            # authorize_url, code/refresh exchange, validate
            ├── role_mapper.rs         # maps ZITADEL roles => Vec<String>
            └── admin.rs               # (optional) admin API placeholder
```

## 🔐 Authentication (ZITADEL OIDC + PKCE)

**Flow:** Authorization Code + PKCE (no client secret).  
**Tokens:**
- **Access Token (JWT):** used for API auth + roles (from ZITADEL’s project role claim).
- **ID Token (JWT):** identity fields (email, preferred_username, etc.) if enabled.
- **Refresh Token:** optional; for silent renewal.

**Cookies set by backend:**
- `access_token` — short‑lived; used by the extractor to authorize requests.
- `refresh_token` — optional; used to refresh `access_token`.
- `pkce_verifier` — short‑lived; stored during login, used once on callback.

**Required ZITADEL app settings:**
- **Type:** Web
- **Response type:** `code`
- **Grant types:** `authorization_code`, `refresh_token`
- **Authentication method:** `none` (PKCE; no client secret)
- **Redirect URIs:** include your backend callback (e.g. `http://localhost:5000/api/auth/callback`)
- **(Recommended)** “**User Info inside ID Token**”: **ON** to receive `email` / `preferred_username` in the ID token.
- Assign users **project roles** so the access token includes them.

## ⚙️ Configuration (`.env`)
Use this as your `.env.example`:

```env
# --- DB ---
DATABASE_URL=postgres://postgres:postgres@localhost:5432/hestixdb
DB_MAX_CONNECTIONS=5

# --- Server ---
HOST=localhost
PORT=5000
LOG_FILTER=info

# Exact frontend origin (no trailing slash)
CORS_ALLOWED_ORIGIN=http://localhost:5173

# Where to send the browser after successful login
FRONTEND_URL=http://localhost:5173

# --- OIDC (ZITADEL) ---
# Public issuer URL (your ZITADEL base URL)
OIDC_ISSUER_URL=http://localhost:8080

# Your ZITADEL Application’s Client ID
OIDC_CLIENT_ID=334480673379254275

# PKCE uses no client secret (leave empty if you selected “None” auth method)
OIDC_CLIENT_SECRET=

# Must EXACTLY match a Redirect URI configured on the ZITADEL app
OIDC_REDIRECT_URL=http://localhost:5000/api/auth/callback

# Space‑separated scopes
# - openid (required), profile/email (identity), offline_access (refresh token)
OIDC_SCOPES="openid profile email offline_access"
```

> **Docker note:** if your API runs in Docker and ZITADEL is another container, set `OIDC_ISSUER_URL=http://zitadel:8080` (service name), not `localhost`. The browser‑facing redirect URI should still use `http://localhost:5000/...`.

## 🚀 Running Locally

1) **Run migrations**
```bash
sqlx migrate run
# If you use SQLx offline mode:
cargo sqlx prepare
```

2) **Start the API**
```bash
cargo run
```
API listens on `http://localhost:5000`.

3) **Auth endpoints**
- `GET /api/auth/login` → redirect to ZITADEL (starts PKCE)
- `GET /api/auth/callback` → exchanges code, sets cookies, redirects to `FRONTEND_URL`
- `POST /api/auth/refresh` → refreshes `access_token` if `refresh_token` cookie is present
- `POST /api/auth/logout` → clears cookies
- `GET /api/me` → returns validated token claims (via extractor)

## 👤 User Sync

On successful login the backend validates tokens and **creates/updates** a user in the DB using:
- Stable identity pair: **`iss` + `sub`**
- Preferred identity data from **ID token** (if enabled), falling back to `email` or `sub`

Roles are derived from the access token’s ZITADEL claim:
- `urn:zitadel:iam:org:project:roles` → `Vec<String>` (e.g., `["user"]`)

## 🔑 Role Checks

Example macros:
```rust
require_role!(claims, "admin");
require_any_role!(claims, ["editor", "admin"]);
```
Your extractor fills `claims.roles` from ZITADEL’s role object.

## 🔌 Typical Frontend Flow

1. SPA calls `GET /api/auth/login` → browser is redirected to ZITADEL.
2. User authenticates → ZITADEL redirects back to `/api/auth/callback?code=...`.
3. Backend exchanges code (with PKCE), sets cookies, then redirects to `FRONTEND_URL`.
4. SPA calls API with cookies. Extractor validates tokens; protected routes use `require_role!` macros.

## 🧪 Local Testing Tips

- **Redirect URI mismatch** → Ensure `OIDC_REDIRECT_URL` exactly matches ZITADEL’s config.
- **Can’t login** → Confirm the test user is in the project and has required roles.
- **Missing email/username** → Turn on **User Info inside ID Token** and request `profile email` scopes; ensure the user has an email set (and optionally verified).
- **CORS errors** → `CORS_ALLOWED_ORIGIN` must exactly match the SPA origin.
- **Cookies not sent** → If cross‑site in prod, configure `SameSite=None; Secure` and TLS.

## ✨ What changed vs Keycloak?

- Replaced Keycloak‑specific config with **generic OIDC** for ZITADEL.
- Switched to **PKCE** (no client secret).
- Introduced generic OIDC module (`src/oidc/*`) and a **ZITADEL provider** (`src/providers/zitadel/*`).
- User sync now keys off **(iss, sub)** instead of a Keycloak‑specific UUID.
- Role mapping comes from ZITADEL’s `urn:zitadel:iam:org:project:roles` claim.
