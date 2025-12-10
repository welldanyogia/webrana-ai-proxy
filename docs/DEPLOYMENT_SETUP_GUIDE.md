# Webrana AI Proxy - Deployment Setup Guide

Panduan lengkap untuk setup SSH keys, GitHub, Cloudflare, dan deployment ke VPS.

## Table of Contents
1. [Setup SSH Keys untuk GitHub](#1-setup-ssh-keys-untuk-github)
2. [Setup SSH Keys untuk VPS](#2-setup-ssh-keys-untuk-vps)
3. [Setup Cloudflare](#3-setup-cloudflare)
4. [Deployment ke VPS](#4-deployment-ke-vps)

---

## 1. Setup SSH Keys untuk GitHub

### A. Generate SSH Key (di Local Machine)

**Windows (PowerShell/Git Bash):**
```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "welldanyogia01@gmail.com"

# Jika sistem tidak support ed25519, gunakan RSA
ssh-keygen -t rsa -b 4096 -C "welldanyogia01@gmai.com"
```

Saat diminta:
- **File location:** Tekan Enter untuk default (`~/.ssh/id_ed25519`)
- **Passphrase:** Masukkan passphrase (opsional tapi recommended)

### B. Tambahkan SSH Key ke SSH Agent

**Windows (PowerShell sebagai Admin):**
```powershell
# Start SSH Agent
Get-Service -Name ssh-agent | Set-Service -StartupType Manual
Start-Service ssh-agent

# Tambahkan key
ssh-add $env:USERPROFILE\.ssh\id_ed25519
```

**Windows (Git Bash):**
```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

### C. Copy Public Key

**Windows (PowerShell):**
```powershell
Get-Content $env:USERPROFILE\.ssh\id_ed25519.pub | Set-Clipboard
```

**Git Bash:**
```bash
cat ~/.ssh/id_ed25519.pub | clip
```

### D. Tambahkan ke GitHub

1. Buka https://github.com/settings/keys
2. Klik **"New SSH key"**
3. Title: `Webrana Dev Machine` (atau nama yang deskriptif)
4. Key type: **Authentication Key**
5. Paste public key
6. Klik **"Add SSH key"**

### E. Test Koneksi

```bash
ssh -T git@github.com
```

Output yang diharapkan:
```
Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

### F. Update Git Remote ke SSH

```bash
# Cek remote saat ini
git remote -v

# Ubah dari HTTPS ke SSH
git remote set-url origin git@github.com:welldanyogia/webrana-ai-proxy.git

# Verifikasi
git remote -v
```

---

## 2. Setup SSH Keys untuk VPS

### A. Generate SSH Key untuk VPS (jika belum ada)

Gunakan key yang sama dengan GitHub atau buat key terpisah:

```bash
# Key terpisah untuk VPS (recommended untuk security)
ssh-keygen -t ed25519 -C "webrana-vps" -f ~/.ssh/webrana_vps
```

### B. Copy Public Key ke VPS

**Opsi 1: Menggunakan ssh-copy-id (Linux/Mac/Git Bash)**
```bash
ssh-copy-id -i ~/.ssh/webrana_vps.pub root@YOUR_VPS_IP
```

**Opsi 2: Manual Copy**
```bash
# Copy public key
cat ~/.ssh/webrana_vps.pub

# SSH ke VPS dengan password
ssh root@YOUR_VPS_IP

# Di VPS, tambahkan key
mkdir -p ~/.ssh
echo "PASTE_PUBLIC_KEY_HERE" >> ~/.ssh/authorized_keys
chmod 700 ~/.ssh
chmod 600 ~/.ssh/authorized_keys
```

### C. Setup SSH Config (Local Machine)

Buat/edit file `~/.ssh/config`:

```
# GitHub
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519

# Webrana VPS Staging
Host webrana-staging
    HostName YOUR_VPS_IP
    User webrana
    IdentityFile ~/.ssh/webrana_vps
    Port 22

# Webrana VPS Production
Host webrana-prod
    HostName YOUR_PROD_VPS_IP
    User webrana
    IdentityFile ~/.ssh/webrana_vps
    Port 22
```

Sekarang bisa SSH dengan:
```bash
ssh webrana-staging
```

### D. Setup Deploy Key di VPS (untuk git pull)

Di VPS:
```bash
# Login sebagai user webrana
sudo su - webrana

# Generate deploy key
ssh-keygen -t ed25519 -C "webrana-vps-deploy" -f ~/.ssh/deploy_key

# Tampilkan public key
cat ~/.ssh/deploy_key.pub
```

Tambahkan ke GitHub:
1. Buka repo → Settings → Deploy keys
2. Klik "Add deploy key"
3. Title: `Webrana VPS Deploy`
4. Paste public key
5. ✅ Allow write access (jika perlu push)
6. Klik "Add key"

Di VPS, setup SSH config:
```bash
cat >> ~/.ssh/config << 'EOF'
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/deploy_key
EOF

chmod 600 ~/.ssh/config
```

Test:
```bash
ssh -T git@github.com
```

---

## 3. Setup Cloudflare

### A. Tambahkan Domain ke Cloudflare

1. Login ke https://dash.cloudflare.com
2. Klik **"Add a Site"**
3. Masukkan domain: `webrana.id`
4. Pilih plan **Free**
5. Cloudflare akan scan DNS records existing

### B. Update Nameservers

Di domain registrar (Niagahoster/Namecheap/dll):
1. Ubah nameservers ke yang diberikan Cloudflare:
   - `xxx.ns.cloudflare.com`
   - `yyy.ns.cloudflare.com`
2. Tunggu propagasi (bisa sampai 24 jam)

### C. Setup DNS Records

Di Cloudflare Dashboard → DNS → Records:

| Type | Name | Content | Proxy | TTL |
|------|------|---------|-------|-----|
| A | `@` | `YOUR_VPS_IP` | ✅ Proxied | Auto |
| A | `api` | `YOUR_VPS_IP` | ✅ Proxied | Auto |
| A | `staging` | `YOUR_STAGING_IP` | ✅ Proxied | Auto |
| A | `api.staging` | `YOUR_STAGING_IP` | ✅ Proxied | Auto |
| CNAME | `www` | `webrana.id` | ✅ Proxied | Auto |

### D. SSL/TLS Settings

Di Cloudflare Dashboard → SSL/TLS:

1. **Overview:** Pilih **Full (strict)**
2. **Edge Certificates:**
   - Always Use HTTPS: ✅ ON
   - Automatic HTTPS Rewrites: ✅ ON
   - Minimum TLS Version: TLS 1.2
3. **Origin Server:**
   - Klik "Create Certificate"
   - Generate Cloudflare Origin Certificate (15 tahun)
   - Download certificate dan private key
   - Simpan di VPS:
     ```bash
     sudo mkdir -p /etc/cloudflare
     sudo nano /etc/cloudflare/cert.pem    # Paste certificate
     sudo nano /etc/cloudflare/key.pem     # Paste private key
     sudo chmod 600 /etc/cloudflare/*.pem
     ```

### E. Security Settings

Di Cloudflare Dashboard → Security:

1. **WAF:**
   - Enable Managed Rules (Free tier)
2. **Bots:**
   - Bot Fight Mode: ✅ ON
3. **Settings:**
   - Security Level: Medium
   - Challenge Passage: 30 minutes
   - Browser Integrity Check: ✅ ON

### F. Speed Settings

Di Cloudflare Dashboard → Speed:

1. **Optimization:**
   - Auto Minify: ✅ JavaScript, CSS, HTML
   - Brotli: ✅ ON
   - Early Hints: ✅ ON
   - Rocket Loader: ❌ OFF (bisa conflict dengan Next.js)

### G. Caching Rules

Di Cloudflare Dashboard → Caching:

1. **Configuration:**
   - Caching Level: Standard
   - Browser Cache TTL: 4 hours
2. **Cache Rules:** Buat rule untuk API
   - If: `hostname contains "api"`
   - Then: Bypass cache

### H. Page Rules (Optional)

Buat page rules untuk optimasi:

1. **API No Cache:**
   - URL: `api.webrana.id/*`
   - Settings: Cache Level = Bypass

2. **Static Assets Cache:**
   - URL: `webrana.id/_next/static/*`
   - Settings: Cache Level = Cache Everything, Edge TTL = 1 month

---

## 4. Deployment ke VPS

### A. Initial VPS Setup

```bash
# SSH ke VPS sebagai root
ssh root@YOUR_VPS_IP

# Download dan jalankan setup script
curl -fsSL https://raw.githubusercontent.com/welldanyogia/webrana-ai-proxy/main/infrastructure/scripts/setup-vps.sh | bash
```

Atau manual:
```bash
# Clone repo
git clone git@github.com:welldanyogia/webrana-ai-proxy.git /opt/webrana/app
cd /opt/webrana/app

# Jalankan setup
chmod +x infrastructure/scripts/setup-vps.sh
sudo ./infrastructure/scripts/setup-vps.sh
```

### B. Configure Environment

```bash
# Login sebagai webrana
sudo su - webrana
cd /opt/webrana/app

# Copy dan edit .env
cp infrastructure/docker/.env.example infrastructure/docker/.env
nano infrastructure/docker/.env
```

Generate secrets:
```bash
# POSTGRES_PASSWORD
openssl rand -base64 24

# MASTER_ENCRYPTION_KEY (32 bytes)
openssl rand -base64 32

# JWT_SECRET (64 bytes)
openssl rand -base64 64

# NEXTAUTH_SECRET
openssl rand -base64 32
```

### C. Setup Nginx dengan Cloudflare SSL

```bash
# Copy Cloudflare certificates (sudah di-download sebelumnya)
sudo mkdir -p /etc/nginx/ssl
sudo cp /etc/cloudflare/cert.pem /etc/nginx/ssl/
sudo cp /etc/cloudflare/key.pem /etc/nginx/ssl/
```

### D. Deploy

```bash
cd /opt/webrana/app

# Full deployment
./infrastructure/scripts/deploy-staging.sh deploy

# Atau step by step
./infrastructure/scripts/deploy-staging.sh build
./infrastructure/scripts/deploy-staging.sh start
./infrastructure/scripts/deploy-staging.sh verify
```

### E. Verify Deployment

```bash
# Check services
./infrastructure/scripts/deploy-staging.sh status

# Check logs
./infrastructure/scripts/deploy-staging.sh logs

# Test endpoints
curl -I https://staging.webrana.id
curl -I https://api.staging.webrana.id/health
```

---

## Troubleshooting

### SSH Connection Refused
```bash
# Check SSH service
sudo systemctl status sshd

# Check firewall
sudo ufw status
sudo ufw allow 22/tcp
```

### Git Permission Denied
```bash
# Test SSH key
ssh -vT git@github.com

# Check key permissions
chmod 600 ~/.ssh/id_ed25519
chmod 644 ~/.ssh/id_ed25519.pub
```

### Cloudflare 522 Error (Connection Timed Out)
- Pastikan VPS firewall allow port 80 dan 443
- Pastikan Nginx/Docker running
- Check: `sudo ufw allow 80/tcp && sudo ufw allow 443/tcp`

### Cloudflare 525 Error (SSL Handshake Failed)
- Pastikan SSL mode = Full (strict)
- Pastikan Origin Certificate sudah di-install di Nginx
- Check certificate: `openssl x509 -in /etc/nginx/ssl/cert.pem -text -noout`

---

## Quick Reference

### SSH Commands
```bash
# Connect to staging
ssh webrana-staging

# Connect to production
ssh webrana-prod

# Copy file to VPS
scp local_file.txt webrana-staging:/opt/webrana/
```

### Deployment Commands
```bash
./deploy-staging.sh deploy    # Full deployment
./deploy-staging.sh build     # Build only
./deploy-staging.sh start     # Start services
./deploy-staging.sh stop      # Stop services
./deploy-staging.sh logs      # View logs
./deploy-staging.sh status    # Check status
./deploy-staging.sh rollback  # Rollback to previous
```

### Cloudflare API (Optional)
```bash
# Purge cache
curl -X POST "https://api.cloudflare.com/client/v4/zones/ZONE_ID/purge_cache" \
     -H "Authorization: Bearer CF_API_TOKEN" \
     -H "Content-Type: application/json" \
     --data '{"purge_everything":true}'
```
