# Design Document - Week 1: Foundation

## Overview

Week 1 establishes the core backend infrastructure for Webrana AI Proxy using Rust/Axum. The system provides secure user authentication, encrypted API key storage (AES-256-GCM), and a proxy endpoint for OpenAI requests. The architecture follows Domain-Driven Design with clear separation between routes, services, and models.

## Architecture

```mermaid
graph TD
    subgraph "Client Layer"
        A[Developer App] -->|HTTPS| B[Cloudflare CDN]
    end
    
    subgraph "API Gateway"
        B --> C[Axum HTTP Server]
        C --> D[Auth Middleware]
        C --> E[Rate Limit Middleware]
    end
    
    subgraph "Route Handlers"
        D --> F[/auth/* routes]
        D --> G[/api-keys/* routes]
        D --> H[/v1/chat/completions]
    end
    
    subgraph "Services Layer"
        F --> I[AuthService]
        G --> J[ApiKeyService]
        H --> K[ProxyService]
        I --> L[EncryptionUtils]
        J --> L
    end
    
    subgraph "Data Layer"
        I --> M[(PostgreSQL)]
        J --> M
        K --> N[(Redis)]
    end
    
    subgraph "External"
        K -->|Forward Request| O[OpenAI API]
    end
```

## Components and Interfaces

### Component 1: AuthService

**Purpose:** Handles user registration, login, JWT token management, and login rate limiting.

**Interface:**
```rust
pub trait AuthService {
    async fn register(&self, email: String, password: String) -> Result<User, AuthError>;
    async fn login(&self, email: String, password: String) -> Result<TokenPair, AuthError>;
    async fn refresh_token(&self, refresh_token: String) -> Result<TokenPair, AuthError>;
    async fn verify_token(&self, token: String) -> Result<Claims, AuthError>;
    async fn check_login_rate_limit(&self, email: &str) -> Result<(), AuthError>;
    async fn record_failed_login(&self, email: &str) -> Result<(), AuthError>;
}

pub struct TokenPair {
    pub access_token: String,  // JWT, 24h expiry
    pub refresh_token: String, // JWT, 7d expiry
}
```

**Rate Limiting Logic:**
- Track failed login attempts per email in Redis
- Key format: `login_attempts:{email}`
- Window: 15 minutes (TTL on Redis key)
- Threshold: 5 failed attempts
- Lockout duration: 30 minutes
- On successful login: Clear the failed attempts counter

### Component 2: ApiKeyService

**Purpose:** Manages provider API key encryption/decryption and proxy API key generation.

**Interface:**
```rust
pub trait ApiKeyService {
    // Provider API Keys (user's OpenAI/Anthropic keys)
    async fn store_provider_key(&self, user_id: Uuid, provider: Provider, key: String) -> Result<ApiKeyId, ApiKeyError>;
    async fn get_provider_key(&self, user_id: Uuid, provider: Provider) -> Result<String, ApiKeyError>;
    async fn list_provider_keys(&self, user_id: Uuid) -> Result<Vec<MaskedApiKey>, ApiKeyError>;
    async fn delete_provider_key(&self, user_id: Uuid, key_id: Uuid) -> Result<(), ApiKeyError>;
    fn validate_key_format(&self, provider: Provider, key: &str) -> Result<(), ApiKeyError>;
    async fn invalidate_corrupted_key(&self, user_id: Uuid, key_id: Uuid) -> Result<(), ApiKeyError>;
    
    // Proxy API Keys (Webrana-issued keys)
    async fn generate_proxy_key(&self, user_id: Uuid, name: String) -> Result<ProxyKeyCreated, ApiKeyError>;
    async fn validate_proxy_key(&self, key: String) -> Result<UserId, ApiKeyError>;
    async fn list_proxy_keys(&self, user_id: Uuid) -> Result<Vec<ProxyKeyInfo>, ApiKeyError>;
    async fn revoke_proxy_key(&self, user_id: Uuid, key_id: Uuid) -> Result<(), ApiKeyError>;
}

pub struct MaskedApiKey {
    pub id: Uuid,
    pub provider: Provider,
    pub masked_key: String, // e.g., "sk-...abc123"
    pub created_at: DateTime<Utc>,
}
```

**Provider Key Format Validation:**
| Provider | Expected Pattern | Example |
|----------|------------------|---------|
| OpenAI | `sk-*` | `sk-abc123...` |
| Anthropic | `sk-ant-*` | `sk-ant-api03-...` |
| Google | `AIza*` | `AIzaSy...` |
| Qwen | `sk-*` | `sk-...` |

**Decryption Failure Handling:**
- If GCM authentication tag verification fails, the key is considered corrupted/tampered
- Call `invalidate_corrupted_key()` to mark the key as invalid in the database
- Return HTTP 500 to the user and log the incident (without exposing key material)
```

### Component 3: ProxyService

**Purpose:** Forwards requests to AI providers with proper authentication and response handling.

**Interface:**
```rust
pub trait ProxyService {
    async fn forward_chat_completion(
        &self,
        user_id: Uuid,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProxyError>;
}

pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}
```

### Component 4: EncryptionUtils

**Purpose:** Provides AES-256-GCM encryption/decryption for API keys.

**Interface:**
```rust
pub trait EncryptionUtils {
    fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError>;
    fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, EncryptionError>;
}

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: [u8; 12],      // 12-byte nonce for GCM
    pub auth_tag: [u8; 16], // 16-byte authentication tag
}
```

## Data Models

### Entity: User

```rust
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,  // Argon2id hash
    pub plan_tier: PlanTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PlanTier {
    Free,      // 1,000 requests/month, 1 API key
    Starter,   // 10,000 requests/month, 5 API keys
    Pro,       // 50,000 requests/month, unlimited
    Team,      // 200,000 requests/month, unlimited + 10 users
}
```

### Entity: ApiKey (Provider Keys)

```rust
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: Provider,
    pub encrypted_key: Vec<u8>,
    pub iv: Vec<u8>,
    pub auth_tag: Vec<u8>,
    pub key_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    Qwen,
}
```

### Entity: ProxyApiKey

```rust
pub struct ProxyApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,    // Argon2id hash
    pub key_prefix: String,  // "wbr_abc..." for display
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

## Database Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan_tier VARCHAR(20) NOT NULL DEFAULT 'free',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Provider API Keys (encrypted)
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(20) NOT NULL,
    encrypted_key BYTEA NOT NULL,
    iv BYTEA NOT NULL,
    auth_tag BYTEA NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, provider)
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);

-- Proxy API Keys (hashed)
CREATE TABLE proxy_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(20) NOT NULL,
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_proxy_api_keys_user_id ON proxy_api_keys(user_id);
CREATE INDEX idx_proxy_api_keys_key_hash ON proxy_api_keys(key_hash);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Encryption Round-Trip Consistency
*For any* valid API key string, encrypting then decrypting the key SHALL produce the original plaintext key.
**Validates: Requirements 4.5**

### Property 2: Password Hashing Security
*For any* password, the stored hash SHALL NOT equal the plaintext password, AND verifying the correct password against the hash SHALL succeed.
**Validates: Requirements 1.2, 6.2**

### Property 3: Authentication Round-Trip
*For any* registered user with valid credentials, logging in SHALL succeed AND the returned token SHALL be valid for authentication.
**Validates: Requirements 2.2, 7.2**

### Property 4: Sensitive Data Masking
*For any* stored API key (provider or proxy), listing keys SHALL return only masked/prefix versions, never the full key.
**Validates: Requirements 3.4, 6.4**

### Property 5: Proxy Key Format Invariant
*For any* generated proxy API key, the key SHALL be exactly 32 bytes of random data prefixed with "wbr_".
**Validates: Requirements 6.1**

### Property 6: New User Default Plan
*For any* newly registered user, the assigned plan tier SHALL be "Free".
**Validates: Requirements 1.5**

### Property 7: Proxy Request Integrity
*For any* valid proxy request, the forwarded request to OpenAI SHALL contain the correct decrypted API key in the Authorization header, AND the response status code SHALL match OpenAI's response.
**Validates: Requirements 5.2, 5.3, 5.4**

### Property 8: Unique IV Per Encryption
*For any* two encryption operations (even with the same plaintext), the generated IVs SHALL be different.
**Validates: Requirements 3.1**

## Error Handling

| Error Scenario | Response | HTTP Status |
|----------------|----------|-------------|
| Invalid email format | `{"error": "Invalid email format"}` | 400 |
| Email already registered | `{"error": "Email already registered"}` | 409 |
| Invalid credentials | `{"error": "Invalid credentials"}` | 401 |
| Rate limit exceeded (login) | `{"error": "Too many attempts. Try again in X minutes"}` | 429 |
| Invalid/expired token | `{"error": "Invalid or expired token"}` | 401 |
| API key not configured | `{"error": "{Provider} API key not configured"}` | 400 |
| API key limit reached | `{"error": "API key limit reached for your plan"}` | 403 |
| Encryption failure | `{"error": "Internal server error"}` (log details) | 500 |
| Decryption failure (corrupted/tampered) | `{"error": "Internal server error"}` (invalidate key, log details) | 500 |
| Invalid API key format | `{"error": "Invalid API key format for {provider}"}` | 400 |
| Login rate limit exceeded | `{"error": "Too many login attempts. Try again in X minutes"}` | 429 |
| Provider API error | Forward provider's error response | 4xx/5xx |

## Testing Strategy

### Unit Tests
- Test individual encryption/decryption functions
- Test password hashing and verification
- Test JWT token generation and validation
- Test API key format validation
- Target: 80% code coverage

### Property-Based Tests
- Use `proptest` crate for Rust property-based testing
- Minimum 100 iterations per property
- Tag format: `**Feature: week1-foundation, Property {number}: {property_text}**`

**Property tests to implement:**
1. Encryption round-trip (Property 1)
2. Password hashing security (Property 2)
3. Unique IV generation (Property 8)
4. Proxy key format (Property 5)
5. Sensitive data masking (Property 4)

### Integration Tests
- Test full registration → login → create API key → proxy request flow
- Test authentication middleware with valid/invalid tokens
- Test rate limiting behavior
- Test database constraints (unique email, foreign keys)
