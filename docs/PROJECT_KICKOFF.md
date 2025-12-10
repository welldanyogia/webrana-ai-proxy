# Webrana AI Proxy - Project Kickoff
**Company**: Webrana AI Proxy
**Project**: AI Proxy Service (API Key MVP)
**Launch Date**: Week 4 (January 6, 2025)
**Status**: ✅ APPROVED - READY TO START

---

## PROJECT SUMMARY

**What We're Building**:
Hosted AI Proxy SaaS yang memungkinkan developers mengakses multiple AI models (GPT, Claude, Gemini, Qwen) melalui unified API endpoint dengan analytics, team management, dan Rupiah billing.

**Target Market**: Indonesian developers dengan multiple AI subscriptions

**Timeline**: 3-4 weeks to launch
**Budget**: Rp 95.3M (~$6,069 USD)
**Expected Revenue (Month 3)**: Rp 8.8M MRR (100 paying users)

---

## KEY DECISIONS (FINALIZED)

### Product Scope
- ✅ **Architecture**: API Key mode (users provide own API keys)
- ✅ **Providers**: OpenAI, Anthropic, Google AI, Alibaba Qwen
- ✅ **Pricing**: Rp 0 / 49K / 99K / 299K / Custom (Rupiah)
- ✅ **Features**: Unified API + Analytics + Team Management + Billing

### Infrastructure
- ✅ **Cloud Provider**: DigitalOcean Kubernetes (Singapore)
- ✅ **Database**: PostgreSQL (managed) + Redis (K8s)
- ✅ **CDN**: Cloudflare (free tier)
- ✅ **Cost**: Rp 2.27M/month (~$144.50)

### Payment & Business
- ✅ **Company Name**: Webrana AI Proxy
- ✅ **Domain**: webrana.id
- ✅ **Payment Methods**:
  - QRIS (GoPay, OVO, Dana, LinkAja, ShopeePay)
  - Virtual Account (BCA, BNI, BRI, Mandiri, Permata)
  - Credit/Debit Card (Visa, Mastercard, JCB)
- ✅ **Payment Gateway**: Midtrans

### Launch
- ✅ **Target Date**: Week 4 (January 6, 2025)
- ✅ **Strategy**: ProductHunt launch + community beta

---

## TEAM & ROLES

| Role | Person | Commitment | Responsibilities |
|------|--------|------------|------------------|
| **Team Lead & Orchestrator** | NEXUS | Full-time | Coordination, unblocking, decision-making |
| **Backend Engineer** | FORGE | Full-time | Rust/Axum development, encryption, proxy logic |
| **DevOps Engineer** | ATLAS | Part-time (20h/week) | K8s, CI/CD, monitoring, infrastructure |
| **Product Manager** | COMPASS | Part-time (10h/week) | Requirements, designer hiring, documentation |
| **Security Advisor** | SENTINEL | On-call | Code review, security audit, encryption review |
| **Frontend Developer** | TBD | Contract/Internal | Next.js 15, landing page, dashboards |
| **UI/UX Designer** | TBD (hiring) | Contract | Brand, mockups, design system |

---

## 3-WEEK SPRINT BREAKDOWN

### Week 1: Foundation (Dec 9-15, 2024)
**Goal**: Backend core + DevOps automation + Designer hired

**Deliverables**:
- [] GitHub repo created
- [] DigitalOcean K8s cluster (staging)
- [] PostgreSQL + Redis deployed
- [] Authentication (signup, login)
- [] API key encryption (AES-256-GCM)
- [] Proxy endpoint (OpenAI only)
- [] CI/CD pipeline (GitHub Actions)
- [] UI/UX designer hired and onboarded

**Success Criteria**:
- Backend can accept API key, encrypt, store in DB
- Proxy forwards request to OpenAI, returns response
- Deployed to staging via CI/CD
- Designer has brand guidelines ready

---

### Week 2: Multi-Provider + Frontend (Dec 16-22, 2024)
**Goal**: All providers working + Landing page + Dashboard skeleton

**Deliverables**:
- [ ] Anthropic proxy support (Claude models)
- [ ] Google AI proxy support (Gemini models)
- [ ] Alibaba Qwen proxy support
- [ ] Request format transformation (OpenAI ↔ Anthropic/Google/Qwen)
- [ ] Streaming support (SSE passthrough)
- [ ] Usage logging (tokens, latency, costs)
- [ ] Landing page (Next.js, based on designer mockups)
- [ ] User dashboard skeleton (authentication, layout)
- [ ] API key management UI (add, list, delete, test)

**Success Criteria**:
- All 4 providers accessible via unified endpoint
- Landing page live (staging.aiproxy.id)
- Users can add API keys via dashboard

---

### Week 3: Analytics + Billing + Polish (Dec 23-29, 2024)
**Goal**: Complete MVP features + Testing + Launch prep

**Deliverables**:
- [ ] Usage analytics dashboard (charts, logs, export CSV)
- [ ] Midtrans integration (QRIS, VA, Card)
- [ ] Subscription lifecycle (create, upgrade, cancel)
- [ ] Invoice generation (PDF)
- [ ] Admin dashboard (user management, platform stats)
- [ ] Rate limiting enforcement (per plan tier)
- [ ] Email notifications (quota alerts, payment success/failure)
- [ ] Security audit (SENTINEL review)
- [ ] Load testing (100 concurrent users)
- [ ] Bug fixes + polish

**Success Criteria**:
- Users can subscribe via Midtrans, receive invoice
- Analytics dashboard shows usage in real-time
- Admin can view all users, suspend accounts
- Passes load test (0% errors, <500ms p95 latency)

---

### Week 4: Launch! (Dec 30 - Jan 6, 2025)
**Goal**: Public beta launch + ProductHunt + User onboarding

**Activities**:
- [ ] ProductHunt submission (Tuesday 12:01 AM PST)
- [ ] Blog post (announcement + technical deep-dive)
- [ ] Social media campaign (Twitter, LinkedIn)
- [ ] Email to Webrana community
- [ ] Monitor server health (all hands on deck)
- [ ] Respond to user feedback
- [ ] Bug fixes (critical issues only)

**Launch Day Goals**:
- ✅ 100+ signups
- ✅ 20+ active users (added API key)
- ✅ 5+ paying users
- ✅ No critical downtime
- ✅ Positive ProductHunt reviews

---

## BUDGET BREAKDOWN (3 Months)

### One-Time Costs
| Category | Amount (IDR) | Amount (USD) | Status |
|----------|--------------|--------------|--------|
| UI/UX Designer | Rp 10,000,000 | $637 | Hiring Week 1 |
| Security Audit | Rp 78,500,000 | $5,000 | Week 3 |
| Legal (ToS, Privacy) | Rp 5,000,000 | $318 | Week 2-3 |
| Domain (1 year) | Rp 200,000 | $13 | Week 1 |
| **TOTAL** | **Rp 93,700,000** | **$5,968** | |

### Monthly Recurring Costs
| Category | Amount (IDR/mo) | Amount (USD/mo) | Notes |
|----------|-----------------|-----------------|-------|
| Infrastructure (DO K8s) | Rp 2,270,000 | $144.50 | 3 nodes, DB, Redis, LB |
| Email Service | Rp 160,000 | $10 | SendGrid/Resend |
| Domain | Rp 17,000 | $1 | Amortized annual |
| **TOTAL** | **Rp 2,447,000** | **$155.50** | |

### 3-Month Total
```
One-time:  Rp 93.7M  ($5,968)
Recurring: Rp  7.3M  ($466) x 3 months
───────────────────────────────────
TOTAL:     Rp 101M   ($6,434)
```

**Budget Approved**: Rp 95.3M → **Within budget** ✅

---

## PAYMENT INTEGRATION DETAILS

### Midtrans Configuration
**Payment Methods to Enable**:

1. **QRIS** (QR Code - Instant)
   - GoPay, OVO, Dana, LinkAja, ShopeePay
   - Fee: 0.7% per transaction
   - Settlement: D+1

2. **Virtual Account** (Bank Transfer)
   - BCA, BNI, BRI, Mandiri, Permata, CIMB
   - Fee: Rp 4,000 per transaction (flat)
   - Settlement: D+2
   - Auto-expire: 24 hours

3. **Credit/Debit Card**
   - Visa, Mastercard, JCB
   - Fee: 2.9% + Rp 2,000 per transaction
   - Settlement: D+7 (card), D+1 (debit)
   - 3D Secure required

### Subscription Flow
```
User selects plan (Starter Rp 49K)
    ↓
POST /billing/subscribe → Midtrans Snap API
    ↓
Redirect to Midtrans payment page
    ↓
User pays via QRIS/VA/Card
    ↓
Midtrans webhook → POST /webhook/midtrans
    ↓
Verify signature → Update subscription status
    ↓
Send email confirmation + invoice PDF
```

### Invoice Format
```
─────────────────────────────────────────
INVOICE #WEB-2024-12-001

Webrana AI Proxy
Singapore Data Center
support@aiproxy.id

Bill To:
John Doe
john@example.com

Date: 2024-12-09
Due Date: 2025-01-09

Description          Qty    Price      Amount
─────────────────────────────────────────
Starter Plan         1x   Rp 49,000  Rp 49,000

Subtotal:                           Rp 49,000
Tax (PPN 11%):                      Rp  5,390
─────────────────────────────────────────
TOTAL:                              Rp 54,390

Payment Method: QRIS (GoPay)
Status: PAID
Transaction ID: MT-20241209-ABC123
─────────────────────────────────────────
```

**Note**: PPN (Pajak Pertambahan Nilai) 11% applies for Indonesian customers.

---

## DOMAIN (CONFIRMED)

**Domain**: **webrana.id** ✅

**Benefits**:
- Matches company name (Webrana AI Proxy)
- .id extension (Indonesia-first positioning)
- Short, memorable
- Strong brand identity

**URLs**:
- Production: `https://webrana.id`
- API Endpoint: `https://api.webrana.id/v1`
- Staging: `https://staging.webrana.id`
- Admin: `https://admin.webrana.id`

**Cost**: ~Rp 150-200K/year

**Action**: ATLAS to register Monday (Week 1 Day 1)

---

## SUCCESS METRICS

### Week 1 (Foundation)
- ✅ Backend deployed to staging
- ✅ CI/CD pipeline working
- ✅ Designer hired
- ✅ All tests passing

### Week 2 (Multi-Provider)
- ✅ 4 providers working (OpenAI, Anthropic, Google, Qwen)
- ✅ Landing page live
- ✅ Dashboard functional

### Week 3 (Complete MVP)
- ✅ Billing working (Midtrans)
- ✅ Analytics dashboard
- ✅ Admin panel
- ✅ Security audit passed
- ✅ Load test passed

### Week 4 (Launch)
- ✅ 100+ signups
- ✅ 20+ active users
- ✅ 5+ paying users (Rp 245K MRR)
- ✅ ProductHunt top 5 product of the day
- ✅ 99% uptime

### Month 3 (Growth)
- ✅ 500 signups
- ✅ 100 paying users (Rp 8.8M MRR)
- ✅ Break-even (30 paying users minimum)
- ✅ <5% churn rate

---

## RISK MANAGEMENT

### Critical Risks
| Risk | Mitigation | Owner |
|------|------------|-------|
| Designer delays | Start outreach Day 1, have 3 backup candidates | COMPASS |
| Midtrans integration bugs | Test in sandbox Week 2, allocate 3 days | FORGE |
| Infrastructure costs overrun | Monitor daily, set up billing alerts | ATLAS |
| Security vulnerability | SENTINEL review Week 3, penetration test | SENTINEL |
| Slow user adoption | Aggressive marketing, ProductHunt launch | COMPASS |

### Contingency Plans
- **If Week 1 delayed**: Extend to Week 5 launch (Jan 13)
- **If designer unavailable**: Use Tailwind UI templates (delay custom design)
- **If Midtrans integration fails**: Fallback to manual bank transfer (temp)
- **If infrastructure issues**: Downgrade to smaller K8s cluster (save costs)

---

## COMMUNICATION & DAILY RITUALS

### Slack Channels
- **#aiproxy-dev**: Daily standup, technical discussions
- **#aiproxy-design**: Designer collaboration
- **#aiproxy-launch**: Marketing, ProductHunt prep

### Daily Standup (Async)
**Time**: Every morning by 10 AM (WIB)
**Format** (post in #aiproxy-dev):
```
@username:
Yesterday: ✅ Completed tasks
Today: 📋 Planned tasks
Blockers: 🚧 Any issues (or "None")
```

### Weekly Sync (Optional)
**Time**: Friday 3 PM WIB
**Format**: Video call (30 mins)
**Agenda**:
- Demo progress
- Retrospective (what went well, what to improve)
- Plan next week

---

## WEEK 1 DAY 1 CHECKLIST (MONDAY)

### Morning (9 AM - 12 PM)

**NEXUS**:
- [ ] Create GitHub repo: `webrana-ai/ai-proxy-mvp`
- [ ] Setup monorepo structure (backend, frontend, infrastructure, docs)
- [ ] Create GitHub Project board
- [ ] Setup Slack channel: #aiproxy-dev
- [ ] Post kickoff message (share this doc)

**ATLAS**:
- [ ] Create DigitalOcean account (or use existing)
- [ ] Apply for startup credits (if eligible)
- [ ] Provision K8s cluster (3 nodes, 2vCPU/4GB)
- [ ] Setup PostgreSQL (managed, dev tier)
- [ ] Deploy Redis (K8s StatefulSet)

**COMPASS**:
- [ ] Post UI/UX designer job listing:
  - Platforms: Upwork, Sribu, Projects.co.id
  - Budget: Rp 10M
  - Timeline: 2 weeks
  - Deliverables: Brand guidelines (Webrana brand), landing page mockup, dashboard wireframes

### Afternoon (1 PM - 6 PM)

**FORGE**:
- [ ] Clone repo
- [ ] Setup Rust environment (cargo, rustc, sqlx-cli)
- [ ] Create Cargo workspace (backend/Cargo.toml)
- [ ] Add dependencies (axum, tokio, sqlx, redis, aes-gcm, etc.)
- [ ] Create basic main.rs (HTTP server on :3000)
- [ ] Run locally: `cargo run`

**ATLAS**:
- [ ] Configure Cloudflare account
- [ ] Register domain: **webrana.id**
- [ ] Setup DNS (point to DigitalOcean K8s LB)
- [ ] Configure SSL (Let's Encrypt)
- [ ] Create subdomains:
  - api.webrana.id (proxy API endpoint)
  - staging.webrana.id (staging environment)
  - admin.webrana.id (admin dashboard)
- [ ] Create K8s namespaces (backend, frontend, monitoring)

**COMPASS**:
- [ ] Review designer applications (first batch)
- [ ] Schedule interviews (2-3 candidates)
- [ ] Draft brand guidelines (color palette, typography, tone)

### Evening (Optional)

**NEXUS**:
- [ ] Review Day 1 progress (Slack check-in)
- [ ] Identify any blockers
- [ ] Adjust Day 2 plan if needed

---

## NEXT MILESTONES

### Week 1 End (Dec 15)
- Backend MVP deployed to staging
- Designer hired, brand guidelines delivered
- CI/CD pipeline working

### Week 2 End (Dec 22)
- All 4 providers working
- Landing page live
- Dashboard functional

### Week 3 End (Dec 29)
- Billing integration complete
- Security audit passed
- Ready for launch

### Week 4 End (Jan 6, 2025)
- **PUBLIC LAUNCH** 🚀
- 100+ signups
- 5+ paying users

---

## EMERGENCY CONTACTS

**Technical Issues**:
- Backend: @FORGE (Slack DM)
- Infrastructure: @ATLAS (Slack DM)
- Security: @SENTINEL (Slack DM)

**Product/Business**:
- Product questions: @COMPASS (Slack DM)
- Coordination: @NEXUS (Slack DM)

**External**:
- DigitalOcean support: https://cloud.digitalocean.com/support
- Midtrans support: support@midtrans.com
- Cloudflare support: https://support.cloudflare.com

---

## FINAL CHECKLIST BEFORE KICKOFF

- [x] Budget approved: Rp 95.3M
- [x] Scope finalized: API Key MVP
- [x] Company name: Webrana AI Proxy
- [x] Payment methods: QRIS, VA, Card (Midtrans)
- [x] Launch date: Week 4 (Jan 6, 2025)
- [x] Team assignments clear
- [x] Week 1 sprint plan ready
- [x] Domain chosen: webrana.id
- [ ] Domain registered (Week 1 Day 1)
- [ ] DigitalOcean account ready (Week 1 Day 1)
- [ ] GitHub repo created (Week 1 Day 1)
- [ ] Designer job posted (Week 1 Day 1)

---

## LET'S GO! 🚀

**Status**: ✅ READY TO START
**Start Date**: Monday, December 9, 2024
**First Milestone**: Week 1 End (Dec 15, 2024)
**Launch Date**: Week 4 End (Jan 6, 2025)

**Team motto**: *"Ship fast, iterate faster. 3 weeks to launch!"*

---

**Document Owner**: NEXUS
**Last Updated**: 2024-12-09
**Version**: 1.0 (FINAL)
