# =============================================================================
# Webrana AI Proxy - AWS MVP Infrastructure (EC2 All-in-One)
# Estimated Cost: ~$43/month
# =============================================================================

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# -----------------------------------------------------------------------------
# Provider Configuration
# -----------------------------------------------------------------------------
provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "webrana"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# -----------------------------------------------------------------------------
# Variables
# -----------------------------------------------------------------------------
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-southeast-1" # Singapore
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t3.medium" # 2 vCPU, 4GB RAM - $30/month
}

variable "domain_name" {
  description = "Domain name for the application"
  type        = string
  default     = "webrana.id"
}

variable "ssh_key_name" {
  description = "Name of the SSH key pair"
  type        = string
}

variable "admin_cidr" {
  description = "CIDR block for SSH access (restrict to your IP)"
  type        = string
  default     = "0.0.0.0/0" # Override in terraform.tfvars with your IP/32
}

variable "enable_monitoring" {
  description = "Enable CloudWatch monitoring alarms"
  type        = bool
  default     = true
}

# -----------------------------------------------------------------------------
# Data Sources
# -----------------------------------------------------------------------------
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# -----------------------------------------------------------------------------
# VPC & Networking
# -----------------------------------------------------------------------------
resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "webrana-vpc"
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "webrana-igw"
  }
}

resource "aws_subnet" "public" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = "10.0.1.0/24"
  availability_zone       = data.aws_availability_zones.available.names[0]
  map_public_ip_on_launch = true

  tags = {
    Name = "webrana-public-subnet"
  }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "webrana-public-rt"
  }
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

# -----------------------------------------------------------------------------
# Security Group
# -----------------------------------------------------------------------------
resource "aws_security_group" "webrana" {
  name        = "webrana-sg"
  description = "Security group for Webrana EC2 instance"
  vpc_id      = aws_vpc.main.id

  # SSH access - RESTRICT THIS IN PRODUCTION
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [var.admin_cidr]
    description = "SSH access - restricted to admin IP"
  }

  # HTTP
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "HTTP access"
  }

  # HTTPS
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "HTTPS access"
  }

  # Backend API - internal only (nginx proxies to this)
  # Remove this rule in production - access via nginx on 80/443
  ingress {
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"] # VPC internal only
    description = "Backend API - internal"
  }

  # All outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    description = "All outbound traffic"
  }

  tags = {
    Name = "webrana-sg"
  }
}

# -----------------------------------------------------------------------------
# Elastic IP
# -----------------------------------------------------------------------------
resource "aws_eip" "webrana" {
  domain = "vpc"

  tags = {
    Name = "webrana-eip"
  }
}

resource "aws_eip_association" "webrana" {
  instance_id   = aws_instance.webrana.id
  allocation_id = aws_eip.webrana.id
}

# -----------------------------------------------------------------------------
# EC2 Instance
# -----------------------------------------------------------------------------
resource "aws_instance" "webrana" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = var.instance_type
  key_name               = var.ssh_key_name
  subnet_id              = aws_subnet.public.id
  vpc_security_group_ids = [aws_security_group.webrana.id]
  iam_instance_profile   = aws_iam_instance_profile.webrana.name
  monitoring             = true # Enable detailed monitoring

  root_block_device {
    volume_size           = 30
    volume_type           = "gp3"
    iops                  = 3000
    throughput            = 125
    delete_on_termination = true
    encrypted             = true
  }

  user_data = base64encode(templatefile("${path.module}/user-data.sh", {
    domain_name = var.domain_name
  }))

  tags = {
    Name = "webrana-server"
  }

  lifecycle {
    create_before_destroy = true
  }
}

# -----------------------------------------------------------------------------
# S3 Bucket for Backups
# -----------------------------------------------------------------------------
resource "aws_s3_bucket" "backups" {
  bucket = "webrana-backups-${random_id.bucket_suffix.hex}"

  tags = {
    Name = "webrana-backups"
  }
}

resource "random_id" "bucket_suffix" {
  byte_length = 4
}

resource "aws_s3_bucket_versioning" "backups" {
  bucket = aws_s3_bucket.backups.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "backups" {
  bucket = aws_s3_bucket.backups.id

  rule {
    id     = "cleanup-old-backups"
    status = "Enabled"

    expiration {
      days = 30
    }

    noncurrent_version_expiration {
      noncurrent_days = 7
    }
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "backups" {
  bucket = aws_s3_bucket.backups.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Block all public access to backup bucket
resource "aws_s3_bucket_public_access_block" "backups" {
  bucket = aws_s3_bucket.backups.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# -----------------------------------------------------------------------------
# IAM Role for EC2 (S3 backup access)
# -----------------------------------------------------------------------------
resource "aws_iam_role" "webrana_ec2" {
  name = "webrana-ec2-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "webrana_s3_backup" {
  name = "webrana-s3-backup-policy"
  role = aws_iam_role.webrana_ec2.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:GetObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.backups.arn,
          "${aws_s3_bucket.backups.arn}/*"
        ]
      }
    ]
  })
}

resource "aws_iam_instance_profile" "webrana" {
  name = "webrana-instance-profile"
  role = aws_iam_role.webrana_ec2.name
}

# -----------------------------------------------------------------------------
# CloudWatch Monitoring & Alarms
# -----------------------------------------------------------------------------
resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  count               = var.enable_monitoring ? 1 : 0
  alarm_name          = "webrana-cpu-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = 300
  statistic           = "Average"
  threshold           = 80
  alarm_description   = "CPU utilization exceeds 80%"
  
  dimensions = {
    InstanceId = aws_instance.webrana.id
  }

  tags = {
    Name = "webrana-cpu-alarm"
  }
}

resource "aws_cloudwatch_metric_alarm" "disk_high" {
  count               = var.enable_monitoring ? 1 : 0
  alarm_name          = "webrana-disk-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "disk_used_percent"
  namespace           = "CWAgent"
  period              = 300
  statistic           = "Average"
  threshold           = 85
  alarm_description   = "Disk usage exceeds 85%"

  dimensions = {
    InstanceId = aws_instance.webrana.id
  }

  tags = {
    Name = "webrana-disk-alarm"
  }
}

resource "aws_cloudwatch_metric_alarm" "status_check" {
  count               = var.enable_monitoring ? 1 : 0
  alarm_name          = "webrana-status-check-failed"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "StatusCheckFailed"
  namespace           = "AWS/EC2"
  period              = 60
  statistic           = "Maximum"
  threshold           = 0
  alarm_description   = "EC2 instance status check failed"

  dimensions = {
    InstanceId = aws_instance.webrana.id
  }

  tags = {
    Name = "webrana-status-alarm"
  }
}

# -----------------------------------------------------------------------------
# Route 53 (Optional - if managing DNS in AWS)
# -----------------------------------------------------------------------------
# Uncomment if you want to manage DNS in Route 53
# resource "aws_route53_zone" "main" {
#   name = var.domain_name
# }
#
# resource "aws_route53_record" "api" {
#   zone_id = aws_route53_zone.main.zone_id
#   name    = "api.${var.domain_name}"
#   type    = "A"
#   ttl     = 300
#   records = [aws_eip.webrana.public_ip]
# }

# -----------------------------------------------------------------------------
# Outputs
# -----------------------------------------------------------------------------
output "instance_id" {
  description = "EC2 instance ID"
  value       = aws_instance.webrana.id
}

output "public_ip" {
  description = "Public IP address (Elastic IP)"
  value       = aws_eip.webrana.public_ip
}

output "ssh_command" {
  description = "SSH command to connect"
  value       = "ssh -i ~/.ssh/${var.ssh_key_name}.pem ubuntu@${aws_eip.webrana.public_ip}"
}

output "s3_backup_bucket" {
  description = "S3 bucket for backups"
  value       = aws_s3_bucket.backups.bucket
}

output "monthly_cost_estimate" {
  description = "Estimated monthly cost"
  value       = "~$43/month (EC2: $30, EBS: $3, EIP: $4, S3: $1, Transfer: $5)"
}
