# CompileX7 AWS ECS Fargate Deployment Guide

## Overview

This guide covers deploying CompileX7 backend to AWS ECS Fargate using the CLI tool with automated ECR image management, deployment orchestration, and rollback capabilities.

## Architecture

- **ECR**: Container image storage and scanning
- **ECS Fargate**: Serverless container orchestration
- **ALB**: Application Load Balancer for traffic distribution
- **Secrets Manager**: Secure credential management
- **CloudWatch**: Logging and monitoring
- **Auto Scaling**: Automatic capacity management based on CPU/Memory

## Prerequisites

1. AWS Account with appropriate IAM permissions
2. AWS CLI v2 installed and configured
3. Docker installed locally
4. Rust toolchain (for building the application)
5. CLI tool installed (`cx7` command)

## Setup Steps

### 1. Configure AWS Credentials

```bash
aws configure
# Enter AWS Access Key ID, Secret Access Key, Region (us-east-1), Output format (json)
```

### 2. Set Environment Variables

```bash
# Create .env file for ECS deployment
cp aws/.env.ecs .env

# Update with your AWS account details
export AWS_ACCOUNT_ID=123456789012
export AWS_REGION=us-east-1
export ECR_REPOSITORY=compilex7
export ECS_CLUSTER=compilex7-cluster
export ECS_SERVICE=compilex7-service
export TASK_FAMILY=compilex7-task
```

### 3. Create ECR Repository

```bash
aws ecr create-repository \
  --repository-name compilex7 \
  --image-scanning-configuration scanOnPush=true \
  --region us-east-1
```

### 4. Deploy Infrastructure with CloudFormation

```bash
# Update subnet IDs and security group IDs in cloudformation.yaml
aws cloudformation create-stack \
  --stack-name compilex7-stack \
  --template-body file://aws/cloudformation.yaml \
  --parameters ParameterKey=ImageUri,ParameterValue=<IMAGE_URI> \
  --capabilities CAPABILITY_IAM \
  --region us-east-1

# Monitor stack creation
aws cloudformation wait stack-create-complete \
  --stack-name compilex7-stack \
  --region us-east-1
```

### 5. Store Secrets in AWS Secrets Manager

```bash
# Create database secret
aws secretsmanager create-secret \
  --name compilex7/database-url \
  --secret-string "postgresql://user:password@db.example.com:5432/compilex7" \
  --region us-east-1

# Create JWT secret
aws secretsmanager create-secret \
  --name compilex7/jwt-secret \
  --secret-string "your-jwt-secret-key" \
  --region us-east-1
```

## Deployment Commands

### Build and Deploy to ECS

```bash
# Deploy with automatic image building and ECR push
cx7 aws deploy --dockerfile Dockerfile

# Deploy with custom tag
cx7 aws deploy --dockerfile Dockerfile --tag v1.0.0

# Or use the Rust CLI directly
cargo run --bin cx7 -- aws deploy
```

This command:
1. Builds Docker image locally
2. Authenticates with ECR
3. Tags and pushes image to ECR
4. Updates ECS task definition
5. Updates ECS service
6. Monitors deployment until stable (max 10 minutes)

### Check Deployment Status

```bash
cx7 aws status

# Output example:
# Service: compilex7-service
# Running: 2/2
# Pending: 0
# Status: ACTIVE
```

### Manage Secrets

```bash
# Set a secret
cx7 aws secrets set DATABASE_URL "postgresql://user:pass@host:5432/db"

# Get a secret
cx7 aws secrets get DATABASE_URL

# Delete a secret
cx7 aws secrets delete DATABASE_URL
```

### Rollback Deployment

```bash
cx7 aws rollback --previous-tag v1.0.0
```

## Environment Variables

### Required
- `AWS_ACCOUNT_ID`: Your AWS account ID
- `AWS_REGION`: AWS region (default: us-east-1)
- `ECR_REPOSITORY`: ECR repository name (default: compilex7)
- `ECS_CLUSTER`: ECS cluster name (default: compilex7-cluster)
- `ECS_SERVICE`: ECS service name (default: compilex7-service)
- `TASK_FAMILY`: ECS task family (default: compilex7-task)

### Optional
- `TASK_CPU`: Task CPU units (default: 256)
- `TASK_MEMORY`: Task memory MB (default: 512)
- `CONTAINER_PORT`: Container port (default: 8080)
- `LOG_GROUP`: CloudWatch log group (default: /ecs/compilex7)

## Monitoring

### CloudWatch Logs

```bash
# View logs in real-time
aws logs tail /ecs/compilex7 --follow

# View specific time range
aws logs tail /ecs/compilex7 --since 1h

# Filter logs
aws logs filter-log-events \
  --log-group-name /ecs/compilex7 \
  --filter-pattern "ERROR"
```

### ECS Console

Visit AWS Console → ECS → Clusters → compilex7-cluster → Services → compilex7-service

## Troubleshooting

### Deployment Fails

1. Check task definition syntax:
```bash
aws ecs describe-task-definition --task-definition compilex7-task
```

2. Check CloudWatch logs for application errors:
```bash
aws logs tail /ecs/compilex7 --follow
```

3. Verify IAM permissions for secrets access

### Service Won't Start

1. Check container health:
```bash
aws ecs describe-tasks \
  --cluster compilex7-cluster \
  --tasks <task-arn> \
  --query 'tasks[0].{lastStatus:lastStatus,containers:containers[0]}'
```

2. Verify Docker image exists in ECR:
```bash
aws ecr describe-images --repository-name compilex7
```

### High CPU/Memory Usage

1. Check application logs for issues
2. Increase task CPU/Memory in CloudFormation
3. Adjust auto-scaling thresholds

## Best Practices

1. **Versioning**: Use semantic versioning for image tags (v1.0.0, v1.1.0, etc.)
2. **Health Checks**: Ensure `/health` endpoint responds with 200 OK
3. **Secrets Management**: Never commit secrets to version control
4. **Logging**: Use structured JSON logging for better CloudWatch filtering
5. **Deployment Windows**: Deploy during low-traffic periods
6. **Monitoring**: Set up CloudWatch alarms for CPU/Memory/Error rates
7. **Backup**: Keep previous task definitions for quick rollback

## Security

1. Enable image scanning in ECR for vulnerability detection
2. Use IAM roles with minimal required permissions
3. Rotate secrets regularly
4. Use VPC security groups to restrict traffic
5. Enable CloudTrail for audit logging
6. Use HTTPS/TLS for all external communications

## Cost Optimization

1. Use Fargate Spot for development/staging (up to 70% savings)
2. Right-size task CPU/Memory based on actual usage
3. Delete unused ECR images (>10 versions)
4. Monitor CloudWatch costs (logs retention: 30 days)
5. Use Reserved Capacity for predictable production workloads

## Advanced Configuration

### Custom Domain with Route 53

```bash
# Create Route 53 record
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123ABC \
  --change-batch '{
    "Changes": [{
      "Action": "CREATE",
      "ResourceRecordSet": {
        "Name": "api.compilex7.com",
        "Type": "A",
        "AliasTarget": {
          "HostedZoneId": "<ALB-ZONE-ID>",
          "DNSName": "<ALB-DNS-NAME>",
          "EvaluateTargetHealth": true
        }
      }
    }]
  }'
```

### HTTPS with AWS Certificate Manager

```bash
# Create SSL certificate
aws acm request-certificate \
  --domain-name api.compilex7.com \
  --validation-method DNS \
  --region us-east-1

# Use certificate in ALB listener (update CloudFormation template)
```

## Support

For issues or questions:
1. Check AWS documentation: https://docs.aws.amazon.com/ecs/
2. Review CloudWatch logs
3. Check CLI tool help: `cx7 aws --help`
4. Report issues with: `cx7 --debug` for verbose logging
