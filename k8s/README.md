# Future Economy Indices - Kubernetes Deployment Guide

This guide walks you through deploying the Future Economy Indices API server to Google Kubernetes Engine (GKE) using Helm.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Connect to GKE Cluster](#step-1-connect-to-gke-cluster)
- [Step 2: Create Kubernetes Secrets](#step-2-create-kubernetes-secrets)
- [Step 3: Install Helm Chart](#step-3-install-helm-chart)
- [Step 4: Verify Deployment](#step-4-verify-deployment)
- [Step 5: Access the API](#step-5-access-the-api)
- [Upgrading](#upgrading)
- [Uninstalling](#uninstalling)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying, ensure you have:

1. **GKE Cluster Running**: Your GKE cluster should be provisioned via Terraform
2. **gcloud CLI**: Installed and authenticated
3. **kubectl**: Installed and configured
4. **Helm 3**: Installed (minimum version 3.0+)
5. **GKE Auth Plugin**: Installed for kubectl authentication
6. **Docker Image**: Built and pushed to a container registry (GCR/Artifact Registry)

### Install Required Tools

```bash
# Install kubectl (if not installed)
gcloud components install kubectl

# Install Helm (if not installed)
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify installations
kubectl version --client
helm version
gke-gcloud-auth-plugin --version
```

### Build and Push Docker Image

```bash
# From project root directory
cd /home/dsidorenko/develop/future-economy-indices

# Build the Docker image
docker build -t gcr.io/urania-economy-indices-dev/api-server:develop .

# Authenticate Docker with GCR
gcloud auth configure-docker

# Push the image
docker push gcr.io/urania-economy-indices-dev/api-server:develop
```

## Step 1: Connect to GKE Cluster

### Development Environment

```bash
# Set your project
gcloud config set project urania-economy-indices-dev

# Get cluster credentials
gcloud container clusters get-credentials dev-gke-cluster \
  --region us-central1 \
  --project urania-economy-indices-dev

# Verify connection
kubectl cluster-info
kubectl get nodes
```

### Production Environment

```bash
# Set production project
gcloud config set project urania-economy-indices-prod

# Get cluster credentials
gcloud container clusters get-credentials prod-gke-cluster \
  --region us-central1 \
  --project urania-economy-indices-prod

# Verify connection
kubectl cluster-info
kubectl get nodes
```

## Step 2: Create Kubernetes Secrets

The application requires three secrets stored in Kubernetes:

1. **DATABASE_URL**: PostgreSQL connection string
2. **POLYGON_API_KEY**: Polygon.io API key for market data
3. **ANTHROPIC_API_KEY**: Anthropic API key for AI features

### Option A: Create Secrets from Command Line

```bash
# Create the namespace first
kubectl create namespace future-economy-indices

# Create secrets (replace with your actual values)
kubectl create secret generic api-server-secrets \
  --namespace=future-economy-indices \
  --from-literal=DATABASE_URL='postgresql://user:password@host:5432/database' \
  --from-literal=POLYGON_API_KEY='your_polygon_api_key' \
  --from-literal=ANTHROPIC_API_KEY='your_anthropic_api_key'

# Verify secret was created
kubectl get secrets -n future-economy-indices
kubectl describe secret api-server-secrets -n future-economy-indices
```

### Option B: Create Secrets from Environment File

```bash
# From project root
cd /home/dsidorenko/develop/future-economy-indices

# Copy and edit environment file
cp .env.example .env
nano .env  # Edit with your actual values

# Create secret from file
kubectl create secret generic api-server-secrets \
  --namespace=future-economy-indices \
  --from-env-file=.env

# Clean up the .env file (important for security!)
rm .env
```

### Option C: Create Secrets from Google Secret Manager (Recommended for Production)

```bash
# Enable Secret Manager API
gcloud services enable secretmanager.googleapis.com

# Create secrets in Google Secret Manager
echo -n "postgresql://user:password@host:5432/database" | \
  gcloud secrets create database-url --data-file=-

echo -n "your_polygon_api_key" | \
  gcloud secrets create polygon-api-key --data-file=-

echo -n "your_anthropic_api_key" | \
  gcloud secrets create anthropic-api-key --data-file=-

# Grant the GKE service account access to secrets
# (This should already be configured via Terraform)
```

### Get Database URL from Terraform

If your database was created with Terraform, get the connection string:

```bash
cd ../terraform

# Get database connection info
terraform output -json | jq '.database_connection_string.value'

# Or use gcloud to get Cloud SQL instance details
gcloud sql instances describe dev-postgres-instance \
  --project urania-economy-indices-dev
```

## Step 3: Install Helm Chart

### Development Environment

```bash
# Navigate to k8s directory
cd /home/dsidorenko/develop/future-economy-indices/k8s

# Install the Helm chart
helm install future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-dev.yaml \
  --namespace future-economy-indices \
  --create-namespace

# Watch the deployment
kubectl get pods -n future-economy-indices -w
```

### Production Environment

```bash
# Navigate to k8s directory
cd /home/dsidorenko/develop/future-economy-indices/k8s

# Install the Helm chart
helm install future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-prod.yaml \
  --namespace future-economy-indices \
  --create-namespace

# Watch the deployment
kubectl get pods -n future-economy-indices -w
```

### Dry Run (Recommended First)

Before actually installing, do a dry run to check for issues:

```bash
# Dry run for dev
helm install future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-dev.yaml \
  --namespace future-economy-indices \
  --create-namespace \
  --dry-run --debug

# Review the output for any errors
```

## Step 4: Verify Deployment

### Check All Resources

```bash
# Check namespace
kubectl get namespace future-economy-indices

# Check all resources in namespace
kubectl get all -n future-economy-indices

# Check deployment status
kubectl get deployment -n future-economy-indices
kubectl rollout status deployment/future-economy-indices -n future-economy-indices

# Check pods
kubectl get pods -n future-economy-indices
kubectl describe pod -n future-economy-indices

# Check services
kubectl get svc -n future-economy-indices

# Check ingress
kubectl get ingress -n future-economy-indices

# Check HPA (Horizontal Pod Autoscaler)
kubectl get hpa -n future-economy-indices
```

### View Logs

```bash
# Get pod name
POD_NAME=$(kubectl get pods -n future-economy-indices -l app=future-economy-indices -o jsonpath='{.items[0].metadata.name}')

# View logs
kubectl logs -n future-economy-indices $POD_NAME

# Follow logs in real-time
kubectl logs -n future-economy-indices $POD_NAME -f

# View logs from all pods
kubectl logs -n future-economy-indices -l app=future-economy-indices --all-containers=true
```

### Check Health Endpoints

```bash
# Port-forward to test locally
kubectl port-forward -n future-economy-indices service/future-economy-indices 3000:80

# In another terminal, test the API
curl http://localhost:3000/health
curl http://localhost:3000/api/v1/indices
```

## Step 5: Access the API

### Via Ingress (External Access)

Once the ingress is configured and the load balancer is provisioned:

```bash
# Get ingress address
kubectl get ingress -n future-economy-indices

# Wait for external IP to be assigned (may take 5-10 minutes)
kubectl get ingress -n future-economy-indices -w

# Test the API (replace with your domain)
curl https://api.dev.urania.fund/health
```

### Configure DNS

Point your domain to the ingress IP address:

```bash
# Get the external IP
INGRESS_IP=$(kubectl get ingress future-economy-indices -n future-economy-indices -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

echo "Configure your DNS:"
echo "api.dev.urania.fund A $INGRESS_IP"
```

## Upgrading

### Upgrade to New Image Version

```bash
# Method 1: Using helm upgrade with new values
helm upgrade future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-dev.yaml \
  --set image.tag=v1.2.3 \
  --namespace future-economy-indices

# Method 2: Edit values file and upgrade
# Edit values-dev.yaml to change image.tag
nano ./helm/future-economy-indices/values-dev.yaml

helm upgrade future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-dev.yaml \
  --namespace future-economy-indices

# Watch the rolling update
kubectl rollout status deployment/future-economy-indices -n future-economy-indices
```

### Rollback to Previous Version

```bash
# View release history
helm history future-economy-indices -n future-economy-indices

# Rollback to previous release
helm rollback future-economy-indices -n future-economy-indices

# Rollback to specific revision
helm rollback future-economy-indices 2 -n future-economy-indices
```

## Uninstalling

### Remove the Helm Release

```bash
# Uninstall the chart
helm uninstall future-economy-indices -n future-economy-indices

# Delete the namespace (optional, will delete all resources)
kubectl delete namespace future-economy-indices

# Verify resources are deleted
kubectl get all -n future-economy-indices
```

### Clean Up Secrets

```bash
# Delete secrets if needed
kubectl delete secret api-server-secrets -n future-economy-indices

# Or delete via namespace deletion (above)
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl get pods -n future-economy-indices

# Describe pod for events
kubectl describe pod <pod-name> -n future-economy-indices

# Check logs
kubectl logs <pod-name> -n future-economy-indices

# Check if secrets exist
kubectl get secrets -n future-economy-indices
```

### Image Pull Errors

```bash
# Verify image exists in registry
gcloud container images list --repository=gcr.io/urania-economy-indices-dev

# Check service account permissions
kubectl describe serviceaccount default -n future-economy-indices

# Manually test image pull
docker pull gcr.io/urania-economy-indices-dev/api-server:develop
```

### Database Connection Issues

```bash
# Check if secret is correctly configured
kubectl get secret api-server-secrets -n future-economy-indices -o yaml

# Decode secret to verify (be careful with sensitive data!)
kubectl get secret api-server-secrets -n future-economy-indices -o jsonpath='{.data.DATABASE_URL}' | base64 --decode

# Test database connection from pod
kubectl exec -it <pod-name> -n future-economy-indices -- /bin/sh
# Then inside pod: psql $DATABASE_URL
```

### Ingress Not Working

```bash
# Check ingress status
kubectl describe ingress -n future-economy-indices

# Check if static IP was created
gcloud compute addresses list

# Check if SSL certificate is provisioned (if using managed certs)
kubectl describe managedcertificate -n future-economy-indices

# View ingress controller logs
kubectl logs -n kube-system -l k8s-app=glbc
```

### View All Events

```bash
# View recent events in namespace
kubectl get events -n future-economy-indices --sort-by='.lastTimestamp'

# Watch events in real-time
kubectl get events -n future-economy-indices --watch
```

### Reset Everything

```bash
# Complete reset (use with caution!)
helm uninstall future-economy-indices -n future-economy-indices
kubectl delete namespace future-economy-indices
kubectl create namespace future-economy-indices

# Recreate secrets and reinstall
# (Follow steps 2 and 3 again)
```

## Common Helm Commands

```bash
# List all releases
helm list -n future-economy-indices

# Get values of installed release
helm get values future-economy-indices -n future-economy-indices

# Get all information about release
helm get all future-economy-indices -n future-economy-indices

# Show chart information
helm show chart ./helm/future-economy-indices
helm show values ./helm/future-economy-indices

# Test template rendering
helm template future-economy-indices \
  ./helm/future-economy-indices \
  --values ./helm/future-economy-indices/values-dev.yaml
```

## Environment-Specific Notes

### Development
- Uses `values-dev.yaml`
- Single replica
- Lower resource limits
- Debug logging enabled
- Domain: `api.dev.urania.fund`

### Production
- Uses `values-prod.yaml`
- Multiple replicas for HA
- Higher resource limits
- Info-level logging
- Domain: `api.urania.fund`

## Next Steps

1. Set up CI/CD pipeline for automated deployments
2. Configure monitoring and alerting (Prometheus/Grafana)
3. Set up log aggregation (Cloud Logging)
4. Configure backup strategy for database
5. Implement canary deployments
6. Add SSL/TLS certificates
7. Configure CDN for static assets

## Support

For issues or questions:
- Check application logs: `kubectl logs -n future-economy-indices -l app=future-economy-indices`
- Review Kubernetes events: `kubectl get events -n future-economy-indices`
- Consult the main README.md in project root
