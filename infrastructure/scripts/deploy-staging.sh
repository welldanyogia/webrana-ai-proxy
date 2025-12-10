#!/bin/bash
# ===========================================
# Webrana AI Proxy - Staging Deployment Script
# ===========================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEPLOY_DIR="/opt/webrana"
APP_DIR="${DEPLOY_DIR}/app"
# Use SSH for GitHub (requires deploy key setup)
REPO_URL="${GIT_REPO_URL:-git@github.com:welldanyogia/webrana-ai-proxy.git}"
BRANCH="${BRANCH:-main}"
COMPOSE_FILE="infrastructure/docker/docker-compose.prod.yml"

# Cloudflare Configuration (optional - for cache purging)
CF_ZONE_ID="${CF_ZONE_ID:-}"
CF_API_TOKEN="${CF_API_TOKEN:-}"

# Domain Configuration
DOMAIN="${DOMAIN:-staging.webrana.id}"
API_DOMAIN="${API_DOMAIN:-api.staging.webrana.id}"

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}  Webrana Staging Deployment Script     ${NC}"
echo -e "${GREEN}=========================================${NC}"

# Check if running as correct user
if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Please run as non-root user (webrana)${NC}"
    exit 1
fi

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# Check Docker Compose
if ! docker compose version &> /dev/null; then
    echo -e "${RED}Docker Compose is not installed. Please install Docker Compose first.${NC}"
    exit 1
fi

# Function to check if .env exists
check_env() {
    if [ ! -f ".env" ]; then
        echo -e "${RED}.env file not found!${NC}"
        echo -e "${YELLOW}Creating template .env file...${NC}"
        cat > .env << 'EOF'
# ===========================================
# Webrana Staging Environment
# IMPORTANT: Update these values before deploying!
# ===========================================

# Database
POSTGRES_USER=webrana
POSTGRES_PASSWORD=CHANGE_ME_GENERATE_RANDOM
POSTGRES_DB=webrana

# Application
DATABASE_URL=postgresql://webrana:CHANGE_ME_GENERATE_RANDOM@postgres:5432/webrana
REDIS_URL=redis://redis:6379

# Security (GENERATE NEW KEYS!)
MASTER_ENCRYPTION_KEY=CHANGE_ME_GENERATE_BASE64_32
JWT_SECRET=CHANGE_ME_GENERATE_BASE64_64

# Logging
RUST_LOG=info

# Frontend
NEXT_PUBLIC_API_URL=https://api.staging.webrana.id
NEXTAUTH_URL=https://staging.webrana.id
NEXTAUTH_SECRET=CHANGE_ME_GENERATE_BASE64_32

# Email (Resend)
RESEND_API_KEY=re_xxxxxxxxxxxx

# Payment (Midtrans Sandbox)
MIDTRANS_SERVER_KEY=SB-Mid-server-xxxx
MIDTRANS_CLIENT_KEY=SB-Mid-client-xxxx
MIDTRANS_IS_SANDBOX=true
EOF
        echo -e "${YELLOW}Please edit .env file with your values and run again.${NC}"
        echo -e "${YELLOW}Generate secrets with:${NC}"
        echo "  POSTGRES_PASSWORD: openssl rand -base64 24"
        echo "  MASTER_ENCRYPTION_KEY: openssl rand -base64 32"
        echo "  JWT_SECRET: openssl rand -base64 64"
        exit 1
    fi
}

# Function to pull latest code
pull_code() {
    echo -e "${YELLOW}Pulling latest code from ${BRANCH}...${NC}"
    git fetch origin
    git checkout ${BRANCH}
    git pull origin ${BRANCH}
    echo -e "${GREEN}Code updated successfully${NC}"
}

# Function to build images
build_images() {
    echo -e "${YELLOW}Building Docker images...${NC}"
    docker compose -f ${COMPOSE_FILE} build --no-cache
    echo -e "${GREEN}Images built successfully${NC}"
}

# Function to deploy
deploy() {
    echo -e "${YELLOW}Deploying services...${NC}"
    
    # Stop existing containers
    docker compose -f ${COMPOSE_FILE} down --remove-orphans
    
    # Start new containers
    docker compose -f ${COMPOSE_FILE} up -d
    
    # Wait for services to be healthy
    echo -e "${YELLOW}Waiting for services to be healthy...${NC}"
    sleep 10
    
    # Check health
    docker compose -f ${COMPOSE_FILE} ps
    
    echo -e "${GREEN}Deployment completed successfully${NC}"
}

# Function to run migrations
run_migrations() {
    echo -e "${YELLOW}Running database migrations...${NC}"
    
    # Wait for PostgreSQL to be ready
    until docker compose -f ${COMPOSE_FILE} exec -T postgres pg_isready -U webrana; do
        echo "Waiting for PostgreSQL..."
        sleep 2
    done
    
    # Run migrations (if migration command exists)
    # docker compose -f ${COMPOSE_FILE} exec -T backend ./migrate
    
    echo -e "${GREEN}Migrations completed${NC}"
}

# Function to verify deployment
verify() {
    echo -e "${YELLOW}Verifying deployment...${NC}"
    
    # Check backend health
    if curl -sf http://localhost:3000/health > /dev/null; then
        echo -e "${GREEN}✓ Backend is healthy${NC}"
    else
        echo -e "${RED}✗ Backend health check failed${NC}"
        docker compose -f ${COMPOSE_FILE} logs backend --tail=50
        exit 1
    fi
    
    # Check frontend
    if curl -sf http://localhost:3001 > /dev/null; then
        echo -e "${GREEN}✓ Frontend is healthy${NC}"
    else
        echo -e "${RED}✗ Frontend health check failed${NC}"
        docker compose -f ${COMPOSE_FILE} logs frontend --tail=50
        exit 1
    fi
    
    # Check PostgreSQL
    if docker compose -f ${COMPOSE_FILE} exec -T postgres pg_isready -U webrana > /dev/null; then
        echo -e "${GREEN}✓ PostgreSQL is healthy${NC}"
    else
        echo -e "${RED}✗ PostgreSQL health check failed${NC}"
        exit 1
    fi
    
    # Check Redis
    if docker compose -f ${COMPOSE_FILE} exec -T redis redis-cli ping > /dev/null; then
        echo -e "${GREEN}✓ Redis is healthy${NC}"
    else
        echo -e "${RED}✗ Redis health check failed${NC}"
        exit 1
    fi
    
    # Check external endpoints (via Cloudflare)
    echo -e "${YELLOW}Checking external endpoints...${NC}"
    if curl -sf "https://${DOMAIN}" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Frontend accessible via https://${DOMAIN}${NC}"
    else
        echo -e "${YELLOW}⚠ Frontend not accessible externally (check Cloudflare/DNS)${NC}"
    fi
    
    if curl -sf "https://${API_DOMAIN}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ API accessible via https://${API_DOMAIN}${NC}"
    else
        echo -e "${YELLOW}⚠ API not accessible externally (check Cloudflare/DNS)${NC}"
    fi
    
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}  All services are running!              ${NC}"
    echo -e "${GREEN}=========================================${NC}"
}

# Function to purge Cloudflare cache
purge_cache() {
    if [ -z "$CF_ZONE_ID" ] || [ -z "$CF_API_TOKEN" ]; then
        echo -e "${YELLOW}Cloudflare credentials not set, skipping cache purge${NC}"
        echo -e "${YELLOW}Set CF_ZONE_ID and CF_API_TOKEN to enable${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}Purging Cloudflare cache...${NC}"
    
    RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${CF_ZONE_ID}/purge_cache" \
        -H "Authorization: Bearer ${CF_API_TOKEN}" \
        -H "Content-Type: application/json" \
        --data '{"purge_everything":true}')
    
    if echo "$RESPONSE" | grep -q '"success":true'; then
        echo -e "${GREEN}✓ Cloudflare cache purged successfully${NC}"
    else
        echo -e "${RED}✗ Failed to purge Cloudflare cache${NC}"
        echo "$RESPONSE"
    fi
}

# Function to setup SSH for GitHub
setup_ssh() {
    echo -e "${YELLOW}Setting up SSH for GitHub...${NC}"
    
    # Check if deploy key exists
    if [ ! -f ~/.ssh/deploy_key ]; then
        echo -e "${YELLOW}Generating deploy key...${NC}"
        ssh-keygen -t ed25519 -C "webrana-deploy" -f ~/.ssh/deploy_key -N ""
        
        echo -e "${GREEN}Deploy key generated!${NC}"
        echo -e "${YELLOW}Add this public key to GitHub repo → Settings → Deploy keys:${NC}"
        echo ""
        cat ~/.ssh/deploy_key.pub
        echo ""
        echo -e "${YELLOW}Press Enter after adding the key to GitHub...${NC}"
        read
    fi
    
    # Setup SSH config
    if ! grep -q "github.com" ~/.ssh/config 2>/dev/null; then
        cat >> ~/.ssh/config << 'EOF'
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/deploy_key
    StrictHostKeyChecking no
EOF
        chmod 600 ~/.ssh/config
    fi
    
    # Test connection
    echo -e "${YELLOW}Testing GitHub SSH connection...${NC}"
    if ssh -T git@github.com 2>&1 | grep -q "successfully authenticated"; then
        echo -e "${GREEN}✓ GitHub SSH connection successful${NC}"
    else
        echo -e "${RED}✗ GitHub SSH connection failed${NC}"
        echo -e "${YELLOW}Make sure deploy key is added to GitHub${NC}"
        exit 1
    fi
}

# Function to setup Cloudflare SSL certificates
setup_cloudflare_ssl() {
    echo -e "${YELLOW}Setting up Cloudflare Origin SSL...${NC}"
    
    SSL_DIR="/etc/nginx/ssl"
    
    if [ -f "${SSL_DIR}/cert.pem" ] && [ -f "${SSL_DIR}/key.pem" ]; then
        echo -e "${GREEN}✓ SSL certificates already exist${NC}"
        return 0
    fi
    
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}  Cloudflare Origin Certificate Setup   ${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
    echo "1. Go to Cloudflare Dashboard → SSL/TLS → Origin Server"
    echo "2. Click 'Create Certificate'"
    echo "3. Keep defaults (RSA 2048, 15 years)"
    echo "4. Add hostnames: ${DOMAIN}, ${API_DOMAIN}, *.webrana.id"
    echo "5. Click 'Create'"
    echo ""
    
    sudo mkdir -p ${SSL_DIR}
    
    echo -e "${YELLOW}Paste the Origin Certificate (PEM format), then press Ctrl+D:${NC}"
    sudo tee ${SSL_DIR}/cert.pem > /dev/null
    
    echo -e "${YELLOW}Paste the Private Key (PEM format), then press Ctrl+D:${NC}"
    sudo tee ${SSL_DIR}/key.pem > /dev/null
    
    sudo chmod 600 ${SSL_DIR}/*.pem
    
    echo -e "${GREEN}✓ SSL certificates saved${NC}"
}

# Function to show logs
show_logs() {
    docker compose -f ${COMPOSE_FILE} logs -f
}

# Function to rollback
rollback() {
    echo -e "${YELLOW}Rolling back to previous version...${NC}"
    git checkout HEAD~1
    docker compose -f ${COMPOSE_FILE} down
    docker compose -f ${COMPOSE_FILE} up -d
    echo -e "${GREEN}Rollback completed${NC}"
}

# Main script
case "${1:-deploy}" in
    deploy)
        check_env
        pull_code
        build_images
        deploy
        run_migrations
        verify
        purge_cache
        ;;
    build)
        build_images
        ;;
    start)
        docker compose -f ${COMPOSE_FILE} up -d
        ;;
    stop)
        docker compose -f ${COMPOSE_FILE} down
        ;;
    restart)
        docker compose -f ${COMPOSE_FILE} restart
        ;;
    logs)
        show_logs
        ;;
    status)
        docker compose -f ${COMPOSE_FILE} ps
        ;;
    verify)
        verify
        ;;
    rollback)
        rollback
        ;;
    purge-cache)
        purge_cache
        ;;
    setup-ssh)
        setup_ssh
        ;;
    setup-ssl)
        setup_cloudflare_ssl
        ;;
    setup)
        setup_ssh
        setup_cloudflare_ssl
        check_env
        ;;
    *)
        echo "Usage: $0 {deploy|build|start|stop|restart|logs|status|verify|rollback|purge-cache|setup-ssh|setup-ssl|setup}"
        echo ""
        echo "Commands:"
        echo "  deploy      - Full deployment (pull, build, deploy, verify, purge cache)"
        echo "  build       - Build Docker images only"
        echo "  start       - Start services"
        echo "  stop        - Stop services"
        echo "  restart     - Restart services"
        echo "  logs        - View logs"
        echo "  status      - Show service status"
        echo "  verify      - Verify deployment health"
        echo "  rollback    - Rollback to previous version"
        echo "  purge-cache - Purge Cloudflare cache"
        echo "  setup-ssh   - Setup SSH deploy key for GitHub"
        echo "  setup-ssl   - Setup Cloudflare Origin SSL certificates"
        echo "  setup       - Run all setup steps (ssh, ssl, env check)"
        exit 1
        ;;
esac
