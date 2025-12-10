//! Usage Logger Service for tracking proxy request analytics.
//!
//! Requirements: 5.1-5.5 - Usage logging for analytics
//!
//! Asynchronously logs request metadata including tokens, latency, and cost.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::services::transformers::Provider;

/// Usage log entry for a proxy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLog {
    pub user_id: Uuid,
    pub proxy_key_id: Option<Uuid>,
    pub provider: Provider,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub latency_ms: i32,
    pub estimated_cost_idr: i64,
    pub status_code: i16,
    pub error_message: Option<String>,
}

/// Provider pricing configuration (per 1M tokens in IDR)
#[derive(Debug, Clone)]
pub struct ProviderPricing {
    pub input_per_million: i64,
    pub output_per_million: i64,
}

impl ProviderPricing {
    /// Get pricing for a provider and model
    /// Prices are approximate conversions to IDR (1 USD ≈ 15,500 IDR)
    pub fn for_model(provider: Provider, model: &str) -> Self {
        match provider {
            Provider::OpenAI => Self::openai_pricing(model),
            Provider::Anthropic => Self::anthropic_pricing(model),
            Provider::Google => Self::google_pricing(model),
            Provider::Qwen => Self::qwen_pricing(model),
        }
    }

    fn openai_pricing(model: &str) -> Self {
        // GPT-4 Turbo: $10/1M input, $30/1M output
        // GPT-4: $30/1M input, $60/1M output
        // GPT-3.5 Turbo: $0.50/1M input, $1.50/1M output
        if model.contains("gpt-4-turbo") || model.contains("gpt-4o") {
            Self { input_per_million: 155_000, output_per_million: 465_000 }
        } else if model.starts_with("gpt-4") {
            Self { input_per_million: 465_000, output_per_million: 930_000 }
        } else if model.starts_with("o1") {
            Self { input_per_million: 232_500, output_per_million: 930_000 }
        } else {
            // GPT-3.5 Turbo
            Self { input_per_million: 7_750, output_per_million: 23_250 }
        }
    }

    fn anthropic_pricing(model: &str) -> Self {
        // Claude 3 Opus: $15/1M input, $75/1M output
        // Claude 3 Sonnet: $3/1M input, $15/1M output
        // Claude 3 Haiku: $0.25/1M input, $1.25/1M output
        if model.contains("opus") {
            Self { input_per_million: 232_500, output_per_million: 1_162_500 }
        } else if model.contains("sonnet") {
            Self { input_per_million: 46_500, output_per_million: 232_500 }
        } else {
            // Haiku
            Self { input_per_million: 3_875, output_per_million: 19_375 }
        }
    }

    fn google_pricing(model: &str) -> Self {
        // Gemini 1.5 Pro: $3.50/1M input, $10.50/1M output
        // Gemini 1.5 Flash: $0.075/1M input, $0.30/1M output
        if model.contains("flash") {
            Self { input_per_million: 1_163, output_per_million: 4_650 }
        } else {
            // Pro models
            Self { input_per_million: 54_250, output_per_million: 162_750 }
        }
    }

    fn qwen_pricing(model: &str) -> Self {
        // Qwen pricing (approximate, varies by region)
        // Qwen-Max: ~$2/1M input, ~$6/1M output
        // Qwen-Plus: ~$0.50/1M input, ~$1.50/1M output
        // Qwen-Turbo: ~$0.10/1M input, ~$0.30/1M output
        if model.contains("max") {
            Self { input_per_million: 31_000, output_per_million: 93_000 }
        } else if model.contains("plus") {
            Self { input_per_million: 7_750, output_per_million: 23_250 }
        } else {
            // Turbo
            Self { input_per_million: 1_550, output_per_million: 4_650 }
        }
    }
}

/// Token counter for estimating token usage
pub struct TokenCounter;

impl TokenCounter {
    /// Estimate token count from text
    /// Uses chars/4 approximation as fallback
    /// Requirements: 5.2, 5.5
    pub fn estimate_tokens(text: &str) -> i32 {
        // Simple estimation: ~4 characters per token for English
        // This is a reasonable approximation when tiktoken is not available
        (text.len() as f64 / 4.0).ceil() as i32
    }

    /// Count tokens from messages
    pub fn count_message_tokens(messages: &[crate::services::transformers::Message]) -> i32 {
        messages.iter()
            .map(|m| Self::estimate_tokens(&m.content) + Self::estimate_tokens(&m.role) + 4)
            .sum::<i32>() + 3 // Base overhead
    }
}

/// Usage logger service
pub struct UsageLogger;

impl UsageLogger {
    /// Log a proxy request asynchronously
    /// Requirements: 5.1, 5.3, 5.4
    pub async fn log_request(pool: &PgPool, log: UsageLog) -> Result<Uuid, sqlx::Error> {
        let provider_str = match log.provider {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Google => "google",
            Provider::Qwen => "qwen",
        };

        // Use runtime query to avoid compile-time database dependency
        let row = sqlx::query(
            r#"
            INSERT INTO proxy_requests (
                user_id, proxy_key_id, provider, model,
                prompt_tokens, completion_tokens, total_tokens,
                latency_ms, estimated_cost_idr, status_code, error_message
            )
            VALUES ($1, $2, $3::ai_provider, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
        .bind(log.user_id)
        .bind(log.proxy_key_id)
        .bind(provider_str)
        .bind(&log.model)
        .bind(log.prompt_tokens)
        .bind(log.completion_tokens)
        .bind(log.total_tokens)
        .bind(log.latency_ms)
        .bind(log.estimated_cost_idr)
        .bind(log.status_code as i32)
        .bind(&log.error_message)
        .fetch_one(pool)
        .await?;

        let id: Uuid = row.get("id");

        Ok(id)
    }

    /// Calculate estimated cost in IDR
    /// Requirements: 5.2
    pub fn calculate_cost(
        provider: Provider,
        model: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
    ) -> i64 {
        let pricing = ProviderPricing::for_model(provider, model);
        
        let input_cost = (prompt_tokens as i64 * pricing.input_per_million) / 1_000_000;
        let output_cost = (completion_tokens as i64 * pricing.output_per_million) / 1_000_000;
        
        input_cost + output_cost
    }

    /// Spawn async logging task to avoid blocking response
    /// Requirements: 5.3
    pub fn log_async(pool: PgPool, log: UsageLog) {
        tokio::spawn(async move {
            if let Err(e) = Self::log_request(&pool, log).await {
                tracing::error!("Failed to log usage: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        // ~4 chars per token
        assert_eq!(TokenCounter::estimate_tokens("Hello"), 2); // 5 chars -> 2 tokens
        assert_eq!(TokenCounter::estimate_tokens("Hello, world!"), 4); // 13 chars -> 4 tokens
        assert_eq!(TokenCounter::estimate_tokens(""), 0);
    }

    #[test]
    fn test_cost_calculation_openai() {
        // GPT-4 Turbo: 155,000 IDR/1M input, 465,000 IDR/1M output
        let cost = UsageLogger::calculate_cost(
            Provider::OpenAI,
            "gpt-4-turbo",
            1000, // 1K input tokens
            500,  // 500 output tokens
        );
        
        // Expected: (1000 * 155,000 / 1M) + (500 * 465,000 / 1M) = 155 + 232 = 387 IDR
        assert!(cost > 0);
        assert!(cost < 1000); // Sanity check
    }

    #[test]
    fn test_cost_calculation_anthropic() {
        let cost = UsageLogger::calculate_cost(
            Provider::Anthropic,
            "claude-3-haiku",
            1000,
            500,
        );
        
        // Haiku is cheaper
        assert!(cost > 0);
        assert!(cost < 100);
    }

    #[test]
    fn test_pricing_tiers() {
        // GPT-4 should be more expensive than GPT-3.5
        let gpt4_pricing = ProviderPricing::for_model(Provider::OpenAI, "gpt-4");
        let gpt35_pricing = ProviderPricing::for_model(Provider::OpenAI, "gpt-3.5-turbo");
        
        assert!(gpt4_pricing.input_per_million > gpt35_pricing.input_per_million);
        assert!(gpt4_pricing.output_per_million > gpt35_pricing.output_per_million);
    }

    #[test]
    fn test_message_token_count() {
        use crate::services::transformers::Message;
        
        let messages = vec![
            Message { role: "user".to_string(), content: "Hello".to_string() },
            Message { role: "assistant".to_string(), content: "Hi there!".to_string() },
        ];
        
        let count = TokenCounter::count_message_tokens(&messages);
        assert!(count > 0);
    }
}
