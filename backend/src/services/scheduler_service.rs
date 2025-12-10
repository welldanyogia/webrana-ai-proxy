//! Scheduler Service for background jobs
//!
//! Handles scheduled tasks like inactive user reminders, subscription expiry checks, etc.
//! Requirements: 5.5 - Send reminder email to users without API key after 24h

use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

use super::email_service::EmailService;
use super::onboarding_service::OnboardingService;

/// Scheduler error types
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Email error: {0}")]
    Email(String),
}

/// Scheduler Service for running background jobs
/// Requirements: 5.5
pub struct SchedulerService {
    pool: PgPool,
    email_service: Arc<EmailService>,
    onboarding_service: OnboardingService,
}

impl SchedulerService {
    pub fn new(pool: PgPool, email_service: Arc<EmailService>) -> Self {
        let onboarding_service = OnboardingService::new(pool.clone());
        Self {
            pool,
            email_service,
            onboarding_service,
        }
    }

    /// Start all scheduled jobs
    /// This should be called once at application startup
    pub async fn start_all_jobs(self: Arc<Self>) {
        let scheduler = self.clone();
        
        // Spawn inactive user reminder job (runs every hour)
        tokio::spawn(async move {
            scheduler.run_inactive_user_reminder_job().await;
        });

        let scheduler2 = self.clone();
        // Spawn subscription expiry check job (runs every 6 hours)
        tokio::spawn(async move {
            scheduler2.run_subscription_expiry_job().await;
        });

        tracing::info!("Scheduler service started with all background jobs");
    }

    /// Run inactive user reminder job
    /// Requirements: 5.5 - Send reminder to users without API key after 24h
    async fn run_inactive_user_reminder_job(&self) {
        let mut interval = interval(TokioDuration::from_secs(3600)); // Every hour

        loop {
            interval.tick().await;
            
            if let Err(e) = self.send_inactive_user_reminders().await {
                tracing::error!(error = %e, "Failed to send inactive user reminders");
            }
        }
    }

    /// Send reminders to inactive users
    /// Requirements: 5.5
    pub async fn send_inactive_user_reminders(&self) -> Result<u32, SchedulerError> {
        // Find users who signed up >24h ago without adding API key
        let inactive_users = self.onboarding_service
            .find_inactive_users(24)
            .await
            .map_err(|e| SchedulerError::Database(sqlx::Error::Protocol(e.to_string())))?;

        let mut sent_count = 0;

        for user in inactive_users {
            // Send reminder email
            let result = self.email_service
                .send_onboarding_reminder(
                    &user.email,
                    user.name.clone(),
                    "id", // Default to Indonesian
                )
                .await;

            match result {
                Ok(_) => {
                    // Mark reminder as sent
                    if let Err(e) = self.onboarding_service.mark_reminder_sent(user.user_id).await {
                        tracing::error!(
                            user_id = %user.user_id,
                            error = %e,
                            "Failed to mark reminder as sent"
                        );
                    }
                    sent_count += 1;
                    tracing::info!(
                        user_id = %user.user_id,
                        email = %user.email,
                        hours_since_signup = user.hours_since_signup,
                        "Sent onboarding reminder email"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        user_id = %user.user_id,
                        error = %e,
                        "Failed to send onboarding reminder"
                    );
                }
            }
        }

        tracing::info!(sent_count = sent_count, "Inactive user reminder job completed");
        Ok(sent_count)
    }


    /// Run subscription expiry check job
    async fn run_subscription_expiry_job(&self) {
        let mut interval = interval(TokioDuration::from_secs(21600)); // Every 6 hours

        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_expiring_subscriptions().await {
                tracing::error!(error = %e, "Failed to check expiring subscriptions");
            }
        }
    }

    /// Check for expiring subscriptions and send reminders
    pub async fn check_expiring_subscriptions(&self) -> Result<u32, SchedulerError> {
        // Find subscriptions expiring in 7 days
        let threshold = Utc::now() + Duration::days(7);
        
        let rows = sqlx::query(
            r#"
            SELECT 
                s.id, s.user_id, s.plan_tier::text as plan_tier, s.current_period_end,
                u.email, u.name
            FROM subscriptions s
            JOIN users u ON u.id = s.user_id
            WHERE s.status = 'active'
              AND s.current_period_end <= $1
              AND s.current_period_end > NOW()
              AND NOT EXISTS (
                SELECT 1 FROM email_logs el
                WHERE el.recipient = u.email
                  AND el.template = 'subscription_expiring'
                  AND el.sent_at > NOW() - INTERVAL '7 days'
              )
            ORDER BY s.current_period_end ASC
            LIMIT 100
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;

        let mut sent_count = 0;

        for row in rows {
            use sqlx::Row;
            let email: String = row.get("email");
            let name: Option<String> = row.get("name");
            let plan_tier: String = row.get("plan_tier");
            let period_end: chrono::DateTime<Utc> = row.get("current_period_end");
            
            let days_remaining = (period_end - Utc::now()).num_days() as i32;

            let result = self.email_service
                .send_subscription_expiring(
                    &email,
                    name,
                    &plan_tier,
                    days_remaining,
                    "id",
                )
                .await;

            match result {
                Ok(_) => {
                    sent_count += 1;
                    tracing::info!(
                        email = %email,
                        plan = %plan_tier,
                        days_remaining = days_remaining,
                        "Sent subscription expiring reminder"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        email = %email,
                        error = %e,
                        "Failed to send subscription expiring reminder"
                    );
                }
            }
        }

        tracing::info!(sent_count = sent_count, "Subscription expiry check completed");
        Ok(sent_count)
    }

    /// Manual trigger for inactive user check (for testing/admin)
    pub async fn trigger_inactive_user_check(&self) -> Result<u32, SchedulerError> {
        self.send_inactive_user_reminders().await
    }

    /// Manual trigger for subscription expiry check (for testing/admin)
    pub async fn trigger_subscription_expiry_check(&self) -> Result<u32, SchedulerError> {
        self.check_expiring_subscriptions().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_error_display() {
        let db_error = SchedulerError::Database(sqlx::Error::RowNotFound);
        assert!(db_error.to_string().contains("Database error"));

        let email_error = SchedulerError::Email("test error".to_string());
        assert!(email_error.to_string().contains("Email error"));
    }
}
