-- Migration: Create proxy_requests table (usage logging)
-- Requirements: Usage logs for analytics

CREATE TABLE proxy_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    proxy_key_id UUID REFERENCES proxy_api_keys(id) ON DELETE SET NULL,
    -- Request details
    provider ai_provider NOT NULL,
    model VARCHAR(100) NOT NULL,
    -- Token usage
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    -- Performance
    latency_ms INTEGER NOT NULL,
    -- Cost tracking (in IDR, stored as integer cents)
    estimated_cost_idr BIGINT NOT NULL DEFAULT 0,
    -- Status
    status_code INTEGER NOT NULL,
    error_message TEXT,
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for user usage queries
CREATE INDEX idx_proxy_requests_user_id ON proxy_requests(user_id);

-- Index for time-based analytics
CREATE INDEX idx_proxy_requests_created_at ON proxy_requests(created_at);

-- Index for provider analytics
CREATE INDEX idx_proxy_requests_provider ON proxy_requests(provider, created_at);

-- Composite index for user dashboard queries
CREATE INDEX idx_proxy_requests_user_time ON proxy_requests(user_id, created_at DESC);

-- Partitioning hint: Consider partitioning by month for large datasets
COMMENT ON TABLE proxy_requests IS 'Usage logs for proxy requests. Consider partitioning by created_at for scale.';
