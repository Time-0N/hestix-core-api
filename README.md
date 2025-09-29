# Hestix Core API — ZITADEL (OIDC + PKCE)

The **Hestix Core API** is the central backend for the Hestix ecosystem, designed to run on a Raspberry Pi 5 as a home API server. It handles authentication via **ZITADEL** (OIDC + PKCE), user syncing, and exposes a clean, secure, type‑safe HTTP API built on **Rust/Axum** with enterprise-grade security.

## 🧰 Tech Stack
- **Rust**
- **Axum** (HTTP framework)
- **Tokio** (async runtime)
- **SQLx** (compile‑time checked Postgres client)
- **PostgreSQL** (relational datastore)
- **ZITADEL** (OIDC: Authorization Code + PKCE)
- **Moka** (in‑memory async cache)
- **anyhow**, **tracing**, **tower-http**

## 📁 Project Structure (Auxums Design Pattern)
```text
├── Cargo.toml
├── migrations/                        # SQLx migrations
├── SECURITY.md                        # Security implementation documentation
├── ENVIRONMENT.md                     # Environment configuration guide
└── src/
    ├── main.rs                        # Entry point
    ├── bootstrap.rs                   # Application initialization
    ├── app_state.rs                   # Application state composition
    ├── domain/                        # Domain layer (entities, repositories)
    │   ├── entities/                  # Domain entities (User, etc.)
    │   └── repositories/              # Repository traits
    ├── application/                   # Application layer (services, DTOs)
    │   ├── auth_service.rs            # Authentication business logic
    │   ├── user_service.rs            # User management with integrated cache
    │   ├── user_sync.rs               # Automated user synchronization
    │   └── dto/                       # Data transfer objects
    ├── infrastructure/                # Infrastructure layer
    │   ├── config/                    # Configuration management
    │   ├── persistence/               # Database implementations
    │   ├── oidc/                      # OIDC providers and implementations
    │   │   ├── providers/zitadel/     # ZITADEL-specific implementation
    │   │   ├── claims.rs              # JWT claims structure
    │   │   ├── discovery.rs           # OIDC discovery
    │   │   └── provider.rs            # OIDC provider traits
    │   └── web/                       # Web layer (routes, handlers)
    │       ├── handlers/              # HTTP request handlers
    │       ├── routes/                # Route definitions
    │       └── cookies/               # Cookie management
    └── shared/                        # Shared utilities
        ├── middleware/                # Clean middleware architecture
        │   ├── auth/                  # Authentication middleware
        │   ├── cors.rs                # CORS configuration
        │   ├── headers.rs             # Security headers
        │   └── layers.rs              # Middleware composition
        ├── errors/                    # Error handling
        └── role.rs                    # Role-based access control macros
```

## 🔐 Authentication & Security

### Enhanced OIDC + PKCE Flow
- **Authorization Code + PKCE** with enhanced security (512-bit PKCE verifier)
- **Constant-time state validation** to prevent timing attacks
- **Enhanced entropy** for all cryptographic operations
- **Token expiration validation** with defense-in-depth approach
- **Provider token revocation** on logout

### Tokens & Lifetimes
- **Access Token (JWT):** 1 hour, used for API auth + roles
- **Refresh Token:** 7 days, for token renewal (reduced from 30 days for security)
- **OAuth State:** 10 minutes, for CSRF protection (384-bit entropy)
- **PKCE Verifier:** 10 minutes, for code exchange security (512-bit entropy)

### Environment-Aware Cookie Security
- **Development Mode** (`ENVIRONMENT=development`): HTTP-compatible for local testing
- **Production Mode** (`ENVIRONMENT=production`): HTTPS-only for secure deployment
- All cookies use `HttpOnly=true` and `SameSite=Lax` for XSS/CSRF protection

**Required ZITADEL app settings:**
- **Type:** Web
- **Response type:** `code`
- **Grant types:** `authorization_code`, `refresh_token`
- **Authentication method:** `none` (PKCE; no client secret)
- **Redirect URIs:** include your backend callback (e.g. `http://localhost:5000/api/auth/callback`)
- **(Recommended)** “**User Info inside ID Token**”: **ON** to receive `email` / `preferred_username` in the ID token.
- Assign users **project roles** so the access token includes them.

## ⚙️ Configuration

### Environment Variables
Copy `.env.example` to `.env` and configure:

```env
# =========================
# Database (PostgreSQL)
# =========================
DATABASE_URL=postgres://postgres:postgres@localhost:5432/hestixdb
DB_MAX_CONNECTIONS=5

# =========================
# Server (Axum)
# =========================
HOST=localhost
PORT=5000
LOG_FILTER=info

# Environment mode: "development" or "production"
# In development mode, cookies will not require HTTPS (secure=false)
# In production mode, cookies will require HTTPS (secure=true)
ENVIRONMENT=development

# CORS: the exact origin of your frontend that will call the API
CORS_ALLOWED_ORIGIN=http://localhost:5173

# Optional: where to redirect the browser after successful login
FRONTEND_URL=http://localhost:5173

# =========================
# OIDC (ZITADEL) — Code Flow + PKCE
# =========================
OIDC_ISSUER_URL=http://localhost:8080
OIDC_CLIENT_ID=334480673379254275
OIDC_REDIRECT_URL=http://localhost:5000/api/auth/callback
OIDC_SCOPES="openid profile email offline_access"

# =========================
# ZITADEL User Sync (Optional)
# =========================
# Option 1: Personal Access Token as string
# ZITADEL_SERVICE_TOKEN=your_personal_access_token_here

# Option 2: Path to token file
# ZITADEL_SERVICE_TOKEN_PATH=/path/to/token.pat
```

### User Synchronization
- **Manual Sync**: Users are synced on login automatically
- **Automated Sync**: Set `ZITADEL_SERVICE_TOKEN` or `ZITADEL_SERVICE_TOKEN_PATH` for background sync every 24 hours
- **Cache Integration**: User data is cached in memory for performance

> **Docker note:** if your API runs in Docker and ZITADEL is another container, set `OIDC_ISSUER_URL=http://zitadel:8080` (service name), not `localhost`. The browser‑facing redirect URI should still use `http://localhost:5000/...`.

## 🚀 Getting Started

### Prerequisites
- **Rust** (latest stable)
- **PostgreSQL** database
- **ZITADEL** instance (local or hosted)

### Setup
1) **Clone and configure**
```bash
git clone <repository>
cd hestix-core-api
cp .env.example .env
# Edit .env with your configuration
```

2) **Database setup**
```bash
# Run migrations
sqlx migrate run

# For offline compilation (optional)
cargo sqlx prepare
```

3) **Start the API**
```bash
cargo run
# Look for: "Booting with environment: development"
```

### API Endpoints
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/auth/login` | GET | Initiate OIDC login with PKCE |
| `/api/auth/callback` | GET | Handle OIDC callback, set cookies |
| `/api/auth/refresh` | POST | Refresh access token |
| `/api/auth/logout` | POST | Logout with provider token revocation |
| `/api/auth/me` | GET | Get current user claims |
| `/api/user/info` | GET | Get current user information |

## 🏗️ Architecture Features

### Clean Architecture (Auxums Pattern)
- **Domain Layer**: Entities and repository traits
- **Application Layer**: Business logic and services with integrated caching
- **Infrastructure Layer**: Database, OIDC providers, web framework
- **Shared Layer**: Middleware, errors, utilities

### Performance & Scalability
- **In-Memory Caching**: User data cached with Moka for fast access
- **Connection Pooling**: SQLx connection pool for database efficiency
- **Async Throughout**: Full async/await with Tokio runtime
- **Memory Efficient**: Smart caching with TTL and capacity limits

### Security Features
- **Enterprise-Grade**: A- security rating with comprehensive protections
- **Defense in Depth**: Multiple layers of security validation
- **Environment Aware**: Automatic security configuration based on deployment mode
- **Attack Resistant**: Timing attack prevention, enhanced entropy
- **Comprehensive Logging**: Security events without exposing sensitive data

## 🔑 Role-Based Access Control

### Usage Examples
```rust
// Single role requirement
require_role!(claims, "admin");

// Multiple role options
require_any_role!(claims, ["editor", "admin"]);
```

### Role Extraction
- Roles from ZITADEL access token: `urn:zitadel:iam:org:project:roles`
- Automatic mapping to `Vec<String>` (e.g., `["user", "admin"]`)
- Compile-time safety with descriptive error messages

## 🔌 Typical Frontend Flow

1. SPA calls `GET /api/auth/login` → browser is redirected to ZITADEL.
2. User authenticates → ZITADEL redirects back to `/api/auth/callback?code=...`.
3. Backend exchanges code (with PKCE), sets cookies, then redirects to `FRONTEND_URL`.
4. SPA calls API with cookies. Extractor validates tokens; protected routes use `require_role!` macros.

## 🏠 Raspberry Pi 5 Deployment

### Production Configuration
```bash
# Set production environment
ENVIRONMENT=production

# Use HTTPS URLs
OIDC_ISSUER_URL=https://your-zitadel-domain.com
OIDC_REDIRECT_URL=https://your-pi5-domain.com:5000/api/auth/callback
FRONTEND_URL=https://your-frontend-domain.com
```

### Security Considerations
- **HTTPS Required**: Production mode enforces secure cookies
- **Firewall**: Restrict access to necessary ports
- **Regular Updates**: Keep OS and dependencies updated
- **Monitoring**: Enable comprehensive logging
- **Backup**: Regular database backups

### Performance Optimization
- **Database**: Optimize PostgreSQL for Pi5 resources
- **Cache Settings**: Adjust cache size based on available memory
- **Connection Limits**: Configure appropriate database connection limits

## 🧪 Troubleshooting

| Issue | Solution |
|-------|----------|
| **Redirect URI mismatch** | Ensure `OIDC_REDIRECT_URL` exactly matches ZITADEL config |
| **Login fails** | Verify user has project roles and email set |
| **CORS errors** | `CORS_ALLOWED_ORIGIN` must exactly match SPA origin |
| **Cookies not working** | Check `ENVIRONMENT` setting and HTTPS in production |
| **Database connection** | Verify `DATABASE_URL` and PostgreSQL is running |
| **Token validation fails** | Check ZITADEL issuer URL and client ID |

## 📚 Documentation

- **[SECURITY.md](SECURITY.md)**: Comprehensive security implementation details
- **[ENVIRONMENT.md](ENVIRONMENT.md)**: Environment configuration guide
- **[API Documentation]**: Interactive API docs available at `/docs` when running

## 🔄 Migration from Keycloak

### Key Changes
- **Generic OIDC**: Replaced Keycloak-specific implementation
- **PKCE Security**: Enhanced security with Proof Key for Code Exchange
- **Clean Architecture**: Restructured to Auxums design pattern
- **Integrated Caching**: Removed separate resolver layer
- **Enhanced Security**: Multiple security improvements and timing attack prevention

### Migration Steps
1. Update OIDC configuration for ZITADEL
2. Configure PKCE in ZITADEL application settings
3. Update environment variables
4. Run database migrations
5. Test authentication flow

---

**Security Rating: A-** | **Production Ready** | **Pi5 Optimized**
