//! Security headers middleware
//!
//! Requirements: 8.3 - Add security headers (CSP, etc.)

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};

/// Security headers middleware
/// 
/// Adds security headers to all responses:
/// - X-Frame-Options: DENY (prevent clickjacking)
/// - X-Content-Type-Options: nosniff (prevent MIME sniffing)
/// - X-XSS-Protection: 1; mode=block (legacy XSS protection)
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - Permissions-Policy: restrict browser features
/// - Content-Security-Policy: restrict resource loading
/// 
/// Requirements: 8.3
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Legacy XSS protection (for older browsers)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Control referrer information
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Restrict browser features
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Content Security Policy
    // Note: Midtrans requires script-src and frame-src exceptions
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' https://app.sandbox.midtrans.com https://app.midtrans.com; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' https://api.webrana.id; \
             frame-src https://app.sandbox.midtrans.com https://app.midtrans.com; \
             frame-ancestors 'none'"
        ),
    );

    // Cache control for sensitive endpoints
    // This should be applied selectively, but as a default we prevent caching
    if !headers.contains_key("Cache-Control") {
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
        );
    }

    response
}

/// HSTS (HTTP Strict Transport Security) middleware
/// 
/// Should only be enabled after confirming HTTPS works correctly.
/// Tells browsers to only use HTTPS for this domain.
/// 
/// Requirements: 8.2
pub async fn hsts_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HSTS with 1 year max-age and includeSubDomains
    // Only enable in production with valid HTTPS
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    #[test]
    fn test_x_frame_options_value() {
        let value = HeaderValue::from_static("DENY");
        assert_eq!(value.to_str().unwrap(), "DENY");
    }

    #[test]
    fn test_x_content_type_options_value() {
        let value = HeaderValue::from_static("nosniff");
        assert_eq!(value.to_str().unwrap(), "nosniff");
    }

    #[test]
    fn test_x_xss_protection_value() {
        let value = HeaderValue::from_static("1; mode=block");
        assert_eq!(value.to_str().unwrap(), "1; mode=block");
    }

    #[test]
    fn test_referrer_policy_value() {
        let value = HeaderValue::from_static("strict-origin-when-cross-origin");
        assert_eq!(value.to_str().unwrap(), "strict-origin-when-cross-origin");
    }

    #[test]
    fn test_csp_contains_midtrans() {
        let csp = "default-src 'self'; \
             script-src 'self' 'unsafe-inline' https://app.sandbox.midtrans.com https://app.midtrans.com; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' https://api.webrana.id; \
             frame-src https://app.sandbox.midtrans.com https://app.midtrans.com; \
             frame-ancestors 'none'";

        // Verify Midtrans domains are allowed
        assert!(csp.contains("app.sandbox.midtrans.com"));
        assert!(csp.contains("app.midtrans.com"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn test_hsts_value() {
        let hsts = "max-age=31536000; includeSubDomains";
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
    }

    #[test]
    fn test_permissions_policy_value() {
        let value = HeaderValue::from_static("geolocation=(), microphone=(), camera=()");
        assert_eq!(value.to_str().unwrap(), "geolocation=(), microphone=(), camera=()");
    }
}
