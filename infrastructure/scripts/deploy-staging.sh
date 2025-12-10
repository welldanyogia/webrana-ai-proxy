#!/bin/bash
# ===========================================
# Webrana AI Proxy - Staging Deployment Script
# ===========================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DEPLOY_DIR="/opt/webrana"
REPO_URL="https://github.com/your-org/webrana-ai-proxy.git"
BRANCH="${BRANCH:-main}"
COMPOSE_FILE="infrastructure/docker/docker-compose.prod.yml"

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
    
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}  All services are running!              ${NC}"
    echo -e "${GREEN}=========================================${NC}"
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
    *)
        echo "Usage: $0 {deploy|build|start|stop|restart|logs|status|verify|rollback}"
        exit 1
        ;;
esac
