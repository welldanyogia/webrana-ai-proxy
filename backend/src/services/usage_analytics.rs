use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Usage statistics for a given period
#[derive(Debug, Serialize, Default)]
pub struct UsageStats {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub total_cost_idr: i64,
    pub avg_latency_ms: f64,
}

/// Usage breakdown by provider
#[derive(Debug, Serialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost_idr: i64,
}

/// Usage breakdown by model
#[derive(Debug, Serialize)]
pub struct ModelUsage {
    pub model: String,
    pub provider: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost_idr: i64,
}

/// Daily usage for time series charts
#[derive(Debug, Serialize)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost_idr: i64,
}

/// Date range filter
#[derive(Debug, Deserialize, Clone)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    /// Create date range for last N days
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::days(days);
        Self { start, end }
    }

    pub fn last_7_days() -> Self {
        Self::last_days(7)
    }

    pub fn last_30_days() -> Self {
        Self::last_days(30)
    }
}


/// Usage Analytics Service
/// Requirements: 1.2, 1.3, 1.4 - Usage aggregation and filtering
pub struct UsageAnalyticsService {
    pool: PgPool,
}

impl UsageAnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get aggregated usage stats for a user within date range
    pub async fn get_usage_stats(
        &self,
        user_id: Uuid,
        range: &DateRange,
    ) -> Result<UsageStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(COUNT(*), 0)::bigint as total_requests,
                COALESCE(SUM(prompt_tokens), 0)::bigint as total_input_tokens,
                COALESCE(SUM(completion_tokens), 0)::bigint as total_output_tokens,
                COALESCE(SUM(total_tokens), 0)::bigint as total_tokens,
                COALESCE(SUM(estimated_cost_idr), 0)::bigint as total_cost_idr,
                COALESCE(AVG(latency_ms), 0)::float8 as avg_latency_ms
            FROM proxy_requests
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at <= $3
              AND status_code < 400
            "#,
        )
        .bind(user_id)
        .bind(range.start)
        .bind(range.end)
        .fetch_one(&self.pool)
        .await?;

        Ok(UsageStats {
            total_requests: row.get("total_requests"),
            total_input_tokens: row.get("total_input_tokens"),
            total_output_tokens: row.get("total_output_tokens"),
            total_tokens: row.get("total_tokens"),
            total_cost_idr: row.get("total_cost_idr"),
            avg_latency_ms: row.get("avg_latency_ms"),
        })
    }

    /// Get usage breakdown by provider
    pub async fn get_usage_by_provider(
        &self,
        user_id: Uuid,
        range: &DateRange,
    ) -> Result<Vec<ProviderUsage>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                provider::text as provider,
                COUNT(*)::bigint as request_count,
                COALESCE(SUM(total_tokens), 0)::bigint as total_tokens,
                COALESCE(SUM(estimated_cost_idr), 0)::bigint as total_cost_idr
            FROM proxy_requests
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at <= $3
              AND status_code < 400
            GROUP BY provider
            ORDER BY request_count DESC
            "#,
        )
        .bind(user_id)
        .bind(range.start)
        .bind(range.end)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ProviderUsage {
                provider: r.get("provider"),
                request_count: r.get("request_count"),
                total_tokens: r.get("total_tokens"),
                total_cost_idr: r.get("total_cost_idr"),
            })
            .collect())
    }


    /// Get usage breakdown by model
    pub async fn get_usage_by_model(
        &self,
        user_id: Uuid,
        range: &DateRange,
    ) -> Result<Vec<ModelUsage>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                model,
                provider::text as provider,
                COUNT(*)::bigint as request_count,
                COALESCE(SUM(total_tokens), 0)::bigint as total_tokens,
                COALESCE(SUM(estimated_cost_idr), 0)::bigint as total_cost_idr
            FROM proxy_requests
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at <= $3
              AND status_code < 400
            GROUP BY model, provider
            ORDER BY request_count DESC
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .bind(range.start)
        .bind(range.end)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ModelUsage {
                model: r.get("model"),
                provider: r.get("provider"),
                request_count: r.get("request_count"),
                total_tokens: r.get("total_tokens"),
                total_cost_idr: r.get("total_cost_idr"),
            })
            .collect())
    }

    /// Get daily usage for time series chart
    pub async fn get_daily_usage(
        &self,
        user_id: Uuid,
        range: &DateRange,
    ) -> Result<Vec<DailyUsage>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(created_at AT TIME ZONE 'Asia/Jakarta') as date,
                COUNT(*)::bigint as request_count,
                COALESCE(SUM(total_tokens), 0)::bigint as total_tokens,
                COALESCE(SUM(estimated_cost_idr), 0)::bigint as total_cost_idr
            FROM proxy_requests
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at <= $3
              AND status_code < 400
            GROUP BY DATE(created_at AT TIME ZONE 'Asia/Jakarta')
            ORDER BY date ASC
            "#,
        )
        .bind(user_id)
        .bind(range.start)
        .bind(range.end)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DailyUsage {
                date: r.get("date"),
                request_count: r.get("request_count"),
                total_tokens: r.get("total_tokens"),
                total_cost_idr: r.get("total_cost_idr"),
            })
            .collect())
    }

    /// Export usage data as CSV
    pub async fn export_csv(
        &self,
        user_id: Uuid,
        range: &DateRange,
    ) -> Result<String, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                created_at,
                provider::text as provider,
                model,
                prompt_tokens,
                completion_tokens,
                estimated_cost_idr,
                latency_ms
            FROM proxy_requests
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at <= $3
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(range.start)
        .bind(range.end)
        .fetch_all(&self.pool)
        .await?;

        let mut csv = String::from("timestamp,provider,model,input_tokens,output_tokens,cost_idr,latency_ms\n");

        for row in rows {
            let ts: DateTime<Utc> = row.get("created_at");
            let provider: String = row.get("provider");
            let model: String = row.get("model");
            let input: i32 = row.get("prompt_tokens");
            let output: i32 = row.get("completion_tokens");
            let cost: i64 = row.get("estimated_cost_idr");
            let latency: i32 = row.get("latency_ms");

            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                ts.format("%Y-%m-%d %H:%M:%S"),
                provider,
                model,
                input,
                output,
                cost,
                latency
            ));
        }

        Ok(csv)
    }
}
