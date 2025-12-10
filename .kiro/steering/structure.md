---
inclusion: always
---

# Architect Persona & Project Structure

> When creating or reviewing files, adopt the mindset of a **Software Architect** enforcing Domain-Driven Design principles and clean architecture boundaries.

## Monorepo Structure (Webrana AI Proxy)

```
webrana-ai-proxy/
├── backend/                    # Rust/Axum API server
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/            # HTTP route handlers
│   │   │   ├── auth.rs        # /auth/* endpoints
│   │   │   ├── api_keys.rs    # /api-keys/* endpoints
│   │   │   ├── proxy.rs       # /v1/chat/completions
│   │   │   ├── usage.rs       # /usage/* endpoints
│   │   │   └── billing.rs     # /billing/* + webhooks
│   │   ├── services/          # Business logic
│   │   │   ├── api_key_service.rs    # Encrypt/decrypt, validate
│   │   │   ├── proxy_service.rs      # Route requests, transform
│   │   │   ├── usage_tracker.rs      # Log requests, count tokens
│   │   │   └── billing_service.rs    # Midtrans integration
│   │   ├── models/            # Database models (SQLx)
│   │   ├── middleware/        # Auth, rate limiting, logging
│   │   └── utils/             # Encryption, helpers
│   ├── migrations/            # SQL migrations
│   └── Cargo.toml
│
├── frontend/                   # Next.js 16 App (React 19)
│   ├── src/
│   │   ├── app/               # Next.js App Router
│   │   │   ├── (marketing)/   # Landing page, pricing
│   │   │   │   ├── page.tsx
│   │   │   │   └── pricing/page.tsx
│   │   │   ├── (dashboard)/   # User dashboard (auth required)
│   │   │   │   ├── layout.tsx
│   │   │   │   ├── overview/page.tsx
│   │   │   │   ├── api-keys/page.tsx
│   │   │   │   ├── usage/page.tsx
│   │   │   │   ├── billing/page.tsx
│   │   │   │   └── settings/page.tsx
│   │   │   ├── (admin)/       # Admin dashboard
│   │   │   │   └── admin/page.tsx
│   │   │   └── api/           # Next.js API routes (auth)
│   │   ├── components/        # React components
│   │   │   ├── ui/            # shadcn/ui components
│   │   │   ├── charts/        # Recharts wrappers
│   │   │   └── forms/         # Form components
│   │   ├── lib/               # Utilities
│   │   │   ├── api-client.ts  # TanStack Query setup
│   │   │   └── utils.ts
│   │   └── types/             # TypeScript types
│   └── package.json
│
├── infrastructure/             # Terraform + K8s configs
│   ├── terraform/             # DigitalOcean provisioning
│   ├── k8s/                   # Kubernetes manifests
│   └── docker/                # Dockerfiles
│
├── docs/                       # Project documentation
│   ├── PRD_AI_PROXY_SERVICE.md
│   ├── API_KEY_MVP_SCOPE.md
│   └── ...
│
└── .kiro/                      # Kiro configuration
    ├── steering/
    ├── hooks/
    ├── settings/
    └── specs/
```

## Domain Boundaries (Backend)

### Bounded Contexts
| Domain | Responsibility | Key Entities |
|--------|----------------|--------------|
| **auth** | User authentication, sessions | User, Session |
| **api-keys** | Provider API key management | ApiKey, ProxyApiKey |
| **proxy** | Request routing, transformation | ProxyRequest, Provider |
| **usage** | Analytics, logging, reporting | UsageLog, UsageStats |
| **billing** | Subscriptions, payments | Subscription, Invoice |

### Layer Dependencies (Backend)
```
routes/ → services/ → models/
              ↓
          middleware/
              ↓
           utils/
```

- **routes/** MAY import from: `services/`, `models/`, `middleware/`
- **services/** MAY import from: `models/`, `utils/`
- **models/** MUST NOT import from other layers
- **middleware/** MAY import from: `services/`, `utils/`

## Frontend Architecture

### Route Groups
| Group | Purpose | Auth Required |
|-------|---------|---------------|
| `(marketing)` | Landing page, pricing, docs | No |
| `(dashboard)` | User dashboard | Yes |
| `(admin)` | Admin panel | Yes (admin role) |

### Component Organization
```
components/
├── ui/                 # shadcn/ui primitives (Button, Card, etc.)
├── charts/             # Analytics charts (UsageChart, ProviderPie)
├── forms/              # Form components (ApiKeyForm, LoginForm)
├── layout/             # Layout components (Sidebar, Header)
└── features/           # Feature-specific components
    ├── api-keys/       # ApiKeyList, ApiKeyCard
    ├── usage/          # UsageTable, UsageStats
    └── billing/        # PlanCard, InvoiceList
```

## File Naming Conventions

| File Type | Location | Naming | Example |
|-----------|----------|--------|---------|
| Rust module | `backend/src/` | snake_case.rs | `api_key_service.rs` |
| SQL migration | `backend/migrations/` | `YYYYMMDD_name.sql` | `20241209_create_users.sql` |
| React Page | `frontend/src/app/` | `page.tsx` | `overview/page.tsx` |
| React Component | `frontend/src/components/` | PascalCase.tsx | `ApiKeyCard.tsx` |
| TypeScript Type | `frontend/src/types/` | kebab-case.ts | `api-response.ts` |
| Utility | `frontend/src/lib/` | kebab-case.ts | `api-client.ts` |

## Architect Review Checklist

When a new file is created, verify:

- [ ] File is in the correct directory per structure above
- [ ] File naming follows the convention for its type
- [ ] Import statements respect layer boundaries
- [ ] No circular dependencies introduced
- [ ] Related test file created (if applicable)
- [ ] Aligns with PRD domain boundaries
