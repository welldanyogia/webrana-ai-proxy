//! Authentication middleware for JWT and API key validation.

use axum::{
    extract::{Extension, Request},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use uuid::Uuid;

use crate::services::auth_service::Claims;

/// Error response for authentication failures
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub code: String,
}

/// Authenticated user information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub plan: String,
}

/// JWT authentication middleware
/// 
/// Extracts and validates Bearer token from Authorization header.
/// On success, attaches AuthUser to request extensions.
/// 
/// # Arguments
/// * `request` - The incoming HTTP request
/// * `next` - The next middleware/handler in the chain
/// 
/// # Returns
/// Response from the next handler or an authentication error
pub async fn jwt_auth(
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            header.trim_start_matches("Bearer ").trim()
        }
        Some(_) => {
            return auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format",
                "INVALID_AUTH_HEADER",
            );
        }
        None => {
            return auth_error(
                StatusCode::UNAUTHORIZED,
                "Missing authorization header",
                "MISSING_AUTH_HEADER",
            );
        }
    };

    if token.is_empty() {
        return auth_error(
            StatusCode::UNAUTHORIZED,
            "Empty token",
            "EMPTY_TOKEN",
        );
    }

    // Get JWT secret from environment
    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            tracing::error!("JWT_SECRET not configured");
            return auth_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error",
                "CONFIG_ERROR",
            );
        }
    };

    // Decode and validate token
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let validation = Validation::default();

    let claims = match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            let (message, code) = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    ("Token has expired", "TOKEN_EXPIRED")
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    ("Invalid token", "INVALID_TOKEN")
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    ("Invalid token signature", "INVALID_SIGNATURE")
                }
                _ => ("Token validation failed", "TOKEN_VALIDATION_FAILED"),
            };
            return auth_error(StatusCode::UNAUTHORIZED, message, code);
        }
    };

    // Verify token type is "access"
    if claims.token_type != "access" {
        return auth_error(
            StatusCode::UNAUTHORIZED,
            "Invalid token type",
            "INVALID_TOKEN_TYPE",
        );
    }

    // Parse user ID
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid user ID in token",
                "INVALID_USER_ID",
            );
        }
    };

    // Create AuthUser and attach to request extensions
    let auth_user = AuthUser {
        user_id,
        email: claims.email,
        plan: claims.plan,
    };

    request.extensions_mut().insert(auth_user);

    // Continue to next handler
    next.run(request).await
}

/// Authenticated API key user information
#[derive(Debug, Clone)]
pub struct ApiKeyUser {
    pub key_id: Uuid,
    pub user_id: Uuid,
}

/// Proxy API key authentication middleware
/// 
/// Validates proxy API keys (wbr_* format) for API access.
/// Requirements: 7.1, 7.2, 7.3, 7.4, 7.5
/// 
/// # Arguments
/// * `state` - Application state containing database pool
/// * `request` - The incoming HTTP request
/// * `next` - The next middleware/handler in the chain
/// 
/// # Returns
/// Response from the next handler or an authentication error
pub async fn api_key_auth(
    Extension(state): Extension<std::sync::Arc<crate::AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    use crate::services::proxy_key_service::ProxyKeyService;
    use crate::models::proxy_api_key::PROXY_KEY_PREFIX;

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let api_key = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let key = header.trim_start_matches("Bearer ").trim();
            // Check if it's a proxy API key (wbr_* format)
            if key.starts_with(PROXY_KEY_PREFIX) {
                key
            } else {
                return auth_error(
                    StatusCode::UNAUTHORIZED,
                    "Invalid API key format",
                    "INVALID_KEY_FORMAT",
                );
            }
        }
        Some(_) => {
            return auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format",
                "INVALID_AUTH_HEADER",
            );
        }
        None => {
            // Requirement 7.4: Missing Authorization header
            return auth_error(
                StatusCode::UNAUTHORIZED,
                "API key required",
                "API_KEY_REQUIRED",
            );
        }
    };

    if api_key.is_empty() {
        return auth_error(
            StatusCode::UNAUTHORIZED,
            "Empty API key",
            "EMPTY_API_KEY",
        );
    }

    // Validate the API key (Requirement 7.1, 7.2)
    match ProxyKeyService::validate_key(&state.db, api_key).await {
        Ok((key_id, user_id)) => {
            // Requirement 7.5: Associate request with user account
            let api_key_user = ApiKeyUser { key_id, user_id };
            request.extensions_mut().insert(api_key_user);
            next.run(request).await
        }
        Err(_) => {
            // Requirement 7.3: Invalid or revoked key
            auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid or revoked API key",
                "INVALID_API_KEY",
            )
        }
    }
}

/// Helper function to create authentication error responses
fn auth_error(status: StatusCode, message: &str, code: &str) -> Response {
    let body = Json(AuthErrorResponse {
        error: message.to_string(),
        code: code.to_string(),
    });

    (status, body).into_response()
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use crate::services::auth_service::Claims;

    // Helper to create a valid JWT token
    fn create_test_token(secret: &str, token_type: &str, expired: bool) -> String {
        let now = Utc::now();
        let exp = if expired {
            now - Duration::hours(1)
        } else {
            now + Duration::hours(24)
        };

        let claims = Claims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            email: "test@example.com".to_string(),
            plan: "free".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: token_type.to_string(),
        };

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key).unwrap()
    }

    // ============================================================
    // Unit Tests for Auth Middleware (Task 8.2)
    // **Validates: Requirements 7.3, 7.4**
    // ============================================================

    #[test]
    fn test_auth_error_response_format() {
        let response = auth_error(
            StatusCode::UNAUTHORIZED,
            "Test error message",
            "TEST_CODE",
        );
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_create_valid_token() {
        let secret = "test-secret";
        let token = create_test_token(secret, "access", false);
        
        // Token should be non-empty
        assert!(!token.is_empty());
        
        // Token should be decodable
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();
        let result = decode::<Claims>(&token, &decoding_key, &validation);
        
        assert!(result.is_ok());
        let claims = result.unwrap().claims;
        assert_eq!(claims.token_type, "access");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_create_expired_token() {
        let secret = "test-secret";
        let token = create_test_token(secret, "access", true);
        
        // Token should be non-empty
        assert!(!token.is_empty());
        
        // Token should fail validation due to expiration
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();
        let result = decode::<Claims>(&token, &decoding_key, &validation);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_type() {
        let secret = "test-secret";
        let token = create_test_token(secret, "refresh", false);
        
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();
        let result = decode::<Claims>(&token, &decoding_key, &validation);
        
        assert!(result.is_ok());
        let claims = result.unwrap().claims;
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_auth_user_struct() {
        let auth_user = AuthUser {
            user_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            email: "test@example.com".to_string(),
            plan: "free".to_string(),
        };
        
        assert_eq!(auth_user.email, "test@example.com");
        assert_eq!(auth_user.plan, "free");
    }

    #[test]
    fn test_auth_error_response_struct() {
        let error = AuthErrorResponse {
            error: "Test error".to_string(),
            code: "TEST_CODE".to_string(),
        };
        
        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, "TEST_CODE");
    }

    // Test token extraction from header
    #[test]
    fn test_bearer_token_extraction() {
        let header_value = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let token = header_value.trim_start_matches("Bearer ").trim();
        
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test");
    }

    #[test]
    fn test_invalid_header_format() {
        // Header without "Bearer " prefix
        let header_value = "Basic dXNlcjpwYXNz";
        let starts_with_bearer = header_value.starts_with("Bearer ");
        
        assert!(!starts_with_bearer);
    }

    #[test]
    fn test_empty_token_after_bearer() {
        let header_value = "Bearer ";
        let token = header_value.trim_start_matches("Bearer ").trim();
        
        assert!(token.is_empty());
    }

    // Test UUID parsing
    #[test]
    fn test_valid_uuid_parsing() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(uuid_str);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_uuid_parsing() {
        let invalid_uuid = "not-a-valid-uuid";
        let result = Uuid::parse_str(invalid_uuid);
        
        assert!(result.is_err());
    }

    // Test different token validation scenarios
    #[test]
    fn test_token_with_wrong_secret() {
        let token = create_test_token("secret1", "access", false);
        
        // Try to decode with different secret
        let decoding_key = DecodingKey::from_secret("secret2".as_bytes());
        let validation = Validation::default();
        let result = decode::<Claims>(&token, &decoding_key, &validation);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_token() {
        let malformed_token = "not.a.valid.jwt.token";
        
        let decoding_key = DecodingKey::from_secret("secret".as_bytes());
        let validation = Validation::default();
        let result = decode::<Claims>(malformed_token, &decoding_key, &validation);
        
        assert!(result.is_err());
    }

    // Note: Full integration tests for the middleware would require
    // setting up a test server with the middleware applied.
    // These unit tests verify the individual components work correctly.

    // ============================================================
    // Unit Tests for API Key Middleware (Task 12.2)
    // **Validates: Requirements 7.3**
    // ============================================================

    #[test]
    fn test_api_key_user_struct() {
        let api_key_user = ApiKeyUser {
            key_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            user_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
        };
        
        assert_eq!(api_key_user.key_id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(api_key_user.user_id.to_string(), "660e8400-e29b-41d4-a716-446655440001");
    }

    #[test]
    fn test_api_key_extraction_valid() {
        let header_value = "Bearer wbr_abc123xyz789";
        let key = header_value.trim_start_matches("Bearer ").trim();
        
        assert_eq!(key, "wbr_abc123xyz789");
        assert!(key.starts_with("wbr_"));
    }

    #[test]
    fn test_api_key_extraction_invalid_prefix() {
        let header_value = "Bearer sk-abc123xyz789";
        let key = header_value.trim_start_matches("Bearer ").trim();
        
        // Key doesn't start with wbr_ prefix
        assert!(!key.starts_with("wbr_"));
    }

    #[test]
    fn test_api_key_missing_bearer() {
        let header_value = "wbr_abc123xyz789";
        let starts_with_bearer = header_value.starts_with("Bearer ");
        
        assert!(!starts_with_bearer);
    }

    #[test]
    fn test_api_key_empty_after_bearer() {
        let header_value = "Bearer ";
        let key = header_value.trim_start_matches("Bearer ").trim();
        
        assert!(key.is_empty());
    }

    #[test]
    fn test_api_key_format_validation() {
        use crate::models::proxy_api_key::PROXY_KEY_PREFIX;
        
        // Valid key format
        let valid_key = "wbr_abc123xyz789abcdefghijklmnopqrstuvwxyz12";
        assert!(valid_key.starts_with(PROXY_KEY_PREFIX));
        
        // Invalid key format (wrong prefix)
        let invalid_key = "sk-abc123xyz789";
        assert!(!invalid_key.starts_with(PROXY_KEY_PREFIX));
        
        // Invalid key format (no prefix)
        let no_prefix_key = "abc123xyz789";
        assert!(!no_prefix_key.starts_with(PROXY_KEY_PREFIX));
    }

    #[test]
    fn test_auth_error_api_key_required() {
        let response = auth_error(
            StatusCode::UNAUTHORIZED,
            "API key required",
            "API_KEY_REQUIRED",
        );
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_auth_error_invalid_api_key() {
        let response = auth_error(
            StatusCode::UNAUTHORIZED,
            "Invalid or revoked API key",
            "INVALID_API_KEY",
        );
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
