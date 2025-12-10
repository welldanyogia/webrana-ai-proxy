-- Migration: Create analytics_events table
-- Week 4: Launch - Analytics Tracking
-- Requirements: 9.1 - Track user acquisition source, activation funnel, retention

-- Analytics events table
CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    source VARCHAR(50),  -- producthunt, organic, referral, direct
    session_id VARCHAR(100),
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for event type queries
CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);

-- Index for time-based queries
CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at);

-- Index for user-specific queries
CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events(user_id) WHERE user_id IS NOT NULL;

-- Index for source analysis
CREATE INDEX IF NOT EXISTS idx_analytics_events_source ON analytics_events(source) WHERE source IS NOT NULL;

-- Composite index for funnel analysis
CREATE INDEX IF NOT EXISTS idx_analytics_funnel ON analytics_events(event_type, created_at);

-- Comment for documentation
COMMENT ON TABLE analytics_events IS 'Tracks user events for acquisition, activation, and retention analytics';
COMMENT ON COLUMN analytics_events.event_type IS 'Event types: signup, api_key_added, first_request, upgrade, etc.';
COMMENT ON COLUMN analytics_events.source IS 'Acquisition source: producthunt, organic, referral, direct';
COMMENT ON COLUMN analytics_events.properties IS 'Additional event-specific data in JSON format';
