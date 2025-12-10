use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use redis::AsyncCommands;
use serde::Serialize;
use uuid::Uuid;

use crate::services::billing_service::PlanTier;

/// Rate limit check result
#[derive(Debug, Serialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: i64,
    pub limit: i64,
    pub reset_at: DateTime<Utc>,
    pub retry_after_secs: Option<i64>,
}

/// Rate limit usage info
#[derive(Debug, Serialize)]
pub struct RateLimitUsage {
    pub monthly_used: i64,
    pub monthly_limit: i64,
    pub minute_used: i64,
    pub minute_limit: i64,
}

/// Rate limiter error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Rate limit exceeded")]
    LimitExceeded,
}

/// Per-minute burst limit
const BURST_LIMIT: i64 = 60;

/// Rate Limiter Service using Redis
/// Requirements: 5.1, 5.2, 5.5
pub struct RateLimiter {
    redis: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let redis = redis::Client::open(redis_url)?;
        Ok(Self { redis })
    }

    /// Get monthly key for user
    fn monthly_key(user_id: Uuid) -> String {
        let now = Utc::now();
        format!("rate:{}:{}:{}", user_id, now.year(), now.month())
    }

    /// Get minute key for user (for burst limiting)
    fn minute_key(user_id: Uuid) -> String {
        let now = Utc::now();
        format!("rate:{}:minute:{}", user_id, now.timestamp() / 60)
    }


    /// Check rate limit and increment counter if allowed
    /// Requirements: 5.1, 5.5
    /// Property 5: Rate Limiting Enforcement
    pub async fn check_and_increment(
        &self,
        user_id: Uuid,
        plan: PlanTier,
    ) -> Result<RateLimitResult, RateLimitError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let monthly_limit = plan.request_limit();
        let monthly_key = Self::monthly_key(user_id);
        let minute_key = Self::minute_key(user_id);

        // Check monthly limit
        let monthly_used: i64 = conn.get(&monthly_key).await.unwrap_or(0);
        
        if monthly_used >= monthly_limit {
            let reset_at = Self::next_month_start();
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                limit: monthly_limit,
                reset_at,
                retry_after_secs: Some((reset_at - Utc::now()).num_seconds()),
            });
        }

        // Check per-minute burst limit
        let minute_used: i64 = conn.get(&minute_key).await.unwrap_or(0);
        
        if minute_used >= BURST_LIMIT {
            let reset_at = Utc::now() + Duration::seconds(60 - (Utc::now().timestamp() % 60));
            return Ok(RateLimitResult {
                allowed: false,
                remaining: monthly_limit - monthly_used,
                limit: monthly_limit,
                reset_at,
                retry_after_secs: Some(60 - (Utc::now().timestamp() % 60)),
            });
        }

        // Increment both counters
        let _: () = redis::pipe()
            .atomic()
            .incr(&monthly_key, 1)
            .expire(&monthly_key, Self::seconds_until_month_end())
            .incr(&minute_key, 1)
            .expire(&minute_key, 60)
            .query_async(&mut conn)
            .await?;

        let reset_at = Self::next_month_start();
        Ok(RateLimitResult {
            allowed: true,
            remaining: monthly_limit - monthly_used - 1,
            limit: monthly_limit,
            reset_at,
            retry_after_secs: None,
        })
    }

    /// Get current usage without incrementing
    pub async fn get_usage(
        &self,
        user_id: Uuid,
        plan: PlanTier,
    ) -> Result<RateLimitUsage, RateLimitError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let monthly_key = Self::monthly_key(user_id);
        let minute_key = Self::minute_key(user_id);

        let monthly_used: i64 = conn.get(&monthly_key).await.unwrap_or(0);
        let minute_used: i64 = conn.get(&minute_key).await.unwrap_or(0);

        Ok(RateLimitUsage {
            monthly_used,
            monthly_limit: plan.request_limit(),
            minute_used,
            minute_limit: BURST_LIMIT,
        })
    }

    /// Check if user is at quota warning threshold (80%)
    /// Requirements: 5.3
    pub fn is_at_warning_threshold(used: i64, limit: i64) -> bool {
        let percentage = (used as f64 / limit as f64) * 100.0;
        percentage >= 80.0 && percentage < 100.0
    }

    /// Calculate seconds until end of current month
    fn seconds_until_month_end() -> i64 {
        let now = Utc::now();
        let next_month = if now.month() == 12 {
            now.with_year(now.year() + 1)
                .and_then(|d| d.with_month(1))
                .and_then(|d| d.with_day(1))
        } else {
            now.with_month(now.month() + 1)
                .and_then(|d| d.with_day(1))
        };
        
        next_month
            .map(|d| (d - now).num_seconds())
            .unwrap_or(30 * 24 * 60 * 60) // Default to 30 days
    }

    /// Get start of next month
    fn next_month_start() -> DateTime<Utc> {
        let now = Utc::now();
        if now.month() == 12 {
            now.with_year(now.year() + 1)
                .and_then(|d| d.with_month(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.with_hour(0))
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .unwrap_or(now)
        } else {
            now.with_month(now.month() + 1)
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.with_hour(0))
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .unwrap_or(now)
        }
    }
}
