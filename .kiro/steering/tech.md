---
inclusion: always
---

# Technology Stack & Constraints

This document defines mandatory technologies, patterns, and constraints for the Webrana AI Proxy project. Follow these rules strictly when generating or modifying code.

## Backend (Rust/Axum)

**Required Stack:**
- Rust Nightly (rustlang/rust:nightly-bookworm via Docker)
- Rust 2021 Edition, Axum 0.7.x, SQLx 0.7.x (compile-time queries)
- PostgreSQL 15+ (postgres:15-alpine), Redis 7.x (redis:7-alpine)
- redis crate 0.24.x (tokio-comp feature)
- aes-gcm 0.10.x (AES-256-GCM encryption), argon2 0.5.x (password hashing)
- reqwest 0.11.x (HTTP client for proxying)
- proptest 1.x (property-based testing)
- tokio 1.x (full features), tower 0.4.x, tower-http 0.5.x

**Code Patterns:**
- Use `async` handlers; never block the async runtime
- Use SQLx parameterized queries with `query!` or `query_as!` macros
- Return `Result<T, AppError>` from service functions
- Use `tracing` for structured logging
- Use `thiserror` for custom error types
- Use `anyhow` for error propagation in application code

**Forbidden:**
- Plaintext API key storage (must use AES-256-GCM)
- SQL string concatenation or interpolation
- Hardcoded secrets (use environment variables)
- Blocking I/O (`std::fs`, `std::thread::sleep`) in async contexts

## Frontend (Next.js 16)

**Required Stack:**
- Next.js 16.x with App Router, TypeScript 5.7.x (strict mode)
- React 19.x, React DOM 19.x
- Tailwind CSS 3.4.x + shadcn/ui components (Radix UI primitives)
- Zustand 5.x (state), TanStack Query 5.62.x (data fetching), Zod 3.24.x (validation)
- NextAuth.js 5.x beta (auth), Recharts 2.15.x (charts), Vitest 2.1.x (testing)
- date-fns 4.x (date utilities), lucide-react 0.468.x (icons)

**Code Patterns:**
- Use Server Components by default; add `'use client'` only when needed
- Use named exports for components (except `page.tsx`, `layout.tsx`)
- Fetch data with TanStack Query hooks in client components
- Validate forms with Zod schemas
- Use Radix UI primitives via shadcn/ui (Dialog, DropdownMenu, Label, Select, Tabs, Toast)

**Forbidden:**
- `getServerSideProps` / `getStaticProps` (use App Router patterns)
- Redux (use Zustand)
- Axios (use native fetch with custom wrapper)
- Moment.js (use date-fns)
- Class components (functional only)
- Default exports for non-page components

## Database Schema

All tables must include:
```sql
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
created_at TIMESTAMP DEFAULT NOW(),
updated_at TIMESTAMP DEFAULT NOW()
```

**Core Tables:** `users`, `api_keys`, `proxy_api_keys`, `proxy_requests`, `subscriptions`, `invoices`

## Security Requirements

| Concern | Implementation |
|---------|----------------|
| API Key Encryption | AES-256-GCM (aes-gcm 0.10), unique 12-byte IV per encryption |
| Password Hashing | Argon2id (argon2 0.5) |
| JWT | jsonwebtoken 9.x, 24h access / 7d refresh |
| Rate Limiting | 100 req/min per IP (Redis-backed) |
| Master Key | K8s Secret, rotate every 90 days |

## Environment Variables

**Backend:** `DATABASE_URL`, `REDIS_URL`, `MASTER_ENCRYPTION_KEY` (32 bytes base64), `JWT_SECRET`, `MIDTRANS_SERVER_KEY`, `MIDTRANS_CLIENT_KEY`

**Frontend:** `NEXT_PUBLIC_API_URL`, `NEXTAUTH_SECRET`, `NEXTAUTH_URL`

## Performance Targets

| Metric | Target |
|--------|--------|
| Proxy latency overhead | <100ms p95 |
| API response time | <200ms p95 |
| Frontend LCP | <2s |
| Uptime | 99.5% |

## Infrastructure

- Cloud: DigitalOcean (Singapore region)
- Orchestration: Kubernetes (3 nodes, 2vCPU/4GB each)
- CDN: Cloudflare (free tier)
- Payment: Midtrans (QRIS, VA, Card)
- Monitoring: Prometheus + Grafana

## Development Environment

**Docker Compose (Required for local development):**
- All backend commands must be run via Docker Compose
- Config location: `infrastructure/docker/docker-compose.yml`

**Running Backend Tests:**
```bash
# From infrastructure/docker directory
docker compose run --rm backend-dev cargo test

# Run specific tests
docker compose run --rm backend-dev cargo test property_tests --no-fail-fast -- --nocapture

# Run with filter
docker compose run --rm backend-dev cargo test <test_name>
```

**Running Backend Build:**
```bash
docker compose run --rm backend-dev cargo build
```

**Starting Development Services:**
```bash
# Start all services (postgres, redis, backend-dev)
docker compose up -d

# View logs
docker compose logs -f backend-dev
```

**Note:** Cargo is NOT installed locally. Always use Docker for Rust commands.
