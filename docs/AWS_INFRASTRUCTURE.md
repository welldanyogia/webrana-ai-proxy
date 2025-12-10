# AWS Infrastructure Options - Webrana AI Proxy

## Architecture Diagrams

### Option 1: EC2 All-in-One (MVP - ~$43/month)

```
┌─────────────────────────────────────────────────────────────────┐
│                        AWS Cloud (ap-southeast-1)               │
│                                                                 │
│  ┌─────────────┐     ┌─────────────────────────────────────┐   │
│  │  Route 53   │────▶│         EC2 (t3.medium)             │   │
│  │ webrana.id  │     │  ┌─────────────────────────────┐    │   │
│  └─────────────┘     │  │     Docker Compose          │    │   │
│                      │  │  ┌─────────┐ ┌───────────┐  │    │   │
│  ┌─────────────┐     │  │  │ Backend │ │ Frontend  │  │    │   │
│  │ CloudFlare  │────▶│  │  │ (Rust)  │ │ (Next.js) │  │    │   │
│  │  CDN/WAF    │     │  │  └────┬────┘ └─────┬─────┘  │    │   │
│  └─────────────┘     │  │       │            │        │    │   │
│                      │  │  ┌────▼────┐ ┌─────▼─────┐  │    │   │
│                      │  │  │PostgreSQL│ │   Redis   │  │    │   │
│                      │  │  │ (Docker) │ │ (Docker)  │  │    │   │
│                      │  │  └──────────┘ └───────────┘  │    │   │
│                      │  └─────────────────────────────┘    │   │
│                      │                                     │   │
│                      │  ┌─────────────────────────────┐    │   │
│                      │  │    EBS Volume (30GB gp3)    │    │   │
│                      │  │    - PostgreSQL data        │    │   │
│                      │  │    - Redis persistence      │    │   │
│                      │  └─────────────────────────────┘    │   │
│                      └─────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────┐     ┌─────────────┐                           │
│  │  S3 Bucket  │     │ Elastic IP  │                           │
│  │  (Backups)  │     │ (Static IP) │                           │
│  └─────────────┘     └─────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

### Option 2: ECS Fargate (Growth - ~$92/month)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        AWS Cloud (ap-southeast-1)                       │
│                                                                         │
│  ┌─────────────┐                                                        │
│  │  Route 53   │                                                        │
│  │ webrana.id  │                                                        │
│  └──────┬──────┘                                                        │
│         │                                                               │
│  ┌──────▼──────┐     ┌─────────────────────────────────────────────┐   │
│  │ CloudFlare  │     │              VPC (10.0.0.0/16)              │   │
│  │  CDN/WAF    │     │                                             │   │
│  └──────┬──────┘     │  ┌─────────────────────────────────────┐   │   │
│         │            │  │     Public Subnets (Multi-AZ)       │   │   │
│         │            │  │  ┌───────────────────────────────┐  │   │   │
│         └───────────▶│  │  │    Application Load Balancer  │  │   │   │
│                      │  │  │    (api.webrana.id:443)       │  │   │   │
│                      │  │  └───────────────┬───────────────┘  │   │   │
│                      │  └─────────────────┼───────────────────┘   │   │
│                      │                    │                       │   │
│                      │  ┌─────────────────▼───────────────────┐   │   │
│                      │  │     Private Subnets (Multi-AZ)      │   │   │
│                      │  │                                     │   │   │
│                      │  │  ┌─────────────────────────────┐    │   │   │
│                      │  │  │      ECS Fargate Cluster    │    │   │   │
│                      │  │  │  ┌─────────┐  ┌─────────┐   │    │   │   │
│                      │  │  │  │ Backend │  │ Backend │   │    │   │   │
│                      │  │  │  │ Task 1  │  │ Task 2  │   │    │   │   │
│                      │  │  │  └────┬────┘  └────┬────┘   │    │   │   │
│                      │  │  └───────┼────────────┼────────┘    │   │   │
│                      │  │          │            │             │   │   │
│                      │  │  ┌───────▼────────────▼───────┐     │   │   │
│                      │  │  │                            │     │   │   │
│                      │  │  │  ┌──────────┐ ┌─────────┐  │     │   │   │
│                      │  │  │  │   RDS    │ │ElastiCache│ │     │   │   │
│                      │  │  │  │PostgreSQL│ │  Redis   │  │     │   │   │
│                      │  │  │  │(db.t3.micro)│(cache.t3)│  │     │   │   │
│                      │  │  │  └──────────┘ └─────────┘  │     │   │   │
│                      │  │  │     Database Subnet        │     │   │   │
│                      │  │  └────────────────────────────┘     │   │   │
│                      │  └─────────────────────────────────────┘   │   │
│                      └─────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                     │
│  │     ECR     │  │  S3 Bucket  │  │ CloudWatch  │                     │
│  │  (Images)   │  │  (Backups)  │  │   (Logs)    │                     │
│  └─────────────┘  └─────────────┘  └─────────────┘                     │
└─────────────────────────────────────────────────────────────────────────┘
```

### Option 3: EKS Kubernetes (Scale - ~$255/month)

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                           AWS Cloud (ap-southeast-1)                          │
│                                                                               │
│  ┌─────────────┐                                                              │
│  │  Route 53   │                                                              │
│  │ webrana.id  │                                                              │
│  └──────┬──────┘                                                              │
│         │                                                                     │
│  ┌──────▼──────┐        ┌───────────────────────────────────────────────┐    │
│  │ CloudFlare  │        │                VPC (10.0.0.0/16)              │    │
│  │  CDN/WAF    │        │                                               │    │
│  └──────┬──────┘        │  ┌─────────────────────────────────────────┐  │    │
│         │               │  │         Public Subnets (3 AZs)          │  │    │
│         │               │  │  ┌─────────────────────────────────┐    │  │    │
│         └──────────────▶│  │  │   AWS ALB Ingress Controller    │    │  │    │
│                         │  │  │   (api.webrana.id:443)          │    │  │    │
│                         │  │  └───────────────┬─────────────────┘    │  │    │
│                         │  └──────────────────┼──────────────────────┘  │    │
│                         │                     │                         │    │
│                         │  ┌──────────────────▼──────────────────────┐  │    │
│                         │  │         Private Subnets (3 AZs)         │  │    │
│                         │  │                                         │  │    │
│                         │  │  ┌─────────────────────────────────┐    │  │    │
│                         │  │  │         EKS Cluster             │    │  │    │
│                         │  │  │  ┌───────────────────────────┐  │    │  │    │
│                         │  │  │  │    EKS Control Plane      │  │    │  │    │
│                         │  │  │  │    (AWS Managed)          │  │    │  │    │
│                         │  │  │  └───────────────────────────┘  │    │  │    │
│                         │  │  │                                 │    │  │    │
│                         │  │  │  ┌─────────────────────────────────┐ │  │    │
│                         │  │  │  │      Worker Node Group          │ │  │    │
│                         │  │  │  │  ┌───────┐┌───────┐┌───────┐    │ │  │    │
│                         │  │  │  │  │Node 1 ││Node 2 ││Node 3 │    │ │  │    │
│                         │  │  │  │  │t3.med ││t3.med ││t3.med │    │ │  │    │
│                         │  │  │  │  └───┬───┘└───┬───┘└───┬───┘    │ │  │    │
│                         │  │  │  │      │        │        │        │ │  │    │
│                         │  │  │  │  ┌───▼────────▼────────▼───┐    │ │  │    │
│                         │  │  │  │  │      Kubernetes Pods    │    │ │  │    │
│                         │  │  │  │  │ ┌────────┐ ┌──────────┐ │    │ │  │    │
│                         │  │  │  │  │ │Backend │ │ Frontend │ │    │ │  │    │
│                         │  │  │  │  │ │(2 pods)│ │ (2 pods) │ │    │ │  │    │
│                         │  │  │  │  │ └────────┘ └──────────┘ │    │ │  │    │
│                         │  │  │  │  └─────────────────────────┘    │ │  │    │
│                         │  │  │  └─────────────────────────────────┘ │  │    │
│                         │  │  └─────────────────────────────────┘    │  │    │
│                         │  │                                         │  │    │
│                         │  │  ┌─────────────────────────────────┐    │  │    │
│                         │  │  │       Database Subnet           │    │  │    │
│                         │  │  │  ┌──────────┐  ┌─────────────┐  │    │  │    │
│                         │  │  │  │   RDS    │  │ ElastiCache │  │    │  │    │
│                         │  │  │  │PostgreSQL│  │    Redis    │  │    │  │    │
│                         │  │  │  │(Multi-AZ)│  │  (Cluster)  │  │    │  │    │
│                         │  │  │  └──────────┘  └─────────────┘  │    │  │    │
│                         │  │  └─────────────────────────────────┘    │  │    │
│                         │  └─────────────────────────────────────────┘  │    │
│                         └───────────────────────────────────────────────┘    │
│                                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │     ECR     │  │  S3 Bucket  │  │ CloudWatch  │  │   Secrets   │          │
│  │  (Images)   │  │  (Backups)  │  │   (Logs)    │  │   Manager   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘          │
└───────────────────────────────────────────────────────────────────────────────┘
```

## Cost Breakdown Summary

| Component | EC2 All-in-One | ECS Fargate | EKS |
|-----------|----------------|-------------|-----|
| Compute | $30 (t3.medium) | $30 (2 tasks) | $173 (EKS + 3 nodes) |
| Database | $0 (Docker) | $15 (RDS) | $30 (RDS Multi-AZ) |
| Cache | $0 (Docker) | $13 (ElastiCache) | $13 (ElastiCache) |
| Load Balancer | $0 (Cloudflare) | $22 (ALB) | $22 (ALB) |
| Storage | $7 (EBS + S3) | $7 (ECR + S3) | $12 (ECR + S3) |
| Network | $6 (EIP + Transfer) | $5 (Transfer) | $5 (Transfer) |
| **Total** | **~$43/month** | **~$92/month** | **~$255/month** |



---

## Deployment Instructions

### Prerequisites

1. Install required tools:
```bash
# AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip && sudo ./aws/install

# Terraform
wget https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip
unzip terraform_1.6.0_linux_amd64.zip && sudo mv terraform /usr/local/bin/

# kubectl (for EKS)
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
```

2. Configure AWS credentials:
```bash
aws configure
# Enter: AWS Access Key ID, Secret Access Key, Region (ap-southeast-1)
```

---

### Option 1: EC2 All-in-One (MVP)

```bash
cd infrastructure/terraform/aws-mvp

# 1. Create SSH key pair in AWS Console first
# EC2 > Key Pairs > Create key pair > Name: webrana-key

# 2. Copy and edit variables
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values

# 3. Initialize and deploy
terraform init
terraform plan
terraform apply

# 4. SSH into server
ssh -i ~/.ssh/webrana-key.pem ubuntu@<PUBLIC_IP>

# 5. Setup application
cd /opt/webrana
cp .env.template .env
# Edit .env with secrets:
# - DB_PASSWORD: openssl rand -base64 24
# - MASTER_ENCRYPTION_KEY: openssl rand -base64 32
# - JWT_SECRET: openssl rand -base64 64

# 6. Build and push Docker images (from local machine)
docker build -t webrana/backend:latest -f infrastructure/docker/Dockerfile.backend .
docker build -t webrana/frontend:latest -f infrastructure/docker/Dockerfile.frontend .

# 7. Start services
docker compose up -d

# 8. Setup SSL
sudo certbot --nginx -d webrana.id -d api.webrana.id
```

---

### Option 2: ECS Fargate

```bash
cd infrastructure/terraform/aws-ecs

# 1. Create terraform.tfvars
cat > terraform.tfvars << EOF
aws_region     = "ap-southeast-1"
environment    = "production"
domain_name    = "webrana.id"
db_password    = "$(openssl rand -base64 24)"
encryption_key = "$(openssl rand -base64 32)"
jwt_secret     = "$(openssl rand -base64 64)"
EOF

# 2. Deploy infrastructure
terraform init
terraform plan
terraform apply

# 3. Build and push to ECR
aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin <ACCOUNT_ID>.dkr.ecr.ap-southeast-1.amazonaws.com

docker build -t webrana/backend:latest -f infrastructure/docker/Dockerfile.backend .
docker tag webrana/backend:latest <ECR_URL>:latest
docker push <ECR_URL>:latest

# 4. Force new deployment
aws ecs update-service --cluster webrana-cluster --service webrana-backend --force-new-deployment

# 5. Setup DNS
# Point api.webrana.id to ALB DNS name in Route 53 or Cloudflare
```

---

### Option 3: EKS Kubernetes

```bash
cd infrastructure/terraform/aws-eks

# 1. Create terraform.tfvars
cat > terraform.tfvars << EOF
aws_region     = "ap-southeast-1"
environment    = "production"
cluster_name   = "webrana-eks"
db_password    = "$(openssl rand -base64 24)"
encryption_key = "$(openssl rand -base64 32)"
jwt_secret     = "$(openssl rand -base64 64)"
EOF

# 2. Deploy infrastructure
terraform init
terraform plan
terraform apply

# 3. Configure kubectl
aws eks update-kubeconfig --region ap-southeast-1 --name webrana-eks

# 4. Build and push to ECR
aws ecr get-login-password --region ap-southeast-1 | docker login --username AWS --password-stdin <ACCOUNT_ID>.dkr.ecr.ap-southeast-1.amazonaws.com

docker build -t webrana/backend:latest -f infrastructure/docker/Dockerfile.backend .
docker tag webrana/backend:latest <ECR_URL>:latest
docker push <ECR_URL>:latest

# 5. Deploy to Kubernetes
kubectl apply -f infrastructure/k8s/namespace.yaml
kubectl apply -f infrastructure/k8s/secret.yaml
kubectl apply -f infrastructure/k8s/configmap.yaml
kubectl apply -f infrastructure/k8s/backend-deployment.yaml
kubectl apply -f infrastructure/k8s/ingress.yaml

# 6. Install AWS Load Balancer Controller
helm repo add eks https://aws.github.io/eks-charts
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
  -n kube-system \
  --set clusterName=webrana-eks \
  --set serviceAccount.create=false \
  --set serviceAccount.name=aws-load-balancer-controller

# 7. Verify deployment
kubectl get pods -n webrana
kubectl get ingress -n webrana
```

---

## Migration Path

### Phase 1 → Phase 2 (EC2 → ECS)

1. Export PostgreSQL data:
```bash
# On EC2
docker exec webrana-postgres pg_dump -U webrana webrana > backup.sql
aws s3 cp backup.sql s3://webrana-backups/migration/
```

2. Deploy ECS infrastructure with Terraform

3. Import data to RDS:
```bash
aws s3 cp s3://webrana-backups/migration/backup.sql .
psql -h <RDS_ENDPOINT> -U webrana -d webrana < backup.sql
```

4. Update DNS to point to ALB

5. Terminate EC2 instance

### Phase 2 → Phase 3 (ECS → EKS)

1. Deploy EKS infrastructure with Terraform

2. RDS and ElastiCache can be reused (update security groups)

3. Deploy K8s manifests

4. Update ALB target groups or create new Ingress

5. Decommission ECS cluster

---

## Cost Optimization Tips

1. **Reserved Instances**: Save 30-40% on EC2/RDS with 1-year commitment
2. **Spot Instances**: Use for non-critical workloads (up to 90% savings)
3. **Right-sizing**: Monitor CloudWatch and downsize if underutilized
4. **S3 Lifecycle**: Auto-delete old backups after 30 days
5. **NAT Gateway**: Consider NAT Instance for lower traffic (~$5 vs $45/month)

---

## Monitoring & Alerts

### CloudWatch Alarms (Recommended)

```bash
# CPU > 80%
aws cloudwatch put-metric-alarm \
  --alarm-name "webrana-high-cpu" \
  --metric-name CPUUtilization \
  --namespace AWS/EC2 \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2

# Database connections > 80%
aws cloudwatch put-metric-alarm \
  --alarm-name "webrana-db-connections" \
  --metric-name DatabaseConnections \
  --namespace AWS/RDS \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2
```

### Grafana Dashboard (Optional)

For EKS, install Prometheus + Grafana:
```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack -n monitoring --create-namespace
```
