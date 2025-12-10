-- Migration: Create invoices table
-- Requirements: 4.1, 4.2, 4.3 - Invoice generation

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES subscriptions(id),
    -- Invoice details
    invoice_number VARCHAR(50) UNIQUE NOT NULL,  -- Format: WEB-YYYY-MM-XXX
    -- Amounts in IDR
    subtotal_idr BIGINT NOT NULL,
    ppn_idr BIGINT NOT NULL,  -- 11% VAT
    total_idr BIGINT NOT NULL,
    -- Payment info
    payment_method VARCHAR(50),
    midtrans_transaction_id VARCHAR(100),
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    paid_at TIMESTAMP WITH TIME ZONE,
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for user invoices
CREATE INDEX idx_invoices_user_id ON invoices(user_id);

-- Index for invoice number lookups
CREATE INDEX idx_invoices_number ON invoices(invoice_number);

-- Sequence for invoice numbers per month
CREATE SEQUENCE invoice_number_seq START 1;
