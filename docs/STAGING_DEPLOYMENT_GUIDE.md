# Staging Deployment Guide - Webrana AI Proxy

## Overview

Panduan ini untuk deploy Webrana ke staging environment menggunakan VPS (DigitalOcean, Vultr, Hetzner, atau provider lokal Indonesia).

**Target:** Single VPS dengan Docker Compose untuk staging/MVP.

---

## Staging Readiness Checklist

### ✅ Code Ready
- [x] Backend services (Rust/Axum)
- [x] Frontend (Next.js 16)
- [x] Database migrations
- [x] Onboarding & Analytics services
- [x] Email service dengan templates
- [x] Scheduler service untuk cron jobs
- [x] Property-based tests (214 tests pass)

### ⏳ Infrastructure Needed
- [ ] VPS provisioned
- [ ] Domain configured
- [ ] SSL certificates
- [ ] Environment variables set

---

## Minimum VPS Specifications

### Staging Environment (Recommended)

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 vCPU | 2-4 vCPU |
| RAM | 4 GB | 4-8 GB |
| Storage | 40 GB SSD | 80 GB NVMe |
| Bandwidth | 2 TB/month | 4 TB/month |
| Location | Singapore | Singapore |

### Provider Options (Non-AWS)

| Provider | Plan | Price/Month | Specs | Notes |
|----------|------|-------------|-------|-------|
| **DigitalOcean** | Basic Droplet | $24 | 2 vCPU, 4GB, 80GB | Best docs, easy setup |
| **Vultr** | Cloud Compute | $24 | 2 vCPU, 4GB, 80GB | Good API, hourly billing |
| **Hetzner** | CX31 | €7.50 (~$8) | 2 vCPU, 8GB, 80GB | Best value, Germany DC |
| **Contabo** | VPS S | €6.99 (~$7.50) | 4 vCPU, 8GB, 100GB | Cheapest, Singapore DC |
| **IDCloudHost** | VPS KVM | Rp 150k (~$9.50) | 2 vCPU, 4GB, 40GB | Jakarta DC, lowest latency |

**Rekomendasi untuk Staging:**
- **Budget:** Contabo atau Hetzner (~$8/month)
- **Reliable:** DigitalOcean atau Vultr (~$24/month)
- **Lokal Indonesia:** IDCloudHost (~Rp 150k/month)

---

## Architecture (Single VPS)

```
┌─────────────────────────────────────────────────────────────┐
│                    VPS (2 vCPU / 4 GB RAM)                  │
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
│  │  ┌──────▼──────┐  ┌──────┴──────┐                   │   │
│  │  │ Let's       │  │             │                   │   │
│  │  │ Encrypt     │  │  ┌──────────▼──────┐           │   │
│  │  │ (SSL)       │  │  │   PostgreSQL    │           │   │
│  │  └─────────────┘  │  │   (Docker)      │           │   │
│  │                   │  └─────────────────┘           │   │
│  │                   │                                │   │
│  │                   │  ┌─────────────────┐           │   │
│  │                   └──│     Redis       │           │   │
│  │                      │   (Docker)      │           │   │
│  │                      └─────────────────┘           │   │
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

## Step-by-Step Deployment

### Step 1: Provision VPS

#### DigitalOcean
```bash
# Using doctl CLI
doctl compute droplet create webrana-staging \
  --region sgp1 \
  --size s-2vcpu-4gb \
  --image ubuntu-22-04-x64 \
  --ssh-keys <YOUR_SSH_KEY_ID>
```

#### Vultr
```bash
# Using vultr-cli
vultr-cli instance create \
  --region sgp \
  --plan vc2-2c-4gb \
  --os 387 \
  --ssh-keys <YOUR_SSH_KEY_ID>
```

#### Manual (Any Provider)
1. Create Ubuntu 22.04 LTS instance
2. Select Singapore region
3. Choose 2 vCPU / 4 GB RAM plan
4. Add your SSH key

### Step 2: Initial Server Setup

```bash
# SSH into your VPS
ssh root@YOUR_VPS_IP

# Update system
apt update && apt upgrade -y

# Create non-root user
adduser webrana
usermod -aG sudo webrana

# Setup SSH key for new user
mkdir -p /home/webrana/.ssh
cp ~/.ssh/authorized_keys /home/webrana/.ssh/
chown -R webrana:webrana /home/webrana/.ssh
chmod 700 /home/webrana/.ssh
chmod 600 /home/webrana/.ssh/authorized_keys

# Disable root login
sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
systemctl restart sshd
```

### Step 3: Install Docker

```bash
# Switch to webrana user
su - webrana

# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker webrana

# Install Docker Compose plugin
sudo apt install docker-compose-plugin -y

# Verify installation
docker --version
docker compose version

# Re-login to apply group changes
exit
ssh webrana@YOUR_VPS_IP
```

### Step 4: Setup Firewall

```bash
# Install UFW
sudo apt install ufw -y

# Allow SSH, HTTP, HTTPS
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable
sudo ufw status
```

### Step 5: Clone Repository

```bash
# Clone repository
git clone https://github.com/your-org/webrana-ai-proxy.git
cd webrana-ai-proxy

# Create directories
sudo mkdir -p /opt/webrana/backups
sudo chown -R webrana:webrana /opt/webrana
```

### Step 6: Configure Environment

```bash
# Create production environment file
cat > .env << EOF
# ===========================================
# Webrana Staging Environment
# ===========================================

# Database
POSTGRES_USER=webrana
POSTGRES_PASSWORD=$(openssl rand -base64 24)
POSTGRES_DB=webrana

# Application
DATABASE_URL=postgresql://webrana:\${POSTGRES_PASSWORD}@postgres:5432/webrana
REDIS_URL=redis://redis:6379

# Security (GENERATE NEW KEYS!)
MASTER_ENCRYPTION_KEY=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 64)

# Logging
RUST_LOG=info

# Frontend
NEXT_PUBLIC_API_URL=https://api.staging.webrana.id

# Email (Resend)
RESEND_API_KEY=re_xxxxxxxxxxxx

# Payment (Midtrans Sandbox)
MIDTRANS_SERVER_KEY=SB-Mid-server-xxxx
MIDTRANS_CLIENT_KEY=SB-Mid-client-xxxx
MIDTRANS_IS_SANDBOX=true
EOF

# Secure the file
chmod 600 .env
```

### Step 7: Build and Deploy

```bash
# Build Docker images
docker compose -f infrastructure/docker/docker-compose.prod.yml build

# Start services
docker compose -f infrastructure/docker/docker-compose.prod.yml up -d

# Check status
docker compose ps
docker compose logs -f
```

### Step 8: Run Database Migrations

```bash
# Run migrations
docker compose exec backend ./migrate

# Or manually
docker compose exec postgres psql -U webrana -d webrana -f /migrations/20241209001_create_users.sql
```

### Step 9: Setup SSL with Certbot

```bash
# Install Nginx and Certbot
sudo apt install nginx certbot python3-certbot-nginx -y

# Create Nginx config
sudo tee /etc/nginx/sites-available/webrana << 'EOF'
server {
    listen 80;
    server_name staging.webrana.id api.staging.webrana.id;

    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl http2;
    server_name staging.webrana.id;

    ssl_certificate /etc/letsencrypt/live/staging.webrana.id/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/staging.webrana.id/privkey.pem;

    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}

server {
    listen 443 ssl http2;
    server_name api.staging.webrana.id;

    ssl_certificate /etc/letsencrypt/live/staging.webrana.id/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/staging.webrana.id/privkey.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/webrana /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Get SSL certificate
sudo certbot --nginx -d staging.webrana.id -d api.staging.webrana.id

# Verify auto-renewal
sudo certbot renew --dry-run
```

### Step 10: Setup Automated Backups

```bash
# Create backup script
sudo tee /opt/webrana/backup.sh << 'EOF'
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

sudo chmod +x /opt/webrana/backup.sh

# Add to crontab (daily at 2 AM)
(crontab -l 2>/dev/null; echo "0 2 * * * /opt/webrana/backup.sh >> /var/log/webrana-backup.log 2>&1") | crontab -
```

---

## Verification

### Health Checks

```bash
# Check all services
docker compose ps

# Check backend health
curl -f http://localhost:3000/health

# Check frontend
curl -f http://localhost:3001

# Check database
docker compose exec postgres pg_isready -U webrana

# Check Redis
docker compose exec redis redis-cli ping
```

### Test Endpoints

```bash
# API health
curl https://api.staging.webrana.id/health

# Frontend
curl https://staging.webrana.id

# Test auth endpoint
curl -X POST https://api.staging.webrana.id/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!","name":"Test User"}'
```

---

## Troubleshooting

### Common Issues

**1. Docker permission denied**
```bash
sudo usermod -aG docker $USER
# Then logout and login again
```

**2. Port already in use**
```bash
sudo lsof -i :3000
sudo kill -9 <PID>
```

**3. Database connection failed**
```bash
# Check PostgreSQL logs
docker compose logs postgres

# Verify connection
docker compose exec postgres psql -U webrana -d webrana -c "SELECT 1"
```

**4. SSL certificate issues**
```bash
# Check certificate status
sudo certbot certificates

# Force renewal
sudo certbot renew --force-renewal
```

### Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f postgres

# Nginx
sudo tail -f /var/log/nginx/error.log
```

---

## Monitoring (Optional)

### Basic Monitoring with htop

```bash
sudo apt install htop -y
htop
```

### Docker Stats

```bash
docker stats
```

### Setup Uptime Monitoring

Use free services like:
- [UptimeRobot](https://uptimerobot.com/) - Free 50 monitors
- [Freshping](https://www.freshworks.com/website-monitoring/) - Free 50 monitors
- [Healthchecks.io](https://healthchecks.io/) - Free cron monitoring

---

## Next Steps

After staging is deployed:

1. **Test all features** - Auth, API keys, proxy, billing
2. **Load testing** - Use k6 script from `infrastructure/scripts/load-test.js`
3. **Security audit** - Follow `docs/SECURITY_AUDIT_GUIDE.md`
4. **Setup monitoring** - Grafana + Prometheus (optional)
5. **Configure Cloudflare** - CDN, WAF, rate limiting
6. **Production deployment** - Scale to 3-node K8s cluster
