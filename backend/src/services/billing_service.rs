use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Plan tier pricing in IDR (before PPN)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanTier {
    Free,
    Starter,
    Pro,
    Team,
}

impl PlanTier {
    /// Get base price in IDR (before PPN)
    pub fn price_idr(&self) -> i64 {
        match self {
            PlanTier::Free => 0,
            PlanTier::Starter => 49_000,
            PlanTier::Pro => 99_000,
            PlanTier::Team => 299_000,
        }
    }

    /// Get monthly request limit
    pub fn request_limit(&self) -> i64 {
        match self {
            PlanTier::Free => 1_000,
            PlanTier::Starter => 10_000,
            PlanTier::Pro => 50_000,
            PlanTier::Team => 200_000,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PlanTier::Free => "free",
            PlanTier::Starter => "starter",
            PlanTier::Pro => "pro",
            PlanTier::Team => "team",
        }
    }
}

impl std::fmt::Display for PlanTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


/// Subscription status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    PendingPayment,
    Active,
    Expired,
    Cancelled,
}

/// Subscription entity
#[derive(Debug, Serialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_tier: String,
    pub price_idr: i64,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub midtrans_order_id: Option<String>,
    pub midtrans_transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Midtrans Snap token response
#[derive(Debug, Serialize)]
pub struct MidtransSnapToken {
    pub token: String,
    pub redirect_url: String,
    pub order_id: String,
}

/// Midtrans webhook notification payload
#[derive(Debug, Deserialize)]
pub struct MidtransWebhook {
    pub order_id: String,
    pub status_code: String,
    pub gross_amount: String,
    pub signature_key: String,
    pub transaction_status: String,
    pub transaction_id: String,
    pub payment_type: String,
}

/// Billing error types
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Midtrans API error: {0}")]
    MidtransApi(String),
    #[error("Invalid webhook signature")]
    InvalidSignature,
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("Invalid plan tier")]
    InvalidPlanTier,
}

/// PPN (VAT) rate in Indonesia: 11%
const PPN_RATE: f64 = 0.11;

/// Calculate total amount with PPN
/// Property 2: Payment Amount Calculation
pub fn calculate_total_with_ppn(base_price: i64) -> (i64, i64, i64) {
    let ppn = (base_price as f64 * PPN_RATE).round() as i64;
    let total = base_price + ppn;
    (base_price, ppn, total)
}


/// Billing Service for Midtrans integration
/// Requirements: 2.1, 2.3, 2.4, 2.5, 2.6, 3.1
pub struct BillingService {
    pool: PgPool,
    http_client: Client,
    server_key: String,
    client_key: String,
    is_sandbox: bool,
}

impl BillingService {
    pub fn new(pool: PgPool, server_key: String, client_key: String, is_sandbox: bool) -> Self {
        Self {
            pool,
            http_client: Client::new(),
            server_key,
            client_key,
            is_sandbox,
        }
    }

    /// Get reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    fn snap_url(&self) -> &str {
        if self.is_sandbox {
            "https://app.sandbox.midtrans.com/snap/v1/transactions"
        } else {
            "https://app.midtrans.com/snap/v1/transactions"
        }
    }

    /// Create subscription and get Midtrans Snap token
    /// Requirements: 2.1, 2.3
    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        plan: PlanTier,
        user_email: &str,
    ) -> Result<MidtransSnapToken, BillingError> {
        if plan == PlanTier::Free {
            return Err(BillingError::InvalidPlanTier);
        }

        let (subtotal, ppn, total) = calculate_total_with_ppn(plan.price_idr());
        let order_id = format!("WEB-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), &user_id.to_string()[..8]);

        // Create pending subscription in database
        let subscription_id = Uuid::new_v4();
        let now = Utc::now();
        let period_end = now + Duration::days(30);
        
        sqlx::query(
            r#"
            INSERT INTO subscriptions (id, user_id, plan_tier, price_idr, status, midtrans_order_id, current_period_start, current_period_end, created_at, updated_at)
            VALUES ($1, $2, $3::plan_tier, $4, 'pending', $5, $6, $7, NOW(), NOW())
            "#,
        )
        .bind(subscription_id)
        .bind(user_id)
        .bind(plan.as_str())
        .bind(total)
        .bind(&order_id)
        .bind(now)
        .bind(period_end)
        .execute(&self.pool)
        .await?;

        // Create Midtrans Snap transaction
        let snap_request = serde_json::json!({
            "transaction_details": {
                "order_id": order_id,
                "gross_amount": total
            },
            "item_details": [{
                "id": plan.as_str(),
                "price": subtotal,
                "quantity": 1,
                "name": format!("Webrana {} Plan", plan.as_str().to_uppercase())
            }, {
                "id": "ppn",
                "price": ppn,
                "quantity": 1,
                "name": "PPN 11%"
            }],
            "customer_details": {
                "email": user_email
            },
            "callbacks": {
                "finish": format!("https://webrana.id/dashboard/billing?order_id={}", order_id)
            },
            "custom_field1": subscription_id.to_string(),
            "custom_field2": user_id.to_string()
        });

        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:", self.server_key),
        );

        let response = self
            .http_client
            .post(self.snap_url())
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&snap_request)
            .send()
            .await
            .map_err(|e| BillingError::MidtransApi(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BillingError::MidtransApi(error_text));
        }

        #[derive(Deserialize)]
        struct SnapResponse {
            token: String,
            redirect_url: String,
        }

        let snap_response: SnapResponse = response
            .json()
            .await
            .map_err(|e| BillingError::MidtransApi(e.to_string()))?;

        Ok(MidtransSnapToken {
            token: snap_response.token,
            redirect_url: snap_response.redirect_url,
            order_id,
        })
    }


    /// Verify Midtrans webhook signature
    /// Requirements: 2.4, 2.5
    /// Property 6: Webhook Signature Verification
    pub fn verify_signature(&self, webhook: &MidtransWebhook) -> bool {
        let signature_input = format!(
            "{}{}{}{}",
            webhook.order_id, webhook.status_code, webhook.gross_amount, self.server_key
        );
        
        let mut hasher = Sha512::new();
        hasher.update(signature_input.as_bytes());
        let computed = format!("{:x}", hasher.finalize());
        
        computed == webhook.signature_key
    }

    /// Handle Midtrans webhook notification
    /// Requirements: 2.4, 2.6, 3.1
    pub async fn handle_webhook(&self, webhook: MidtransWebhook) -> Result<(), BillingError> {
        // Verify signature first
        if !self.verify_signature(&webhook) {
            tracing::warn!(
                order_id = %webhook.order_id,
                "Invalid webhook signature - potential security threat"
            );
            return Err(BillingError::InvalidSignature);
        }

        match webhook.transaction_status.as_str() {
            "capture" | "settlement" => {
                self.activate_subscription(&webhook.order_id, &webhook.transaction_id, &webhook.payment_type)
                    .await?;
            }
            "pending" => {
                tracing::info!(order_id = %webhook.order_id, "Payment pending");
            }
            "deny" | "cancel" | "expire" => {
                self.cancel_pending_subscription(&webhook.order_id).await?;
            }
            _ => {
                tracing::warn!(
                    order_id = %webhook.order_id,
                    status = %webhook.transaction_status,
                    "Unknown transaction status"
                );
            }
        }

        Ok(())
    }

    /// Activate subscription after successful payment
    /// Requirements: 3.1
    async fn activate_subscription(
        &self,
        order_id: &str,
        transaction_id: &str,
        payment_type: &str,
    ) -> Result<(), BillingError> {
        let now = Utc::now();
        let end_date = now + Duration::days(30);

        // Get subscription and user info
        let row = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.plan_tier::text as plan_tier, s.price_idr
            FROM subscriptions s
            WHERE s.midtrans_order_id = $1 AND s.status = 'pending'
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(BillingError::SubscriptionNotFound)?;
        let subscription_id: Uuid = row.get("id");
        let user_id: Uuid = row.get("user_id");
        let plan_tier: String = row.get("plan_tier");
        let price_idr: i64 = row.get("price_idr");

        // Update subscription to active
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET status = 'active', midtrans_transaction_id = $1, current_period_start = $2, current_period_end = $3, updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(transaction_id)
        .bind(now)
        .bind(end_date)
        .bind(subscription_id)
        .execute(&self.pool)
        .await?;

        // Update user plan tier
        sqlx::query("UPDATE users SET plan_tier = $1::plan_tier, updated_at = NOW() WHERE id = $2")
            .bind(&plan_tier)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Generate invoice
        self.generate_invoice(user_id, subscription_id, price_idr, transaction_id, payment_type)
            .await?;

        tracing::info!(
            order_id = %order_id,
            user_id = %user_id,
            plan = %plan_tier,
            "Subscription activated"
        );

        Ok(())
    }
    
    /// Generate invoice after payment
    /// Requirements: 4.2, 4.3
    async fn generate_invoice(
        &self,
        user_id: Uuid,
        subscription_id: Uuid,
        total_idr: i64,
        transaction_id: &str,
        payment_type: &str,
    ) -> Result<Uuid, BillingError> {
        let now = Utc::now();
        let ppn = (total_idr as f64 * PPN_RATE / (1.0 + PPN_RATE)).round() as i64;
        let subtotal = total_idr - ppn;
        
        // Generate invoice number: WEB-YYYY-MM-XXX
        let invoice_number = format!(
            "WEB-{}-{:03}",
            now.format("%Y-%m"),
            now.timestamp_millis() % 1000
        );
        
        let invoice_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO invoices (id, user_id, subscription_id, invoice_number, subtotal_idr, ppn_idr, total_idr, payment_method, midtrans_transaction_id, status, paid_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'paid', $10, NOW())
            "#,
        )
        .bind(invoice_id)
        .bind(user_id)
        .bind(subscription_id)
        .bind(&invoice_number)
        .bind(subtotal)
        .bind(ppn)
        .bind(total_idr)
        .bind(payment_type)
        .bind(transaction_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        tracing::info!(invoice_number = %invoice_number, "Invoice generated");
        Ok(invoice_id)
    }

    /// Cancel pending subscription
    async fn cancel_pending_subscription(&self, order_id: &str) -> Result<(), BillingError> {
        sqlx::query(
            "UPDATE subscriptions SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW() WHERE midtrans_order_id = $1",
        )
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(order_id = %order_id, "Subscription cancelled");
        Ok(())
    }

    /// Get user's active subscription
    pub async fn get_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>, BillingError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, plan_tier::text as plan_tier, price_idr, status::text as status, 
                   current_period_start, current_period_end, midtrans_order_id, midtrans_transaction_id, 
                   created_at, updated_at
            FROM subscriptions
            WHERE user_id = $1 AND status = 'active'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Subscription {
            id: r.get("id"),
            user_id: r.get("user_id"),
            plan_tier: r.get("plan_tier"),
            price_idr: r.get("price_idr"),
            status: r.get("status"),
            current_period_start: r.get("current_period_start"),
            current_period_end: r.get("current_period_end"),
            midtrans_order_id: r.get("midtrans_order_id"),
            midtrans_transaction_id: r.get("midtrans_transaction_id"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Cancel subscription (allow access until period ends)
    /// Requirements: 3.5
    pub async fn cancel_subscription(&self, user_id: Uuid) -> Result<(), BillingError> {
        sqlx::query(
            "UPDATE subscriptions SET cancel_at_period_end = true, cancelled_at = NOW(), updated_at = NOW() WHERE user_id = $1 AND status = 'active'",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Upgrade subscription to a higher tier with proration
    /// Requirements: 3.4 - Calculate prorated amount for remaining days
    pub async fn upgrade_subscription(
        &self,
        user_id: Uuid,
        new_plan: PlanTier,
        user_email: &str,
    ) -> Result<UpgradeResult, BillingError> {
        // Get current active subscription
        let current_sub = self.get_subscription(user_id).await?;
        
        let current_sub = match current_sub {
            Some(sub) => sub,
            None => {
                // No active subscription, create new one
                let snap_token = self.create_subscription(user_id, new_plan, user_email).await?;
                return Ok(UpgradeResult {
                    prorated_amount: 0,
                    new_total: calculate_total_with_ppn(new_plan.price_idr()).2,
                    snap_token: Some(snap_token),
                    remaining_days: 30,
                });
            }
        };

        // Parse current plan tier
        let current_plan = match current_sub.plan_tier.as_str() {
            "free" => PlanTier::Free,
            "starter" => PlanTier::Starter,
            "pro" => PlanTier::Pro,
            "team" => PlanTier::Team,
            _ => return Err(BillingError::InvalidPlanTier),
        };

        // Can't downgrade or stay same
        if new_plan.price_idr() <= current_plan.price_idr() {
            return Err(BillingError::InvalidPlanTier);
        }

        // Calculate remaining days
        let now = Utc::now();
        let remaining_days = (current_sub.current_period_end - now).num_days().max(0);

        // Calculate prorated amount: (new_price - old_price) * (remaining_days / 30)
        let price_diff = new_plan.price_idr() - current_plan.price_idr();
        let prorated_base = ((price_diff as f64 * remaining_days as f64) / 30.0).round() as i64;
        let (_, ppn, prorated_total) = calculate_total_with_ppn(prorated_base);

        // Create order for prorated amount
        let order_id = format!("WEB-UPG-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), &user_id.to_string()[..8]);

        // Create pending upgrade subscription
        let subscription_id = Uuid::new_v4();
        let period_end = current_sub.current_period_end; // Keep same end date
        
        sqlx::query(
            r#"
            INSERT INTO subscriptions (id, user_id, plan_tier, price_idr, status, midtrans_order_id, current_period_start, current_period_end, is_upgrade, previous_subscription_id, created_at, updated_at)
            VALUES ($1, $2, $3::plan_tier, $4, 'pending', $5, $6, $7, true, $8, NOW(), NOW())
            "#,
        )
        .bind(subscription_id)
        .bind(user_id)
        .bind(new_plan.as_str())
        .bind(prorated_total)
        .bind(&order_id)
        .bind(now)
        .bind(period_end)
        .bind(Uuid::parse_str(&current_sub.id.to_string()).ok())
        .execute(&self.pool)
        .await?;

        // Create Midtrans Snap transaction for prorated amount
        let snap_request = serde_json::json!({
            "transaction_details": {
                "order_id": order_id,
                "gross_amount": prorated_total
            },
            "item_details": [{
                "id": format!("upgrade-{}", new_plan.as_str()),
                "price": prorated_base,
                "quantity": 1,
                "name": format!("Upgrade to {} Plan (Prorated {} days)", new_plan.as_str().to_uppercase(), remaining_days)
            }, {
                "id": "ppn",
                "price": ppn,
                "quantity": 1,
                "name": "PPN 11%"
            }],
            "customer_details": {
                "email": user_email
            },
            "callbacks": {
                "finish": format!("https://webrana.id/dashboard/billing?order_id={}", order_id)
            },
            "custom_field1": subscription_id.to_string(),
            "custom_field2": user_id.to_string(),
            "custom_field3": "upgrade"
        });

        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:", self.server_key),
        );

        let response = self
            .http_client
            .post(self.snap_url())
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&snap_request)
            .send()
            .await
            .map_err(|e| BillingError::MidtransApi(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BillingError::MidtransApi(error_text));
        }

        #[derive(Deserialize)]
        struct SnapResponse {
            token: String,
            redirect_url: String,
        }

        let snap_response: SnapResponse = response
            .json()
            .await
            .map_err(|e| BillingError::MidtransApi(e.to_string()))?;

        tracing::info!(
            user_id = %user_id,
            from_plan = %current_plan.as_str(),
            to_plan = %new_plan.as_str(),
            prorated_amount = prorated_total,
            remaining_days = remaining_days,
            "Upgrade subscription initiated"
        );

        Ok(UpgradeResult {
            prorated_amount: prorated_total,
            new_total: prorated_total,
            snap_token: Some(MidtransSnapToken {
                token: snap_response.token,
                redirect_url: snap_response.redirect_url,
                order_id,
            }),
            remaining_days: remaining_days as i32,
        })
    }

    /// Check and expire subscriptions that have passed their end date
    /// Requirements: 3.3 - Downgrade to Free tier on expiration
    /// This should be called by a scheduled task (cron job) daily
    pub async fn check_expired_subscriptions(&self) -> Result<ExpiredSubscriptionsResult, BillingError> {
        let now = Utc::now();
        
        // Find all active subscriptions that have expired
        let expired_rows = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.plan_tier::text as plan_tier
            FROM subscriptions s
            WHERE s.status = 'active'
              AND s.current_period_end < $1
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let mut expired_count = 0;
        let mut downgraded_users = Vec::new();

        for row in expired_rows {
            let subscription_id: Uuid = row.get("id");
            let user_id: Uuid = row.get("user_id");
            let plan_tier: String = row.get("plan_tier");

            // Update subscription status to expired
            sqlx::query(
                "UPDATE subscriptions SET status = 'expired', updated_at = NOW() WHERE id = $1",
            )
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;

            // Downgrade user to Free tier
            sqlx::query(
                "UPDATE users SET plan_tier = 'free'::plan_tier, updated_at = NOW() WHERE id = $1",
            )
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            tracing::info!(
                user_id = %user_id,
                old_plan = %plan_tier,
                "Subscription expired, downgraded to Free tier"
            );

            expired_count += 1;
            downgraded_users.push(user_id);
        }

        Ok(ExpiredSubscriptionsResult {
            expired_count,
            downgraded_users,
        })
    }

    /// Check subscriptions expiring soon (for sending reminder emails)
    /// Requirements: 3.2 - Send email 7 days before expiry
    pub async fn get_expiring_subscriptions(&self, days_before: i64) -> Result<Vec<ExpiringSubscription>, BillingError> {
        let now = Utc::now();
        let expiry_threshold = now + Duration::days(days_before);
        
        let rows = sqlx::query(
            r#"
            SELECT s.id, s.user_id, s.plan_tier::text as plan_tier, s.current_period_end,
                   u.email, u.name
            FROM subscriptions s
            JOIN users u ON u.id = s.user_id
            WHERE s.status = 'active'
              AND s.current_period_end <= $1
              AND s.current_period_end > $2
              AND (s.cancel_at_period_end IS NULL OR s.cancel_at_period_end = false)
            "#,
        )
        .bind(expiry_threshold)
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ExpiringSubscription {
                subscription_id: r.get("id"),
                user_id: r.get("user_id"),
                user_email: r.get("email"),
                user_name: r.get("name"),
                plan_tier: r.get("plan_tier"),
                expires_at: r.get("current_period_end"),
            })
            .collect())
    }
}

/// Result of expired subscriptions check
#[derive(Debug, Serialize)]
pub struct ExpiredSubscriptionsResult {
    pub expired_count: i32,
    pub downgraded_users: Vec<Uuid>,
}

/// Result of subscription upgrade
#[derive(Debug, Serialize)]
pub struct UpgradeResult {
    pub prorated_amount: i64,
    pub new_total: i64,
    pub snap_token: Option<MidtransSnapToken>,
    pub remaining_days: i32,
}

/// Subscription expiring soon (for reminder emails)
#[derive(Debug, Serialize)]
pub struct ExpiringSubscription {
    pub subscription_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: Option<String>,
    pub plan_tier: String,
    pub expires_at: DateTime<Utc>,
}


