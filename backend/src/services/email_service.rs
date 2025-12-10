//! Email Service for sending transactional emails
//!
//! Requirements: 7.1, 7.2, 7.3, 7.5, 7.6
//! Sends emails via SendGrid/Resend API with retry logic

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

/// Email template types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailTemplate {
    Welcome,
    PaymentSuccess,
    PaymentFailed,
    QuotaWarning,
    QuotaExceeded,
    SubscriptionExpiring,
    OnboardingReminder,
}

impl EmailTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailTemplate::Welcome => "welcome",
            EmailTemplate::PaymentSuccess => "payment_success",
            EmailTemplate::PaymentFailed => "payment_failed",
            EmailTemplate::QuotaWarning => "quota_warning",
            EmailTemplate::QuotaExceeded => "quota_exceeded",
            EmailTemplate::SubscriptionExpiring => "subscription_expiring",
            EmailTemplate::OnboardingReminder => "onboarding_reminder",
        }
    }
}

/// Email send request
#[derive(Debug, Clone)]
pub struct EmailRequest {
    pub to: String,
    pub to_name: Option<String>,
    pub template: EmailTemplate,
    pub data: EmailData,
    pub language: String, // "id" or "en"
}

/// Email template data
#[derive(Debug, Clone, Serialize)]
pub struct EmailData {
    pub user_name: Option<String>,
    pub plan_name: Option<String>,
    pub amount: Option<String>,
    pub invoice_number: Option<String>,
    pub usage_percent: Option<u8>,
    pub days_remaining: Option<i32>,
    pub error_reason: Option<String>,
}

impl Default for EmailData {
    fn default() -> Self {
        Self {
            user_name: None,
            plan_name: None,
            amount: None,
            invoice_number: None,
            usage_percent: None,
            days_remaining: None,
            error_reason: None,
        }
    }
}


/// Email log entry
#[derive(Debug, Serialize)]
pub struct EmailLog {
    pub id: Uuid,
    pub recipient: String,
    pub template: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: DateTime<Utc>,
}

/// Email service error
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

/// Retry configuration
const MAX_RETRIES: u32 = 3;
const RETRY_DELAYS_SECS: [u64; 3] = [60, 300, 1800]; // 1min, 5min, 30min

/// Email Service using Resend API
/// Requirements: 7.1, 7.5
pub struct EmailService {
    pool: PgPool,
    http_client: Client,
    api_key: String,
    from_email: String,
    from_name: String,
}

impl EmailService {
    pub fn new(pool: PgPool, api_key: String) -> Self {
        Self {
            pool,
            http_client: Client::new(),
            api_key,
            from_email: "noreply@webrana.id".to_string(),
            from_name: "Webrana".to_string(),
        }
    }

    /// Send email with retry logic
    /// Requirements: 7.5 - 3 retries with exponential backoff
    pub async fn send_email(&self, request: EmailRequest) -> Result<(), EmailError> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match self.send_email_internal(&request).await {
                Ok(_) => {
                    self.log_email(&request.to, request.template.as_str(), "sent", None)
                        .await?;
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        let delay = Duration::from_secs(RETRY_DELAYS_SECS[attempt as usize]);
                        tracing::warn!(
                            attempt = attempt + 1,
                            delay_secs = delay.as_secs(),
                            "Email send failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // Log failure after all retries
        let error_msg = last_error.as_ref().map(|e| e.to_string());
        self.log_email(&request.to, request.template.as_str(), "failed", error_msg.as_deref())
            .await?;

        Err(EmailError::MaxRetriesExceeded)
    }

    /// Internal send without retry
    async fn send_email_internal(&self, request: &EmailRequest) -> Result<(), EmailError> {
        let (subject, html_body) = self.render_template(request);

        let payload = serde_json::json!({
            "from": format!("{} <{}>", self.from_name, self.from_email),
            "to": [request.to.clone()],
            "subject": subject,
            "html": html_body,
        });

        let response = self
            .http_client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmailError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmailError::ApiError(error_text));
        }

        tracing::info!(
            to = %request.to,
            template = %request.template.as_str(),
            "Email sent successfully"
        );

        Ok(())
    }

    /// Log email to database
    /// Requirements: 7.6
    async fn log_email(
        &self,
        recipient: &str,
        template: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), EmailError> {
        sqlx::query(
            r#"
            INSERT INTO email_logs (id, recipient, template, status, error_message, sent_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(recipient)
        .bind(template)
        .bind(status)
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    /// Render email template
    /// Requirements: 7.2, 7.3 - Bilingual templates (ID/EN)
    fn render_template(&self, request: &EmailRequest) -> (String, String) {
        let is_indonesian = request.language == "id";
        let name = request.data.user_name.clone().unwrap_or_else(|| "Pengguna".to_string());

        match request.template {
            EmailTemplate::Welcome => {
                if is_indonesian {
                    (
                        "Selamat Datang di Webrana! 🎉".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #3B82F6;">Selamat Datang, {}!</h1>
<p>Terima kasih telah bergabung dengan Webrana. Anda sekarang dapat mengakses semua model AI melalui satu API.</p>
<p>Mulai dengan:</p>
<ul>
<li>Tambahkan API key provider Anda di Dashboard</li>
<li>Gunakan endpoint proxy kami untuk request</li>
<li>Pantau penggunaan dan biaya secara real-time</li>
</ul>
<p>Butuh bantuan? Hubungi kami di support@webrana.id</p>
<p>Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name
                        ),
                    )
                } else {
                    (
                        "Welcome to Webrana! 🎉".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #3B82F6;">Welcome, {}!</h1>
<p>Thank you for joining Webrana. You can now access all AI models through a single API.</p>
<p>Get started by:</p>
<ul>
<li>Adding your provider API keys in the Dashboard</li>
<li>Using our proxy endpoint for requests</li>
<li>Monitoring usage and costs in real-time</li>
</ul>
<p>Need help? Contact us at support@webrana.id</p>
<p>Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name
                        ),
                    )
                }
            }

            EmailTemplate::PaymentSuccess => {
                let amount = request.data.amount.clone().unwrap_or_default();
                let invoice = request.data.invoice_number.clone().unwrap_or_default();
                let plan = request.data.plan_name.clone().unwrap_or_default();

                if is_indonesian {
                    (
                        format!("Pembayaran Berhasil - {}", invoice),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #10B981;">Pembayaran Berhasil! ✅</h1>
<p>Halo {},</p>
<p>Pembayaran Anda telah berhasil diproses.</p>
<div style="background: #F3F4F6; padding: 20px; border-radius: 8px; margin: 20px 0;">
<p><strong>Invoice:</strong> {}</p>
<p><strong>Plan:</strong> {}</p>
<p><strong>Total:</strong> {}</p>
</div>
<p>Langganan Anda sekarang aktif selama 30 hari.</p>
<p>Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name, invoice, plan, amount
                        ),
                    )
                } else {
                    (
                        format!("Payment Successful - {}", invoice),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #10B981;">Payment Successful! ✅</h1>
<p>Hello {},</p>
<p>Your payment has been successfully processed.</p>
<div style="background: #F3F4F6; padding: 20px; border-radius: 8px; margin: 20px 0;">
<p><strong>Invoice:</strong> {}</p>
<p><strong>Plan:</strong> {}</p>
<p><strong>Total:</strong> {}</p>
</div>
<p>Your subscription is now active for 30 days.</p>
<p>Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name, invoice, plan, amount
                        ),
                    )
                }
            }

            EmailTemplate::PaymentFailed => {
                let reason = request.data.error_reason.clone().unwrap_or_else(|| "Unknown error".to_string());

                if is_indonesian {
                    (
                        "Pembayaran Gagal ❌".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #EF4444;">Pembayaran Gagal</h1>
<p>Halo {},</p>
<p>Maaf, pembayaran Anda tidak dapat diproses.</p>
<p><strong>Alasan:</strong> {}</p>
<p>Silakan coba lagi atau gunakan metode pembayaran lain.</p>
<p>Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name, reason
                        ),
                    )
                } else {
                    (
                        "Payment Failed ❌".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #EF4444;">Payment Failed</h1>
<p>Hello {},</p>
<p>Sorry, your payment could not be processed.</p>
<p><strong>Reason:</strong> {}</p>
<p>Please try again or use a different payment method.</p>
<p>Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name, reason
                        ),
                    )
                }
            }

            EmailTemplate::QuotaWarning => {
                let percent = request.data.usage_percent.unwrap_or(80);

                if is_indonesian {
                    (
                        format!("Peringatan Kuota - {}% Terpakai ⚠️", percent),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #F59E0B;">Peringatan Kuota ⚠️</h1>
<p>Halo {},</p>
<p>Anda telah menggunakan <strong>{}%</strong> dari kuota bulanan Anda.</p>
<p>Pertimbangkan untuk upgrade plan agar tidak terganggu.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Upgrade Sekarang</a>
<p style="margin-top: 20px;">Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name, percent
                        ),
                    )
                } else {
                    (
                        format!("Quota Warning - {}% Used ⚠️", percent),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #F59E0B;">Quota Warning ⚠️</h1>
<p>Hello {},</p>
<p>You have used <strong>{}%</strong> of your monthly quota.</p>
<p>Consider upgrading your plan to avoid interruption.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Upgrade Now</a>
<p style="margin-top: 20px;">Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name, percent
                        ),
                    )
                }
            }

            EmailTemplate::QuotaExceeded => {
                if is_indonesian {
                    (
                        "Kuota Habis - Upgrade Diperlukan 🚫".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #EF4444;">Kuota Bulanan Habis 🚫</h1>
<p>Halo {},</p>
<p>Kuota bulanan Anda telah habis. Request API akan ditolak hingga bulan depan atau Anda upgrade plan.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Upgrade Sekarang</a>
<p style="margin-top: 20px;">Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name
                        ),
                    )
                } else {
                    (
                        "Quota Exceeded - Upgrade Required 🚫".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #EF4444;">Monthly Quota Exceeded 🚫</h1>
<p>Hello {},</p>
<p>Your monthly quota has been exceeded. API requests will be rejected until next month or you upgrade your plan.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Upgrade Now</a>
<p style="margin-top: 20px;">Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name
                        ),
                    )
                }
            }

            EmailTemplate::SubscriptionExpiring => {
                let days = request.data.days_remaining.unwrap_or(7);
                let plan = request.data.plan_name.clone().unwrap_or_default();

                if is_indonesian {
                    (
                        format!("Langganan Berakhir dalam {} Hari ⏰", days),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #F59E0B;">Langganan Akan Berakhir ⏰</h1>
<p>Halo {},</p>
<p>Langganan <strong>{}</strong> Anda akan berakhir dalam <strong>{} hari</strong>.</p>
<p>Perpanjang sekarang untuk terus menikmati layanan tanpa gangguan.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Perpanjang Sekarang</a>
<p style="margin-top: 20px;">Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name, plan, days
                        ),
                    )
                } else {
                    (
                        format!("Subscription Expiring in {} Days ⏰", days),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #F59E0B;">Subscription Expiring Soon ⏰</h1>
<p>Hello {},</p>
<p>Your <strong>{}</strong> subscription will expire in <strong>{} days</strong>.</p>
<p>Renew now to continue enjoying uninterrupted service.</p>
<a href="https://webrana.id/dashboard/billing" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Renew Now</a>
<p style="margin-top: 20px;">Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name, plan, days
                        ),
                    )
                }
            }

            EmailTemplate::OnboardingReminder => {
                if is_indonesian {
                    (
                        "Jangan Lupa Tambahkan API Key Anda! 🔑".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #3B82F6;">Halo {}! 👋</h1>
<p>Kami perhatikan Anda belum menambahkan API key ke akun Webrana Anda.</p>
<p>Untuk mulai menggunakan proxy AI kami, Anda perlu:</p>
<ol>
<li>Login ke Dashboard Webrana</li>
<li>Pergi ke halaman API Keys</li>
<li>Tambahkan API key dari provider AI Anda (OpenAI, Anthropic, dll)</li>
<li>Mulai kirim request melalui endpoint proxy kami!</li>
</ol>
<a href="https://webrana.id/dashboard/api-keys" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Tambah API Key Sekarang</a>
<p style="margin-top: 20px;">Butuh bantuan? Balas email ini atau hubungi support@webrana.id</p>
<p>Salam,<br>Tim Webrana</p>
</body></html>"#,
                            name
                        ),
                    )
                } else {
                    (
                        "Don't Forget to Add Your API Key! 🔑".to_string(),
                        format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #3B82F6;">Hello {}! 👋</h1>
<p>We noticed you haven't added an API key to your Webrana account yet.</p>
<p>To start using our AI proxy, you need to:</p>
<ol>
<li>Login to your Webrana Dashboard</li>
<li>Go to the API Keys page</li>
<li>Add your AI provider API key (OpenAI, Anthropic, etc.)</li>
<li>Start sending requests through our proxy endpoint!</li>
</ol>
<a href="https://webrana.id/dashboard/api-keys" style="display: inline-block; background: #3B82F6; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px;">Add API Key Now</a>
<p style="margin-top: 20px;">Need help? Reply to this email or contact support@webrana.id</p>
<p>Best regards,<br>The Webrana Team</p>
</body></html>"#,
                            name
                        ),
                    )
                }
            }
        }
    }
}

// Convenience methods for common email types
impl EmailService {
    /// Send welcome email
    /// Requirements: 7.2
    pub async fn send_welcome(&self, email: &str, name: Option<String>, language: &str) -> Result<(), EmailError> {
        self.send_email(EmailRequest {
            to: email.to_string(),
            to_name: name.clone(),
            template: EmailTemplate::Welcome,
            data: EmailData {
                user_name: name,
                ..Default::default()
            },
            language: language.to_string(),
        })
        .await
    }

    /// Send quota warning email
    /// Requirements: 5.3, 7.2
    pub async fn send_quota_warning(
        &self,
        email: &str,
        name: Option<String>,
        usage_percent: u8,
        language: &str,
    ) -> Result<(), EmailError> {
        self.send_email(EmailRequest {
            to: email.to_string(),
            to_name: name.clone(),
            template: EmailTemplate::QuotaWarning,
            data: EmailData {
                user_name: name,
                usage_percent: Some(usage_percent),
                ..Default::default()
            },
            language: language.to_string(),
        })
        .await
    }

    /// Send subscription expiring email
    /// Requirements: 3.2, 7.2
    pub async fn send_subscription_expiring(
        &self,
        email: &str,
        name: Option<String>,
        plan_name: &str,
        days_remaining: i32,
        language: &str,
    ) -> Result<(), EmailError> {
        self.send_email(EmailRequest {
            to: email.to_string(),
            to_name: name.clone(),
            template: EmailTemplate::SubscriptionExpiring,
            data: EmailData {
                user_name: name,
                plan_name: Some(plan_name.to_string()),
                days_remaining: Some(days_remaining),
                ..Default::default()
            },
            language: language.to_string(),
        })
        .await
    }

    /// Send onboarding reminder email for users who haven't added API key
    /// Requirements: 5.5
    pub async fn send_onboarding_reminder(
        &self,
        email: &str,
        name: Option<String>,
        language: &str,
    ) -> Result<(), EmailError> {
        self.send_email(EmailRequest {
            to: email.to_string(),
            to_name: name.clone(),
            template: EmailTemplate::OnboardingReminder,
            data: EmailData {
                user_name: name,
                ..Default::default()
            },
            language: language.to_string(),
        })
        .await
    }
}
