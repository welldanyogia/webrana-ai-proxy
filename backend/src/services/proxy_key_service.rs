//! Proxy API Key service for Webrana-issued keys.
//!
//! Requirements: 6.1-6.5 - Proxy API key generation, hashing, and validation

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::proxy_api_key::{
    CreateProxyApiKey, ProxyApiKey, ProxyApiKeyCreated, ProxyApiKeyInfo, PROXY_KEY_PREFIX,
};
use crate::models::user::PlanTier;
use crate::utils::password::{hash_password, verify_password, PasswordError};

/// Proxy key service error
#[derive(Debug)]
pub enum ProxyKeyError {
    HashingError(PasswordError),
    DatabaseError(sqlx::Error),
    NotFound,
    KeyLimitReached { limit: u32, plan: PlanTier },
    Revoked,
}

impl std::fmt::Display for ProxyKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyKeyError::HashingError(e) => write!(f, "Hashing error: {}", e),
            ProxyKeyError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ProxyKeyError::NotFound => write!(f, "Proxy API key not found"),
            ProxyKeyError::KeyLimitReached { limit, plan } => {
                write!(f, "API key limit ({}) reached for {:?} plan", limit, plan)
            }
            ProxyKeyError::Revoked => write!(f, "Proxy API key has been revoked"),
        }
    }
}

impl std::error::Error for ProxyKeyError {}

impl From<PasswordError> for ProxyKeyError {
    fn from(e: PasswordError) -> Self {
        ProxyKeyError::HashingError(e)
    }
}

impl From<sqlx::Error> for ProxyKeyError {
    fn from(e: sqlx::Error) -> Self {
        ProxyKeyError::DatabaseError(e)
    }
}

/// Proxy key service implementation
pub struct ProxyKeyService;

impl ProxyKeyService {
    /// Generate a new proxy API key
    /// Requirements: 6.1, 6.2, 6.3, 6.5
    pub async fn generate_key(
        pool: &PgPool,
        user_id: Uuid,
        plan: PlanTier,
        input: CreateProxyApiKey,
    ) -> Result<ProxyApiKeyCreated, ProxyKeyError> {
        // Check API key limit (Requirement 6.5)
        if let Some(limit) = plan.api_key_limit() {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM proxy_api_keys WHERE user_id = $1 AND is_active = true",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

            if count.0 >= limit as i64 {
                return Err(ProxyKeyError::KeyLimitReached { limit, plan });
            }
        }

        // Generate 32-byte cryptographically secure random key (Requirement 6.1)
        let mut key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key_bytes);
        let key_base64 = URL_SAFE_NO_PAD.encode(key_bytes);
        let plaintext_key = format!("{}{}", PROXY_KEY_PREFIX, key_base64);

        // Create prefix for display (first 8 chars after wbr_)
        let key_prefix = format!("{}{}...", PROXY_KEY_PREFIX, &key_base64[..8]);

        // Hash the key with Argon2id (Requirement 6.2)
        let key_hash = hash_password(&plaintext_key)?;

        // Store in database
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO proxy_api_keys (id, user_id, key_hash, key_prefix, name, is_active, request_count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, 0, $6, $6)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(&key_hash)
        .bind(&key_prefix)
        .bind(&input.name)
        .bind(now)
        .execute(pool)
        .await?;

        // Return plaintext key exactly once (Requirement 6.3)
        Ok(ProxyApiKeyCreated {
            id,
            key: plaintext_key,
            prefix: key_prefix,
            name: input.name,
            created_at: now,
        })
    }


    /// List proxy API keys for a user (prefix and metadata only)
    /// Requirement: 6.4
    pub async fn list_keys(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ProxyApiKeyInfo>, ProxyKeyError> {
        let keys: Vec<ProxyApiKey> = sqlx::query_as(
            r#"
            SELECT id, user_id, key_hash, key_prefix, name, is_active, last_used_at, request_count, created_at, updated_at
            FROM proxy_api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(keys.into_iter().map(ProxyApiKeyInfo::from).collect())
    }

    /// Revoke a proxy API key (soft delete)
    /// Requirement: 6.1
    pub async fn revoke_key(
        pool: &PgPool,
        user_id: Uuid,
        key_id: Uuid,
    ) -> Result<(), ProxyKeyError> {
        let result = sqlx::query(
            r#"
            UPDATE proxy_api_keys
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(key_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ProxyKeyError::NotFound);
        }

        Ok(())
    }

    /// Validate a proxy API key and return user_id if valid
    /// Requirement: 7.1, 7.2
    pub async fn validate_key(
        pool: &PgPool,
        key: &str,
    ) -> Result<(Uuid, Uuid), ProxyKeyError> {
        // Key must start with prefix
        if !key.starts_with(PROXY_KEY_PREFIX) {
            return Err(ProxyKeyError::NotFound);
        }

        // Get all active keys and check against hash
        let keys: Vec<ProxyApiKey> = sqlx::query_as(
            r#"
            SELECT id, user_id, key_hash, key_prefix, name, is_active, last_used_at, request_count, created_at, updated_at
            FROM proxy_api_keys
            WHERE is_active = true
            "#,
        )
        .fetch_all(pool)
        .await?;

        for proxy_key in keys {
            if verify_password(key, &proxy_key.key_hash).unwrap_or(false) {
                // Update last_used_at and increment request_count
                sqlx::query(
                    "UPDATE proxy_api_keys SET last_used_at = NOW(), request_count = request_count + 1 WHERE id = $1",
                )
                .bind(proxy_key.id)
                .execute(pool)
                .await?;

                return Ok((proxy_key.id, proxy_key.user_id));
            }
        }

        Err(ProxyKeyError::NotFound)
    }

    /// Get key count for a user
    pub async fn get_key_count(pool: &PgPool, user_id: Uuid) -> Result<i64, ProxyKeyError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM proxy_api_keys WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}

/// Generate a proxy key (for testing)
pub fn generate_proxy_key_string() -> String {
    let mut key_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key_bytes);
    let key_base64 = URL_SAFE_NO_PAD.encode(key_bytes);
    format!("{}{}", PROXY_KEY_PREFIX, key_base64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Property Test 5: Proxy Key Format Invariant
    // Validates: Requirements 6.1 - Keys always start with "wbr_" prefix
    #[test]
    fn test_proxy_key_format() {
        let key = generate_proxy_key_string();
        assert!(key.starts_with(PROXY_KEY_PREFIX));
        // Key should be wbr_ + 43 chars (32 bytes base64 URL-safe no padding)
        assert_eq!(key.len(), 4 + 43);
    }

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(10))]
        #[test]
        fn prop_proxy_key_always_has_prefix(_seed in 0u64..1000) {
            let key = generate_proxy_key_string();
            prop_assert!(key.starts_with(PROXY_KEY_PREFIX));
            prop_assert!(key.len() > 40); // Reasonable length
        }
    }

    #[test]
    fn test_proxy_key_uniqueness() {
        let key1 = generate_proxy_key_string();
        let key2 = generate_proxy_key_string();
        assert_ne!(key1, key2);
    }

    // Property Test: Generated keys are unique
    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(5))]
        #[test]
        fn prop_proxy_keys_unique(_seed in 0u64..100) {
            let key1 = generate_proxy_key_string();
            let key2 = generate_proxy_key_string();
            prop_assert_ne!(key1, key2);
        }
    }
}
