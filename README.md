# Webrana AI Proxy

Unified API for all AI models with analytics, team management, and Rupiah billing.

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 20+
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+

### Local Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/webrana-ai/ai-proxy-mvp.git
   cd ai-proxy-mvp
   ```

2. **Start infrastructure**
   ```bash
   docker-compose -f infrastructure/docker/docker-compose.yml up -d postgres redis
   ```

3. **Setup backend**
   ```bash
   cd backend
   cp .env.example .env
   # Edit .env with your configuration
   cargo run
   ```

4. **Setup frontend**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

5. **Access the application**
   - Frontend: http://localhost:3001
   - Backend API: http://localhost:3000
   - Health check: http://localhost:3000/health

## 📁 Project Structure

```
webrana-ai-proxy/
├── backend/           # Rust/Axum API server
│   ├── src/
│   │   ├── routes/    # HTTP route handlers
│   │   ├── services/  # Business logic
│   │   ├── models/    # Database models
│   │   ├── middleware/# Auth, rate limiting
│   │   └── utils/     # Encryption, helpers
│   └── migrations/    # SQL migrations
├── frontend/          # Next.js 15 App
│   └── src/app/       # App Router pages
├── infrastructure/    # Docker, K8s, Terraform
└── docs/              # Documentation
```

## 🔧 Tech Stack

### Backend
- **Framework**: Rust + Axum 0.7
- **Database**: PostgreSQL 15 + SQLx
- **Cache**: Redis 7
- **Encryption**: AES-256-GCM
- **Auth**: Argon2id + JWT

### Frontend
- **Framework**: Next.js 15 (App Router)
- **Styling**: Tailwind CSS 4 + shadcn/ui
- **State**: Zustand + TanStack Query
- **Validation**: Zod

### Infrastructure
- **Cloud**: DigitalOcean (Singapore)
- **Orchestration**: Kubernetes
- **CDN**: Cloudflare
- **Payment**: Midtrans

## 💰 Pricing (Rupiah)

| Tier | Price | Requests/Month |
|------|-------|----------------|
| Free | Rp 0 | 1,000 |
| Starter | Rp 49,000 | 10,000 |
| Pro | Rp 99,000 | 50,000 |
| Team | Rp 299,000 | 200,000 |

## 📄 License

Copyright © 2024 Webrana AI Proxy. All rights reserved.
