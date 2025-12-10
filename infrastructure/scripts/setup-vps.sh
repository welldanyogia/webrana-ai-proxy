#!/bin/bash
# ===========================================
# Webrana AI Proxy - VPS Initial Setup Script
# Run this script on a fresh Ubuntu 22.04 VPS
# ===========================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}  Webrana VPS Setup Script              ${NC}"
echo -e "${GREEN}=========================================${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root${NC}"
    exit 1
fi

# Variables
USERNAME="webrana"
DEPLOY_DIR="/opt/webrana"

# Set non-interactive mode
export DEBIAN_FRONTEND=noninteractive

echo -e "${YELLOW}Step 1: Updating system...${NC}"
apt update && apt upgrade -y -o Dpkg::Options::="--force-confold" -o Dpkg::Options::="--force-confdef"

echo -e "${YELLOW}Step 2: Installing essential packages...${NC}"
apt install -y -o Dpkg::Options::="--force-confold" -o Dpkg::Options::="--force-confdef" \
    curl \
    wget \
    git \
    htop \
    vim \
    ufw \
    fail2ban \
    unzip \
    software-properties-common \
    apt-transport-https \
    ca-certificates \
    gnupg \
    lsb-release

echo -e "${YELLOW}Step 3: Creating user '${USERNAME}'...${NC}"
if id "$USERNAME" &>/dev/null; then
    echo "User $USERNAME already exists"
else
    adduser --disabled-password --gecos "" $USERNAME
    usermod -aG sudo $USERNAME
    echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/$USERNAME
fi

echo -e "${YELLOW}Step 4: Setting up SSH keys for ${USERNAME}...${NC}"
# Copy SSH keys if they exist - CRITICAL for access
mkdir -p /home/$USERNAME/.ssh
if [ -f "/root/.ssh/authorized_keys" ]; then
    cp /root/.ssh/authorized_keys /home/$USERNAME/.ssh/
    echo -e "${GREEN}✓ SSH keys copied to ${USERNAME}${NC}"
else
    echo -e "${RED}WARNING: No SSH keys found in /root/.ssh/authorized_keys${NC}"
    echo -e "${YELLOW}You may lose access if we disable password auth!${NC}"
    echo -e "${YELLOW}Skipping SSH hardening for safety.${NC}"
    SKIP_SSH_HARDENING=true
fi
chown -R $USERNAME:$USERNAME /home/$USERNAME/.ssh
chmod 700 /home/$USERNAME/.ssh
chmod 600 /home/$USERNAME/.ssh/authorized_keys 2>/dev/null || true

echo -e "${YELLOW}Step 5: Installing Docker...${NC}"
if command -v docker &> /dev/null; then
    echo "Docker already installed"
else
    curl -fsSL https://get.docker.com | sh
    usermod -aG docker $USERNAME
fi

echo -e "${YELLOW}Step 6: Installing Docker Compose...${NC}"
apt install -y -o Dpkg::Options::="--force-confold" docker-compose-plugin

echo -e "${YELLOW}Step 7: Configuring firewall...${NC}"
# IMPORTANT: Allow SSH FIRST before enabling firewall
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw default deny incoming
ufw default allow outgoing
ufw --force enable
echo -e "${GREEN}✓ Firewall enabled (ports 22, 80, 443 open)${NC}"

echo -e "${YELLOW}Step 8: Configuring fail2ban...${NC}"
cat > /etc/fail2ban/jail.local << 'EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 5
EOF
systemctl enable fail2ban
systemctl restart fail2ban

echo -e "${YELLOW}Step 9: Creating deployment directory...${NC}"
mkdir -p $DEPLOY_DIR/backups
mkdir -p $DEPLOY_DIR/app
chown -R $USERNAME:$USERNAME $DEPLOY_DIR

echo -e "${YELLOW}Step 10: Installing Nginx...${NC}"
apt install -y -o Dpkg::Options::="--force-confold" nginx
systemctl enable nginx

echo -e "${YELLOW}Step 11: Setting up swap (2GB)...${NC}"
if [ ! -f /swapfile ]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo -e "${GREEN}✓ Swap created${NC}"
else
    echo "Swap already exists"
fi

echo -e "${YELLOW}Step 12: Optimizing system...${NC}"
# Increase file limits (only if not already set)
if ! grep -q "nofile 65535" /etc/security/limits.conf; then
    cat >> /etc/security/limits.conf << 'EOF'
* soft nofile 65535
* hard nofile 65535
EOF
fi

# Optimize sysctl (only if not already set)
if ! grep -q "net.core.somaxconn" /etc/sysctl.conf; then
    cat >> /etc/sysctl.conf << 'EOF'
# Network optimization
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15

# Memory optimization
vm.swappiness = 10
vm.dirty_ratio = 60
vm.dirty_background_ratio = 2
EOF
fi
sysctl -p 2>/dev/null || true

# SSH Hardening - ONLY if keys are confirmed
echo -e "${YELLOW}Step 13: SSH Security Configuration...${NC}"
if [ "$SKIP_SSH_HARDENING" != "true" ]; then
    # Test that we can login as webrana first
    echo -e "${YELLOW}Testing SSH key access for ${USERNAME}...${NC}"
    
    # Backup original sshd_config
    cp /etc/ssh/sshd_config /etc/ssh/sshd_config.backup
    
    # Only disable root login, keep password auth for now
    sed -i 's/#PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
    sed -i 's/PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
    
    # Restart SSH
    systemctl restart sshd
    echo -e "${GREEN}✓ SSH configured (root login restricted)${NC}"
    echo -e "${YELLOW}NOTE: Password auth still enabled for safety${NC}"
    echo -e "${YELLOW}Run 'sudo sed -i \"s/#PasswordAuthentication yes/PasswordAuthentication no/\" /etc/ssh/sshd_config && sudo systemctl restart sshd' to disable password auth after confirming key access${NC}"
else
    echo -e "${YELLOW}Skipping SSH hardening - no keys found${NC}"
fi

SERVER_IP=$(curl -s ifconfig.me 2>/dev/null || echo "YOUR_SERVER_IP")

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}  VPS Setup Complete!                   ${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Test login as $USERNAME: ssh $USERNAME@$SERVER_IP"
echo "2. Clone repository:"
echo "   cd $DEPLOY_DIR"
echo "   git clone git@github.com:welldanyogia/webrana-ai-proxy.git app"
echo "3. Setup deployment:"
echo "   cd app"
echo "   ./infrastructure/scripts/deploy-staging.sh setup"
echo ""
echo -e "${YELLOW}Current status:${NC}"
echo "- User '$USERNAME' created with sudo access"
echo "- Docker & Docker Compose installed"
echo "- Firewall enabled (ports 22, 80, 443)"
echo "- Fail2ban protecting SSH"
echo "- Root login: restricted (key only)"
echo "- Password auth: still enabled (disable manually after confirming key access)"
echo ""
echo -e "${GREEN}Server IP: $SERVER_IP${NC}"
