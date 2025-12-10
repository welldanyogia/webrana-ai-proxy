//! Provider API key model for encrypted storage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Supported AI providers matching PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ai_provider", rename_all = "lowercase")]
pub enum AiProvider {
    #[serde(rename = "openai")]
    Openai,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "qwen")]
    Qwen,
}

impl AiProvider {
    /// Get the API base URL for this provider
    pub fn base_url(&self) -> &'static str {
        match self {
            AiProvider::Openai => "https://api.openai.com/v1",
            AiProvider::Anthropic => "https://api.anthropic.com/v1",
            AiProvider::Google => "https://generativelanguage.googleapis.com/v1beta",
            AiProvider::Qwen => "https://dashscope.aliyuncs.com/api/v1",
        }
    }

    /// Validate API key format for this provider
    pub fn validate_key_format(&self, key: &str) -> bool {
        match self {
            AiProvider::Openai => key.starts_with("sk-"),
            AiProvider::Anthropic => key.starts_with("sk-ant-"),
            AiProvider::Google => key.starts_with("AI"),
            AiProvider::Qwen => !key.is_empty(), // Qwen uses various formats
        }
    }
}

/// Provider API key entity (encrypted)
#[derive(Debug, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AiProvider,
    pub key_name: String,
    pub encrypted_key: Vec<u8>,
    pub iv: Vec<u8>,
    pub auth_tag: Vec<u8>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create API key DTO
#[derive(Debug, Deserialize)]
pub struct CreateApiKey {
    pub provider: AiProvider,
    pub key: String,
    pub name: String,
}

/// API key info for listing (masked, no sensitive data)
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub provider: AiProvider,
    pub name: String,
    pub masked_key: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ApiKeyInfo {
    /// Create masked key display (e.g., "sk-...abc123")
    pub fn mask_key(key: &str) -> String {
        if key.len() <= 8 {
            return "***".to_string();
        }
        let prefix = &key[..3];
        let suffix = &key[key.len() - 6..];
        format!("{}...{}", prefix, suffix)
    }
}
