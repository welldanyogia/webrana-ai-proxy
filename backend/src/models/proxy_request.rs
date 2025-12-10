//! Proxy request model for usage logging.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::api_key::AiProvider;

/// Proxy request log entity
#[derive(Debug, FromRow, Serialize)]
pub struct ProxyRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub proxy_key_id: Option<Uuid>,
    pub provider: AiProvider,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub latency_ms: i32,
    pub estimated_cost_idr: i64,
    pub status_code: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Create proxy request log
#[derive(Debug)]
pub struct CreateProxyRequest {
    pub user_id: Uuid,
    pub proxy_key_id: Option<Uuid>,
    pub provider: AiProvider,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub latency_ms: i32,
    pub status_code: i32,
    pub error_message: Option<String>,
}

/// Usage statistics for dashboard
#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_cost_idr: i64,
    pub requests_by_provider: Vec<ProviderUsage>,
}

#[derive(Debug, Serialize)]
pub struct ProviderUsage {
    pub provider: AiProvider,
    pub request_count: i64,
    pub token_count: i64,
    pub cost_idr: i64,
}
