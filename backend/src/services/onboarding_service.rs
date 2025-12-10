//! Onboarding Service
//!
//! Tracks user onboarding progress and triggers engagement emails.
//! Requirements: 5.5, 5.6 - Track onboarding completion, detect inactive users

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Onboarding steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStep {
    AccountCreated,
    ApiKeyAdded,
    FirstRequestMade,
    DashboardViewed,
}

impl OnboardingStep {
    /// Get step weight for completion percentage (25% each)
    pub fn weight(&self) -> u8 {
        25
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OnboardingStep::AccountCreated => "account_created",
            OnboardingStep::ApiKeyAdded => "api_key_added",
            OnboardingStep::FirstRequestMade => "first_request_made",
            OnboardingStep::DashboardViewed => "dashboard_viewed",
        }
    }
}

/// Onboarding progress status
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStatus {
    pub user_id: Uuid,
    pub steps_completed: Vec<OnboardingStep>,
    pub completion_percent: u8,
    pub account_created_at: DateTime<Utc>,
    pub api_key_added_at: Option<DateTime<Utc>>,
    pub first_request_at: Option<DateTime<Utc>>,
    pub dashboard_viewed_at: Option<DateTime<Utc>>,
    pub reminder_sent_at: Option<DateTime<Utc>>,
    pub last_activity: DateTime<Utc>,
}

impl OnboardingStatus {
    /// Calculate completion percentage from completed steps
    pub fn calculate_completion(steps: &[OnboardingStep]) -> u8 {
        steps.iter().map(|s| s.weight()).sum()
    }

    /// Check if onboarding is complete
    pub fn is_complete(&self) -> bool {
        self.completion_percent >= 100
    }

    /// Get next recommended step
    pub fn next_step(&self) -> Option<OnboardingStep> {
        if !self.steps_completed.contains(&OnboardingStep::ApiKeyAdded) {
            Some(OnboardingStep::ApiKeyAdded)
        } else if !self.steps_completed.contains(&OnboardingStep::FirstRequestMade) {
            Some(OnboardingStep::FirstRequestMade)
        } else if !self.steps_completed.contains(&OnboardingStep::DashboardViewed) {
            Some(OnboardingStep::DashboardViewed)
        } else {
            None
        }
    }
}

/// Inactive user for reminder emails
#[derive(Debug, Clone, Serialize)]
pub struct InactiveUser {
    pub user_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub account_created_at: DateTime<Utc>,
    pub hours_since_signup: i64,
}

/// Onboarding error types
#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Onboarding record not found")]
    NotFound,
}

/// Onboarding Service
/// Requirements: 5.5, 5.6
pub struct OnboardingService {
    pool: PgPool,
}

impl OnboardingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get onboarding status for a user
    /// Requirements: 5.6
    pub async fn get_status(&self, user_id: Uuid) -> Result<OnboardingStatus, OnboardingError> {
        let row = sqlx::query(
            r#"
            SELECT 
                user_id, account_created_at, api_key_added_at, 
                first_request_at, dashboard_viewed_at, reminder_sent_at,
                completion_percent, updated_at
            FROM onboarding_progress
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(OnboardingError::NotFound)?;

        let mut steps_completed = vec![OnboardingStep::AccountCreated];
        
        let api_key_added_at: Option<DateTime<Utc>> = row.get("api_key_added_at");
        let first_request_at: Option<DateTime<Utc>> = row.get("first_request_at");
        let dashboard_viewed_at: Option<DateTime<Utc>> = row.get("dashboard_viewed_at");

        if api_key_added_at.is_some() {
            steps_completed.push(OnboardingStep::ApiKeyAdded);
        }
        if first_request_at.is_some() {
            steps_completed.push(OnboardingStep::FirstRequestMade);
        }
        if dashboard_viewed_at.is_some() {
            steps_completed.push(OnboardingStep::DashboardViewed);
        }

        let completion_percent = OnboardingStatus::calculate_completion(&steps_completed);

        Ok(OnboardingStatus {
            user_id: row.get("user_id"),
            steps_completed,
            completion_percent,
            account_created_at: row.get("account_created_at"),
            api_key_added_at,
            first_request_at,
            dashboard_viewed_at,
            reminder_sent_at: row.get("reminder_sent_at"),
            last_activity: row.get("updated_at"),
        })
    }

    /// Mark an onboarding step as complete
    /// Requirements: 5.6
    pub async fn mark_step_complete(
        &self,
        user_id: Uuid,
        step: OnboardingStep,
    ) -> Result<OnboardingStatus, OnboardingError> {
        let column = match step {
            OnboardingStep::AccountCreated => return self.get_status(user_id).await,
            OnboardingStep::ApiKeyAdded => "api_key_added_at",
            OnboardingStep::FirstRequestMade => "first_request_at",
            OnboardingStep::DashboardViewed => "dashboard_viewed_at",
        };

        // Update the specific step timestamp
        let query = format!(
            r#"
            UPDATE onboarding_progress
            SET {} = COALESCE({}, NOW()), updated_at = NOW()
            WHERE user_id = $1
            "#,
            column, column
        );

        let result = sqlx::query(&query)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(OnboardingError::NotFound);
        }

        // Get updated status and recalculate completion
        let status = self.get_status(user_id).await?;

        // Update completion percentage
        sqlx::query(
            "UPDATE onboarding_progress SET completion_percent = $1 WHERE user_id = $2",
        )
        .bind(status.completion_percent as i16)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            user_id = %user_id,
            step = %step.as_str(),
            completion = %status.completion_percent,
            "Onboarding step completed"
        );

        Ok(status)
    }

    /// Find inactive users who haven't added API key after 24 hours
    /// Requirements: 5.5
    pub async fn find_inactive_users(&self, hours_threshold: i64) -> Result<Vec<InactiveUser>, OnboardingError> {
        let threshold = Utc::now() - Duration::hours(hours_threshold);

        let rows = sqlx::query(
            r#"
            SELECT 
                o.user_id, u.email, u.name, o.account_created_at
            FROM onboarding_progress o
            JOIN users u ON u.id = o.user_id
            WHERE o.api_key_added_at IS NULL
              AND o.account_created_at < $1
              AND o.reminder_sent_at IS NULL
            ORDER BY o.account_created_at ASC
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;

        let now = Utc::now();
        Ok(rows
            .into_iter()
            .map(|r| {
                let created_at: DateTime<Utc> = r.get("account_created_at");
                InactiveUser {
                    user_id: r.get("user_id"),
                    email: r.get("email"),
                    name: r.get("name"),
                    account_created_at: created_at,
                    hours_since_signup: (now - created_at).num_hours(),
                }
            })
            .collect())
    }

    /// Mark reminder as sent for a user
    pub async fn mark_reminder_sent(&self, user_id: Uuid) -> Result<(), OnboardingError> {
        sqlx::query(
            "UPDATE onboarding_progress SET reminder_sent_at = NOW(), updated_at = NOW() WHERE user_id = $1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get onboarding completion statistics
    pub async fn get_completion_stats(&self) -> Result<OnboardingStats, OnboardingError> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE api_key_added_at IS NOT NULL) as api_key_added,
                COUNT(*) FILTER (WHERE first_request_at IS NOT NULL) as first_request,
                COUNT(*) FILTER (WHERE dashboard_viewed_at IS NOT NULL) as dashboard_viewed,
                COUNT(*) FILTER (WHERE completion_percent >= 100) as fully_completed,
                AVG(completion_percent)::float8 as avg_completion
            FROM onboarding_progress
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(OnboardingStats {
            total_users: row.get::<i64, _>("total_users"),
            api_key_added: row.get::<i64, _>("api_key_added"),
            first_request: row.get::<i64, _>("first_request"),
            dashboard_viewed: row.get::<i64, _>("dashboard_viewed"),
            fully_completed: row.get::<i64, _>("fully_completed"),
            avg_completion_percent: row.get::<f64, _>("avg_completion"),
        })
    }

    /// Create onboarding record for existing user (if not exists)
    pub async fn ensure_onboarding_record(&self, user_id: Uuid) -> Result<(), OnboardingError> {
        sqlx::query(
            r#"
            INSERT INTO onboarding_progress (user_id, account_created_at)
            VALUES ($1, NOW())
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Onboarding completion statistics
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStats {
    pub total_users: i64,
    pub api_key_added: i64,
    pub first_request: i64,
    pub dashboard_viewed: i64,
    pub fully_completed: i64,
    pub avg_completion_percent: f64,
}
