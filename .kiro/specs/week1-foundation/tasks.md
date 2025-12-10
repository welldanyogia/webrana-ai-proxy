# Implementation Plan - Week 1: Foundation

## Setup & Infrastructure

- [x] 1. Initialize monorepo structure and Rust backend
  - [x] 1.1 Create directory structure: `backend/`, `frontend/`, `infrastructure/`, `docs/`
    - Follow structure.md conventions
    - _Requirements: 8.1_
  - [x] 1.2 Initialize Cargo workspace with dependencies
    - Add: axum, tokio, sqlx, redis, aes-gcm, argon2, jsonwebtoken, serde, uuid
    - _Requirements: 8.1_
  - [x] 1.3 Create basic Axum HTTP server on port 3000
    - Health check endpoint at `/health`
    - _Requirements: 8.1_
  - [x] 1.4 Write unit test for health check endpoint
    - Health check endpoint exists and returns "OK"
    - _Requirements: 8.1_

- [x] 2. Database setup and migrations
  - [x] 2.1 Create SQLx migrations for users table
    - Columns: id, email, password_hash, plan_tier, created_at, updated_at
    - _Requirements: 9.1_
  - [x] 2.2 Create SQLx migrations for api_keys table
    - Columns: id, user_id, provider, encrypted_key, iv, auth_tag, key_name, timestamps
    - _Requirements: 9.2_
  - [x] 2.3 Create SQLx migrations for proxy_api_keys table
    - Columns: id, user_id, key_hash, key_prefix, name, is_active, timestamps
    - _Requirements: 9.3_
  - [x] 2.4 Add indexes and foreign key constraints
    - _Requirements: 9.4, 9.5_

## Core Utilities

- [x] 3. Implement encryption utilities (AES-256-GCM)
  - [x] 3.1 Create EncryptionUtils struct with encrypt/decrypt methods
    - Use aes-gcm crate with 256-bit key
    - Generate unique 12-byte IV per encryption
    - _Requirements: 3.1, 3.2_
  - [x] 3.2 Write property test: Encryption round-trip consistency
    - **Property 1: Encryption Round-Trip Consistency**
    - **Validates: Requirements 4.5**
  - [x] 3.3 Write property test: Unique IV per encryption
    - **Property 8: Unique IV Per Encryption**
    - **Validates: Requirements 3.1**
  - [x] 3.4 Implement master key loading from environment variable
    - _Requirements: 3.3_

- [x] 4. Implement password hashing utilities (Argon2id)
  - [x] 4.1 Create hash_password and verify_password functions
    - Use argon2 crate with Argon2id variant
    - _Requirements: 1.2, 2.2_
  - [x] 4.2 Write property test: Password hashing security
    - **Property 2: Password Hashing Security**
    - **Validates: Requirements 1.2, 6.2**

- [x] 5. Checkpoint - Ensure all tests pass
  - All 15 tests passed (8 encryption + 7 password tests)

## Authentication Module

- [x] 6. Implement user registration
  - [x] 6.1 Create POST /auth/register endpoint
    - Validate email format and password requirements
    - Hash password with Argon2id before storage
    - Return user ID on success
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - [x] 6.2 Write property test: New user default plan



    - **Property 6: New User Default Plan**
    - **Validates: Requirements 1.5**

  - [x] 6.3 Write unit tests for registration validation


    - Test invalid email, weak password, duplicate email
    - _Requirements: 1.3, 1.4_

- [x] 7. Implement user login and JWT tokens
  - [x] 7.1 Create POST /auth/login endpoint
    - Verify credentials against Argon2id hash
    - Return JWT access token (24h) and refresh token (7d)
    - Implement 200ms delay on failed attempts
    - _Requirements: 2.1, 2.2, 2.3_
  - [x] 7.2 Create POST /auth/refresh endpoint
    - Validate refresh token and issue new access token
    - _Requirements: 2.5_
  - [x] 7.3 Write property test: Authentication round-trip



    - **Property 3: Authentication Round-Trip**
    - **Validates: Requirements 2.2, 7.2**
  - [x] 7.4 Implement rate limiting for login attempts
    - Block after 5 failed attempts for 30 minutes
    - Use Redis for tracking
    - _Requirements: 2.4_

- [x] 8. Implement auth middleware
  - [x] 8.1 Create JWT validation middleware for protected routes
    - Extract and verify Bearer token from Authorization header
    - Attach user claims to request context
    - _Requirements: 7.1, 7.3, 7.4_
  - [x] 8.2 Write unit tests for middleware




    - Test valid token, expired token, missing token
    - _Requirements: 7.3, 7.4_

- [x] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## API Key Management

- [x] 10. Implement provider API key storage
  - [x] 10.1 Create POST /api-keys/provider endpoint
    - Validate key format per provider (OpenAI: sk-*, Anthropic: sk-ant-*)
    - Encrypt key with AES-256-GCM before storage
    - _Requirements: 3.1, 3.2, 3.6_
  - [x] 10.2 Create GET /api-keys/provider endpoint
    - Return masked keys only (e.g., "sk-...abc123")
    - _Requirements: 3.4_
  - [x] 10.3 Create DELETE /api-keys/provider/:id endpoint
    - Remove encrypted key from database
    - _Requirements: 3.1_
  - [x] 10.4 Write property test: Sensitive data masking
    - Added tests in api_key_service.rs: test_mask_key_short, test_mask_key_openai, test_mask_key_anthropic, test_mask_key_never_reveals_full_key
    - **Property 4: Sensitive Data Masking**
    - **Validates: Requirements 3.4, 6.4**

- [x] 11. Implement proxy API key generation
  - [x] 11.1 Create POST /api-keys/proxy endpoint
    - Generate 32-byte cryptographically secure random key with "wbr_" prefix
    - Hash with Argon2id before storage
    - Return plaintext key exactly once
    - _Requirements: 6.1, 6.2, 6.3_
  - [x] 11.2 Create GET /api-keys/proxy endpoint
    - Return key prefix and metadata only
    - _Requirements: 6.4_
  - [x] 11.3 Create DELETE /api-keys/proxy/:id endpoint
    - Revoke key (set is_active = false)
    - _Requirements: 6.1_
  - [x] 11.4 Write property test: Proxy key format invariant
    - Added tests in proxy_key_service.rs: test_proxy_key_format, prop_proxy_key_always_has_prefix, test_proxy_key_uniqueness, prop_proxy_keys_unique
    - **Property 5: Proxy Key Format Invariant**
    - **Validates: Requirements 6.1**
  - [x] 11.5 Implement API key limit enforcement per plan
    - Free: 1, Starter: 5, Pro/Team: unlimited
    - _Requirements: 6.5_

- [x] 12. Implement proxy API key validation
  - [x] 12.1 Create middleware to validate proxy API keys
    - Hash provided key and compare against stored hashes
    - Associate request with user account
    - _Requirements: 7.1, 7.2, 7.5_
  - [x] 12.2 Write unit tests for key validation

    - Added 8 tests in auth.rs: test_api_key_user_struct, test_api_key_extraction_valid, test_api_key_extraction_invalid_prefix, test_api_key_missing_bearer, test_api_key_empty_after_bearer, test_api_key_format_validation, test_auth_error_api_key_required, test_auth_error_invalid_api_key
    - Test valid key, invalid key, revoked key
    - _Requirements: 7.3_

- [x] 13. Checkpoint - Ensure all tests pass

  - Ensure all tests pass, ask the user if questions arise.

## OpenAI Proxy

- [x] 14. Implement OpenAI proxy endpoint
  - [x] 14.1 Create POST /v1/chat/completions endpoint
    - Parse ChatCompletionRequest from body
    - Validate model is OpenAI model (gpt-4, gpt-3.5-turbo, etc.)
    - _Requirements: 5.1, 5.2_
  - [x] 14.2 Implement request forwarding to OpenAI
    - Decrypt user's OpenAI API key
    - Add Authorization header with decrypted key
    - Forward request to api.openai.com
    - _Requirements: 5.3_
  - [x] 14.3 Implement response handling
    - Forward OpenAI response with original status code
    - Handle error responses appropriately
    - _Requirements: 5.4, 5.5_
  - [x] 14.4 Write property test: Proxy request integrity
    - Added tests in proxy.rs: test_is_openai_model_gpt4, test_is_openai_model_gpt35, test_is_openai_model_o1, test_is_not_openai_model, test_request_model_validation
    - **Property 7: Proxy Request Integrity**
    - **Validates: Requirements 5.2, 5.3, 5.4**
  - [x] 14.5 Handle missing API key error
    - Return HTTP 400 if OpenAI key not configured
    - _Requirements: 5.6_

- [x] 15. Checkpoint - Ensure all tests pass

  - Ensure all tests pass, ask the user if questions arise.

## Infrastructure & DevOps

- [x] 16. Create Dockerfile for backend
  - [x] 16.1 Write multi-stage Dockerfile
    - Build stage with rust:1.75
    - Runtime stage with debian:bookworm-slim
    - _Requirements: 8.1_
  - [x] 16.2 Create docker-compose.yml for local development
    - Services: backend, postgres, redis
    - _Requirements: 8.2, 8.3_

- [x] 17. Create Kubernetes manifests
  - [x] 17.1 Create Deployment manifest for backend
    - 2 replicas, resource limits, health probes
    - _Requirements: 8.1_
  - [x] 17.2 Create Service and Ingress manifests
    - Expose at api.webrana.id
    - _Requirements: 8.6_
  - [x] 17.3 Create ConfigMap and Secret manifests
    - Created `infrastructure/k8s/configmap.yaml` with non-sensitive config (RUST_LOG, rate limits, JWT expiry, API URLs)
    - Created `infrastructure/k8s/secret.yaml` with sensitive config (DATABASE_URL, REDIS_URL, MASTER_ENCRYPTION_KEY, JWT_SECRET)
    - _Requirements: 8.1_

- [x] 18. Setup CI/CD pipeline
  - [x] 18.1 Create GitHub Actions workflow
    - Build, test, and deploy on push to main
    - _Requirements: 8.5_
  - [x] 18.2 Configure staging deployment
    - Deploy to staging.webrana.id
    - _Requirements: 8.5_

- [x] 19. Final Checkpoint - Ensure all tests pass
  - ✅ All 69 tests passed (Dec 9, 2025)
