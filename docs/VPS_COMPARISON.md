# VPS Provider Comparison - Webrana AI Proxy (MVP)

## Requirement Minimum untuk MVP

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 vCPU | 2-4 vCPU |
| RAM | 2 GB | 4 GB |
| Storage | 40 GB SSD | 80 GB NVMe |
| Bandwidth | 2 TB/month | Unlimited |
| Location | Singapore/Jakarta | Singapore |

---

## Provider Comparison (2 vCPU / 4 GB RAM)

| Provider | Plan | Price/Month | Storage | Bandwidth | Data Center |
|----------|------|-------------|---------|-----------|-------------|
| **DigitalOcean** | Basic Droplet | $24 | 80 GB SSD | 4 TB | Singapore |
| **Vultr** | Cloud Compute | $24 | 80 GB NVMe | 4 TB | Singapore |
| **Linode (Akamai)** | Shared CPU | $24 | 80 GB SSD | 4 TB | Singapore |
| **Hetzner** | CX31 | €7.50 (~$8) | 80 GB SSD | 20 TB | Singapore* |
| **Contabo** | VPS S | €6.99 (~$7.50) | 100 GB NVMe | Unlimited | Singapore |
| **AWS Lightsail** | 2 vCPU/4GB | $20 | 80 GB SSD | 4 TB | Singapore |
| **Google Cloud** | e2-medium | ~$25 | 10 GB (+ extra) | Pay per GB | Singapore |
| **Azure** | B2s | ~$30 | 8 GB (+ extra) | Pay per GB | Singapore |
| **IDCloudHost** | VPS KVM | Rp 150k (~$9.50) | 40 GB SSD | Unlimited | Jakarta |
| **Dewaweb** | Cloud VPS | Rp 200k (~$12.50) | 50 GB SSD | Unlimited | Jakarta |
| **Biznet Gio** | NEO Lite | Rp 150k (~$9.50) | 60 GB SSD | Unlimited | Jakarta |

*Hetzner Singapore masih dalam waitlist, alternatif: Hetzner Germany (~150ms latency ke Indonesia)

---

## Rekomendasi per Budget

### Budget Ketat (< $10/month)

**Contabo VPS S - €6.99/month (~Rp 115k)**
```
✅ 4 vCPU, 8 GB RAM, 100 GB NVMe
✅ Unlimited bandwidth
✅ Singapore datacenter
⚠️ Support response lambat
⚠️ Oversold, performance bisa fluktuatif
```

**Hetzner CX31 - €7.50/month (~Rp 125k)**
```
✅ 2 vCPU, 8 GB RAM, 80 GB SSD
✅ 20 TB bandwidth
✅ Excellent performance/price ratio
⚠️ Singapore datacenter limited availability
⚠️ Germany datacenter = higher latency
```

### Budget Menengah ($10-25/month)

**DigitalOcean Basic Droplet - $24/month (~Rp 380k)**
```
✅ 2 vCPU, 4 GB RAM, 80 GB SSD
✅ Singapore datacenter
✅ Excellent documentation & community
✅ Easy managed database add-on
✅ Free $200 credit for new users
⚠️ Slightly more expensive
```

**Vultr Cloud Compute - $24/month (~Rp 380k)**
```
✅ 2 vCPU, 4 GB RAM, 80 GB NVMe
✅ Singapore datacenter
✅ Hourly billing
✅ Good API for automation
✅ Free $100 credit for new users
```

**AWS Lightsail - $20/month (~Rp 320k)**
```
✅ 2 vCPU, 4 GB RAM, 80 GB SSD
✅ Singapore datacenter
✅ Easy upgrade path to full AWS
✅ Managed database option ($15/month extra)
⚠️ Limited customization vs EC2
```

### Provider Lokal Indonesia

**IDCloudHost VPS KVM - Rp 150k/month**
```
✅ 2 vCPU, 4 GB RAM, 40 GB SSD
✅ Jakarta datacenter (lowest latency)
✅ Rupiah billing
✅ Local support (Indonesian)
⚠️ Smaller community
⚠️ Less documentation
```

**Biznet Gio NEO Lite - Rp 150k/month**
```
✅ 2 vCPU, 4 GB RAM, 60 GB SSD
✅ Jakarta datacenter
✅ Unlimited bandwidth
✅ Good local peering
⚠️ UI less polished
```

---

## Total Cost Comparison (MVP Stack)

### Option A: All-in-One VPS (Cheapest)

Everything runs on single VPS with Docker Compose.

| Provider | VPS | Domain | Cloudflare | Total/Month |
|----------|-----|--------|------------|-------------|
| Contabo | €6.99 | ~$1 | Free | **~$8** |
| Hetzner | €7.50 | ~$1 | Free | **~$9** |
| IDCloudHost | Rp 150k | Rp 15k | Free | **~Rp 165k (~$10)** |
| DigitalOcean | $24 | ~$1 | Free | **~$25** |
| AWS Lightsail | $20 | ~$1 | Free | **~$21** |

### Option B: VPS + Managed Database (Recommended for Production)

| Provider | VPS | Managed DB | Total/Month |
|----------|-----|------------|-------------|
| DigitalOcean | $24 | $15 (PostgreSQL) | **~$40** |
| AWS Lightsail | $20 | $15 (PostgreSQL) | **~$36** |
| Vultr | $24 | $15 (PostgreSQL) | **~$40** |

---

## Architecture Diagram (VPS All-in-One)

```
┌─────────────────────────────────────────────────────────────┐
│                    VPS (Any Provider)                       │
│                    2 vCPU / 4 GB RAM                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                 Docker Compose                       │   │
│  │                                                      │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │   │
│  │  │   Nginx     │  │   Backend   │  │  Frontend   │  │   │
│  │  │  (Reverse   │──│   (Rust)    │  │  (Next.js)  │  │   │
│  │  │   Proxy)    │  │  Port 3000  │  │  Port 3001  │  │   │
│  │  └──────┬──────┘  └──────┬──────┘  └─────────────┘  │   │
│  │         │                │                          │   │
│  │         │         ┌──────┴──────┐                   │   │
│  │         │         │             │                   │   │
│  │  ┌──────▼──────┐  │  ┌──────────▼──────┐           │   │
│  │  │ Let's       │  │  │   PostgreSQL    │           │   │
│  │  │ Encrypt     │  │  │   (Docker)      │           │   │
│  │  │ (SSL)       │  │  └─────────────────┘           │   │
│  │  └─────────────┘  │                                │   │
│  │                   │  ┌─────────────────┐           │   │
│  │                   └──│     Redis       │           │   │
│  │                      │   (Docker)      │           │   │
│  │                      └─────────────────┘           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Persistent Storage                      │   │
│  │  /var/lib/postgresql/data  (PostgreSQL)             │   │
│  │  /var/lib/redis            (Redis)                  │   │
│  │  /opt/webrana/backups      (Daily backups)          │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │
         │ HTTPS (443)
         ▼
┌─────────────────┐     ┌─────────────────┐
│   Cloudflare    │────▶│    Users        │
│   (CDN + WAF)   │     │  (Indonesia)    │
└─────────────────┘     └─────────────────┘
```

---

## Quick Start Guide (Any VPS)

### 1. Initial Server Setup

```bash
# SSH into your VPS
ssh root@YOUR_VPS_IP

# Update system
apt update && apt upgrade -y

# Create non-root user
adduser webrana
usermod -aG sudo webrana

# Setup SSH key authentication
mkdir -p /home/webrana/.ssh
cp ~/.ssh/authorized_keys /home/webrana/.ssh/
chown -R webrana:webrana /home/webrana/.ssh

# Disable root login (optional but recommended)
sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
systemctl restart sshd
```

### 2. Install Docker

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
usermod -aG docker webrana

# Install Docker Compose plugin
apt install docker-compose-plugin -y

# Verify installation
docker --version
docker compose version
```

### 3. Setup Firewall

```bash
# Install UFW
apt install ufw -y

# Allow SSH, HTTP, HTTPS
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp

# Enable firewall
ufw enable
ufw status
```

### 4. Clone and Deploy

```bash
# Switch to webrana user
su - webrana

# Clone repository
git clone https://github.com/your-org/webrana-ai-proxy.git
cd webrana-ai-proxy

# Create production environment file
cat > .env << EOF
# Database
POSTGRES_USER=webrana
POSTGRES_PASSWORD=$(openssl rand -base64 24)
POSTGRES_DB=webrana

# Application
DATABASE_URL=postgresql://webrana:\${POSTGRES_PASSWORD}@postgres:5432/webrana
REDIS_URL=redis://redis:6379
MASTER_ENCRYPTION_KEY=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 64)
RUST_LOG=info
EOF

# Start services
docker compose -f infrastructure/docker/docker-compose.prod.yml up -d

# Check status
docker compose ps
docker compose logs -f
```

### 5. Setup SSL with Certbot

```bash
# Install Certbot
apt install certbot python3-certbot-nginx -y

# Get SSL certificate
certbot --nginx -d webrana.id -d api.webrana.id

# Auto-renewal is configured automatically
certbot renew --dry-run
```

### 6. Setup Automated Backups

```bash
# Create backup script
cat > /opt/webrana/backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/opt/webrana/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup PostgreSQL
docker exec webrana-postgres pg_dump -U webrana webrana | gzip > $BACKUP_DIR/db_$TIMESTAMP.sql.gz

# Keep only last 7 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete

echo "Backup completed: db_$TIMESTAMP.sql.gz"
EOF

chmod +x /opt/webrana/backup.sh

# Add to crontab (daily at 2 AM)
(crontab -l 2>/dev/null; echo "0 2 * * * /opt/webrana/backup.sh >> /var/log/webrana-backup.log 2>&1") | crontab -
```

---

## Production Docker Compose

```yaml
# infrastructure/docker/docker-compose.prod.yml
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    container_name: webrana-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
    depends_on:
      - backend
      - frontend
    networks:
      - webrana

  backend:
    image: webrana/backend:latest
    container_name: webrana-backend
    restart: unless-stopped
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
      - MASTER_ENCRYPTION_KEY=${MASTER_ENCRYPTION_KEY}
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=${RUST_LOG:-info}
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - webrana
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    image: webrana/frontend:latest
    container_name: webrana-frontend
    restart: unless-stopped
    environment:
      - NEXT_PUBLIC_API_URL=https://api.webrana.id
    depends_on:
      - backend
    networks:
      - webrana

  postgres:
    image: postgres:15-alpine
    container_name: webrana-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_USER=${POSTGRES_USER}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
      - POSTGRES_DB=${POSTGRES_DB}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - webrana
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: webrana-redis
    restart: unless-stopped
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - webrana
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:

networks:
  webrana:
    driver: bridge
```

---

## Recommendation Summary

| Scenario | Provider | Cost | Why |
|----------|----------|------|-----|
| **Paling Murah** | Contabo | ~$8/month | Best specs for price, unlimited bandwidth |
| **Best Value** | Hetzner | ~$9/month | Excellent performance, good support |
| **Lokal Indonesia** | IDCloudHost | ~Rp 150k | Lowest latency, Rupiah billing |
| **Easiest Setup** | DigitalOcean | ~$25/month | Best docs, 1-click apps, managed DB option |
| **AWS Ecosystem** | Lightsail | ~$21/month | Easy upgrade to full AWS later |

**Rekomendasi untuk Webrana MVP:**
1. **Development/Testing**: Contabo atau Hetzner (~$8-9/month)
2. **Production (Budget)**: IDCloudHost atau Biznet Gio (~Rp 150k/month)
3. **Production (Reliable)**: DigitalOcean atau Vultr (~$24/month)
