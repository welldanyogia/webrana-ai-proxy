#!/bin/bash
set -e

# =============================================================================
# Webrana AI Proxy - EC2 User Data Script
# Installs Docker, Docker Compose, and sets up the application
# =============================================================================

echo "🚀 Starting Webrana setup..."

# Update system
apt-get update -y
apt-get upgrade -y

# Install required packages
apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    unzip \
    htop \
    nginx \
    certbot \
    python3-certbot-nginx

# Install Docker
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
apt-get update -y
apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Add ubuntu user to docker group
usermod -aG docker ubuntu

# Install AWS CLI v2
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
./aws/install
rm -rf aws awscliv2.zip

# Create application directory
mkdir -p /opt/webrana
chown ubuntu:ubuntu /opt/webrana

# Create docker-compose.yml for production
cat > /opt/webrana/docker-compose.yml << 'EOF'
version: '3.8'

services:
  backend:
    image: webrana/backend:latest
    container_name: webrana-backend
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://webrana:${DB_PASSWORD}@postgres:5432/webrana
      - REDIS_URL=redis://redis:6379
      - MASTER_ENCRYPTION_KEY=${MASTER_ENCRYPTION_KEY}
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - webrana-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    image: webrana/frontend:latest
    container_name: webrana-frontend
    restart: unless-stopped
    ports:
      - "3001:3000"
    environment:
      - NEXT_PUBLIC_API_URL=https://api.${domain_name}
    depends_on:
      - backend
    networks:
      - webrana-network

  postgres:
    image: postgres:15-alpine
    container_name: webrana-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_USER=webrana
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=webrana
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - webrana-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U webrana"]
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
      - webrana-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:

networks:
  webrana-network:
    driver: bridge
EOF

# Create .env template
cat > /opt/webrana/.env.template << 'EOF'
# Database
DB_PASSWORD=CHANGE_ME_GENERATE_SECURE_PASSWORD

# Encryption (generate with: openssl rand -base64 32)
MASTER_ENCRYPTION_KEY=CHANGE_ME_GENERATE_WITH_OPENSSL

# JWT Secret (generate with: openssl rand -base64 64)
JWT_SECRET=CHANGE_ME_GENERATE_WITH_OPENSSL
EOF

# Create Nginx configuration
cat > /etc/nginx/sites-available/webrana << 'EOF'
# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name ${domain_name} api.${domain_name};
    return 301 https://$server_name$request_uri;
}

# API Backend
server {
    listen 443 ssl http2;
    server_name api.${domain_name};

    # SSL will be configured by certbot
    # ssl_certificate /etc/letsencrypt/live/api.${domain_name}/fullchain.pem;
    # ssl_certificate_key /etc/letsencrypt/live/api.${domain_name}/privkey.pem;

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
        proxy_read_timeout 300s;
        proxy_connect_timeout 75s;
    }
}

# Frontend
server {
    listen 443 ssl http2;
    server_name ${domain_name};

    # SSL will be configured by certbot
    # ssl_certificate /etc/letsencrypt/live/${domain_name}/fullchain.pem;
    # ssl_certificate_key /etc/letsencrypt/live/${domain_name}/privkey.pem;

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
EOF

# Enable Nginx site
ln -sf /etc/nginx/sites-available/webrana /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Create backup script
cat > /opt/webrana/backup.sh << 'BACKUP_EOF'
#!/bin/bash
set -e

BACKUP_DIR="/tmp/webrana-backup"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
# S3 bucket name will be set after terraform apply
S3_BUCKET=$(aws s3 ls | grep webrana-backups | awk '{print $3}')

mkdir -p $BACKUP_DIR

# Backup PostgreSQL
docker exec webrana-postgres pg_dump -U webrana webrana > $BACKUP_DIR/postgres_$TIMESTAMP.sql

# Compress
tar -czf $BACKUP_DIR/webrana_backup_$TIMESTAMP.tar.gz -C $BACKUP_DIR postgres_$TIMESTAMP.sql

# Upload to S3
aws s3 cp $BACKUP_DIR/webrana_backup_$TIMESTAMP.tar.gz s3://$S3_BUCKET/

# Cleanup
rm -rf $BACKUP_DIR

echo "Backup completed: webrana_backup_$TIMESTAMP.tar.gz"
BACKUP_EOF

chmod +x /opt/webrana/backup.sh

# Add backup cron job (daily at 2 AM)
echo "0 2 * * * /opt/webrana/backup.sh >> /var/log/webrana-backup.log 2>&1" | crontab -u ubuntu -

# Set permissions
chown -R ubuntu:ubuntu /opt/webrana

echo "✅ Webrana setup complete!"
echo ""
echo "Next steps:"
echo "1. SSH into the server"
echo "2. cd /opt/webrana"
echo "3. Copy .env.template to .env and fill in secrets"
echo "4. Run: docker compose up -d"
echo "5. Setup SSL: sudo certbot --nginx -d ${domain_name} -d api.${domain_name}"
