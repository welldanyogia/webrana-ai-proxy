//! API Key service for provider key encryption and proxy key management.
//!
//! Requirements: 3.1, 3.2, 3.4, 3.6 - Provider API key storage with AES-256-GCM encryption

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::api_key::{AiProvider, ApiKey, ApiKeyInfo, CreateApiKey};
use crate::utils::encryption::{EncryptedData, EncryptionError, EncryptionUtils};

/// API Key service error
#[derive(Debug)]
pub enum ApiKeyError {
    InvalidKeyFormat(String),
    EncryptionError(EncryptionError),
    DatabaseError(sqlx::Error),
    NotFound,
    Unauthorized,
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::InvalidKeyFormat(msg) => write!(f, "Invalid key format: {}", msg),
            ApiKeyError::EncryptionError(e) => write!(f, "Encryption error: {}", e),
            ApiKeyError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ApiKeyError::NotFound => write!(f, "API key not found"),
            ApiKeyError::Unauthorized => write!(f, "Unauthorized access to API key"),
        }
    }
}

impl std::error::Error for ApiKeyError {}

impl From<EncryptionError> for ApiKeyError {
    fn from(e: EncryptionError) -> Self {
        ApiKeyError::EncryptionError(e)
    }
}

impl From<sqlx::Error> for ApiKeyError {
    fn from(e: sqlx::Error) -> Self {
        ApiKeyError::DatabaseError(e)
    }
}

/// Stored provider API key result
#[derive(Debug)]
pub struct StoredApiKey {
    pub id: Uuid,
    pub provider: AiProvider,
    pub name: String,
    pub masked_key: String,
    pub created_at: DateTime<Utc>,
}

/// API Key service implementation
pub struct ApiKeyServiceImpl {
    encryption: EncryptionUtils,
}

impl ApiKeyServiceImpl {
    /// Create new API key service from environment
    pub fn from_env() -> Result<Self, EncryptionError> {
        let encryption = EncryptionUtils::from_env()?;
        Ok(Self { encryption })
    }

    /// Store a provider API key (encrypted)
    /// Requirements: 3.1, 3.2, 3.6
    pub async fn store_provider_key(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        input: CreateApiKey,
    ) -> Result<StoredApiKey, ApiKeyError> {
        // Validate key format per provider (Requirement 3.6)
        if !input.provider.validate_key_format(&input.key) {
            return Err(ApiKeyError::InvalidKeyFormat(format!(
                "Invalid {} API key format",
                format!("{:?}", input.provider).to_lowercase()
            )));
        }

        // Encrypt the API key (Requirements 3.1, 3.2)
        let encrypted = self.encryption.encrypt(&input.key)?;

        // Store in database
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO api_keys (id, user_id, provider, key_name, encrypted_key, iv, auth_tag, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(input.provider)
        .bind(&input.name)
        .bind(&encrypted.ciphertext)
        .bind(&encrypted.iv.to_vec())
        .bind(&encrypted.auth_tag.to_vec())
        .bind(now)
        .execute(pool)
        .await?;

        Ok(StoredApiKey {
            id,
            provider: input.provider,
            name: input.name,
            masked_key: ApiKeyInfo::mask_key(&input.key),
            created_at: now,
        })
    }


    /// List provider API keys for a user (masked)
    /// Requirement: 3.4
    pub async fn list_provider_keys(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ApiKeyInfo>, ApiKeyError> {
        let keys: Vec<ApiKey> = sqlx::query_as(
            r#"
            SELECT id, user_id, provider, key_name, encrypted_key, iv, auth_tag, is_active, last_used_at, created_at, updated_at
            FROM api_keys
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        // Decrypt keys to create masked versions
        let mut result = Vec::with_capacity(keys.len());
        for key in keys {
            let encrypted = EncryptedData {
                ciphertext: key.encrypted_key,
                iv: key.iv.try_into().unwrap_or([0u8; 12]),
                auth_tag: key.auth_tag.try_into().unwrap_or([0u8; 16]),
            };

            // Decrypt to get original key for masking
            let decrypted = self.encryption.decrypt(&encrypted)?;
            let masked_key = ApiKeyInfo::mask_key(&decrypted);

            result.push(ApiKeyInfo {
                id: key.id,
                provider: key.provider,
                name: key.key_name,
                masked_key,
                is_active: key.is_active,
                last_used_at: key.last_used_at,
                created_at: key.created_at,
            });
        }

        Ok(result)
    }

    /// Delete a provider API key
    /// Requirement: 3.1
    pub async fn delete_provider_key(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        key_id: Uuid,
    ) -> Result<(), ApiKeyError> {
        let result = sqlx::query(
            r#"
            DELETE FROM api_keys
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(key_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiKeyError::NotFound);
        }

        Ok(())
    }

    /// Get decrypted provider API key for proxy use
    /// Requirement: 4.1, 4.2
    pub async fn get_decrypted_key(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        provider: AiProvider,
    ) -> Result<String, ApiKeyError> {
        let key: Option<ApiKey> = sqlx::query_as(
            r#"
            SELECT id, user_id, provider, key_name, encrypted_key, iv, auth_tag, is_active, last_used_at, created_at, updated_at
            FROM api_keys
            WHERE user_id = $1 AND provider = $2 AND is_active = true
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .fetch_optional(pool)
        .await?;

        let key = key.ok_or(ApiKeyError::NotFound)?;

        let encrypted = EncryptedData {
            ciphertext: key.encrypted_key,
            iv: key.iv.try_into().unwrap_or([0u8; 12]),
            auth_tag: key.auth_tag.try_into().unwrap_or([0u8; 16]),
        };

        // Update last_used_at
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key.id)
            .execute(pool)
            .await?;

        Ok(self.encryption.decrypt(&encrypted)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Property Test 4: Sensitive Data Masking
    // Validates: Requirements 3.4, 6.4
    #[test]
    fn test_mask_key_short() {
        let masked = ApiKeyInfo::mask_key("short");
        assert_eq!(masked, "***");
    }

    #[test]
    fn test_mask_key_openai() {
        let masked = ApiKeyInfo::mask_key("sk-proj-abc123xyz789");
        assert!(masked.starts_with("sk-"));
        assert!(masked.contains("..."));
        assert!(masked.ends_with("xyz789"));
    }

    #[test]
    fn test_mask_key_anthropic() {
        let masked = ApiKeyInfo::mask_key("sk-ant-api03-abcdefghijklmnop");
        assert!(masked.starts_with("sk-"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn test_mask_key_never_reveals_full_key() {
        let key = "sk-proj-verylongapikeythatshouldbehidden123456";
        let masked = ApiKeyInfo::mask_key(key);
        // Masked key should be much shorter than original
        assert!(masked.len() < key.len());
        // Should not contain the middle part
        assert!(!masked.contains("verylongapikey"));
    }
}
