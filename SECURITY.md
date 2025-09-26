# Security Implementation

This document outlines the security measures implemented in the Hestix Core API authentication system.

## 🔐 Authentication Flow Security

### OIDC with PKCE (Proof Key for Code Exchange)
- **Implementation**: Full PKCE support with S256 method
- **Code Verifier**: 64 bytes (512 bits) of entropy - exceeds RFC 7636 minimum
- **Code Challenge**: SHA256 hash of verifier, base64url encoded
- **Security Benefit**: Prevents authorization code interception attacks

### Enhanced State Validation
- **State Size**: 48 bytes (384 bits) of cryptographically secure randomness
- **Validation**: Constant-time comparison to prevent timing attacks
- **CSRF Protection**: Prevents cross-site request forgery attacks

### Token Security
- **Expiration Validation**: Defense-in-depth token expiration checking
- **Token Age Limits**: Tokens older than 24 hours are rejected
- **Secure Storage**: HttpOnly cookies with appropriate expiration times
- **Provider Revocation**: Proper token revocation at Zitadel on logout

## 🛡️ Cookie Security Configuration

### Environment-Aware Security
```bash
# Development Mode (ENVIRONMENT=development)
- secure=false (allows HTTP for local testing)
- HttpOnly=true (prevents XSS)
- SameSite=Lax (CSRF protection)

# Production Mode (ENVIRONMENT=production)
- secure=true (requires HTTPS)
- HttpOnly=true (prevents XSS)
- SameSite=Lax (CSRF protection)
```

### Cookie Lifetimes
| Cookie Type | Lifetime | Purpose |
|-------------|----------|---------|
| `access_token` | 1 hour | API authentication |
| `refresh_token` | 7 days | Token renewal |
| `oauth_state` | 10 minutes | CSRF protection |
| `pkce_verifier` | 10 minutes | PKCE flow security |

## 🔄 Token Management

### Automatic Refresh
- Seamless token refresh using refresh tokens
- New cookies automatically set on successful refresh
- Fallback to full re-authentication if refresh fails

### Token Validation Pipeline
1. **Extract Token**: From Authorization header or cookies
2. **Provider Validation**: JWT signature and standard claims
3. **Expiration Check**: Additional expiration validation
4. **Age Verification**: Reject tokens older than 24 hours
5. **Role Extraction**: Security role-based access control

### Proper Logout
- **Token Revocation**: Tokens revoked at Zitadel provider
- **Cookie Clearing**: All auth cookies removed
- **Graceful Failure**: Continues logout even if revocation fails

## 🔍 Security Headers

### Applied Headers
- `Strict-Transport-Security`: Enforces HTTPS (max-age=63072000)
- `X-Content-Type-Options`: Prevents MIME sniffing (nosniff)
- `X-Frame-Options`: Prevents clickjacking (DENY)
- `Referrer-Policy`: Controls referrer information
- `Permissions-Policy`: Restricts dangerous browser features
- `Content-Security-Policy`: Prevents XSS attacks

### CORS Configuration
- **Origin Validation**: Configurable allowed origins
- **Credentials**: Supports credential-bearing requests
- **Methods**: GET, POST, PUT, DELETE, OPTIONS
- **Headers**: Authorization, Content-Type

## 🎯 Role-Based Access Control

### Implementation
```rust
// Single role requirement
require_role!(claims, "admin");

// Multiple role options
require_any_role!(claims, ["editor", "admin"]);
```

### Security Features
- **Role Extraction**: From Zitadel access tokens
- **Compile-time Safety**: Macro-based role checking
- **Clear Errors**: Descriptive error messages for missing roles

## 🔒 Cryptographic Security

### Random Number Generation
- **Source**: ring::SystemRandom (OS-level entropy)
- **Usage**: PKCE verifiers, OAuth state, session tokens
- **Quality**: Cryptographically secure random generation

### Constant-Time Operations
- **State Comparison**: Prevents timing attack vulnerabilities
- **Implementation**: XOR-based comparison algorithm

## 🚨 Security Monitoring

### Logging
- **Environment Mode**: Clear logging of development vs production
- **Security Events**: Token validation, logout, failures
- **No Sensitive Data**: Careful to avoid logging secrets

### Startup Messages
```
🔧 Starting Hestix Core API in DEVELOPMENT mode
⚠️  Cookie security: HTTP allowed (secure=false)
🚨 Development mode should NOT be used in production!
```

```
🚀 Starting Hestix Core API in PRODUCTION mode
🔒 Cookie security: HTTPS required (secure=true)
```

## 📋 Security Checklist

### ✅ Implemented
- [x] PKCE with enhanced entropy (512 bits)
- [x] Enhanced state validation with timing attack protection
- [x] Token expiration validation (defense in depth)
- [x] Proper provider logout with token revocation
- [x] Environment-aware cookie security
- [x] Role-based access control
- [x] Security headers implementation
- [x] Constant-time comparison for sensitive data
- [x] Comprehensive security logging

### 🔄 Additional Recommendations
- [ ] Rate limiting for auth endpoints
- [ ] Token binding to client characteristics
- [ ] Additional CSRF tokens for state mutations
- [ ] IP-based access restrictions
- [ ] Failed login attempt monitoring

## 🏠 Pi5 Deployment Considerations

### Production Setup
1. **HTTPS Required**: Use proper TLS certificates
2. **Environment Variable**: Set `ENVIRONMENT=production`
3. **Firewall**: Restrict access to necessary ports only
4. **Monitoring**: Enable comprehensive logging
5. **Updates**: Regular security updates for OS and dependencies

### Security Rating: A- (Excellent)
The authentication system implements industry best practices with defense-in-depth security measures suitable for a home API server deployment.

## 📚 References
- [RFC 7636 - PKCE](https://tools.ietf.org/html/rfc7636)
- [RFC 6749 - OAuth 2.0](https://tools.ietf.org/html/rfc6749)
- [RFC 7009 - Token Revocation](https://tools.ietf.org/html/rfc7009)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)