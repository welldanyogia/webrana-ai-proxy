-- Migration: Create proxy_api_keys table
-- Requirements: 9.3 - User's proxy API keys (Argon2id hashed)

CREATE TABLE proxy_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Argon2id hashed key (never store plaintext)
    key_hash VARCHAR(255) NOT NULL,
    -- First 8 chars for identification (wbr_xxxx)
    key_prefix VARCHAR(12) NOT NULL,
    -- User-friendly name
    name VARCHAR(100) NOT NULL,
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Usage tracking
    last_used_at TIMESTAMP WITH TIME ZONE,
    request_count BIGINT NOT NULL DEFAULT 0,
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT unique_user_key_name UNIQUE (user_id, name)
);

-- Index for key validation (hash lookup)
CREATE INDEX idx_proxy_api_keys_prefix ON proxy_api_keys(key_prefix) WHERE is_active = true;

-- Index for user's proxy keys
CREATE INDEX idx_proxy_api_keys_user_id ON proxy_api_keys(user_id);

-- Trigger for updated_at
CREATE TRIGGER update_proxy_api_keys_updated_at
    BEFORE UPDATE ON proxy_api_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
