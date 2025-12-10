# Design Document - Week 3: Analytics + Billing + Polish

## Overview

Week 3 completes the MVP with usage analytics dashboard, Midtrans payment integration for Indonesian payment methods (QRIS, VA, Card), subscription lifecycle management, and admin panel. Security audit and load testing ensure production readiness.

## Architecture

```mermaid
graph TD
    subgraph "Frontend Dashboard"
        A[Usage Dashboard] --> B[Charts/Recharts]
        C[Billing Page] --> D[Plan Selection]
        E[Admin Panel] --> F[User Management]
    end
    
    subgraph "Backend Services"
        G[UsageService] --> H[(PostgreSQL)]
        I[BillingService] --> J[Midtrans API]
        K[RateLimiter] --> L[(Redis)]
        M[EmailService] --> N[SendGrid/Resend]
    end
    
    subgraph "Midtrans Flow"
        D -->|Create Transaction| I
        I -->|Snap Token| O[Midtrans Snap]
        O -->|Payment| P[User Pays]
        P -->|Webhook| Q[/webhook/midtrans]
        Q --> I
    end
    
    subgraph "Rate Limiting"
        R[Proxy Request] --> K
        K -->|Check Limit| L
        K -->|Allowed| S[Forward to Provider]
        K -->|Exceeded| T[HTTP 429]
    end
```

## Components and Interfaces

### Component 1: BillingService

**Purpose:** Handles Midtrans integration, subscription lifecycle, and invoice generation.

**Interface:**
```rust
pub trait BillingService {
    async fn create_subscription(&self, user_id: Uuid, plan: PlanTier) -> Result<MidtransSnapToken, BillingError>;
    async fn handle_webhook(&self, payload: MidtransWebhook) -> Result<(), BillingError>;
    async fn get_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>, BillingError>;
    async fn cancel_subscription(&self, user_id: Uuid) -> Result<(), BillingError>;
    async fn generate_invoice(&self, subscription_id: Uuid) -> Result<InvoicePdf, BillingError>;
}

pub struct MidtransSnapToken {
    pub token: String,
    pub redirect_url: String,
}

pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_tier: PlanTier,
    pub status: SubscriptionStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub payment_id: Option<String>,
}
```

### Component 2: UsageAnalyticsService

**Purpose:** Aggregates and queries usage data for dashboard display.

**Interface:**
```rust
pub trait UsageAnalyticsService {
    async fn get_usage_stats(&self, user_id: Uuid, period: DateRange) -> Result<UsageStats, AnalyticsError>;
    async fn get_usage_by_provider(&self, user_id: Uuid, period: DateRange) -> Result<Vec<ProviderUsage>, AnalyticsError>;
    async fn get_usage_by_model(&self, user_id: Uuid, period: DateRange) -> Result<Vec<ModelUsage>, AnalyticsError>;
    async fn export_csv(&self, user_id: Uuid, period: DateRange) -> Result<String, AnalyticsError>;
}

pub struct UsageStats {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_idr: i64,
    pub avg_latency_ms: f64,
}
```

### Component 3: RateLimiter

**Purpose:** Enforces request limits per plan tier using Redis.

**Interface:**
```rust
pub trait RateLimiter {
    async fn check_and_increment(&self, user_id: Uuid) -> Result<RateLimitResult, RateLimitError>;
    async fn get_usage(&self, user_id: Uuid) -> Result<RateLimitUsage, RateLimitError>;
    async fn reset_monthly(&self) -> Result<(), RateLimitError>;
}

pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: i64,
    pub limit: i64,
    pub reset_at: DateTime<Utc>,
}

pub struct RateLimitUsage {
    pub monthly_used: i64,
    pub monthly_limit: i64,
    pub minute_used: i64,
    pub minute_limit: i64,
}
```

### Component 4: EmailService

**Purpose:** Sends transactional emails via SendGrid/Resend.

**Interface:**
```rust
pub trait EmailService {
    async fn send_welcome(&self, user: &User) -> Result<(), EmailError>;
    async fn send_payment_success(&self, user: &User, invoice: &Invoice) -> Result<(), EmailError>;
    async fn send_payment_failed(&self, user: &User, reason: &str) -> Result<(), EmailError>;
    async fn send_quota_warning(&self, user: &User, usage_percent: u8) -> Result<(), EmailError>;
    async fn send_subscription_expiring(&self, user: &User, days_left: u8) -> Result<(), EmailError>;
}
```

## Data Models

### Entity: Subscription

```rust
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_tier: PlanTier,
    pub status: SubscriptionStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub payment_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
    PendingPayment,
}
```

### Entity: Invoice

```rust
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Uuid,
    pub invoice_number: String,  // WEB-2024-12-001
    pub subtotal_idr: i64,
    pub ppn_idr: i64,            // 11% VAT
    pub total_idr: i64,
    pub payment_method: String,
    pub midtrans_transaction_id: String,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
}
```

## Database Schema Addition

```sql
-- Subscriptions table
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_tier VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending_payment',
    start_date TIMESTAMP WITH TIME ZONE,
    end_date TIMESTAMP WITH TIME ZONE,
    payment_id VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);

-- Invoices table
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES subscriptions(id),
    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    subtotal_idr BIGINT NOT NULL,
    ppn_idr BIGINT NOT NULL,
    total_idr BIGINT NOT NULL,
    payment_method VARCHAR(50),
    midtrans_transaction_id VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_invoices_user_id ON invoices(user_id);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Usage Aggregation Correctness
*For any* date range and user, the aggregated usage stats (total_requests, total_tokens, total_cost) SHALL equal the sum of individual proxy_requests records in that range.
**Validates: Requirements 1.2, 1.3, 1.4**

### Property 2: Payment Amount Calculation
*For any* plan tier, the Midtrans transaction amount SHALL equal the plan price plus 11% PPN (rounded to nearest Rupiah).
**Validates: Requirements 2.1, 4.2**

### Property 3: Subscription Lifecycle Integrity
*For any* confirmed payment, the subscription end_date SHALL be exactly 30 days after start_date, AND expired subscriptions SHALL result in Free tier access.
**Validates: Requirements 3.1, 3.3**

### Property 4: Proration Calculation
*For any* mid-cycle upgrade, the prorated amount SHALL equal (new_price - old_price) * (remaining_days / 30).
**Validates: Requirements 3.4**

### Property 5: Rate Limiting Enforcement
*For any* user at or above their monthly limit, subsequent requests SHALL return HTTP 429, AND users below limit SHALL be allowed.
**Validates: Requirements 5.1, 5.4**

### Property 6: Webhook Signature Verification
*For any* Midtrans webhook, the signature SHALL be verified using SHA512(order_id + status_code + gross_amount + server_key), AND invalid signatures SHALL be rejected.
**Validates: Requirements 2.4**

### Property 7: Invoice Number Uniqueness
*For any* generated invoice, the invoice_number SHALL be unique and follow format WEB-YYYY-MM-XXX.
**Validates: Requirements 4.2**

### Property 8: CSV Export Completeness
*For any* CSV export, the output SHALL contain columns: timestamp, provider, model, input_tokens, output_tokens, cost_idr, latency_ms.
**Validates: Requirements 1.5**

## Error Handling

| Error Scenario | Response | HTTP Status |
|----------------|----------|-------------|
| Monthly limit exceeded | `{"error": "Monthly request limit exceeded. Upgrade your plan."}` | 429 |
| Per-minute limit exceeded | `{"error": "Rate limit exceeded. Try again in X seconds."}` | 429 |
| Invalid webhook signature | Log security alert, return 400 | 400 |
| Payment failed | `{"error": "Payment failed: {reason}"}` | 402 |
| Subscription not found | `{"error": "No active subscription"}` | 404 |
| Admin access denied | `{"error": "Admin access required"}` | 403 |

## Testing Strategy

### Unit Tests
- Test usage aggregation queries
- Test PPN calculation (11%)
- Test proration calculation
- Test rate limit logic
- Test webhook signature verification

### Property-Based Tests
- Use `proptest` crate for Rust
- Minimum 100 iterations per property
- Tag format: `**Feature: week3-billing-analytics, Property {number}: {property_text}**`

**Property tests to implement:**
1. Usage aggregation correctness (Property 1)
2. Payment amount calculation (Property 2)
3. Rate limiting enforcement (Property 5)
4. Webhook signature verification (Property 6)

### Integration Tests
- Test full payment flow with Midtrans sandbox
- Test subscription activation after webhook
- Test rate limiting with Redis
- Test email sending (mock SMTP)

### Load Tests
- 100 concurrent users for 10 minutes
- Target: p95 latency <500ms, 0% error rate
- Verify rate limiting under load
