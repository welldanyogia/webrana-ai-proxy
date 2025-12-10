# Security Fixes Implementation Guide

**Based on:** Security Audit Report - Task 21
**Date:** December 10, 2024
**Priority:** IMMEDIATE (Before Production)

---

## Fix 1: Remove JWT Secret Fallback [CRITICAL]

**Risk:** HIGH - Default JWT secret could be exploited
**Effort:** 5 minutes
**Files to modify:** 3

### Implementation

#### File 1: `backend/src/routes/auth.rs`

**Lines 106-107, 130-131, 167-168:**
```rust
// BEFORE (3 occurrences)
let jwt_secret = std::env::var("JWT_SECRET")
    .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

// AFTER
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("CRITICAL: JWT_SECRET environment variable must be set");
```

**Search and replace:**
```bash
# In backend/src/routes/auth.rs, replace all 3 occurrences
sed -i 's/.unwrap_or_else(|_| "default-secret-change-in-production".to_string())/.expect("CRITICAL: JWT_SECRET environment variable must be set")/g' backend/src/routes/auth.rs
```

#### File 2: `backend/src/middleware/auth.rs`

**Lines 79-82:**
```rust
// BEFORE
let jwt_secret = std::env::var("JWT_SECRET")
    .unwrap_or_else(|_| {
        tracing::error!("JWT_SECRET not configured");
        "default-secret-change-in-production".to_string()
    });

// AFTER
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("CRITICAL: JWT_SECRET environment variable must be set");
```

### Testing
```bash
# Test that app fails to start without JWT_SECRET
unset JWT_SECRET
cargo run  # Should panic with clear error message

# Test that app starts with JWT_SECRET
export JWT_SECRET="test-secret-at-least-32-chars-long"
cargo run  # Should start successfully
```

---

## Fix 2: Mask Email Addresses in Logs [MEDIUM]

**Risk:** MEDIUM - PII exposure in logs
**Effort:** 30 minutes
**Files to create/modify:** 2

### Implementation

#### Step 1: Create masking utility

**Create file:** `backend/src/utils/masking.rs`
```rust
//! Utilities for masking sensitive data in logs

/// Mask an email address for logging
/// Example: "user@example.com" -> "u***@example.com"
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let username = &email[..at_pos];
        let domain = &email[at_pos..];

        if username.len() > 2 {
            format!("{}***{}", &username[..1], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        "***@***.***".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("john@example.com"), "j***@example.com");
        assert_eq!(mask_email("ab@test.com"), "***@test.com");
        assert_eq!(mask_email("a@b.c"), "***@b.c");
        assert_eq!(mask_email("invalid"), "***@***.***");
    }
}
```

#### Step 2: Export masking module

**File:** `backend/src/utils/mod.rs`
```rust
// Add this line
pub mod masking;

// Existing exports
pub mod encryption;
pub mod password;
```

#### Step 3: Update email service

**File:** `backend/src/services/email_service.rs`

**Add import at top:**
```rust
use crate::utils::masking::mask_email;
```

**Lines 182-186, replace:**
```rust
// BEFORE
tracing::info!(
    to = %request.to,
    template = %request.template.as_str(),
    "Email sent successfully"
);

// AFTER
tracing::info!(
    to = %mask_email(&request.to),
    template = %request.template.as_str(),
    "Email sent successfully"
);
```

### Testing
```bash
# Run unit tests
cargo test mask_email

# Check email service logs
cargo test send_email -- --nocapture
# Verify logs show masked emails like "u***@example.com"
```

---

## Fix 3: Add Origin Header Validation [LOW]

**Risk:** LOW - CSRF defense-in-depth
**Effort:** 1 hour
**Files to create/modify:** 3

### Implementation

#### Step 1: Create CSRF middleware

**Create file:** `backend/src/middleware/csrf.rs`
```rust
//! CSRF protection via Origin/Referer header validation

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};

/// Validate Origin/Referer headers for state-changing requests
pub async fn csrf_protection(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Only check state-changing methods
    if !matches!(
        request.method(),
        &Method::POST | &Method::DELETE | &Method::PUT | &Method::PATCH
    ) {
        return Ok(next.run(request).await);
    }

    // Get Origin or Referer header
    let origin = request
        .headers()
        .get("Origin")
        .and_then(|h| h.to_str().ok());

    let referer = request
        .headers()
        .get("Referer")
        .and_then(|h| h.to_str().ok());

    // Validate against allowed origins
    let allowed_origins = vec![
        "https://webrana.id",
        "https://app.webrana.id",
        "https://api.webrana.id",
        "http://localhost:3000", // Development
    ];

    let is_valid = origin
        .or(referer)
        .map(|header| {
            allowed_origins
                .iter()
                .any(|allowed| header.starts_with(allowed))
        })
        .unwrap_or(false);

    if !is_valid {
        tracing::warn!(
            origin = ?origin,
            referer = ?referer,
            method = ?request.method(),
            path = ?request.uri().path(),
            "CSRF protection: Invalid Origin/Referer header"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

#### Step 2: Export CSRF module

**File:** `backend/src/middleware/mod.rs`
```rust
// Add this line
pub mod csrf;

// Existing exports
pub mod auth;
pub mod rate_limit;
pub mod admin;
pub mod security_headers;
```

#### Step 3: Apply middleware to routes

**File:** `backend/src/main.rs`

**Add import:**
```rust
use middleware::csrf::csrf_protection;
```

**Apply to state-changing routes (around line 84):**
```rust
// Build application router
let app = Router::new()
    .route("/health", get(health_check))
    .route("/health/db", get(health_check_db))
    .nest("/auth", routes::auth::router())
    .nest("/api-keys", api_keys_routes)
    .nest("/usage", usage_routes)
    .nest("/v1", proxy_routes)
    // Apply CSRF protection to all routes except health checks
    .layer(axum_middleware::from_fn(csrf_protection))
    .layer(Extension(state));
```

### Testing
```bash
# Test with valid origin
curl -X POST http://localhost:3000/auth/login \
  -H "Origin: http://localhost:3000" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
# Should succeed

# Test with invalid origin
curl -X POST http://localhost:3000/auth/login \
  -H "Origin: https://evil.com" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
# Should return 403 Forbidden
```

---

## Fix 4: Apply Security Headers Middleware [LOW]

**Risk:** LOW - Missing security headers
**Effort:** 10 minutes
**Files to modify:** 1

### Implementation

**File:** `backend/src/main.rs`

**Add import:**
```rust
use middleware::security_headers::security_headers;
```

**Apply middleware (around line 84):**
```rust
// Build application router
let app = Router::new()
    .route("/health", get(health_check))
    .route("/health/db", get(health_check_db))
    .nest("/auth", routes::auth::router())
    .nest("/api-keys", api_keys_routes)
    .nest("/usage", usage_routes)
    .nest("/v1", proxy_routes)
    // Apply security headers to all responses
    .layer(axum_middleware::from_fn(security_headers))
    .layer(Extension(state));
```

**Note:** HSTS headers should only be enabled in production:
```rust
// Only in production
if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
    app = app.layer(axum_middleware::from_fn(hsts_headers));
}
```

### Testing
```bash
# Check security headers are present
curl -I http://localhost:3000/health

# Should include:
# X-Frame-Options: DENY
# X-Content-Type-Options: nosniff
# X-XSS-Protection: 1; mode=block
# Content-Security-Policy: default-src 'self'; ...
# Referrer-Policy: strict-origin-when-cross-origin
```

---

## Infrastructure Verification Checklist

Before production deployment, DevOps team must verify:

### TLS Configuration

```bash
# Test TLS versions (should only show TLS 1.2 and 1.3)
nmap --script ssl-enum-ciphers -p 443 webrana.id

# Or use testssl.sh
testssl.sh webrana.id

# Expected output:
# TLS 1.2: Supported ✓
# TLS 1.3: Supported ✓
# TLS 1.0: Not supported ✓
# TLS 1.1: Not supported ✓
# SSL 3.0: Not supported ✓
```

### Certificate Validation

```bash
# Check certificate
openssl s_client -connect webrana.id:443 -servername webrana.id

# Verify:
# - Valid certificate from trusted CA
# - Certificate not expired
# - Complete certificate chain
# - SAN/CN matches domain
```

### Load Balancer Configuration

DigitalOcean Load Balancer settings:
- [ ] TLS termination enabled
- [ ] TLS 1.2 minimum version
- [ ] Strong cipher suites only
- [ ] HTTP → HTTPS redirect enabled
- [ ] Health checks configured
- [ ] Sticky sessions (if needed)

### Cloudflare Configuration

If using Cloudflare:
- [ ] SSL/TLS mode: Full (strict)
- [ ] Minimum TLS version: 1.2
- [ ] TLS 1.3: Enabled
- [ ] HSTS enabled (via Cloudflare)
- [ ] Always use HTTPS: On

---

## Environment Variables Checklist

Ensure these are set before deployment:

```bash
# Required
JWT_SECRET="<random-256-bit-secret>"
MASTER_ENCRYPTION_KEY="<base64-encoded-32-byte-key>"
DATABASE_URL="postgresql://..."
REDIS_URL="redis://..."

# Recommended
ENVIRONMENT="production"
RUST_LOG="info"
MIDTRANS_SERVER_KEY="<midtrans-key>"
MIDTRANS_CLIENT_KEY="<midtrans-client-key>"
RESEND_API_KEY="<email-api-key>"
```

---

## Testing After Fixes

### Unit Tests
```bash
cd backend
cargo test
```

### Integration Tests
```bash
# Test authentication
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Password123!"}'

# Verify JWT secret panic (should fail)
unset JWT_SECRET
cargo run  # Should panic with clear error
```

### Security Headers Test
```bash
# Check all security headers
curl -I https://staging.webrana.id/health | grep -E "X-|Content-Security|Strict-Transport"
```

### Log Audit
```bash
# Check logs don't contain PII
docker logs backend-container | grep -E "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
# Should only show masked emails like "u***@example.com"
```

---

## Rollout Plan

### Phase 1: Staging (Immediate)
1. Apply Fix 1 (JWT secret)
2. Apply Fix 2 (Email masking)
3. Run full test suite
4. Deploy to staging
5. Verify with security tests

### Phase 2: Production (After Staging Verification)
1. Verify infrastructure TLS configuration
2. Apply Fix 3 (CSRF middleware)
3. Apply Fix 4 (Security headers)
4. Deploy to production
5. Monitor logs for issues
6. Run penetration tests

### Phase 3: Ongoing (Post-Launch)
1. Weekly dependency audits (`cargo audit`)
2. Monthly security reviews
3. Quarterly penetration tests
4. Continuous log monitoring for anomalies

---

## Verification Commands

After deployment:

```bash
# 1. Check TLS
testssl.sh webrana.id

# 2. Check security headers
curl -I https://webrana.id | grep -E "X-|Content-Security|Strict-Transport"

# 3. Test CSRF protection
curl -X POST https://webrana.id/api/some-endpoint \
  -H "Origin: https://evil.com" \
  # Should return 403

# 4. Verify JWT requirement
curl https://webrana.id/api-keys  # Should return 401 Unauthorized

# 5. Check logs for PII
# Logs should show masked emails only
```

---

## Sign-off

Complete this checklist before production:

- [ ] Fix 1: JWT secret fallback removed
- [ ] Fix 2: Email masking implemented
- [ ] Fix 3: CSRF middleware applied (optional)
- [ ] Fix 4: Security headers middleware applied
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Staging deployment successful
- [ ] Infrastructure TLS verified
- [ ] Security headers verified
- [ ] Log audit confirms no PII exposure
- [ ] DevOps team sign-off
- [ ] Security team sign-off

**Prepared by:** SENTINEL
**Date:** December 10, 2024
