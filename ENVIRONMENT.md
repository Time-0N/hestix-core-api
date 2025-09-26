# Environment Configuration

This project supports environment-aware cookie security configuration to enable local development over HTTP while maintaining security in production.

## Environment Modes

### Development Mode (`ENVIRONMENT=development`)
- **Purpose**: Local development and testing
- **Cookie Security**: `secure=false` - allows cookies over HTTP
- **Use Case**: Testing on `http://localhost:5000` without HTTPS certificates

### Production Mode (`ENVIRONMENT=production`)
- **Purpose**: Production deployment
- **Cookie Security**: `secure=true` - requires HTTPS for all cookies
- **Use Case**: Deployment on Pi5 with proper TLS certificates

## Configuration

Set the `ENVIRONMENT` variable in your `.env` file:

```bash
# For local development
ENVIRONMENT=development

# For production deployment
ENVIRONMENT=production
```

If not set, the application defaults to `development` mode.

## Cookie Security Settings

| Cookie Type | Development | Production |
|-------------|------------|------------|
| `access_token` | `secure=false` | `secure=true` |
| `refresh_token` | `secure=false` | `secure=true` |
| `oauth_state` | `secure=false` | `secure=true` |
| `pkce_verifier` | `secure=false` | `secure=true` |

All cookies maintain the following security settings in both modes:
- `HttpOnly=true` (prevents XSS access)
- `SameSite=Lax` (CSRF protection)
- Appropriate expiration times

## Security Improvements

In addition to environment-aware security, the following improvements have been made:

### Shorter Refresh Token Lifetime
- **Before**: 30 days
- **After**: 7 days (reduced for better security on home servers)

### Token Expiration
- Access tokens: 1 hour
- Refresh tokens: 7 days
- OAuth state/PKCE: 10 minutes

## Usage Examples

### Local Development
```bash
# .env file
ENVIRONMENT=development
HOST=localhost
PORT=5000

# Cookies will work over http://localhost:5000
```

### Production Deployment
```bash
# .env file
ENVIRONMENT=production
HOST=0.0.0.0
PORT=5000

# Requires HTTPS setup for cookies to work
# Example: https://your-pi5.local:5000
```

## Migration

If you're upgrading from a previous version:

1. Add `ENVIRONMENT=development` to your `.env` file for local development
2. Set `ENVIRONMENT=production` when deploying to production
3. Ensure HTTPS is properly configured in production mode

The application will continue to work without this variable (defaulting to development mode) but it's recommended to explicitly set it.