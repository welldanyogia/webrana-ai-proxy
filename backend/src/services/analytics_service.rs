//! Analytics Service
//!
//! Tracks user acquisition, activation, and retention metrics.
//! Requirements: 9.1, 9.2, 9.3

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Analytics event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Signup,
    ApiKeyAdded,
    FirstRequest,
    ProxyRequest,
    DashboardView,
    BillingPageView,
    Upgrade,
    Downgrade,
    Cancellation,
    Login,
    Logout,
    Custom(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Signup => "signup",
            EventType::ApiKeyAdded => "api_key_added",
            EventType::FirstRequest => "first_request",
            EventType::ProxyRequest => "proxy_request",
            EventType::DashboardView => "dashboard_view",
            EventType::BillingPageView => "billing_page_view",
            EventType::Upgrade => "upgrade",
            EventType::Downgrade => "downgrade",
            EventType::Cancellation => "cancellation",
            EventType::Login => "login",
            EventType::Logout => "logout",
            EventType::Custom(s) => s,
        }
    }
}

/// Acquisition source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcquisitionSource {
    ProductHunt,
    Organic,
    Referral,
    Direct,
    Social,
    Unknown,
}

impl AcquisitionSource {
    pub fn as_str(&self) -> &str {
        match self {
            AcquisitionSource::ProductHunt => "producthunt",
            AcquisitionSource::Organic => "organic",
            AcquisitionSource::Referral => "referral",
            AcquisitionSource::Direct => "direct",
            AcquisitionSource::Social => "social",
            AcquisitionSource::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "producthunt" | "product_hunt" => AcquisitionSource::ProductHunt,
            "organic" | "search" => AcquisitionSource::Organic,
            "referral" | "ref" => AcquisitionSource::Referral,
            "direct" => AcquisitionSource::Direct,
            "social" | "twitter" | "linkedin" => AcquisitionSource::Social,
            _ => AcquisitionSource::Unknown,
        }
    }
}

/// Analytics event to track
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsEvent {
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub properties: HashMap<String, JsonValue>,
    pub source: Option<String>,
    pub session_id: Option<String>,
}

/// Analytics error types
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invalid event data")]
    InvalidEvent,
}

/// Acquisition statistics
#[derive(Debug, Clone, Serialize)]
pub struct AcquisitionStats {
    pub total_signups: i64,
    pub by_source: HashMap<String, i64>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Activation funnel data
#[derive(Debug, Clone, Serialize)]
pub struct ActivationFunnel {
    pub total_signups: i64,
    pub api_key_added: i64,
    pub first_request: i64,
    pub active_users: i64,
    pub api_key_rate: f64,
    pub first_request_rate: f64,
    pub activation_rate: f64,
}

/// Retention cohort data
#[derive(Debug, Clone, Serialize)]
pub struct RetentionCohort {
    pub cohort_date: String,
    pub cohort_size: i64,
    pub day_1_retained: i64,
    pub day_7_retained: i64,
    pub day_30_retained: i64,
    pub day_1_rate: f64,
    pub day_7_rate: f64,
    pub day_30_rate: f64,
}

/// User at risk of churn
#[derive(Debug, Clone, Serialize)]
pub struct ChurnRiskUser {
    pub user_id: Uuid,
    pub email: String,
    pub last_activity: DateTime<Utc>,
    pub days_inactive: i64,
    pub plan_tier: String,
}

/// Analytics Service
/// Requirements: 9.1, 9.2, 9.3
pub struct AnalyticsService {
    pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Track an analytics event
    /// Requirements: 9.1
    pub async fn track_event(&self, event: AnalyticsEvent) -> Result<Uuid, AnalyticsError> {
        if event.event_type.is_empty() {
            return Err(AnalyticsError::InvalidEvent);
        }

        let properties_json = serde_json::to_value(&event.properties)
            .unwrap_or(JsonValue::Object(serde_json::Map::new()));

        let event_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO analytics_events (id, user_id, event_type, properties, source, session_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
        )
        .bind(event_id)
        .bind(event.user_id)
        .bind(&event.event_type)
        .bind(properties_json)
        .bind(&event.source)
        .bind(&event.session_id)
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            event_type = %event.event_type,
            user_id = ?event.user_id,
            "Analytics event tracked"
        );

        Ok(event_id)
    }

    /// Get acquisition statistics for a date range
    /// Requirements: 9.1
    pub async fn get_acquisition_stats(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<AcquisitionStats, AnalyticsError> {
        // Total signups
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM analytics_events
            WHERE event_type = 'signup'
              AND created_at >= $1 AND created_at < $2
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await?;

        // By source
        let rows = sqlx::query(
            r#"
            SELECT COALESCE(source, 'unknown') as source, COUNT(*) as count
            FROM analytics_events
            WHERE event_type = 'signup'
              AND created_at >= $1 AND created_at < $2
            GROUP BY source
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        let mut by_source = HashMap::new();
        for row in rows {
            let source: String = row.get("source");
            let count: i64 = row.get("count");
            by_source.insert(source, count);
        }

        Ok(AcquisitionStats {
            total_signups: total,
            by_source,
            period_start: start_date,
            period_end: end_date,
        })
    }

    /// Get activation funnel metrics
    /// Requirements: 9.2
    pub async fn get_activation_funnel(&self) -> Result<ActivationFunnel, AnalyticsError> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(DISTINCT CASE WHEN event_type = 'signup' THEN user_id END) as signups,
                COUNT(DISTINCT CASE WHEN event_type = 'api_key_added' THEN user_id END) as api_key_added,
                COUNT(DISTINCT CASE WHEN event_type = 'first_request' THEN user_id END) as first_request,
                COUNT(DISTINCT CASE WHEN event_type = 'proxy_request' THEN user_id END) as active_users
            FROM analytics_events
            WHERE created_at >= NOW() - INTERVAL '30 days'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let signups: i64 = row.get("signups");
        let api_key_added: i64 = row.get("api_key_added");
        let first_request: i64 = row.get("first_request");
        let active_users: i64 = row.get("active_users");

        let api_key_rate = if signups > 0 {
            (api_key_added as f64 / signups as f64) * 100.0
        } else {
            0.0
        };

        let first_request_rate = if api_key_added > 0 {
            (first_request as f64 / api_key_added as f64) * 100.0
        } else {
            0.0
        };

        let activation_rate = if signups > 0 {
            (first_request as f64 / signups as f64) * 100.0
        } else {
            0.0
        };

        Ok(ActivationFunnel {
            total_signups: signups,
            api_key_added,
            first_request,
            active_users,
            api_key_rate,
            first_request_rate,
            activation_rate,
        })
    }

    /// Identify users at risk of churn (no activity for 7+ days)
    /// Requirements: 9.5
    pub async fn identify_churn_risk(&self, days_inactive: i64) -> Result<Vec<ChurnRiskUser>, AnalyticsError> {
        let threshold = Utc::now() - Duration::days(days_inactive);

        let rows = sqlx::query(
            r#"
            SELECT 
                u.id as user_id, u.email, u.plan_tier::text as plan_tier,
                COALESCE(
                    (SELECT MAX(created_at) FROM analytics_events WHERE user_id = u.id),
                    u.created_at
                ) as last_activity
            FROM users u
            WHERE u.plan_tier != 'free'
              AND NOT EXISTS (
                SELECT 1 FROM analytics_events ae
                WHERE ae.user_id = u.id AND ae.created_at > $1
              )
            ORDER BY last_activity ASC
            LIMIT 100
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;

        let now = Utc::now();
        Ok(rows
            .into_iter()
            .map(|r| {
                let last_activity: DateTime<Utc> = r.get("last_activity");
                ChurnRiskUser {
                    user_id: r.get("user_id"),
                    email: r.get("email"),
                    last_activity,
                    days_inactive: (now - last_activity).num_days(),
                    plan_tier: r.get("plan_tier"),
                }
            })
            .collect())
    }

    /// Track signup event with source
    pub async fn track_signup(
        &self,
        user_id: Uuid,
        source: AcquisitionSource,
    ) -> Result<Uuid, AnalyticsError> {
        let mut properties = HashMap::new();
        properties.insert("source".to_string(), JsonValue::String(source.as_str().to_string()));

        self.track_event(AnalyticsEvent {
            user_id: Some(user_id),
            event_type: EventType::Signup.as_str().to_string(),
            properties,
            source: Some(source.as_str().to_string()),
            session_id: None,
        })
        .await
    }

    /// Track API key added event
    pub async fn track_api_key_added(&self, user_id: Uuid) -> Result<Uuid, AnalyticsError> {
        self.track_event(AnalyticsEvent {
            user_id: Some(user_id),
            event_type: EventType::ApiKeyAdded.as_str().to_string(),
            properties: HashMap::new(),
            source: None,
            session_id: None,
        })
        .await
    }

    /// Track first request event
    pub async fn track_first_request(&self, user_id: Uuid, provider: &str) -> Result<Uuid, AnalyticsError> {
        let mut properties = HashMap::new();
        properties.insert("provider".to_string(), JsonValue::String(provider.to_string()));

        self.track_event(AnalyticsEvent {
            user_id: Some(user_id),
            event_type: EventType::FirstRequest.as_str().to_string(),
            properties,
            source: None,
            session_id: None,
        })
        .await
    }
}
