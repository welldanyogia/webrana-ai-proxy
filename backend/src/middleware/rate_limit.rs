//! Rate limiting middleware using Redis.
//! Requirements: 2.4 - Block after 5 failed attempts for 30 minutes

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use serde::Serialize;
use std::net::SocketAddr;

/// Rate limit configuration
pub struct RateLimitConfig {
    pub max_attempts: u32,
    pub window_seconds: u64,
    pub block_duration_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_seconds: 60,
            block_duration_seconds: 1800, // 30 minutes
        }
    }
}

/// Login rate limit config (5 attempts, block for 30 minutes)
pub fn login_rate_limit_config() -> RateLimitConfig {
    RateLimitConfig {
        max_attempts: 5,
        window_seconds: 60,
        block_duration_seconds: 1800,
    }
}

#[derive(Serialize)]
struct RateLimitError {
    error: String,
    message: String,
    retry_after: Option<u64>,
}

/// Rate limiting middleware (generic)
pub async fn rate_limit(request: Request, next: Next) -> Response {
    // TODO: Implement generic rate limiting
    next.run(request).await
}

/// Rate limiter service for login attempts
pub struct LoginRateLimiter {
    redis: redis::Client,
    config: RateLimitConfig,
}

impl LoginRateLimiter {
    pub fn new(redis: redis::Client) -> Self {
        Self {
            redis,
            config: login_rate_limit_config(),
        }
    }

    /// Check if IP/email is rate limited
    /// Returns Ok(remaining_attempts) or Err(seconds_until_unblock)
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<u32, u64> {
        let mut conn = match self.redis.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(_) => return Ok(self.config.max_attempts), // Fail open if Redis unavailable
        };

        let block_key = format!("login_blocked:{}", identifier);
        let attempts_key = format!("login_attempts:{}", identifier);

        // Check if blocked
        let ttl: i64 = conn.ttl(&block_key).await.unwrap_or(-2);
        if ttl > 0 {
            return Err(ttl as u64);
        }

        // Get current attempts
        let attempts: u32 = conn.get(&attempts_key).await.unwrap_or(0);
        let remaining = self.config.max_attempts.saturating_sub(attempts);

        Ok(remaining)
    }

    /// Record a failed login attempt
    pub async fn record_failed_attempt(&self, identifier: &str) -> Result<u32, u64> {
        let mut conn = match self.redis.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(_) => return Ok(self.config.max_attempts),
        };

        let block_key = format!("login_blocked:{}", identifier);
        let attempts_key = format!("login_attempts:{}", identifier);

        // Increment attempts
        let attempts: u32 = conn.incr(&attempts_key, 1).await.unwrap_or(1);
        
        // Set expiry on attempts key
        let _: () = conn.expire(&attempts_key, self.config.window_seconds as i64)
            .await
            .unwrap_or(());

        // Check if should block
        if attempts >= self.config.max_attempts {
            // Set block key
            let _: () = conn.set_ex(
                &block_key,
                "blocked",
                self.config.block_duration_seconds,
            ).await.unwrap_or(());

            // Clear attempts counter
            let _: () = conn.del(&attempts_key).await.unwrap_or(());

            return Err(self.config.block_duration_seconds);
        }

        Ok(self.config.max_attempts - attempts)
    }

    /// Clear rate limit on successful login
    pub async fn clear_rate_limit(&self, identifier: &str) {
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let attempts_key = format!("login_attempts:{}", identifier);
            let _: Result<(), _> = conn.del(&attempts_key).await;
        }
    }
}

/// Create rate limit exceeded response
pub fn rate_limit_response(retry_after: u64) -> Response {
    let error = RateLimitError {
        error: "rate_limit_exceeded".to_string(),
        message: format!("Too many failed login attempts. Please try again in {} minutes.", retry_after / 60),
        retry_after: Some(retry_after),
    };

    (
        StatusCode::TOO_MANY_REQUESTS,
        [("Retry-After", retry_after.to_string())],
        Json(error),
    ).into_response()
}
