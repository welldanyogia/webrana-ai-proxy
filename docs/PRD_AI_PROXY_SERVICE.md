# PRD: AI Proxy Service (Standalone SaaS)
**Document Version**: 1.2
**Created**: 2024-12-09
**Last Updated**: 2024-12-09
**Owner**: COMPASS
**Status**: REQUIREMENTS APPROVED
**Pricing**: ✅ FINALIZED (Rupiah-based)
**Infrastructure**: ✅ FINALIZED (Self-hosted K8s)

---

## Executive Summary

Build a **hosted AI Proxy SaaS** that enables developers to access multiple AI models (GPT, Claude, Gemini, Qwen) through their existing subscriptions via a unified API endpoint—without self-hosting or managing OAuth flows.

**Positioning**: "CLIProxyAPI as a Service + Team Collaboration + Smart Routing"

---

## Problem Statement

### Current Pain Points
1. **Fragmentation**: Developers with ChatGPT Plus, Claude Pro, and Gemini subscriptions must manage separate APIs/CLIs
2. **OAuth Complexity**: Setting up CLIProxyAPI requires technical expertise (Docker, config files, port management)
3. **No Analytics**: Existing solutions lack visibility into usage, costs, and performance across models
4. **Single-user limitation**: Hard to share access across teams with budget controls

### Market Validation
- **CLIProxyAPI** has proven demand (GitHub stars, active community)
- **Factory Droid** charges premium for multi-model access ($50/mo Max plan)
- **Cursor/Windsurf** locked to single providers—users want flexibility

---

## Proposed Solution

### Product Overview
A **cloud-hosted AI proxy service** with:
- Zero-setup OAuth integration (login once per provider)
- Unified OpenAI/Anthropic-compatible API endpoints
- Real-time analytics dashboard
- Smart routing with auto-fallbacks
- Team collaboration with budget controls

### User Flow
```
1. User signs up → Create account
2. Connect AI providers → OAuth flow (Google/OpenAI/Anthropic)
3. Get API endpoint → https://api.yourservice.com/v1
4. Configure CLI/IDE → Point to your endpoint with your API key
5. Start using → All requests routed through proxy
```

### Differentiation vs CLIProxyAPI
| Feature | CLIProxyAPI | Our Service |
|---------|-------------|-------------|
| Deployment | Self-hosted (Docker) | ✅ Hosted SaaS |
| Setup Time | 30+ mins (technical) | ✅ 2 mins (OAuth only) |
| Analytics | ❌ None | ✅ Real-time dashboard |
| Team Features | ❌ Single-user | ✅ Multi-user with roles |
| Smart Routing | Basic fallbacks | ✅ Cost/performance-based |
| Pricing | Free (BYOK) | ✅ Rp 49K-299K/bln (Indonesia market) |
| Target Market | Global (tech-savvy) | ✅ Indonesia-first (ease of use) |
| Payment | N/A | ✅ Rupiah, local payment (Midtrans/Xendit) |

---

## Target Audience

### Primary Persona: "Pragmatic Indie Developer"
- **Profile**: Freelance/startup developer with multiple AI subscriptions
- **Pain**: Juggling 3+ AI tools, wants unified workflow
- **Budget**: $10-30/mo acceptable if saves time
- **Tech Level**: Comfortable with APIs, prefers simplicity

### Secondary Persona: "Small Dev Team Lead"
- **Profile**: CTO/Lead of 3-10 person team
- **Pain**: Team shares 1-2 AI accounts, no visibility/control
- **Budget**: $50-200/mo for team features
- **Tech Level**: DevOps-aware, values security/compliance

---

## Success Metrics

### North Star Metric
**Monthly Active Proxy Requests** (target: 1M requests by Month 6)

### Supporting Metrics
- User Acquisition: 1000 signups (Month 3), 5000 (Month 6)
- Activation Rate: 60% connect ≥1 provider within 24h
- Revenue: $5K MRR (Month 3), $25K MRR (Month 6)
- NPS: ≥50 by Month 6

### Technical KPIs
- Proxy Latency: <100ms overhead vs direct API
- Uptime: 99.9% SLA
- OAuth Success Rate: >95%

---

## Core Features (MVP)

### Phase 1: Essential Proxy (Month 1-2)
**P0 - Must Have:**
- ✅ OAuth integration (Google, OpenAI, Anthropic, Alibaba)
- ✅ OpenAI-compatible API proxy (`/v1/chat/completions`)
- ✅ Multi-account load balancing (round-robin)
- ✅ API key management (user dashboard)
- ✅ Basic usage tracking (requests/tokens)

**P1 - Should Have:**
- ✅ Anthropic/Gemini-compatible endpoints
- ✅ Streaming support (SSE)
- ✅ Model mapping (unavailable → fallback)
- ✅ Rate limiting per user

### Phase 2: Analytics & Intelligence (Month 3)
**P0:**
- ✅ Real-time dashboard (requests, tokens, costs)
- ✅ Smart routing (cost-optimized, performance-optimized)
- ✅ Model availability monitoring
- ✅ Historical usage reports

**P1:**
- ✅ Cost estimation per request
- ✅ Provider health dashboard
- ✅ Custom routing rules

### Phase 3: Team & Enterprise (Month 4-6)
**P0:**
- ✅ Team workspace creation
- ✅ Multi-user access with roles (Admin/Member)
- ✅ Shared budget limits
- ✅ Usage attribution by team member

**P1:**
- ✅ Audit logs
- ✅ SSO (Google Workspace)
- ✅ Custom rate limits per member
- ✅ Invoice generation

---

## Frontend Requirements

### 1. Landing Page (Marketing Site)

**Purpose**: Convert visitors → signups

**Key Sections:**
- **Hero**:
  - Headline: "Akses Semua AI Model dengan 1 API Key"
  - Subheadline: "GPT, Claude, Gemini, Qwen - Mulai Rp 49K/bulan"
  - CTA: "Mulai Gratis" (signup) + "Lihat Demo"
  - Hero image/video: Terminal demo showing multi-model usage

- **Problem/Solution**:
  - Problem: "Punya subscription GPT + Claude tapi ribet manage API keys?"
  - Solution: "Hubungkan semua model dalam 2 menit. Gunakan dari CLI/IDE favorit Anda."

- **Pricing Table**:
  - Interactive (highlight recommended tier)
  - Currency toggle: IDR/USD
  - FAQ expandables

- **Features Showcase**:
  - Real-time analytics dashboard preview
  - Smart routing demo
  - Team collaboration screenshot
  - Code examples (curl, Python, Droid config)

- **Social Proof**:
  - Usage stats: "X developers, Y million requests served"
  - Testimonials (future)
  - Compatible tools: Droid, Cursor, Continue.dev logos

- **Footer**:
  - Links: Docs, API Reference, Blog, Status Page
  - Legal: Terms, Privacy, Contact

**Design Requirements:**
- Mobile-first responsive
- Dark mode support (developer audience)
- Fast load (<2s LCP)
- SEO optimized (meta tags, structured data)
- Indonesian + English language toggle

---

### 2. User Dashboard (Self-Service Portal)

**Purpose**: Manage accounts, monitor usage, configure settings

**Main Sections:**

**A. Overview (Home)**
- Current plan badge (Free/Starter/Pro/Team)
- Quick stats cards:
  - Requests today/this month (progress bar vs limit)
  - Active OAuth connections
  - Estimated cost savings vs separate subscriptions
- Recent activity feed (last 10 requests)
- Quick actions: "Add Provider", "View API Key", "Upgrade Plan"

**B. OAuth Connections**
- List of connected providers:
  ```
  [OpenAI] john.doe@gmail.com  ✅ Active  [Disconnect]
  [Anthropic] john@company.com ✅ Active  [Refresh Token]
  [Google AI] ---              ➕ Connect
  ```
- Add new provider button → OAuth flow
- Connection status indicators (active, expired, error)
- Account limits display: "2/5 accounts used (Starter)"

**C. API Keys**
- Primary API key display (masked, click to reveal)
- Copy button + regenerate button
- Multiple API keys (future): name, created date, last used
- Usage example snippets (curl, Python, JavaScript)

**D. Usage Analytics**
- Time range selector: 24h, 7d, 30d, custom
- Charts:
  - Requests over time (line chart)
  - Requests by provider (pie chart)
  - Requests by model (bar chart)
  - Token usage over time
- Exportable data (CSV, JSON)
- Detailed logs table:
  - Timestamp, Model, Tokens, Latency, Status, Cost estimate

**E. Billing**
- Current plan details
- Usage vs limits (visual progress bars)
- Upgrade/downgrade buttons
- Payment method management (Midtrans integration)
- Invoice history (downloadable PDFs)
- Estimated next bill

**F. Team (Team/Enterprise only)**
- Team members list (email, role, usage)
- Invite members (email input)
- Role management (Admin/Member)
- Per-member usage analytics
- Shared budget configuration

**G. Settings**
- Profile: Name, email, password change
- Notifications: Email alerts for quota limits, errors
- API settings: Rate limits, webhook URLs (future)
- Danger zone: Delete account

**Design Requirements:**
- Clean, minimal UI (shadcn/ui aesthetic)
- Real-time updates (WebSocket for live usage)
- Responsive (desktop primary, tablet/mobile secondary)
- Dark mode default
- Loading states, error handling, empty states

---

### 3. Admin Dashboard (Operations Panel)

**Purpose**: Monitor platform health, manage users, support operations

**Main Sections:**

**A. Platform Overview**
- KPI cards:
  - Total users (Free/Starter/Pro/Team breakdown)
  - MRR (Monthly Recurring Revenue)
  - Total requests (24h, 7d, 30d)
  - Active subscriptions
  - Churn rate
- Platform health:
  - API uptime (%)
  - Average latency (p50, p95, p99)
  - Error rate (%)
  - Provider status (OpenAI ✅, Anthropic ⚠️, etc.)
- Recent signups (list)
- Recent upgrades/downgrades

**B. User Management**
- Search users (email, ID)
- User list table:
  - Email, Plan, Status, Signup Date, Last Active, MRR
  - Actions: View Details, Impersonate (support), Suspend, Refund
- User detail view:
  - Profile info
  - OAuth connections
  - Usage stats
  - Billing history
  - Activity logs
  - Manual plan change (comp, extend trial)

**C. Provider Management**
- Provider status monitoring:
  - OpenAI: ✅ Healthy (avg latency: 850ms)
  - Anthropic: ⚠️ Degraded (5% error rate)
  - Google: ✅ Healthy
  - Alibaba: ❌ Down (investigating)
- OAuth flow testing tools
- Provider API key rotation (admin-level)
- Fallback routing configuration

**D. Financial**
- Revenue dashboard:
  - Daily/weekly/monthly MRR chart
  - Revenue by plan (breakdown)
  - Churn analysis
  - LTV estimates
- Payment transactions (Midtrans webhook logs)
- Refund management
- Failed payments follow-up

**E. Support**
- Recent support tickets (if integrated)
- User search for quick lookup
- Impersonation mode (view as user)
- Manual credit additions (comp users)
- Ban/suspend abusive users

**F. Analytics**
- Cohort analysis (retention)
- Conversion funnel (signup → paid)
- Feature usage (which providers most popular)
- Geographic distribution (if tracking)
- Custom SQL query runner (advanced)

**G. System Logs**
- Application logs (errors, warnings)
- Audit trail (admin actions)
- OAuth failures
- Rate limit hits
- Security events

**Design Requirements:**
- Desktop-only (no mobile needed)
- Data-dense tables (sortable, filterable)
- Real-time updates (critical metrics)
- Export capabilities (CSV, JSON)
- Role-based access (Super Admin, Support, Finance)

---

## Out of Scope (v1.0)

**Explicitly NOT building:**
- ❌ Custom model fine-tuning
- ❌ On-premise deployment (cloud-only initially)
- ❌ Native CLI tool (users bring their own: Droid, Cursor, etc.)
- ❌ IDE extensions (focus on API compatibility)
- ❌ AI model hosting (pure proxy, not inference)

---

## Technical Architecture

### High-Level Design
```
┌──────────────────┐     ┌──────────────────┐     ┌─────────────┐
│ Landing Page     │     │ User Dashboard   │     │ Admin Panel │
│ (Marketing)      │     │ (Self-service)   │     │ (Operations)│
└────────┬─────────┘     └────────┬─────────┘     └──────┬──────┘
         │                        │                       │
         └────────────────────────┼───────────────────────┘
                                  ▼
                         ┌──────────────────┐
                         │  Web API         │
                         │  (Rust/Axum)     │
                         └────────┬─────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌─────────────┐      ┌──────────────────┐      ┌─────────────┐
│ User's CLI  │─────▶│  Proxy API       │─────▶│ AI Provider │
│ (Droid/etc) │      │  (Rust/Axum)     │      │ (OpenAI/etc)│
└─────────────┘      └──────────┬───────┘      └─────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │ PostgreSQL       │
                       │ - Users/Teams    │
                       │ - OAuth tokens   │
                       │ - Usage logs     │
                       │ - Billing        │
                       └──────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │ Redis            │
                       │ - Rate limiting  │
                       │ - Caching        │
                       │ - Sessions       │
                       └──────────────────┘
```

### Tech Stack Proposal

**Backend:**
- **API Server**: Rust (Axum framework) - reuse Webrana expertise
- **Database**: PostgreSQL (users, sessions, billing) + Redis (caching, rate limiting)
- **Auth**: OAuth 2.0 (Google/OpenAI/Anthropic SDKs)
- **Monitoring**: Prometheus + Grafana

**Frontend:**
- **Framework**: Next.js 15+ (latest stable) with App Router - SSR for landing page, SPA for dashboards
- **React**: React 19+ (latest stable) - avoid CVE vulnerabilities
- **UI Library**: Tailwind CSS 4+ + shadcn/ui (modern, fast development)
- **Charts/Analytics**: Recharts (latest) or tremor (untuk usage graphs)
- **State Management**: Zustand (latest, lightweight)
- **API Client**: TanStack Query v5+ (React Query) - caching & optimistic updates
- **Authentication**: NextAuth.js v5 (Auth.js) - integrate dengan backend OAuth
- **Dependency Management**:
  - Automated Dependabot updates (GitHub)
  - Weekly security patches
  - Pin major versions, auto-update patches

**Infrastructure:**
- **Hosting**: Singapore (Vultr/DigitalOcean) - Phase 1
  - Backend: Docker + Kubernetes (auto-scaling)
  - Frontend: Self-hosted on K8s (Next.js standalone mode)
  - Deployment: Single K8s cluster, separate namespaces (frontend/backend/admin)
- **CDN**: Cloudflare (global caching, DDoS protection, static asset caching)
- **Domain**: .id atau .com (TBD)
- **SSL**: Let's Encrypt (auto-renewal via cert-manager)
- **CI/CD**: GitHub Actions → Docker build → K8s rolling update

### Security Requirements
- ✅ Encrypted OAuth token storage (AES-256)
- ✅ API key rotation
- ✅ HTTPS-only (TLS 1.3)
- ✅ Rate limiting (per user, per endpoint)
- ✅ DDoS protection (Cloudflare)
- ✅ Audit logging (all proxy requests)
- ✅ Dependency security:
  - Latest stable versions (Next.js 15+, React 19+)
  - Automated vulnerability scanning (Dependabot, Snyk)
  - Weekly security patch updates
  - SemVer pinning (^major.x.x for controlled updates)
- ✅ CORS policy (strict origin validation)
- ✅ CSP headers (Content Security Policy)
- ✅ Input validation & sanitization (prevent XSS, SQL injection)

---

## Business Model

### Pricing Tiers (Indonesian Market)
| Tier | Price | Requests/Month | OAuth Accounts | Providers | Features |
|------|-------|----------------|----------------|-----------|----------|
| **Free** | Rp 0 | 1,000 | **1 account** | 1 provider | 1 AI provider (pilih: GPT/Claude/Gemini/Qwen), basic analytics |
| **Starter** | Rp 49.000/bln | 10,000 | **5 accounts** | **Max 2 providers** | Flexible: 5 accounts OpenAI ATAU 2+3 mix (e.g., 2 Claude + 3 GPT), basic smart routing, email support |
| **Pro** | Rp 99.000/bln | 50,000 | Unlimited | All providers | All 4 providers (GPT+Claude+Gemini+Qwen), unlimited accounts, advanced routing, priority support |
| **Team** | Rp 299.000/bln | 200,000 | Unlimited | All providers | 10 users, team analytics, shared budgets, API limits per member, Slack integration |
| **Enterprise** | Custom | Unlimited | Unlimited | All providers | SSO, SLA, Indonesia data residency, dedicated support, audit logs, white-label option |

**Positioning Rationale:**
- **Free**: 1 account = testing & demos. Cukup untuk evaluate 1 provider.
- **Starter (Rp 49K)**: **Super flexible!** 5 accounts di max 2 providers. Bisa:
  - 5x akun OpenAI (untuk load balancing personal projects)
  - 3 Claude + 2 GPT (best of both worlds)
  - 2 Gemini + 3 Qwen (fokus open models)
  - Entry point terjangkau untuk serious developers
- **Pro (Rp 99K)**: Unlock ALL providers + unlimited accounts. 1/3 harga Claude Pro solo subscription!
- **Team (Rp 299K)**: Harga 1 Claude Pro, untuk 10 orang - ROI 10x untuk teams
- **Enterprise**: Custom (Rp 5-20 juta/bulan) - compliance, SLA, dedicated infra

**Upgrade Incentives:**
- Free → Starter: "Need more accounts or mix providers? 5 slots for Rp 49K"
- Starter → Pro: "Want Gemini+Qwen too? Unlock all providers + 5x requests (Rp 99K)"
- Pro → Team: "Share access - only Rp 30K/user untuk 10 people"

**Use Case Examples (Starter Tier):**
1. **Freelancer**: 3 akun GPT-4 (personal + 2 clients) + 2 Claude (backup)
2. **Startup Dev**: 5 akun Claude (team share, poor man's team plan)
3. **Experimenter**: 2 GPT + 2 Gemini + 1 Qwen (compare outputs)

### Revenue Projections (Conservative - IDR)
- **Month 3**: 500 free + 100 Starter + 30 Pro + 3 Team = Rp 8.870.000 MRR (~$565 USD)
- **Month 6**: 2,000 free + 300 Starter + 100 Pro + 10 Team = Rp 27.600.000 MRR (~$1,760 USD)
- **Month 12**: 5,000 free + 800 Starter + 300 Pro + 30 Team = Rp 78.900.000 MRR (~$5,025 USD)
- **+ Enterprise deals**: Potential +Rp 20-50 juta/month from 2-3 corporate clients

### Cost Structure

**Monthly Operating Costs:**
- **Infrastructure (Singapore)**: ~Rp 8.170.000/mo (~$520)
  - K8s cluster (3 nodes, 8GB RAM each): Rp 5.5M
    - Backend pods (Rust/Axum): 2 nodes
    - Frontend pods (Next.js): 1 node
  - Database (PostgreSQL managed): Rp 1.6M
  - Redis cache: Rp 800K
  - Bandwidth (150GB estimate, includes frontend): Rp 1.1M
  - Load balancer: Rp 470K
- **CDN (Cloudflare Free)**: Rp 0
  - Free tier: Unlimited bandwidth
  - Caching for static assets (_next/static/*)
- **Payment Gateway**: 2.9% + Rp 2,000 per transaksi (Midtrans/Xendit)
- **Email Service (Resend/SendGrid)**: Rp 160K/mo ($10 - 10K emails)
- **Monitoring (Grafana Cloud)**: Rp 0 (free tier)
- **Domain + SSL**: Rp 200K/year (amortized: Rp 17K/mo)

**Total Monthly**: ~Rp 8.35M/mo (~$532)

**Savings vs Vercel**: Rp 315K/mo saved ✅

**One-time Development Costs:**
- **UI/UX Design**: Rp 10-15M (freelance designer, 2-3 weeks)
- **Frontend Development**: Included in dev effort (FORGE or contract)
- **Legal (ToS, Privacy Policy)**: Rp 5-8M (Indonesian + English)

**Break-even Analysis**: ~215 paying users (mix of tiers) to cover infrastructure

**Technical Notes (Self-Hosted Frontend):**
- Next.js standalone output mode (minimal Docker image ~150MB vs ~1GB)
- Static assets served via Cloudflare CDN (cache hit ratio ~80%+)
- Server-side rendering on K8s pods (auto-scaling based on CPU/memory)
- Rolling updates with zero downtime (K8s deployment strategy)
- Horizontal pod autoscaling (HPA): 1-3 frontend pods based on traffic

---

## Go-to-Market Strategy

### Launch Phases

**Phase 1: Private Alpha (Week 1-4)**
- Target: 20 hand-picked users (existing Webrana community)
- Goal: Validate OAuth flows, gather feedback
- Channel: Direct outreach, Discord

**Phase 2: Public Beta (Month 2-3)**
- Target: 500 signups
- Goal: Stress-test infrastructure, refine UX
- Channel: ProductHunt, Reddit (r/webdev, r/LocalLLaMA), Twitter

**Phase 3: General Availability (Month 4+)**
- Target: 2,000+ users
- Goal: Achieve $5K MRR
- Channel: SEO (comparison pages), integrations (Droid marketplace?)

### Marketing Channels
1. **Content Marketing**: "How to use Claude + GPT in Droid without API keys"
2. **Community**: Discord, Reddit AMAs
3. **Partnerships**: Reach out to Droid/Factory.ai for official integration
4. **SEO**: "CLIProxyAPI alternative", "hosted AI proxy"

---

## Dependencies

### External Dependencies
- OAuth provider APIs (Google, OpenAI, Anthropic, Alibaba)
- Payment processing (Stripe)
- Email service (SendGrid/AWS SES)

### Internal Dependencies
- ❌ None (standalone product, no Webrana CLI integration)

### Technical Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| OAuth token expiration | HIGH | Auto-refresh mechanism + user notifications |
| Provider rate limits | MEDIUM | Multi-account pooling + smart queuing |
| DDoS attacks | HIGH | Cloudflare + rate limiting |
| Cost overruns | MEDIUM | Usage alerts + hard caps per tier |

---

## Acceptance Criteria

### MVP Launch Criteria (Phase 1)
**MUST HAVE (all):**
- [ ] User can sign up and connect ≥1 AI provider (Google/OpenAI/Anthropic)
- [ ] Proxy successfully forwards OpenAI-compatible requests to connected provider
- [ ] User receives unique API key and can authenticate requests
- [ ] Dashboard shows real-time request count and token usage
- [ ] System handles 100 concurrent requests without errors
- [ ] OAuth tokens auto-refresh before expiration
- [ ] API latency overhead <150ms (p95)

**GIVEN** a user has connected their Claude Pro account
**WHEN** they send a request to `/v1/chat/completions` with model `claude-sonnet-4`
**THEN** the system routes to Anthropic API using user's OAuth token
**AND** returns response in OpenAI-compatible format
**AND** logs usage to dashboard

---

## Open Questions

**CRITICAL (need answers before development):**
1. [✅] **Data Residency**: Start with Singapore servers (low latency, cost-effective), add Indonesia region as premium feature in Phase 3 for enterprise clients
2. [✅] **Legal Entity**: Already exists, ready for payment processing
3. [✅] **ToS Compliance**: Proceeding without formal legal review (user decision)

**IMPORTANT (can be answered during development):**
4. [ ] Should free tier users share a pool of proxy accounts or use their own OAuth?
5. [ ] What happens if user's subscription expires mid-request?
6. [ ] How do we handle model deprecations (e.g., GPT-4 → GPT-5 migration)?

---

## Timeline & Milestones

### Development Roadmap
| Milestone | Target Date | Deliverables |
|-----------|-------------|--------------|
| **M1: Architecture Design** | Week 1 | Tech stack finalized, DB schema, API spec, UI mockups |
| **M2: OAuth Integration (Backend)** | Week 3 | All 4 providers working in dev |
| **M3: Proxy Core** | Week 5 | Request routing, streaming, load balancing |
| **M4: Landing Page** | Week 6 | Marketing site live, SEO optimized |
| **M5: User Dashboard MVP** | Week 8 | Signup, OAuth connect, API keys, basic analytics |
| **M6: Admin Dashboard** | Week 9 | User management, platform monitoring |
| **M7: Billing Integration** | Week 10 | Midtrans, subscription management, invoicing |
| **M8: Private Alpha** | Week 11 | 20 users onboarded, feedback collected |
| **M9: Public Beta** | Week 14 | ProductHunt launch, 500 users target |

**CRITICAL PATH**: OAuth Integration → Proxy Core → User Dashboard → Billing → Launch

**Parallel Workstreams**:
- Backend (FORGE): OAuth + Proxy Core (Rust/Axum)
- Frontend (FORGE/External): Landing Page + Dashboards (Next.js 15+)
- Security (SENTINEL): Audit + hardening
- DevOps (ATLAS): K8s cluster setup (backend + frontend), CI/CD pipeline, monitoring

---

## Risks & Mitigations

### Business Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Low adoption (users prefer self-hosting) | MEDIUM | HIGH | Differentiate with hosted convenience + analytics |
| Provider ToS violations | LOW | CRITICAL | Legal review before launch, add ToS disclaimer |
| Competition from CLIProxyAPI adding SaaS | MEDIUM | MEDIUM | Move fast, build superior UX/features |

### Technical Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| OAuth refresh failures | MEDIUM | HIGH | Robust retry logic + user notifications |
| Scalability issues at 10K+ users | LOW | HIGH | Load testing, horizontal scaling |
| Provider API changes breaking proxy | MEDIUM | MEDIUM | Automated integration tests, version pinning |

---

## Next Steps (Immediate Actions)

**NEXUS to delegate:**
1. **SYNAPSE**: Design OAuth architecture + smart routing algorithms
2. **FORGE**: Build proxy core (Axum + PostgreSQL + Redis)
3. **SENTINEL**: Security audit of OAuth flow + credential storage
4. **ATLAS**: Infrastructure setup (K8s, monitoring, CI/CD)
5. **SCOUT**: Competitive deep-dive (CLIProxyAPI alternatives, pricing benchmarks)

**COMPASS (me) to do:**
- [ ] Legal review: Provider ToS compliance
- [ ] Create wireframes for dashboard
- [ ] Write user onboarding flow documentation
- [ ] Set up project tracking (Jira/Linear)

---

**Approval Required**: NEXUS + User
**Estimated Effort**: 8-12 weeks (2 full-time engineers)
**Investment**: ~$2K infrastructure + opportunity cost
**Potential ROI**: $10K+ MRR by Month 12

---

*Document prepared by COMPASS*
*Awaiting SCOUT competitive analysis + SYNAPSE architecture proposal*
