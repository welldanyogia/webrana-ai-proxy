-- Migration: Create onboarding_progress table
-- Week 4: Launch - User Onboarding Tracking
-- Requirements: 5.6 - Track onboarding completion rate for each step

-- Onboarding progress tracking
CREATE TABLE IF NOT EXISTS onboarding_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    api_key_added_at TIMESTAMP WITH TIME ZONE,
    first_request_at TIMESTAMP WITH TIME ZONE,
    dashboard_viewed_at TIMESTAMP WITH TIME ZONE,
    reminder_sent_at TIMESTAMP WITH TIME ZONE,
    completion_percent SMALLINT NOT NULL DEFAULT 25,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for user lookup
CREATE INDEX IF NOT EXISTS idx_onboarding_user_id ON onboarding_progress(user_id);

-- Index for finding incomplete onboarding (for reminder emails)
CREATE INDEX IF NOT EXISTS idx_onboarding_incomplete ON onboarding_progress(api_key_added_at) 
    WHERE api_key_added_at IS NULL;

-- Index for analytics queries
CREATE INDEX IF NOT EXISTS idx_onboarding_created ON onboarding_progress(created_at);

-- Trigger to auto-create onboarding record when user is created
CREATE OR REPLACE FUNCTION create_onboarding_progress()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO onboarding_progress (user_id, account_created_at)
    VALUES (NEW.id, NOW())
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_create_onboarding ON users;
CREATE TRIGGER trigger_create_onboarding
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_onboarding_progress();

-- Comment for documentation
COMMENT ON TABLE onboarding_progress IS 'Tracks user onboarding progress through key milestones';
COMMENT ON COLUMN onboarding_progress.completion_percent IS 'Calculated: 25% per step (account, api_key, first_request, dashboard)';
