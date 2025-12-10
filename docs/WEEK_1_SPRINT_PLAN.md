# Week 1 Sprint Plan - AI Proxy Service MVP
**Sprint**: Week 1 of 4
**Dates**: 2024-12-09 to 2024-12-15 (7 days)
**Goal**: Foundation setup + Backend core + DevOps automation

---

## SPRINT OBJECTIVES

**Primary Goals**:
1. ✅ Development environment ready (repo, CI/CD, staging cluster)
2. ✅ Backend foundation (Axum server, PostgreSQL, Redis)
3. ✅ API key encryption working (encrypt/decrypt)
4. ✅ Basic proxy endpoint functional (OpenAI only)
5. ✅ UI/UX designer hired and onboarded

**Success Criteria**:
- Backend can accept API key, encrypt it, store in DB
- Proxy endpoint forwards request to OpenAI, returns response
- CI/CD pipeline deploys to staging on every commit
- Designer has brand guidelines and ready to mock landing page

---

## TEAM ASSIGNMENTS

| Role | Person | Commitment | Focus This Week |
|------|--------|------------|-----------------|
| **Team Lead** | NEXUS | Full-time | Coordination, unblocking |
| **Backend Engineer** | FORGE | Full-time | Rust/Axum, encryption, proxy |
| **DevOps** | ATLAS | Part-time (20h) | K8s, CI/CD, monitoring |
| **Product Manager** | COMPASS | Part-time (10h) | Designer hiring, PRD updates |
| **Security Advisor** | SENTINEL | On-call | Code review, encryption review |
| **UI/UX Designer** | TBD (hiring) | Contract | Brand, wireframes, mockups |

---

## DAY-BY-DAY BREAKDOWN

### Day 1 (Monday): Setup & Kickoff
**Morning (NEXUS)**:
- [ ] Create project repository `ai-proxy-mvp` (GitHub)
- [ ] Setup monorepo structure:
  ```
  ai-proxy-mvp/
  ├── backend/          (Rust/Axum)
  ├── frontend/         (Next.js 15)
  ├── infrastructure/   (Terraform)
  ├── docs/             (README, API spec)
  └── .github/workflows/  (CI/CD)
  ```
- [ ] Create project board (GitHub Projects or Linear)
- [ ] Setup Slack channel: `#aiproxy-dev`
- [ ] Kickoff meeting (async: record Loom video or write doc)

**Afternoon (ATLAS)**:
- [ ] Create DigitalOcean account
- [ ] Provision staging K8s cluster (3 nodes, 2vCPU/4GB each)
- [ ] Setup PostgreSQL managed database (dev tier, 1vCPU/2GB)
- [ ] Setup Redis (deploy to K8s, single pod for now)
- [ ] Configure Cloudflare account (DNS, SSL)
- [ ] Register domain (e.g., `aiproxy.id` or `modelproxy.id`)

**Evening (FORGE)**:
- [ ] Clone repo, setup local Rust environment
- [ ] Create Cargo workspace (`backend/Cargo.toml`)
- [ ] Add dependencies:
  ```toml
  [dependencies]
  axum = "0.7"
  tokio = { version = "1", features = ["full"] }
  sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "uuid", "time"] }
  redis = { version = "0.24", features = ["tokio-comp"] }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  uuid = { version = "1.0", features = ["serde", "v4"] }
  aes-gcm = "0.10"
  argon2 = "0.5"
  reqwest = { version = "0.11", features = ["json", "stream"] }
  tower-http = { version = "0.5", features = ["cors", "trace"] }
  ```
- [ ] Create basic `main.rs` (hello world HTTP server)
- [ ] Run locally: `cargo run` → http://localhost:3000

**Deliverable**: Repo created, infra provisioned, backend boots locally

---

### Day 2 (Tuesday): Database + Authentication
**FORGE Tasks**:
- [ ] Create database migrations (`backend/migrations/`)
  ```sql
  -- 001_create_users.sql
  CREATE TABLE users (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      email VARCHAR(255) UNIQUE NOT NULL,
      password_hash VARCHAR(255) NOT NULL,
      plan VARCHAR(50) DEFAULT 'free',
      created_at TIMESTAMP DEFAULT NOW(),
      email_verified BOOLEAN DEFAULT FALSE
  );

  -- 002_create_api_keys.sql
  CREATE TABLE api_keys (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID REFERENCES users(id) ON DELETE CASCADE,
      provider VARCHAR(50) NOT NULL,
      label VARCHAR(100),
      encrypted_key TEXT NOT NULL,
      iv BYTEA NOT NULL,
      created_at TIMESTAMP DEFAULT NOW(),
      last_used_at TIMESTAMP,
      is_valid BOOLEAN DEFAULT TRUE,
      UNIQUE(user_id, provider, label)
  );

  -- 003_create_proxy_api_keys.sql
  CREATE TABLE proxy_api_keys (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID REFERENCES users(id) ON DELETE CASCADE,
      key_prefix VARCHAR(20) NOT NULL,
      key_hash VARCHAR(255) NOT NULL,
      created_at TIMESTAMP DEFAULT NOW(),
      last_used_at TIMESTAMP,
      UNIQUE(key_hash)
  );

  -- 004_create_proxy_requests.sql
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
  ```

- [ ] Run migrations: `sqlx migrate run`
- [ ] Create models (`backend/src/models/`)
  - `user.rs`
  - `api_key.rs`
  - `proxy_request.rs`

- [ ] Implement authentication endpoints:
  ```rust
  POST /auth/signup
    → Hash password (Argon2id)
    → Insert user to DB
    → Generate JWT token
    → Return { user, token }

  POST /auth/login
    → Verify password hash
    → Generate JWT
    → Return { user, token }
  ```

- [ ] Test authentication:
  ```bash
  curl -X POST http://localhost:3000/auth/signup \
    -H "Content-Type: application/json" \
    -d '{"email": "test@example.com", "password": "securepass123"}'
  ```

**ATLAS Tasks**:
- [ ] Configure PostgreSQL connection string (K8s Secret)
- [ ] Test DB connectivity from local → staging DB

**Deliverable**: User signup/login working, DB migrations applied

---

### Day 3 (Wednesday): API Key Encryption
**FORGE Tasks**:
- [ ] Implement encryption service (`backend/src/services/encryption.rs`)
  ```rust
  pub struct EncryptionService {
      master_key: [u8; 32], // AES-256 key
  }

  impl EncryptionService {
      pub fn encrypt(&self, plaintext: &str) -> (Vec<u8>, Vec<u8>) {
          // AES-256-GCM encryption
          // Returns (ciphertext, nonce/IV)
      }

      pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
          // Decrypt and return plaintext
      }
  }
  ```

- [ ] Implement API key management endpoints:
  ```rust
  POST /api-keys
    → Validate provider (openai, anthropic, google, alibaba)
    → Encrypt API key
    → Store encrypted_key + iv in DB
    → Return { id, provider, label, created_at }

  GET /api-keys
    → Fetch user's API keys (WITHOUT decrypting)
    → Return list (keys are masked: sk-ant-***...****)

  DELETE /api-keys/:id
    → Verify ownership (user_id matches)
    → Delete from DB
    → Return success

  POST /api-keys/:id/test
    → Decrypt API key
    → Make test request to provider (e.g., OpenAI list models)
    → Return { valid: true/false, latency_ms }
  ```

- [ ] Test API key CRUD:
  ```bash
  # Add OpenAI key
  curl -X POST http://localhost:3000/api-keys \
    -H "Authorization: Bearer {jwt_token}" \
    -H "Content-Type: application/json" \
    -d '{
      "provider": "openai",
      "label": "Personal GPT",
      "key": "sk-proj-abc123..."
    }'

  # List keys
  curl http://localhost:3000/api-keys \
    -H "Authorization: Bearer {jwt_token}"
  ```

**SENTINEL Task**:
- [ ] Review encryption implementation (code review)
- [ ] Verify master key management (env var, not hardcoded)
- [ ] Test: Can user A access user B's keys? (Should fail)

**Deliverable**: API keys can be added, encrypted, stored, retrieved (encrypted)

---

### Day 4 (Thursday): Proxy Core (OpenAI Only)
**FORGE Tasks**:
- [ ] Implement proxy service (`backend/src/services/proxy.rs`)
  ```rust
  pub async fn proxy_request(
      user_id: Uuid,
      proxy_api_key: &str,
      model: &str,
      messages: Vec<Message>,
      stream: bool,
  ) -> Result<Response> {
      // 1. Detect provider from model name
      let provider = detect_provider(model)?; // "gpt-4" → openai

      // 2. Get user's API key for this provider
      let api_key_record = db.get_api_key(user_id, provider).await?;

      // 3. Decrypt API key
      let decrypted_key = encryption.decrypt(&api_key_record.encrypted_key, &api_key_record.iv)?;

      // 4. Forward request to provider
      let response = reqwest::Client::new()
          .post("https://api.openai.com/v1/chat/completions")
          .bearer_auth(decrypted_key)
          .json(&json!({
              "model": model,
              "messages": messages,
              "stream": stream
          }))
          .send()
          .await?;

      // 5. Log usage
      let tokens = extract_tokens_from_response(&response);
      db.log_request(user_id, api_key_record.id, model, tokens).await?;

      // 6. Return response
      Ok(response)
  }
  ```

- [ ] Implement proxy endpoint:
  ```rust
  POST /v1/chat/completions
    → Extract proxy API key from Authorization header
    → Verify proxy API key (lookup in proxy_api_keys table)
    → Get user_id from proxy key
    → Check rate limit (Redis)
    → Call proxy_service.proxy_request()
    → Return OpenAI-formatted response
  ```

- [ ] Generate proxy API key endpoint:
  ```rust
  POST /proxy-api-keys/generate
    → Generate random key (sk-proxy-{48 random chars})
    → Hash with Argon2id
    → Store hash in proxy_api_keys table
    → Return plaintext key (ONLY ONCE, never again)
  ```

- [ ] Test proxy (OpenAI only for now):
  ```bash
  # Generate proxy key
  curl -X POST http://localhost:3000/proxy-api-keys/generate \
    -H "Authorization: Bearer {jwt_token}"
  # Response: { key: "sk-proxy-abc123..." }

  # Use proxy
  curl -X POST http://localhost:3000/v1/chat/completions \
    -H "Authorization: Bearer sk-proxy-abc123..." \
    -H "Content-Type: application/json" \
    -d '{
      "model": "gpt-4",
      "messages": [{"role": "user", "content": "Hello!"}]
    }'
  ```

**Deliverable**: Proxy works for OpenAI (gpt-4, gpt-3.5-turbo)

---

### Day 5 (Friday): CI/CD + Deployment
**ATLAS Tasks**:
- [ ] Create Dockerfile (`backend/Dockerfile`):
  ```dockerfile
  FROM rust:1.75 AS builder
  WORKDIR /app
  COPY Cargo.toml Cargo.lock ./
  COPY src ./src
  RUN cargo build --release

  FROM debian:bookworm-slim
  RUN apt-get update && apt-get install -y libssl3 ca-certificates
  COPY --from=builder /app/target/release/ai-proxy /usr/local/bin/
  EXPOSE 3000
  CMD ["ai-proxy"]
  ```

- [ ] Create Kubernetes manifests (`infrastructure/k8s/staging/`)
  - `deployment.yaml` (backend pods)
  - `service.yaml` (ClusterIP)
  - `ingress.yaml` (NGINX ingress)
  - `secrets.yaml` (DB credentials, master key)

- [ ] Create GitHub Actions workflow (`.github/workflows/backend.yml`):
  ```yaml
  name: Backend CI/CD
  on:
    push:
      branches: [main, develop]
      paths:
        - 'backend/**'

  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - uses: actions-rs/toolchain@v1
          with:
            toolchain: stable
        - run: cd backend && cargo test

    build-deploy:
      needs: test
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - name: Build Docker image
          run: |
            docker build -t ghcr.io/${{ github.repository }}/backend:${{ github.sha }} backend/
        - name: Push to GHCR
          run: |
            echo ${{ secrets.GITHUB_TOKEN }} | docker login ghcr.io -u ${{ github.actor }} --password-stdin
            docker push ghcr.io/${{ github.repository }}/backend:${{ github.sha }}
        - name: Deploy to K8s
          run: |
            kubectl set image deployment/backend backend=ghcr.io/${{ github.repository }}/backend:${{ github.sha }}
  ```

- [ ] Setup monitoring:
  - Deploy Prometheus + Grafana (kube-prometheus-stack Helm chart)
  - Create basic dashboard (request rate, latency, error rate)
  - Setup Loki for logs

- [ ] Test deployment:
  ```bash
  git commit -m "feat: initial backend"
  git push origin main
  # Watch GitHub Actions run
  # Verify deployment: kubectl get pods -n backend
  # Test staging: curl https://staging.aiproxy.id/health
  ```

**FORGE Task**:
- [ ] Add health check endpoint:
  ```rust
  GET /health
    → Check DB connection
    → Check Redis connection
    → Return { status: "healthy", db: "ok", redis: "ok" }
  ```

**Deliverable**: CI/CD pipeline working, backend deployed to staging

---

### Day 6 (Saturday): Designer Onboarding + Documentation
**COMPASS Tasks**:
- [ ] Post UI/UX designer job listing:
  - Platforms: Upwork, Sribu, Projects.co.id, Dribbble
  - Budget: Rp 10M for 2 weeks work
  - Deliverables: Brand guidelines, landing page mockup, dashboard wireframes
  - Timeline: Week 1 (hire), Week 2 (design), Week 3 (implement)

- [ ] Review designer portfolios
- [ ] Interview 3-5 candidates (video call)
- [ ] Select designer and send contract

**FORGE Tasks**:
- [ ] Write API documentation (`docs/API.md`):
  - Authentication endpoints
  - API key management endpoints
  - Proxy endpoint spec
  - Example requests/responses (curl, Python, JavaScript)

- [ ] Write README.md:
  - Project overview
  - Local development setup
  - Environment variables
  - Running tests
  - Deployment process

**NEXUS Tasks**:
- [ ] Create Sprint 2 plan (Week 2: Multi-provider + Analytics)
- [ ] Review Week 1 progress
- [ ] Identify blockers, adjust timeline if needed

**Deliverable**: Designer hired, documentation complete

---

### Day 7 (Sunday): Testing + Bug Fixes
**FORGE Tasks**:
- [ ] Write unit tests:
  - Encryption/decryption tests
  - API key CRUD tests
  - Authentication tests
  - Proxy logic tests (mocked HTTP client)

- [ ] Integration tests:
  - End-to-end: Signup → Add API key → Proxy request → Check usage log
  - Test with real OpenAI API key (using test account)

- [ ] Fix bugs identified during testing

**ATLAS Tasks**:
- [ ] Load testing (basic):
  ```bash
  # Install k6
  brew install k6

  # Run test script
  k6 run load-test.js
  # Target: 50 concurrent users, 1000 requests
  # Success criteria: <500ms p95 latency, 0% errors
  ```

- [ ] Review logs in Grafana
- [ ] Tune resource limits if needed

**SENTINEL Task**:
- [ ] Security review checklist:
  - [ ] Master key not in git
  - [ ] DB credentials in K8s Secrets
  - [ ] SQL injection prevention (parameterized queries)
  - [ ] API key encryption verified
  - [ ] Rate limiting enforced
  - [ ] CORS configured correctly

**Deliverable**: Tests passing, load test successful, security review complete

---

## WEEK 1 DELIVERABLES CHECKLIST

**Infrastructure**:
- [x] GitHub repo created
- [x] DigitalOcean K8s cluster (staging)
- [x] PostgreSQL database (managed)
- [x] Redis (K8s deployment)
- [x] Domain registered + Cloudflare DNS
- [x] CI/CD pipeline (GitHub Actions)
- [x] Monitoring (Prometheus + Grafana + Loki)

**Backend**:
- [x] Axum server running
- [x] Database migrations (users, api_keys, proxy_api_keys, proxy_requests)
- [x] Authentication (signup, login)
- [x] API key management (add, list, delete, test)
- [x] API key encryption (AES-256-GCM)
- [x] Proxy endpoint (OpenAI only)
- [x] Proxy API key generation
- [x] Usage logging
- [x] Rate limiting (basic, per user)
- [x] Health check endpoint
- [x] Unit tests (>80% coverage)
- [x] Integration tests (E2E happy path)

**DevOps**:
- [x] Docker image builds
- [x] K8s deployment manifests
- [x] Automated deployment (commit → staging)
- [x] Load testing (50 users, <500ms p95)

**Product**:
- [x] UI/UX designer hired
- [x] API documentation
- [x] README.md

**Security**:
- [x] Code review (SENTINEL)
- [x] Security checklist passed

---

## WEEK 1 RISKS

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Designer hiring delays | MEDIUM | MEDIUM | Start outreach Day 1, have 3 backup candidates |
| DigitalOcean account approval slow | LOW | MEDIUM | Use personal account temporarily |
| OpenAI API rate limits (testing) | LOW | LOW | Use test API key with low rate limits |
| Encryption bugs | MEDIUM | HIGH | SENTINEL code review, unit tests |

---

## SUCCESS METRICS (Week 1)

**Code Quality**:
- ✅ All tests passing (cargo test)
- ✅ >80% code coverage
- ✅ No clippy warnings (cargo clippy)
- ✅ SENTINEL security review approved

**Performance**:
- ✅ API startup <500ms
- ✅ Proxy latency <100ms overhead (vs direct OpenAI call)
- ✅ Load test: 50 concurrent users, 0% errors

**Infrastructure**:
- ✅ CI/CD deploys in <10 minutes
- ✅ Staging environment accessible (https://staging.aiproxy.id)
- ✅ 99% uptime (Week 1 target)

---

## DAILY STANDUP FORMAT (Async in Slack)

**Every morning, team posts in #aiproxy-dev**:

```
Yesterday: [completed tasks]
Today: [planned tasks]
Blockers: [anything blocking progress]
```

**Example**:
```
@forge: Yesterday: ✅ DB migrations, authentication endpoints
Today: API key encryption, proxy endpoint
Blockers: None
```

---

## END OF WEEK 1 REVIEW

**Friday EOD (3pm)**:
- [ ] NEXUS: Compile Week 1 achievements
- [ ] Team: Demo backend (video recording or live call)
- [ ] Retrospective: What went well? What to improve?
- [ ] Plan Week 2 sprint (multi-provider support, frontend start)

---

## WEEK 2 PREVIEW

**Goals**:
- ✅ Add Anthropic, Google, Qwen proxy support
- ✅ Frontend Next.js setup + landing page implementation
- ✅ Usage analytics backend (query endpoints)
- ✅ Midtrans integration (payment flow)

**Team Additions**:
- Frontend developer (contract or internal?)
- Designer delivers mockups by Wednesday

---

**Document Owner**: NEXUS
**Sprint**: Week 1 of 4
**Status**: READY TO START
**Start Date**: 2024-12-09

**GO TIME! 🚀**
