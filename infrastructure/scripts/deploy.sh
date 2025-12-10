#!/bin/bash
# =============================================================================
# Webrana AI Proxy - Deployment Script
# Usage: ./deploy.sh [build|deploy|restart|logs|backup|ssl]
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="infrastructure/docker/docker-compose.prod.yml"
PROJECT_NAME="webrana"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if .env exists
check_env() {
    if [ ! -f "infrastructure/docker/.env" ]; then
        log_error ".env file not found!"
        log_info "Copy .env.example to .env and fill in your values:"
        log_info "  cp infrastructure/docker/.env.example infrastructure/docker/.env"
        exit 1
    fi
}

# Build Docker images
build() {
    log_info "Building Docker images..."
    
    # Build backend
    docker build -t webrana/backend:latest -f infrastructure/docker/Dockerfile.backend .
    
    # Build frontend
    docker build -t webrana/frontend:latest -f infrastructure/docker/Dockerfile.frontend frontend/
    
    log_info "Build complete!"
}

# Deploy services
deploy() {
    check_env
    log_info "Deploying services..."
    
    cd infrastructure/docker
    docker compose -f docker-compose.prod.yml -p $PROJECT_NAME up -d
    cd ../..
    
    log_info "Deployment complete!"
    log_info "Checking service health..."
    sleep 5
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME ps
}

# Restart services
restart() {
    log_info "Restarting services..."
    
    cd infrastructure/docker
    docker compose -f docker-compose.prod.yml -p $PROJECT_NAME restart
    cd ../..
    
    log_info "Restart complete!"
}

# View logs
logs() {
    SERVICE=${2:-""}
    if [ -z "$SERVICE" ]; then
        docker compose -f $COMPOSE_FILE -p $PROJECT_NAME logs -f
    else
        docker compose -f $COMPOSE_FILE -p $PROJECT_NAME logs -f $SERVICE
    fi
}

# Backup database
backup() {
    log_info "Creating database backup..."
    
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_DIR="infrastructure/docker/backups"
    mkdir -p $BACKUP_DIR
    
    docker exec webrana-postgres pg_dump -U webrana webrana | gzip > $BACKUP_DIR/db_$TIMESTAMP.sql.gz
    
    log_info "Backup created: $BACKUP_DIR/db_$TIMESTAMP.sql.gz"
    
    # Keep only last 7 backups
    ls -t $BACKUP_DIR/*.sql.gz | tail -n +8 | xargs -r rm
    log_info "Old backups cleaned up (keeping last 7)"
}

# Setup SSL with Let's Encrypt
ssl() {
    DOMAIN=${2:-"webrana.id"}
    log_info "Setting up SSL for $DOMAIN..."
    
    # Stop nginx temporarily
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME stop nginx 2>/dev/null || true
    
    # Get certificate
    sudo certbot certonly --standalone \
        -d $DOMAIN \
        -d api.$DOMAIN \
        -d www.$DOMAIN \
        --non-interactive \
        --agree-tos \
        --email admin@$DOMAIN
    
    # Start nginx
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME up -d nginx
    
    log_info "SSL setup complete!"
}

# Update and redeploy
update() {
    log_info "Pulling latest changes..."
    git pull origin main
    
    log_info "Rebuilding images..."
    build
    
    log_info "Redeploying..."
    deploy
    
    log_info "Update complete!"
}

# Show status
status() {
    log_info "Service status:"
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME ps
    
    echo ""
    log_info "Resource usage:"
    docker stats --no-stream webrana-backend webrana-frontend webrana-postgres webrana-redis webrana-nginx 2>/dev/null || true
}

# Stop all services
stop() {
    log_info "Stopping all services..."
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME down
    log_info "All services stopped."
}

# Clean up (remove volumes)
clean() {
    log_warn "This will remove all data including database!"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        docker compose -f $COMPOSE_FILE -p $PROJECT_NAME down -v
        log_info "Cleanup complete."
    else
        log_info "Cleanup cancelled."
    fi
}

# Help
help() {
    echo "Webrana AI Proxy - Deployment Script"
    echo ""
    echo "Usage: ./deploy.sh [command]"
    echo ""
    echo "Commands:"
    echo "  build     Build Docker images"
    echo "  deploy    Deploy all services"
    echo "  restart   Restart all services"
    echo "  update    Pull, build, and redeploy"
    echo "  stop      Stop all services"
    echo "  status    Show service status"
    echo "  logs      View logs (optional: service name)"
    echo "  backup    Backup database"
    echo "  ssl       Setup SSL certificate"
    echo "  clean     Remove all containers and volumes"
    echo "  help      Show this help message"
}

# Main
case "$1" in
    build)
        build
        ;;
    deploy)
        deploy
        ;;
    restart)
        restart
        ;;
    update)
        update
        ;;
    stop)
        stop
        ;;
    status)
        status
        ;;
    logs)
        logs "$@"
        ;;
    backup)
        backup
        ;;
    ssl)
        ssl "$@"
        ;;
    clean)
        clean
        ;;
    help|--help|-h)
        help
        ;;
    *)
        log_error "Unknown command: $1"
        help
        exit 1
        ;;
esac
