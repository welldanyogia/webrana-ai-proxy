//! Proxy API key model for Webrana-issued keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Proxy API key prefix
pub const PROXY_KEY_PREFIX: &str = "wbr_";

/// Proxy API key entity (hashed)
#[derive(Debug, FromRow)]
pub struct ProxyApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create proxy API key DTO
#[derive(Debug, Deserialize)]
pub struct CreateProxyApiKey {
    pub name: String,
}

/// Proxy API key info for listing (no sensitive data)
#[derive(Debug, Serialize)]
pub struct ProxyApiKeyInfo {
    pub id: Uuid,
    pub prefix: String,
    pub name: String,
    pub is_active: bool,
    pub request_count: i64,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl From<ProxyApiKey> for ProxyApiKeyInfo {
    fn from(key: ProxyApiKey) -> Self {
        Self {
            id: key.id,
            prefix: key.key_prefix,
            name: key.name,
            is_active: key.is_active,
            request_count: key.request_count,
            created_at: key.created_at,
            last_used_at: key.last_used_at,
        }
    }
}

/// Response when creating a new proxy API key (includes plaintext key once)
#[derive(Debug, Serialize)]
pub struct ProxyApiKeyCreated {
    pub id: Uuid,
    pub key: String,  // Plaintext key - shown only once!
    pub prefix: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
