//! Authentication routes for user registration, login, and token refresh.

use axum::{
    routing::post,
    Router, Extension, Json,
    http::StatusCode,
    response::IntoResponse,
    extract::ConnectInfo,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

use crate::AppState;
use crate::models::CreateUser;
use crate::services::auth_service::{AuthService, AuthError};
use crate::middleware::rate_limit::{LoginRateLimiter, rate_limit_response};

pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
}

/// Registration request body
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Login request body
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Refresh token request body
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
        }
    }
}

/// Convert AuthError to HTTP response
fn auth_error_response(err: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    match err {
        AuthError::InvalidEmail => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_email", "Invalid email format")),
        ),
        AuthError::WeakPassword => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("weak_password", "Password must be at least 8 characters")),
        ),
        AuthError::EmailAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorResponse::new("email_exists", "Email already registered")),
        ),
        AuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_credentials", "Invalid email or password")),
        ),
        AuthError::InvalidToken => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("invalid_token", "Invalid or malformed token")),
        ),
        AuthError::TokenExpired => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("token_expired", "Token has expired")),
        ),
        AuthError::DatabaseError(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("server_error", "An internal error occurred")),
        ),
        AuthError::HashingError => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("server_error", "An internal error occurred")),
        ),
    }
}

/// POST /auth/register - Register a new user
async fn register(
    Extension(state): Extension<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    let auth_service = AuthService::new(state.db.clone(), jwt_secret);

    let input = CreateUser {
        email: body.email,
        password: body.password,
    };

    match auth_service.register(input).await {
        Ok(response) => (StatusCode::CREATED, Json(serde_json::to_value(response).unwrap())).into_response(),
        Err(err) => {
            let (status, json) = auth_error_response(err);
            (status, Json(serde_json::to_value(json.0).unwrap())).into_response()
        }
    }
}

/// POST /auth/login - Login user with rate limiting
async fn login(
    Extension(state): Extension<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    // Use email as rate limit identifier
    let identifier = body.email.to_lowercase();
    let rate_limiter = LoginRateLimiter::new(state.redis.clone());

    // Check if rate limited
    if let Err(retry_after) = rate_limiter.check_rate_limit(&identifier).await {
        return rate_limit_response(retry_after);
    }

    let auth_service = AuthService::new(state.db.clone(), jwt_secret);

    match auth_service.login(&body.email, &body.password).await {
        Ok(response) => {
            // Clear rate limit on successful login
            rate_limiter.clear_rate_limit(&identifier).await;
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(err) => {
            // Record failed attempt
            let _ = rate_limiter.record_failed_attempt(&identifier).await;
            
            // Add 200ms delay on failed login attempts (timing attack mitigation)
            sleep(Duration::from_millis(200)).await;
            let (status, json) = auth_error_response(err);
            (status, Json(serde_json::to_value(json.0).unwrap())).into_response()
        }
    }
}

/// POST /auth/refresh - Refresh access token
async fn refresh_token(
    Extension(state): Extension<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> impl IntoResponse {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    let auth_service = AuthService::new(state.db.clone(), jwt_secret);

    match auth_service.refresh_token(&body.refresh_token).await {
        Ok(tokens) => (StatusCode::OK, Json(serde_json::to_value(tokens).unwrap())).into_response(),
        Err(err) => {
            let (status, json) = auth_error_response(err);
            (status, Json(serde_json::to_value(json.0).unwrap())).into_response()
        }
    }
}
