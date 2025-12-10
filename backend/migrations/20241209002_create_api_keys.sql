-- Migration: Create api_keys table (provider API keys)
-- Requirements: 9.2 - Encrypted provider API keys (AES-256-GCM)

CREATE TYPE ai_provider AS ENUM ('openai', 'anthropic', 'google', 'qwen');

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider ai_provider NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    -- AES-256-GCM encrypted data
    encrypted_key BYTEA NOT NULL,
    iv BYTEA NOT NULL,  -- 12 bytes for GCM
    auth_tag BYTEA NOT NULL,  -- 16 bytes authentication tag
    -- Metadata
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- Constraints
    CONSTRAINT unique_user_provider_name UNIQUE (user_id, provider, key_name)
);

-- Index for user's API keys lookup
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);

-- Index for active keys per provider
CREATE INDEX idx_api_keys_user_provider ON api_keys(user_id, provider) WHERE is_active = true;

-- Trigger for updated_at
CREATE TRIGGER update_api_keys_updated_at
    BEFORE UPDATE ON api_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
