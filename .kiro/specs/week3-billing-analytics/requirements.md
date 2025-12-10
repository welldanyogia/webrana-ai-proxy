# Requirements Document - Week 3: Analytics + Billing + Polish

## Introduction

Week 3 completes the MVP with usage analytics dashboard, Midtrans payment integration, subscription management, and admin panel. Security audit and load testing are conducted to ensure production readiness.

**Sprint Duration**: Dec 23-29, 2024
**Goal**: Complete MVP features + Testing + Launch prep

## Glossary

| Term | Definition |
|------|------------|
| Webrana_Billing | The component handling subscription lifecycle and payment processing via Midtrans |
| Midtrans | Indonesian payment gateway supporting QRIS, Virtual Account, and Credit/Debit Card |
| QRIS | Quick Response Code Indonesian Standard - unified QR payment (GoPay, OVO, Dana, etc.) |
| Virtual_Account | Bank transfer method with auto-generated account number for each transaction |
| PPN | Pajak Pertambahan Nilai - Indonesian VAT at 11% |
| Usage_Dashboard | Frontend component displaying analytics charts and usage statistics |

## Requirements

### Requirement 1: Usage Analytics Dashboard

**User Story:** As a user, I want to view my API usage analytics, so that I can track costs and optimize my usage.

#### Acceptance Criteria

1. WHEN a user accesses `/dashboard/usage`, THE Usage_Dashboard SHALL display usage charts for the selected time period (daily/weekly/monthly)
2. THE Usage_Dashboard SHALL show: total requests, total tokens (input/output), total cost (Rp), average latency (ms)
3. THE Usage_Dashboard SHALL display a breakdown by provider (pie chart) and by model (bar chart)
4. THE Usage_Dashboard SHALL allow filtering by date range (last 7 days, 30 days, custom range)
5. THE Usage_Dashboard SHALL provide CSV export of usage logs with columns: timestamp, provider, model, tokens, cost, latency
6. THE Usage_Dashboard SHALL update data within 5 seconds of new requests (near real-time)

---

### Requirement 2: Midtrans Payment Integration

**User Story:** As a user, I want to pay for my subscription using Indonesian payment methods, so that I can upgrade my plan easily.

#### Acceptance Criteria

1. WHEN a user selects a paid plan, THE Webrana_Billing SHALL create a Midtrans Snap transaction with the plan amount plus 11% PPN
2. THE Webrana_Billing SHALL support payment methods: QRIS (0.7% fee), Virtual Account (Rp 4,000 flat fee), Credit/Debit Card (2.9% + Rp 2,000 fee)
3. THE Webrana_Billing SHALL redirect the user to Midtrans payment page with correct amount in Rupiah
4. WHEN Midtrans sends a webhook notification, THE Webrana_Billing SHALL verify the signature using MIDTRANS_SERVER_KEY
5. IF payment signature verification fails, THEN THE Webrana_Billing SHALL reject the webhook and log a security alert
6. THE Webrana_Billing SHALL store transaction details: order_id, payment_type, gross_amount, transaction_status, transaction_time

---

### Requirement 3: Subscription Lifecycle Management

**User Story:** As a user, I want to manage my subscription, so that I can upgrade, downgrade, or cancel my plan.

#### Acceptance Criteria

1. WHEN a payment is confirmed (status: "settlement" or "capture"), THE Webrana_Billing SHALL activate the subscription for 30 days
2. THE Webrana_Billing SHALL send email notification on: subscription activated, 7 days before expiry, subscription expired
3. WHEN a subscription expires, THE Webrana_Backend SHALL downgrade the user to Free tier and enforce Free tier limits
4. WHEN a user requests plan upgrade, THE Webrana_Billing SHALL calculate prorated amount for remaining days
5. WHEN a user requests cancellation, THE Webrana_Billing SHALL allow access until current period ends (no refund)
6. THE Webrana_Billing SHALL store subscription history: user_id, plan_tier, start_date, end_date, status, payment_id

---

### Requirement 4: Invoice Generation

**User Story:** As a user, I want to receive invoices for my payments, so that I can track expenses and claim reimbursements.

#### Acceptance Criteria

1. WHEN a payment is confirmed, THE Webrana_Billing SHALL generate a PDF invoice within 60 seconds
2. THE Invoice SHALL include: invoice number (WEB-YYYY-MM-XXX), company details, customer details, line items, subtotal, PPN (11%), total
3. THE Invoice SHALL display amounts in Indonesian Rupiah with proper formatting (Rp X.XXX.XXX)
4. THE Invoice SHALL include payment method used and Midtrans transaction ID
5. WHEN an invoice is generated, THE Webrana_Billing SHALL send it via email to the user
6. THE User SHALL be able to download past invoices from `/dashboard/billing`

---

### Requirement 5: Rate Limiting Enforcement

**User Story:** As the system, I need to enforce rate limits per plan tier, so that resources are fairly distributed.

#### Acceptance Criteria

1. THE Webrana_Proxy SHALL enforce monthly request limits: Free (1,000), Starter (10,000), Pro (50,000), Team (200,000)
2. THE Webrana_Proxy SHALL track request counts in Redis with key pattern `rate:{user_id}:{month}`
3. WHEN a user exceeds 80% of their monthly limit, THE Webrana_Backend SHALL send an email warning
4. IF a user exceeds their monthly limit, THEN THE Webrana_Proxy SHALL return HTTP 429 Too Many Requests with message "Monthly request limit exceeded. Upgrade your plan."
5. THE Webrana_Proxy SHALL enforce per-minute rate limit of 60 requests for all tiers (burst protection)
6. THE Rate_Limiter SHALL reset monthly counters on the 1st of each month at 00:00 UTC+7

---

### Requirement 6: Admin Dashboard

**User Story:** As an admin, I want to view platform statistics and manage users, so that I can monitor the service health.

#### Acceptance Criteria

1. WHEN an admin accesses `/admin`, THE Webrana_Frontend SHALL display platform-wide statistics: total users, active subscriptions, MRR, total requests today
2. THE Admin_Dashboard SHALL display a list of all users with: email, plan, signup date, last active, total requests
3. THE Admin_Dashboard SHALL allow searching users by email
4. THE Admin_Dashboard SHALL allow admins to: suspend user, change user plan, view user's usage details
5. IF a non-admin user accesses `/admin`, THEN THE Webrana_Frontend SHALL return HTTP 403 Forbidden
6. THE Admin_Dashboard SHALL display system health: API latency (p50, p95, p99), error rate, active connections

---

### Requirement 7: Email Notifications

**User Story:** As a user, I want to receive email notifications, so that I stay informed about my account status.

#### Acceptance Criteria

1. THE Webrana_Backend SHALL send emails via SendGrid/Resend API with proper SPF/DKIM configuration
2. THE Email_Service SHALL send notifications for: welcome (signup), payment success, payment failed, quota warning (80%), quota exceeded, subscription expiring (7 days)
3. THE Email_Templates SHALL be bilingual (ID/EN) based on user's language preference
4. THE Email_Service SHALL include unsubscribe link in all marketing emails (not transactional)
5. IF email delivery fails, THEN THE Email_Service SHALL retry 3 times with exponential backoff (1min, 5min, 30min)
6. THE Email_Service SHALL log all sent emails with: recipient, template, status, timestamp

---

### Requirement 8: Security Audit Compliance

**User Story:** As a security-conscious user, I want assurance that my API keys are secure, so that I can trust the service.

#### Acceptance Criteria

1. THE Webrana_Backend SHALL pass OWASP Top 10 security checks (no SQL injection, XSS, CSRF vulnerabilities)
2. THE Webrana_Backend SHALL enforce HTTPS for all endpoints with TLS 1.2+ minimum
3. THE Webrana_Backend SHALL implement Content Security Policy (CSP) headers
4. THE Webrana_Backend SHALL log all authentication events (login, logout, failed attempts) for audit trail
5. THE Webrana_Backend SHALL comply with Indonesian PP 71/2019 data protection requirements (data stored in Singapore, consent for data processing)
6. THE Security_Audit SHALL verify no PII (email, API keys) appears in application logs

---

### Requirement 9: Load Testing

**User Story:** As a DevOps engineer, I want to verify the system handles expected load, so that we can launch with confidence.

#### Acceptance Criteria

1. THE Load_Test SHALL simulate 100 concurrent users making proxy requests for 10 minutes
2. THE System SHALL maintain p95 latency below 500ms under load (excluding provider response time)
3. THE System SHALL maintain 0% error rate (5xx responses) under normal load
4. THE System SHALL handle 1,000 requests per minute without degradation
5. THE Load_Test SHALL verify Redis rate limiting works correctly under concurrent access
6. THE Load_Test results SHALL be documented with graphs showing latency distribution and throughput
