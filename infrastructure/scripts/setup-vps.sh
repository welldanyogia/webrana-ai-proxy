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

echo -e "${YELLOW}Step 1: Updating system...${NC}"
apt update && apt upgrade -y

echo -e "${YELLOW}Step 2: Installing essential packages...${NC}"
apt install -y \
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

echo -e "${YELLOW}Step 4: Setting up SSH...${NC}"
# Copy SSH keys if they exist
if [ -d "/root/.ssh" ]; then
    mkdir -p /home/$USERNAME/.ssh
    cp /root/.ssh/authorized_keys /home/$USERNAME/.ssh/ 2>/dev/null || true
    chown -R $USERNAME:$USERNAME /home/$USERNAME/.ssh
    chmod 700 /home/$USERNAME/.ssh
    chmod 600 /home/$USERNAME/.ssh/authorized_keys 2>/dev/null || true
fi

# Secure SSH
sed -i 's/#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
systemctl restart sshd

echo -e "${YELLOW}Step 5: Installing Docker...${NC}"
if command -v docker &> /dev/null; then
    echo "Docker already installed"
else
    curl -fsSL https://get.docker.com | sh
    usermod -aG docker $USERNAME
fi

echo -e "${YELLOW}Step 6: Installing Docker Compose...${NC}"
apt install -y docker-compose-plugin

echo -e "${YELLOW}Step 7: Configuring firewall...${NC}"
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable

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
maxretry = 3
EOF
systemctl enable fail2ban
systemctl restart fail2ban

echo -e "${YELLOW}Step 9: Creating deployment directory...${NC}"
mkdir -p $DEPLOY_DIR/backups
chown -R $USERNAME:$USERNAME $DEPLOY_DIR

echo -e "${YELLOW}Step 10: Installing Nginx and Certbot...${NC}"
apt install -y nginx certbot python3-certbot-nginx

echo -e "${YELLOW}Step 11: Setting up swap (2GB)...${NC}"
if [ ! -f /swapfile ]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
fi

echo -e "${YELLOW}Step 12: Optimizing system...${NC}"
# Increase file limits
cat >> /etc/security/limits.conf << 'EOF'
* soft nofile 65535
* hard nofile 65535
EOF

# Optimize sysctl
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
sysctl -p

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}  VPS Setup Complete!                   ${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Login as $USERNAME: ssh $USERNAME@$(curl -s ifconfig.me)"
echo "2. Clone repository: git clone <repo-url> $DEPLOY_DIR/webrana-ai-proxy"
echo "3. Configure .env file"
echo "4. Run deployment: ./infrastructure/scripts/deploy-staging.sh"
echo ""
echo -e "${YELLOW}Important:${NC}"
echo "- Root login is now disabled"
echo "- Password authentication is disabled"
echo "- Firewall is enabled (ports 22, 80, 443)"
echo "- Fail2ban is protecting SSH"
echo ""
echo -e "${GREEN}Server IP: $(curl -s ifconfig.me)${NC}"
