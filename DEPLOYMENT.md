# Deployment Guide - Future Economy Indices

This guide covers deploying the Future Economy Indices application to Google Cloud Platform (GCP) using Kubernetes (GKE).

## Architecture Overview

- **GKE Cluster**: Regional Kubernetes cluster with autoscaling
- **Cloud SQL**: Managed PostgreSQL database with private IP
- **Workload Identity**: Secure authentication between GKE and GCP services
- **Container Registry (GCR)**: Docker image storage
- **Load Balancer**: Google Cloud Load Balancer with managed SSL certificates

## Prerequisites

1. **GCP Project**: Create or use existing project
2. **GCP CLI**: Install and configure `gcloud`
3. **Terraform**: Version >= 1.5.0
4. **kubectl**: Kubernetes CLI
5. **Helm**: Version >= 3.12.0
6. **Docker**: For local builds (optional)

### Required GCP APIs

The Terraform configuration will enable these automatically:
- Compute Engine API
- Kubernetes Engine API
- Cloud SQL Admin API
- Service Networking API
- Cloud Resource Manager API
- IAM API
- Secret Manager API

## Step 1: Configure GCP Project

```bash
# Set your project ID
export PROJECT_ID="your-gcp-project-id"
export REGION="us-central1"

# Authenticate with GCP
gcloud auth login
gcloud config set project $PROJECT_ID

# Enable billing (if not already enabled)
gcloud beta billing projects link $PROJECT_ID \
  --billing-account=BILLING_ACCOUNT_ID
```

## Step 2: Create Terraform State Bucket

Before running Terraform, you need to create a GCS bucket to store the Terraform state file.

### Option A: Using Makefile (Recommended)

```bash
# Set environment variables (if not already set)
export PROJECT_ID="your-gcp-project-id"
export REGION="us-central1"

# Check if bucket exists
make gcs-bucket-check

# Create the bucket
make gcs-bucket-create

# Verify creation
make gcs-bucket-info
```

### Option B: Manual Creation

```bash
# Create bucket for Terraform state
gsutil mb -p $PROJECT_ID -l $REGION gs://future-economy-terraform-state

# Enable versioning for state file history
gsutil versioning set on gs://future-economy-terraform-state

# Verify the bucket was created
gsutil ls -b gs://future-economy-terraform-state
```

**Note**: If you encounter `bucket doesn't exist` error when running `terraform init`, this step was missed.

## Step 3: Deploy Infrastructure with Terraform

```bash
cd terraform

# Initialize Terraform
terraform init

# Review the plan for production
terraform plan -var-file=environments/prod/terraform.tfvars

# Apply the configuration
terraform apply -var-file=environments/prod/terraform.tfvars

# Save outputs for later use
terraform output -json > outputs.json
```

This will create:
- VPC network with private subnets
- GKE cluster with autoscaling node pools
- Cloud SQL PostgreSQL instance
- IAM service accounts and permissions
- Static IP for ingress

**Expected time**: 10-15 minutes

## Step 4: Configure kubectl

```bash
# Get cluster credentials
gcloud container clusters get-credentials prod-gke-cluster \
  --region us-central1 \
  --project $PROJECT_ID

# Verify connection
kubectl cluster-info
kubectl get nodes
```

## Step 5: Set Up Database

```bash
# Get Cloud SQL instance connection details
INSTANCE_NAME=$(terraform output -raw cloudsql_instance_name)
DB_PRIVATE_IP=$(terraform output -raw cloudsql_private_ip)

# Get database password from Secret Manager
DB_PASSWORD=$(gcloud secrets versions access latest \
  --secret=prod-db-password \
  --project=$PROJECT_ID)

# Update Kubernetes secret with database URL
kubectl create secret generic api-server-secrets \
  --namespace=future-economy-indices \
  --from-literal=DATABASE_URL="postgresql://api_server:${DB_PASSWORD}@${DB_PRIVATE_IP}:5432/future_economy_indices" \
  --from-literal=POLYGON_API_KEY="your-polygon-api-key" \
  --from-literal=ANTHROPIC_API_KEY="your-anthropic-api-key" \
  --dry-run=client -o yaml | kubectl apply -f -

# Run database migrations (from a temporary pod)
kubectl run -it --rm db-migrate \
  --image=gcr.io/$PROJECT_ID/api-server:latest \
  --namespace=future-economy-indices \
  --env="DATABASE_URL=postgresql://api_server:${DB_PASSWORD}@${DB_PRIVATE_IP}:5432/future_economy_indices" \
  --command -- /app/api-server db init
```

## Step 6: Build and Push Docker Image

### Option A: Using GitHub Actions (Recommended)

The GitHub Actions workflow will automatically build and deploy when you push to `main` branch.

```bash
# Set up GitHub secrets
gh secret set GCP_PROJECT_ID --body "$PROJECT_ID"
gh secret set GKE_CLUSTER_NAME --body "prod-gke-cluster"
gh secret set WIF_PROVIDER --body "projects/PROJECT_NUMBER/locations/global/workloadIdentityPools/github-pool/providers/github-provider"
gh secret set WIF_SERVICE_ACCOUNT --body "github-actions@$PROJECT_ID.iam.gserviceaccount.com"

# Push to main to trigger deployment
git push origin main
```

### Option B: Manual Build

```bash
# Build and push image
docker build -t gcr.io/$PROJECT_ID/api-server:latest .
docker push gcr.io/$PROJECT_ID/api-server:latest
```

## Step 7: Deploy Application to Kubernetes with Helm

### Option A: Deploy to Production

```bash
# Install Helm (if not already installed)
# macOS: brew install helm
# Or visit: https://helm.sh/docs/intro/install/

# Lint the Helm chart
helm lint k8s/helm/future-economy-indices

# Preview the resources that will be created
helm template future-economy-indices ./k8s/helm/future-economy-indices \
  -f k8s/helm/future-economy-indices/values-prod.yaml

# Deploy to production
helm upgrade --install future-economy-indices ./k8s/helm/future-economy-indices \
  -f k8s/helm/future-economy-indices/values-prod.yaml \
  --set image.repository=gcr.io/$PROJECT_ID/api-server \
  --set image.tag=latest \
  --namespace future-economy-indices \
  --create-namespace

# Watch rollout
kubectl rollout status deployment/future-economy-indices -n future-economy-indices

# Verify pods are running
kubectl get pods -n future-economy-indices
```

### Option B: Deploy to Development

```bash
# Deploy to development
helm upgrade --install future-economy-indices ./k8s/helm/future-economy-indices \
  -f k8s/helm/future-economy-indices/values-dev.yaml \
  --set image.repository=gcr.io/$PROJECT_ID/api-server \
  --set image.tag=develop \
  --namespace future-economy-indices \
  --create-namespace

# Watch rollout
kubectl rollout status deployment/future-economy-indices -n future-economy-indices

# Verify pods are running
kubectl get pods -n future-economy-indices
```

### Using Makefile (Recommended)

```bash
# Lint Helm chart
make helm-lint

# Preview dev templates
make helm-template-dev

# Deploy to dev
make deploy-dev

# Deploy to prod
make deploy-prod
```

## Step 8: Configure DNS and SSL

### Production Environment

```bash
# Get the static IP address for production
PROD_INGRESS_IP=$(terraform output -raw ingress_ip)

# Configure DNS A record for production domain
# Host: api
# Type: A
# Value: $PROD_INGRESS_IP
# Domain: urania.fund

# Google will automatically provision SSL certificate for api.urania.fund
# Check certificate status
kubectl describe managedcertificate api-server-cert -n future-economy-indices
```

### Development Environment

```bash
# Get the static IP address for development
DEV_INGRESS_IP=$(gcloud compute addresses describe future-economy-api-ip-dev \
  --region=$REGION --format="get(address)")

# Configure DNS A record for development domain
# Host: api.dev
# Type: A
# Value: $DEV_INGRESS_IP
# Domain: urania.fund

# Google will automatically provision SSL certificate for api.dev.urania.fund
# Check certificate status
kubectl describe managedcertificate dev-api-server-cert -n future-economy-indices
```

**Note**: SSL certificate provisioning can take 15-60 minutes.

### DNS Configuration Summary

| Environment | Domain | IP Address Variable | Terraform Output |
|-------------|--------|-------------------|------------------|
| Production | api.urania.fund | PROD_INGRESS_IP | `ingress_ip` |
| Development | api.dev.urania.fund | DEV_INGRESS_IP | Created separately |

## Step 9: Verify Deployment

```bash
# Check all resources
kubectl get all -n future-economy-indices

# Check service endpoints
kubectl get ingress -n future-economy-indices

# Test API endpoints (Production)
curl https://api.urania.fund/health
curl https://api.urania.fund/api/indices

# Test API endpoints (Development)
curl https://api.dev.urania.fund/health
curl https://api.dev.urania.fund/api/indices
```

## Monitoring and Logging

### View Logs

```bash
# View API server logs
kubectl logs -f deployment/api-server -n future-economy-indices

# View logs in Cloud Logging
gcloud logging read "resource.type=k8s_container AND resource.labels.namespace_name=future-economy-indices" \
  --limit 50 \
  --format json
```

### Check Metrics

```bash
# Pod metrics
kubectl top pods -n future-economy-indices

# Node metrics
kubectl top nodes

# HPA status
kubectl get hpa -n future-economy-indices
```

## Scaling

### Manual Scaling

```bash
# Scale deployment
kubectl scale deployment api-server --replicas=5 -n future-economy-indices
```

### Horizontal Pod Autoscaler

HPA is configured automatically and will scale based on:
- CPU utilization (target: 70%)
- Memory utilization (target: 80%)
- Min replicas: 2
- Max replicas: 10

## Updates and Rollbacks

### Rolling Update with Helm

```bash
# Update to a new version (production)
helm upgrade future-economy-indices ./k8s/helm/future-economy-indices \
  -f k8s/helm/future-economy-indices/values-prod.yaml \
  --set image.tag=v2.0.0 \
  --namespace future-economy-indices

# Watch rollout
kubectl rollout status deployment/future-economy-indices -n future-economy-indices

# View release history
helm history future-economy-indices --namespace future-economy-indices
```

### Rollback with Helm

```bash
# Rollback to previous release
helm rollback future-economy-indices --namespace future-economy-indices

# Or using Makefile
make helm-rollback

# Rollback to specific revision
helm rollback future-economy-indices 2 --namespace future-economy-indices

# View rollout history
kubectl rollout history deployment/future-economy-indices -n future-economy-indices
```

### Helm Release Management

```bash
# List releases
helm list --namespace future-economy-indices

# Get release status
helm status future-economy-indices --namespace future-economy-indices

# Get all release details
helm get all future-economy-indices --namespace future-economy-indices

# Get values used in release
helm get values future-economy-indices --namespace future-economy-indices
```

## Disaster Recovery

### Database Backups

Cloud SQL automatically creates daily backups. To restore:

```bash
# List backups
gcloud sql backups list --instance=$INSTANCE_NAME

# Restore from backup
gcloud sql backups restore BACKUP_ID \
  --backup-instance=$INSTANCE_NAME \
  --backup-instance=$INSTANCE_NAME
```

### Export Database

```bash
# Export to Cloud Storage
gcloud sql export sql $INSTANCE_NAME \
  gs://future-economy-backups/backup-$(date +%Y%m%d).sql \
  --database=future_economy_indices
```

## Cost Optimization

### Development Environment

For dev/staging, use the `dev` tfvars:

```bash
terraform apply -var-file=environments/dev/terraform.tfvars
```

This uses:
- Smaller instance types (e2-medium)
- Preemptible nodes
- db-f1-micro for database
- Single zone deployment

### Production Cost Estimates

Monthly costs (approximate):
- GKE cluster (2x e2-standard-2): ~$70
- Cloud SQL (db-n1-standard-1): ~$50
- Load Balancer: ~$20
- Container Registry storage: ~$5

**Total**: ~$145/month (can scale up/down as needed)

## Security Best Practices

1. **Workload Identity**: Enabled by default
2. **Private GKE nodes**: Nodes have no public IPs
3. **Cloud SQL Private IP**: Database not exposed to internet
4. **Secrets**: Stored in Secret Manager, not in code
5. **Network Policies**: Restrict pod-to-pod communication
6. **Binary Authorization**: Ensure only verified images run

## Troubleshooting

### Terraform: "bucket doesn't exist" error

**Error message:**
```
Error: Failed to get existing workspaces: querying Cloud Storage failed: storage: bucket doesn't exist
```

**Solution:**
```bash
# Create the GCS bucket for Terraform state
make gcs-bucket-create

# Or manually:
gsutil mb -p $PROJECT_ID -l $REGION gs://future-economy-terraform-state
gsutil versioning set on gs://future-economy-terraform-state

# Then retry terraform init
cd terraform
terraform init
```

### Pods not starting

```bash
kubectl describe pod <pod-name> -n future-economy-indices
kubectl logs <pod-name> -n future-economy-indices
```

### Database connection issues

```bash
# Check if Cloud SQL proxy is needed
kubectl run -it --rm debug \
  --image=gcr.io/cloudsql-docker/gce-proxy:latest \
  --namespace=future-economy-indices \
  -- /cloud_sql_proxy -instances=$INSTANCE_CONNECTION_NAME=tcp:5432
```

### Ingress not working

```bash
# Check ingress status
kubectl describe ingress api-server -n future-economy-indices

# Verify backend services
kubectl get backendconfig -A
```

## Cleanup

To destroy all resources:

```bash
# Delete Kubernetes resources
kubectl delete namespace future-economy-indices

# Destroy Terraform resources
cd terraform
terraform destroy -var-file=environments/prod/terraform.tfvars
```

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-org/future-economy-indices/issues
- Documentation: https://docs.futureeconomy.indices

## Next Steps

1. Set up monitoring dashboards in Cloud Monitoring
2. Configure alerts for critical metrics
3. Set up log-based metrics
4. Implement backup automation
5. Configure CI/CD for the website (Next.js)
