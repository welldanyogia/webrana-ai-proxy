use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::usage_analytics::{
    DateRange, DailyUsage, ModelUsage, ProviderUsage, UsageAnalyticsService, UsageStats,
};

// Re-export for main.rs
pub use crate::services::usage_analytics::UsageAnalyticsService as _;

/// Query parameters for usage endpoints
#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    /// Start date (ISO 8601)
    pub start: Option<DateTime<Utc>>,
    /// End date (ISO 8601)
    pub end: Option<DateTime<Utc>>,
    /// Preset: "7d", "30d", "90d"
    pub preset: Option<String>,
}

impl UsageQuery {
    fn to_date_range(&self) -> DateRange {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            DateRange { start, end }
        } else {
            match self.preset.as_deref() {
                Some("7d") => DateRange::last_7_days(),
                Some("90d") => DateRange::last_days(90),
                _ => DateRange::last_30_days(), // Default to 30 days
            }
        }
    }
}

/// Combined usage response
#[derive(Debug, Serialize)]
pub struct UsageResponse {
    pub stats: UsageStats,
    pub by_provider: Vec<ProviderUsage>,
    pub by_model: Vec<ModelUsage>,
    pub daily: Vec<DailyUsage>,
}

/// Create usage routes
pub fn usage_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(get_usage))
        .route("/stats", get(get_usage_stats))
        .route("/by-provider", get(get_usage_by_provider))
        .route("/by-model", get(get_usage_by_model))
        .route("/daily", get(get_daily_usage))
        .route("/export", get(export_csv))
}


/// Get all usage data (combined endpoint)
/// GET /usage
async fn get_usage(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<UsageResponse>, StatusCode> {
    // TODO: Get user_id from auth middleware
    let user_id = Uuid::nil(); // Placeholder
    
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    let stats = service
        .get_usage_stats(user_id, &range)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let by_provider = service
        .get_usage_by_provider(user_id, &range)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let by_model = service
        .get_usage_by_model(user_id, &range)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let daily = service
        .get_daily_usage(user_id, &range)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UsageResponse {
        stats,
        by_provider,
        by_model,
        daily,
    }))
}

/// Get usage stats only
/// GET /usage/stats
async fn get_usage_stats(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<UsageStats>, StatusCode> {
    let user_id = Uuid::nil(); // TODO: Get from auth
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    service
        .get_usage_stats(user_id, &range)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get usage by provider
/// GET /usage/by-provider
async fn get_usage_by_provider(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<Vec<ProviderUsage>>, StatusCode> {
    let user_id = Uuid::nil();
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    service
        .get_usage_by_provider(user_id, &range)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get usage by model
/// GET /usage/by-model
async fn get_usage_by_model(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<Vec<ModelUsage>>, StatusCode> {
    let user_id = Uuid::nil();
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    service
        .get_usage_by_model(user_id, &range)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get daily usage
/// GET /usage/daily
async fn get_daily_usage(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<Vec<DailyUsage>>, StatusCode> {
    let user_id = Uuid::nil();
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    service
        .get_daily_usage(user_id, &range)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Export usage as CSV
/// GET /usage/export
/// Requirements: 1.5 - CSV export
async fn export_csv(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Response {
    let user_id = Uuid::nil();
    let service = UsageAnalyticsService::new(pool);
    let range = query.to_date_range();

    match service.export_csv(user_id, &range).await {
        Ok(csv) => {
            let headers = [
                (axum::http::header::CONTENT_TYPE, "text/csv"),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    "attachment; filename=\"usage-export.csv\"",
                ),
            ];
            (headers, csv).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
