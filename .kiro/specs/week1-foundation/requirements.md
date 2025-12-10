# Requirements Document - Week 1: Foundation

## Introduction

Week 1 focuses on building the core foundation of Webrana AI Proxy: backend infrastructure, authentication, API key encryption, and basic proxy functionality for OpenAI. This sprint establishes the technical groundwork for all subsequent features.

**Sprint Duration**: Dec 9-15, 2024
**Goal**: Backend core + DevOps automation + Designer hired

## Glossary

The following terms are used throughout this document:

| Term | Definition |
|------|------------|
| Webrana_Backend | The Rust/Axum API server handling authentication, API key management, and proxy requests |
| Webrana_Proxy | The component that forwards requests to AI providers (OpenAI, Anthropic, Google, Qwen) |
| Provider_API_Key | User's own API key for external AI providers (OpenAI, Anthropic, etc.) |
| Proxy_API_Key | API key issued by Webrana for users to authenticate with the proxy service |
| AES-256-GCM | Advanced Encryption Standard with 256-bit key and Galois/Counter Mode for authenticated encryption |
| IV | Initialization Vector - unique random value used per encryption operation |
| Argon2id | Password hashing algorithm combining Argon2i and Argon2d for resistance against side-channel and GPU attacks |
| JWT | JSON Web Token - compact, URL-safe token format for secure claims transmission |
| HTTP Status Codes | Standard response codes: 200 (OK), 400 (Bad Request), 401 (Unauthorized), 403 (Forbidden), 409 (Conflict), 500 (Internal Server Error) |

## Requirements

### Requirement 1: User Registration

**User Story:** As a developer, I want to create an account on Webrana, so that I can access the AI proxy service.

#### Acceptance Criteria

1. WHEN a user submits registration with valid email and password (minimum 8 characters, 1 uppercase, 1 number), THE Webrana_Backend SHALL create a new user account within 500ms
2. WHEN a user submits registration, THE Webrana_Backend SHALL hash the password using Argon2id before storage
3. IF a user submits registration with an email that already exists, THEN THE Webrana_Backend SHALL return HTTP 409 Conflict with error message "Email already registered"
4. IF a user submits registration with invalid email format, THEN THE Webrana_Backend SHALL return HTTP 400 Bad Request with validation errors
5. WHEN a user account is created, THE Webrana_Backend SHALL assign the "Free" plan tier by default

---

### Requirement 2: User Authentication

**User Story:** As a registered user, I want to log in to my account, so that I can manage my API keys and view usage.

#### Acceptance Criteria

1. WHEN a user submits valid email and password, THE Webrana_Backend SHALL return a JWT access token (24-hour expiry) and refresh token (7-day expiry) within 300ms
2. WHEN a user submits login credentials, THE Webrana_Backend SHALL verify the password against the Argon2id hash
3. IF a user submits invalid credentials, THEN THE Webrana_Backend SHALL return HTTP 401 Unauthorized after a 200ms delay (timing attack mitigation)
4. IF a user exceeds 5 failed login attempts within 15 minutes, THEN THE Webrana_Backend SHALL block further attempts for 30 minutes
5. WHEN a user provides a valid refresh token, THE Webrana_Backend SHALL issue a new access token without requiring re-authentication

---

### Requirement 3: Provider API Key Storage

**User Story:** As a user, I want to securely store my AI provider API keys, so that I can use them through the Webrana proxy.

#### Acceptance Criteria

1. WHEN a user submits a Provider_API_Key, THE Webrana_Backend SHALL encrypt the key using AES-256-GCM with a unique 12-byte IV before storage
2. THE Webrana_Backend SHALL store the encrypted Provider_API_Key, IV, and authentication tag in PostgreSQL
3. THE Webrana_Backend SHALL retrieve the master encryption key from environment variable MASTER_ENCRYPTION_KEY (not hardcoded)
4. WHEN a user requests to view stored API keys, THE Webrana_Backend SHALL return only masked versions (e.g., "sk-...abc123")
5. IF encryption fails due to invalid master key, THEN THE Webrana_Backend SHALL return HTTP 500 Internal Server Error and log the error (without exposing key material)
6. WHEN a Provider_API_Key is stored, THE Webrana_Backend SHALL validate the key format matches the provider's expected pattern (e.g., OpenAI keys start with "sk-")

---

### Requirement 4: Provider API Key Retrieval and Decryption

**User Story:** As the proxy service, I need to decrypt stored API keys, so that I can forward requests to AI providers.

#### Acceptance Criteria

1. WHEN the Webrana_Proxy needs to forward a request, THE Webrana_Backend SHALL decrypt the Provider_API_Key using AES-256-GCM within 10ms
2. THE Webrana_Backend SHALL verify the GCM authentication tag during decryption to detect tampering
3. IF decryption fails (invalid tag, corrupted data), THEN THE Webrana_Backend SHALL return HTTP 500 and invalidate the stored key
4. THE Webrana_Backend SHALL never log decrypted API keys or include them in error messages
5. WHEN printing or serializing a Provider_API_Key, THE Webrana_Backend SHALL produce the same key after decryption (round-trip consistency)

---

### Requirement 5: OpenAI Proxy Endpoint

**User Story:** As a developer, I want to send requests to OpenAI through Webrana's unified endpoint, so that I can use my stored API key without exposing it in my application.

#### Acceptance Criteria

1. WHEN a user sends a POST request to `/v1/chat/completions` with valid Proxy_API_Key, THE Webrana_Proxy SHALL forward the request to OpenAI's API within 100ms overhead (excluding OpenAI response time)
2. THE Webrana_Proxy SHALL transform the request to match OpenAI's expected format
3. THE Webrana_Proxy SHALL include the decrypted Provider_API_Key in the Authorization header to OpenAI
4. WHEN OpenAI returns a response, THE Webrana_Proxy SHALL forward the response to the user with original status code
5. IF OpenAI returns an error (4xx, 5xx), THEN THE Webrana_Proxy SHALL forward the error response with appropriate status code
6. IF the user's Provider_API_Key for OpenAI is not configured, THEN THE Webrana_Proxy SHALL return HTTP 400 with message "OpenAI API key not configured"

---

### Requirement 6: Proxy API Key Generation

**User Story:** As a user, I want to generate Proxy API keys, so that I can authenticate my applications with Webrana.

#### Acceptance Criteria

1. WHEN a user requests a new Proxy_API_Key, THE Webrana_Backend SHALL generate a cryptographically secure 32-byte random key prefixed with "wbr_"
2. THE Webrana_Backend SHALL hash the Proxy_API_Key using Argon2id before storage (only hash stored, not plaintext)
3. THE Webrana_Backend SHALL return the plaintext Proxy_API_Key exactly once at creation time
4. WHEN a user lists their Proxy_API_Keys, THE Webrana_Backend SHALL return only the key prefix and creation date (not the full key)
5. IF a user exceeds their plan's API key limit (Free: 1, Starter: 5), THEN THE Webrana_Backend SHALL return HTTP 403 with message "API key limit reached for your plan"

---

### Requirement 7: Request Authentication via Proxy API Key

**User Story:** As the proxy service, I need to authenticate incoming requests, so that only authorized users can access the proxy.

#### Acceptance Criteria

1. WHEN a request includes `Authorization: Bearer wbr_xxx` header, THE Webrana_Proxy SHALL validate the Proxy_API_Key within 50ms
2. THE Webrana_Proxy SHALL hash the provided key and compare against stored Argon2id hashes
3. IF the Proxy_API_Key is invalid or revoked, THEN THE Webrana_Proxy SHALL return HTTP 401 Unauthorized
4. IF the Authorization header is missing, THEN THE Webrana_Proxy SHALL return HTTP 401 with message "API key required"
5. WHEN a valid Proxy_API_Key is provided, THE Webrana_Proxy SHALL associate the request with the corresponding user account

---

### Requirement 8: Infrastructure Setup

**User Story:** As a DevOps engineer, I want automated infrastructure deployment, so that the team can deploy consistently across environments.

#### Acceptance Criteria

1. THE Infrastructure SHALL provision a DigitalOcean Kubernetes cluster with 3 nodes (2vCPU/4GB each) in Singapore region
2. THE Infrastructure SHALL deploy PostgreSQL 15+ as a managed database with SSL connections enforced
3. THE Infrastructure SHALL deploy Redis 7.x for rate limiting and session storage
4. THE Infrastructure SHALL configure Cloudflare as CDN with DDoS protection enabled
5. WHEN code is pushed to the main branch, THE CI/CD Pipeline SHALL automatically build, test, and deploy to staging within 10 minutes
6. THE Infrastructure SHALL expose the API at `api.webrana.id` with valid SSL certificate

---

### Requirement 9: Database Schema

**User Story:** As a backend developer, I need a well-structured database schema, so that I can store user data, API keys, and usage logs efficiently.

#### Acceptance Criteria

1. THE Webrana_Backend SHALL create a `users` table with columns: id (UUID), email (unique), password_hash, plan_tier, created_at, updated_at
2. THE Webrana_Backend SHALL create an `api_keys` table with columns: id (UUID), user_id (FK), provider, encrypted_key, iv, auth_tag, key_name, created_at, updated_at
3. THE Webrana_Backend SHALL create a `proxy_api_keys` table with columns: id (UUID), user_id (FK), key_hash, key_prefix, name, is_active, created_at, last_used_at
4. THE Webrana_Backend SHALL enforce foreign key constraints with ON DELETE CASCADE for user-related tables
5. THE Webrana_Backend SHALL create indexes on frequently queried columns (user_id, email, key_hash)
