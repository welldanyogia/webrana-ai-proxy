//! Admin middleware for role-based access control
//!
//! Requirements: 6.5 - Return 403 for non-admins

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use super::auth::AuthUser;

/// Error response for admin access denial
#[derive(Debug, Serialize)]
pub struct AdminErrorResponse {
    pub error: String,
    pub code: String,
}

/// Admin role check middleware
/// 
/// Checks if the authenticated user has admin role.
/// Must be used after jwt_auth middleware.
/// 
/// Requirements: 6.5 - Return HTTP 403 Forbidden for non-admins
/// 
/// # Arguments
/// * `request` - The incoming HTTP request (must have AuthUser in extensions)
/// * `next` - The next middleware/handler in the chain
/// 
/// # Returns
/// Response from the next handler or a 403 Forbidden error
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Response {
    // Get AuthUser from request extensions (set by jwt_auth middleware)
    let auth_user = request.extensions().get::<AuthUser>();

    match auth_user {
        Some(user) => {
            // Check if user has admin role
            // Admin role is stored in the plan field as "admin" or user has is_admin flag
            if is_admin_user(user) {
                next.run(request).await
            } else {
                admin_error(
                    StatusCode::FORBIDDEN,
                    "Admin access required",
                    "ADMIN_REQUIRED",
                )
            }
        }
        None => {
            // No auth user found - authentication middleware not applied
            admin_error(
                StatusCode::UNAUTHORIZED,
                "Authentication required",
                "AUTH_REQUIRED",
            )
        }
    }
}

/// Check if user has admin privileges
/// 
/// Admin can be determined by:
/// 1. User's plan field is "admin"
/// 2. User's email is in admin whitelist (for MVP)
fn is_admin_user(user: &AuthUser) -> bool {
    // Check plan field for admin role
    if user.plan == "admin" {
        return true;
    }

    // MVP: Whitelist specific admin emails
    let admin_emails = [
        "admin@webrana.id",
        "support@webrana.id",
    ];

    admin_emails.contains(&user.email.as_str())
}

/// Helper function to create admin error responses
fn admin_error(status: StatusCode, message: &str, code: &str) -> Response {
    let body = Json(AdminErrorResponse {
        error: message.to_string(),
        code: code.to_string(),
    });

    (status, body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_admin_user_by_plan() {
        let admin_user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            plan: "admin".to_string(),
        };
        
        assert!(is_admin_user(&admin_user));
    }

    #[test]
    fn test_admin_user_by_email() {
        let admin_user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "admin@webrana.id".to_string(),
            plan: "free".to_string(),
        };
        
        assert!(is_admin_user(&admin_user));
    }

    #[test]
    fn test_non_admin_user() {
        let regular_user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            plan: "pro".to_string(),
        };
        
        assert!(!is_admin_user(&regular_user));
    }

    #[test]
    fn test_admin_error_response() {
        let response = admin_error(
            StatusCode::FORBIDDEN,
            "Admin access required",
            "ADMIN_REQUIRED",
        );
        
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
