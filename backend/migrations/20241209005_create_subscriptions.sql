-- Migration: Create subscriptions table
-- Requirements: Midtrans subscription lifecycle

CREATE TYPE subscription_status AS ENUM ('active', 'pending', 'cancelled', 'expired', 'past_due');

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Midtrans integration
    midtrans_order_id VARCHAR(100) UNIQUE,
    midtrans_transaction_id VARCHAR(100),
    -- Plan details
    plan_tier plan_tier NOT NULL,
    price_idr BIGINT NOT NULL,  -- Price in IDR
    -- Status
    status subscription_status NOT NULL DEFAULT 'pending',
    -- Billing period
    current_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    current_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    -- Cancellation
    cancelled_at TIMESTAMP WITH TIME ZONE,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for user subscriptions
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);

-- Index for active subscriptions
CREATE INDEX idx_subscriptions_status ON subscriptions(status) WHERE status = 'active';

-- Index for Midtrans webhook lookups
CREATE INDEX idx_subscriptions_midtrans_order ON subscriptions(midtrans_order_id);

-- Trigger for updated_at
CREATE TRIGGER update_subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
