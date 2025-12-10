# AI Proxy Service - API Key MVP Scope
**Version**: 2.0 (APPROVED)
**Date**: 2024-12-09
**Status**: READY TO BUILD
**Timeline**: 3-4 weeks to launch

---

## SCOPE CHANGE: OAuth → API Key Mode

**Decision**: User approved **Option B (API Key MVP)** based on NEXUS recommendation.

**Rationale**:
- ✅ 3 weeks vs 14 weeks to market
- ✅ 71% cost reduction (Rp 95M vs Rp 335M)
- ✅ Low security risk vs CRITICAL
- ✅ Works with ALL providers (including Anthropic)
- ✅ Validate demand before heavy OAuth investment

---

## MVP FEATURE SET

### Core Value Proposition
**"Unified API for all AI models with analytics, team management, and Rupiah billing"**

**Target Users**: Indonesian developers with multiple AI subscriptions

**Differentiation vs CLIProxyAPI**:
1. Hosted SaaS (no Docker/self-hosting required)
2. Real-time usage analytics dashboard
3. Rupiah pricing + Midtrans payment
4. Team collaboration features
5. Indonesia-first (ID language, local support)

---

## FEATURES INCLUDED (MVP)

### 1. Backend API ✅

**API Key Management**:
- User creates account (email + password)
- Add API keys per provider (OpenAI, Anthropic, Google, Qwen)
- Keys encrypted at rest (AES-256-GCM)
- Key rotation (manual: user can regenerate)
- Key validation (test connection on save)

**Proxy Endpoints** (OpenAI-compatible):
```
POST /v1/chat/completions
POST /v1/embeddings
POST /v1/images/generations
```

**Request Routing**:
- Detect provider from model name
  - `gpt-4` → OpenAI
  - `claude-sonnet-4` → Anthropic
  - `gemini-2.0-flash` → Google
  - `qwen-max` → Alibaba
- Forward request to correct provider
- Transform request format (OpenAI → Anthropic/Google/Qwen)
- Transform response format (provider → OpenAI)
- Streaming support (SSE passthrough)

**Usage Tracking**:
- Log every request (timestamp, user, model, tokens, latency, status)
- Real-time counters (Redis)
- Usage attribution (per user, per plan)
- Cost estimation (based on provider pricing)

**Rate Limiting** (per plan tier):
- Free: 1,000 requests/month
- Starter: 10,000 requests/month
- Pro: 50,000 requests/month
- Team: 200,000 requests/month

**Billing Integration**:
- Midtrans Snap (redirect to payment page)
- Subscription lifecycle (create, upgrade, downgrade, cancel)
- Webhook handler (payment success/failure)
- Invoice generation (PDF)

---

### 2. User Dashboard ✅

**Authentication**:
- Signup (email + password)
- Login (email + password + 2FA optional in Phase 2)
- Password reset (email flow)
- Email verification

**Dashboard Sections**:

**A. Overview**:
- Current plan badge (Free/Starter/Pro/Team)
- Usage stats: Requests today/month (progress bar vs limit)
- API keys connected (4/4 providers)
- Recent activity (last 10 requests)
- Quick actions: Add API Key, View Proxy URL, Upgrade

**B. API Keys Management**:
```
┌─────────────────────────────────────────────────┐
│ OpenAI                                          │
│ sk-proj-************************************    │
│ Added: 2024-12-09 | Last used: 5 mins ago      │
│ [Test Connection] [Edit] [Delete]              │
├─────────────────────────────────────────────────┤
│ Anthropic                                       │
│ sk-ant-************************************     │
│ Added: 2024-12-09 | Last used: 1 hour ago      │
│ [Test Connection] [Edit] [Delete]              │
├─────────────────────────────────────────────────┤
│ Google AI                                       │
│ [+ Add API Key]                                 │
└─────────────────────────────────────────────────┘
```

- Add new key (provider select + paste key + label)
- Test connection (verify key works)
- Delete key (with confirmation)
- Key limit per tier:
  - Free: 1 provider
  - Starter: 5 keys, max 2 providers
  - Pro: Unlimited
  - Team: Unlimited

**C. Proxy Configuration**:
```
Your Proxy URL:
https://api.yourservice.com/v1

Your API Key:
sk-proxy-abc123xyz789************************ [Copy]

Usage Example (curl):
curl https://api.yourservice.com/v1/chat/completions \
  -H "Authorization: Bearer sk-proxy-abc123..." \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'

Droid CLI Config:
{
  "model": "claude-sonnet-4",
  "base_url": "https://api.yourservice.com",
  "api_key": "sk-proxy-abc123...",
  "provider": "anthropic"
}
```

**D. Usage Analytics**:
- Time range selector (24h, 7d, 30d, all time)
- Charts:
  - Requests over time (line chart)
  - Requests by provider (pie chart)
  - Requests by model (bar chart)
  - Token usage (line chart)
- Table view:
  - Timestamp, Model, Tokens, Latency, Status, Cost estimate
  - Sortable, filterable, searchable
  - Export to CSV

**E. Billing**:
- Current plan + next renewal date
- Usage vs limits (visual progress bars)
- Estimated next bill
- Upgrade/downgrade buttons
- Payment history (invoices downloadable as PDF)
- Payment method (Midtrans: credit card, e-wallet, bank transfer)

**F. Settings**:
- Profile (name, email, change password)
- Notifications (email alerts: quota 80%, quota 100%, payment failed)
- API settings (regenerate proxy API key)
- Danger zone (delete account)

---

### 3. Landing Page ✅

**Hero Section**:
```
Headline: "Akses Semua AI Model dengan 1 API Key"
Subheadline: "GPT, Claude, Gemini, Qwen - Analitik Real-time - Mulai Rp 49K/bulan"
CTA: [Mulai Gratis] [Lihat Demo]
Visual: Terminal screenshot showing unified API usage
```

**Problem/Solution**:
```
Problem: "Punya API key GPT-4, Claude, Gemini - ribet manage?"
Solution: "Hubungkan semua dalam 1 dashboard. Track usage, manage tim, bayar pakai Rupiah."
```

**Features Grid** (4 features):
1. **Unified API** - 1 endpoint untuk semua model
2. **Real-time Analytics** - Dashboard lengkap usage & cost
3. **Team Collaboration** - Share access dengan team (Team tier)
4. **Rupiah Billing** - Midtrans, bayar pakai GoPay/OVO/Bank

**Pricing Table** (Interactive):
```
┌─────────┬──────────┬─────────┬─────────┐
│  Free   │ Starter  │   Pro   │  Team   │
├─────────┼──────────┼─────────┼─────────┤
│  Rp 0   │ Rp 49K   │ Rp 99K  │ Rp 299K │
│         │   /bln   │  /bln   │   /bln  │
├─────────┼──────────┼─────────┼─────────┤
│ 1K req  │ 10K req  │ 50K req │ 200K req│
│ 1 key   │ 5 keys   │ Unlim   │ Unlim   │
│ 1 prov  │ 2 prov   │ All     │ All     │
│ Basic   │ Analytics│ Priority│ 10 users│
│         │          │ Support │ Analytics│
└─────────┴──────────┴─────────┴─────────┘
```

**FAQ** (Collapsible):
- Q: Apakah API key saya aman?
  A: Ya, kami encrypt dengan AES-256-GCM. Hanya Anda yang bisa akses.
- Q: Provider mana yang didukung?
  A: OpenAI (GPT), Anthropic (Claude), Google (Gemini), Alibaba (Qwen)
- Q: Apa bedanya dengan CLIProxyAPI?
  A: CLIProxyAPI self-hosted (ribet setup Docker). Kami hosted + analytics + billing Rupiah.
- Q: Bisakah saya cancel kapan saja?
  A: Ya, cancel kapan saja tanpa penalty.

**Footer**:
- Links: Dokumentasi, API Reference, Blog, Status Page
- Legal: Terms of Service, Privacy Policy, Kontak
- Social: Twitter, GitHub, Discord

**SEO Optimization**:
```html
<title>AI Proxy Indonesia - Unified API untuk GPT, Claude, Gemini | Rp 49K/bulan</title>
<meta name="description" content="Akses semua AI model (GPT-4, Claude, Gemini, Qwen) dengan 1 API key. Real-time analytics, team management, bayar pakai Rupiah. Mulai gratis!">
```

---

### 4. Admin Dashboard ✅ (Simplified for MVP)

**Platform Overview**:
- Total users (Free/Starter/Pro/Team breakdown)
- MRR (Monthly Recurring Revenue)
- Requests (24h, 7d, 30d)
- Active subscriptions
- Recent signups

**User Management**:
- Search users (email, ID)
- User list table (email, plan, status, signup date, last active)
- User details (profile, API keys count, usage, billing history)
- Actions: Suspend user, manual plan change (comp), view logs

**Provider Status**:
- OpenAI: ✅ Healthy (avg latency 850ms)
- Anthropic: ✅ Healthy
- Google: ✅ Healthy
- Alibaba: ⚠️ Degraded (test daily)

**Financial**:
- MRR chart (last 30 days)
- Revenue by plan
- Recent transactions (Midtrans webhooks)

**System Logs**:
- Application errors (last 100)
- Audit trail (admin actions)
- API key additions/deletions

---

## FEATURES DEFERRED TO PHASE 2

**NOT in MVP** (add later based on user feedback):
- ❌ OAuth integration (OpenAI, Google, Anthropic)
- ❌ Multi-account load balancing (users manage 1 key per provider)
- ❌ Smart routing (cost/performance-based)
- ❌ Team workspaces (multi-user access)
- ❌ SSO (Google Workspace)
- ❌ Audit logs (comprehensive)
- ❌ Webhooks (notify on quota limits)
- ❌ Custom rate limits per user
- ❌ White-label option
- ❌ Indonesia data residency (all in Singapore for MVP)

**Add ONLY if users request it.**

---

## TECHNICAL ARCHITECTURE (API Key Mode)

### Backend Stack
```
Rust (Axum 0.7)
├── API Routes
│   ├── /auth/* (signup, login, reset password)
│   ├── /api-keys/* (CRUD for user API keys)
│   ├── /v1/chat/completions (proxy endpoint)
│   ├── /usage (analytics queries)
│   └── /billing (Midtrans integration)
├── Services
│   ├── api_key_service.rs (encrypt/decrypt, validate)
│   ├── proxy_service.rs (route requests, transform formats)
│   ├── usage_tracker.rs (log requests, count tokens)
│   └── billing_service.rs (Midtrans webhooks)
└── Database Models
    ├── users
    ├── api_keys (encrypted)
    ├── proxy_requests (usage logs)
    └── subscriptions
```

### Database Schema (PostgreSQL)
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(50) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW(),
    email_verified BOOLEAN DEFAULT FALSE
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- openai, anthropic, google, alibaba
    label VARCHAR(100), -- "Personal GPT", "Work Claude", etc.
    encrypted_key TEXT NOT NULL, -- AES-256-GCM encrypted
    iv BYTEA NOT NULL, -- Initialization vector for encryption
    created_at TIMESTAMP DEFAULT NOW(),
    last_used_at TIMESTAMP,
    is_valid BOOLEAN DEFAULT TRUE,
    UNIQUE(user_id, provider, label)
);

CREATE TABLE proxy_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    key_prefix VARCHAR(20) NOT NULL, -- "sk-proxy-abc123"
    key_hash VARCHAR(255) NOT NULL, -- Argon2id hash
    created_at TIMESTAMP DEFAULT NOW(),
    last_used_at TIMESTAMP,
    UNIQUE(key_hash)
);

CREATE TABLE proxy_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    api_key_id UUID REFERENCES api_keys(id),
    model VARCHAR(100),
    provider VARCHAR(50),
    tokens_input INT,
    tokens_output INT,
    tokens_total INT,
    latency_ms INT,
    status_code INT,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
CREATE INDEX idx_requests_user_date ON proxy_requests(user_id, created_at DESC);
CREATE INDEX idx_requests_created ON proxy_requests(created_at DESC);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    plan VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'active', -- active, canceled, expired
    midtrans_subscription_id VARCHAR(255),
    current_period_start TIMESTAMP,
    current_period_end TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### Frontend Stack
```
Next.js 15 (App Router, standalone mode)
├── app/
│   ├── (marketing)/
│   │   ├── page.tsx (landing page)
│   │   ├── pricing/page.tsx
│   │   └── docs/page.tsx
│   ├── (dashboard)/
│   │   ├── layout.tsx (auth wrapper)
│   │   ├── overview/page.tsx
│   │   ├── api-keys/page.tsx
│   │   ├── usage/page.tsx
│   │   ├── billing/page.tsx
│   │   └── settings/page.tsx
│   ├── (admin)/
│   │   ├── admin/page.tsx (overview)
│   │   └── admin/users/page.tsx
│   └── api/
│       ├── auth/[...nextauth]/route.ts
│       └── live-updates/route.ts (SSE)
├── components/
│   ├── ui/ (shadcn/ui components)
│   ├── charts/ (Recharts wrappers)
│   └── forms/ (API key forms, etc.)
└── lib/
    ├── api-client.ts (TanStack Query)
    └── utils.ts
```

---

## API SPECIFICATION

### Authentication
```
POST /auth/signup
Body: { email, password }
Response: { user_id, email, token }

POST /auth/login
Body: { email, password }
Response: { token, user: { id, email, plan } }
```

### API Key Management
```
POST /api-keys
Headers: Authorization: Bearer {jwt_token}
Body: { provider: "openai", label: "Personal", key: "sk-..." }
Response: { id, provider, label, created_at, is_valid }

GET /api-keys
Response: [{ id, provider, label, last_used_at, is_valid }]

DELETE /api-keys/:id
Response: { success: true }

POST /api-keys/:id/test
Response: { valid: true, latency_ms: 850 }
```

### Proxy Endpoint (OpenAI-compatible)
```
POST /v1/chat/completions
Headers: Authorization: Bearer {proxy_api_key}
Body: {
  "model": "gpt-4" | "claude-sonnet-4" | "gemini-2.0-flash" | "qwen-max",
  "messages": [...],
  "stream": true | false
}
Response: (OpenAI format, regardless of provider)
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4",
  "choices": [...]
}
```

### Usage Analytics
```
GET /usage/stats?range=7d
Response: {
  "total_requests": 1250,
  "total_tokens": 500000,
  "by_provider": { "openai": 800, "anthropic": 450 },
  "by_model": { "gpt-4": 600, "claude-sonnet-4": 450, "gemini": 200 }
}

GET /usage/logs?limit=100&offset=0
Response: [
  {
    "id": "...",
    "model": "gpt-4",
    "tokens": 1500,
    "latency_ms": 850,
    "status": 200,
    "created_at": "2024-12-09T10:30:00Z"
  },
  ...
]
```

### Billing
```
POST /billing/subscribe
Body: { plan: "starter" | "pro" | "team" }
Response: { payment_url: "https://app.midtrans.com/snap/v2/..." }

POST /webhook/midtrans
Body: (Midtrans webhook payload)
Response: { success: true }
```

---

## SECURITY IMPLEMENTATION (API Key Mode)

### API Key Encryption
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use rand::Rng;

// Encrypt API key before storing in DB
fn encrypt_api_key(plaintext_key: &str, master_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let cipher = Aes256Gcm::new(Key::from_slice(master_key));
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext_key.as_bytes())
        .expect("encryption failed");

    (ciphertext, nonce_bytes.to_vec())
}

// Decrypt when proxying request
fn decrypt_api_key(ciphertext: &[u8], nonce: &[u8], master_key: &[u8]) -> String {
    let cipher = Aes256Gcm::new(Key::from_slice(master_key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .expect("decryption failed");

    String::from_utf8(plaintext).unwrap()
}
```

**Master Key Management**:
- Store in environment variable (K8s Secret)
- Rotate every 90 days
- Use AWS KMS in Phase 2

### Multi-Tenant Isolation
```rust
// PostgreSQL Row-Level Security (RLS)
CREATE POLICY user_api_keys_policy ON api_keys
    USING (user_id = current_user_id());

// Application-level filtering
async fn get_user_api_keys(user_id: Uuid, db: &PgPool) -> Vec<ApiKey> {
    sqlx::query_as!(
        ApiKey,
        "SELECT * FROM api_keys WHERE user_id = $1",
        user_id
    )
    .fetch_all(db)
    .await
    .unwrap()
}
```

### Proxy API Key Generation
```rust
use argon2::{Argon2, PasswordHasher};
use rand::distributions::Alphanumeric;
use rand::Rng;

fn generate_proxy_api_key() -> (String, String) {
    // Generate 256-bit random key
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();

    let full_key = format!("sk-proxy-{}", key);

    // Hash for storage (Argon2id)
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(full_key.as_bytes(), &salt)
        .unwrap()
        .to_string();

    (full_key, hash)
}
```

### Rate Limiting
```rust
use redis::AsyncCommands;

async fn check_rate_limit(
    user_id: Uuid,
    plan: &str,
    redis: &mut redis::aio::Connection
) -> Result<(), RateLimitError> {
    let key = format!("rate_limit:{}:monthly", user_id);
    let count: u64 = redis.incr(&key, 1).await?;

    if count == 1 {
        redis.expire(&key, 30 * 24 * 3600).await?; // 30 days
    }

    let limit = match plan {
        "free" => 1_000,
        "starter" => 10_000,
        "pro" => 50_000,
        "team" => 200_000,
        _ => 0,
    };

    if count > limit {
        return Err(RateLimitError::QuotaExceeded);
    }

    Ok(())
}
```

---

## SUCCESS METRICS (MVP)

**Week 4 Launch**:
- ✅ 100+ signups
- ✅ 20+ active users (added ≥1 API key)
- ✅ 5+ paying users (Rp 245K total MRR)
- ✅ <5 critical bugs
- ✅ 99% uptime

**Month 3**:
- ✅ 500 signups
- ✅ 100 paying users (Rp 8.8M MRR target)
- ✅ 60% activation rate
- ✅ <5% churn

**Key Decision Point (Month 3)**:
- IF users REQUEST OAuth → Build it in Phase 2
- IF users DON'T request OAuth → Saved $30K! ✅

---

## COST ESTIMATE (3 Months)

| Category | Month 1 | Month 2-3 | Total |
|----------|---------|-----------|-------|
| Infrastructure | Rp 2.27M | Rp 4.54M | Rp 6.81M |
| UI/UX Design | Rp 10M | - | Rp 10M |
| Security Audit | Rp 78.5M (~$5K) | - | Rp 78.5M |
| **TOTAL** | **Rp 90.77M** | **Rp 4.54M** | **Rp 95.31M** |

**Budget**: Rp 95.3M (~$6,069 USD) approved

---

## RISK MITIGATION

**Competitive Risk** (CLIProxyAPI adds hosted version):
- Mitigation: Speed to market (3 weeks), Indonesia focus, analytics

**User Adoption Risk** (slow growth):
- Mitigation: Aggressive marketing (ProductHunt, Reddit, blog posts, SEO)

**Security Risk** (API key leakage):
- Mitigation: Encryption, security audit, user education

---

## NEXT STEPS

See `/home/deploy/webrana-ai/docs/WEEK_1_SPRINT_PLAN.md`

**START DATE**: Week of 2024-12-09 (ASAP after approvals)

---

**Document Owner**: NEXUS
**Approved By**: User (2024-12-09)
**Status**: READY TO BUILD 🚀
