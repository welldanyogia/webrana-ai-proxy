-- Migration: Create users table
-- Requirements: 9.1 - User accounts with email, password_hash, plan

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE plan_tier AS ENUM ('free', 'starter', 'pro', 'team');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    plan_tier plan_tier NOT NULL DEFAULT 'free',
    is_active BOOLEAN NOT NULL DEFAULT true,
    email_verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for email lookups (login)
CREATE INDEX idx_users_email ON users(email);

-- Index for active users
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
