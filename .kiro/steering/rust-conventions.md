---
inclusion: fileMatch
fileMatchPattern: "**/*.rs"
---

# Rust Code Conventions (Webrana Backend)

> When writing or reviewing Rust code, adopt the mindset of a **Senior Rust Developer** enforcing idiomatic patterns and safety.

## Rust Version & Toolchain

- **Toolchain**: Rust Nightly (rustlang/rust:nightly-bookworm)
- **Edition**: 2021
- **All commands via Docker**: `docker compose run --rm backend-dev cargo <command>`

## Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files/Modules | snake_case | `api_key_service.rs` |
| Structs | PascalCase | `ApiKeyService` |
| Traits | PascalCase | `Encryptable` |
| Enums | PascalCase | `Provider::OpenAI` |
| Functions | snake_case | `encrypt_api_key` |
| Variables | snake_case | `encrypted_key` |
| Constants | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| Type aliases | PascalCase | `Result<T> = std::result::Result<T, AppError>` |

## Module Organization

```rust
// ✅ CORRECT: Explicit module structure
mod routes;
mod services;
mod models;
mod middleware;
mod utils;

// In each module's mod.rs, re-export public items
pub use api_key_service::ApiKeyService;
pub use proxy_service::ProxyService;
```

## Error Handling

```rust
// ✅ CORRECT: Custom error type with thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("API key not found: {0}")]
    ApiKeyNotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

// Map to HTTP status codes for Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::ApiKeyNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### Error Rules
- Use `thiserror` for custom error types
- Use `anyhow` for error propagation in application code
- Always implement `IntoResponse` for API errors
- Never use `.unwrap()` in production code (use `?` or `.expect("reason")`)
- Log errors with `tracing::error!` before returning

## Async Patterns

```rust
// ✅ CORRECT: Async handler with proper error handling
pub async fn create_api_key(
    State(state): State<AppState>,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<Json<ApiKeyResponse>, AppError> {
    let key = state.api_key_service
        .create(&input)
        .await?;
    
    Ok(Json(ApiKeyResponse::from(key)))
}

// ❌ WRONG: Blocking in async context
pub async fn bad_handler() {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKS!
    std::fs::read_to_string("file.txt"); // BLOCKS!
}

// ✅ CORRECT: Use tokio equivalents
pub async fn good_handler() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    tokio::fs::read_to_string("file.txt").await;
}
```

## Database Patterns (SQLx)

```rust
// ✅ CORRECT: Compile-time checked query with query_as!
let user = sqlx::query_as!(
    User,
    r#"
    SELECT id, email, password_hash, plan, created_at, updated_at
    FROM users
    WHERE id = $1
    "#,
    user_id
)
.fetch_optional(&pool)
.await?;

// ❌ WRONG: String interpolation (SQL injection risk!)
let query = format!("SELECT * FROM users WHERE id = '{}'", user_id);
```

### Database Rules
- Always use `query!` or `query_as!` macros
- Use `fetch_one`, `fetch_optional`, `fetch_all` appropriately
- Wrap multiple operations in transactions
- Use UUIDs for primary keys (`uuid::Uuid`)

## Encryption Patterns

```rust
// ✅ CORRECT: AES-256-GCM with unique IV
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use rand::Rng;

pub fn encrypt_api_key(plaintext: &str, master_key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12]), AppError> {
    let key = Key::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    
    // Generate unique 12-byte IV
    let mut iv = [0u8; 12];
    rand::thread_rng().fill(&mut iv);
    let nonce = Nonce::from_slice(&iv);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Encryption(e.to_string()))?;
    
    Ok((ciphertext, iv))
}
```

## Logging with Tracing

```rust
use tracing::{info, warn, error, instrument};

// ✅ CORRECT: Structured logging with spans
#[instrument(skip(pool, master_key), fields(user_id = %user_id))]
pub async fn create_api_key(
    pool: &PgPool,
    user_id: Uuid,
    master_key: &[u8; 32],
) -> Result<ApiKey, AppError> {
    info!("Creating API key for user");
    
    // ... implementation
    
    if let Err(e) = result {
        error!(error = %e, "Failed to create API key");
        return Err(e);
    }
    
    info!("API key created successfully");
    Ok(result)
}
```

## Testing Patterns

```rust
// Unit test
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let master_key = [0u8; 32]; // Test key
        let plaintext = "sk-test-key-12345";
        
        let (ciphertext, iv) = encrypt_api_key(plaintext, &master_key).unwrap();
        let decrypted = decrypt_api_key(&ciphertext, &iv, &master_key).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}

// Property-based test with proptest
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encryption_roundtrip(plaintext in "[a-zA-Z0-9]{1,100}") {
        let master_key = [0u8; 32];
        let (ciphertext, iv) = encrypt_api_key(&plaintext, &master_key).unwrap();
        let decrypted = decrypt_api_key(&ciphertext, &iv, &master_key).unwrap();
        prop_assert_eq!(plaintext, decrypted);
    }
}
```

### Testing Rules
- Use `#[tokio::test]` for async tests
- Use `proptest` for property-based testing
- Tag property tests with requirement references: `// **Validates: Requirements X.Y**`
- Run tests via Docker: `docker compose run --rm backend-dev cargo test`

## Code Review Checklist

Before approving Rust code:

- [ ] No `.unwrap()` in production code
- [ ] All async functions use `?` for error propagation
- [ ] SQLx queries use compile-time macros (`query!`, `query_as!`)
- [ ] Encryption uses AES-256-GCM with unique IV per operation
- [ ] No blocking I/O in async contexts
- [ ] Structured logging with `tracing`
- [ ] Unit tests exist for core logic
- [ ] Property tests exist for correctness properties
- [ ] No hardcoded secrets (use environment variables)
