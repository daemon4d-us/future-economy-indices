# CI/CD Pipeline Documentation

This document describes the GitHub Actions CI/CD pipeline for the Future Economy Indices project.

## Overview

The pipeline consists of two main workflows:

1. **Build and Push Docker Image** (`build.yml`) - Builds the Docker image and pushes it to Google Container Registry (GCR)
2. **Deploy to GKE with Helm** (`deploy.yml`) - Deploys the application to Google Kubernetes Engine using Helm

## Workflows

### 1. Build and Push Docker Image

**File**: `.github/workflows/build.yml`

**Triggers**:
- Push to `master` or `develop` branches (when code changes in `crates/`, `src/`, `Dockerfile`, or `Cargo.*`)
- Pull requests to `master` or `develop` branches

**What it does**:
1. Checks out the code
2. Determines the environment (prod/dev) based on the branch
3. Authenticates to Google Cloud using Workload Identity Federation
4. Builds the Docker image using BuildKit with layer caching
5. Pushes the image to GCR with appropriate tags:
   - `master` branch → `gcr.io/urania-economy-indices-prod/api-server:latest`
   - `develop` branch → `gcr.io/urania-economy-indices-dev/api-server:develop`
   - Pull requests → `gcr.io/urania-economy-indices-dev/api-server:pr-{number}`
6. Also tags with the short commit SHA for versioning

**Environment Variables**:
- `master` branch uses production GCP project (`urania-economy-indices-prod`)
- `develop` branch uses development GCP project (`urania-economy-indices-dev`)

### 2. Deploy to GKE with Helm

**File**: `.github/workflows/deploy.yml`

**Triggers**:
- Automatically after successful build workflow completion
- Manual trigger via `workflow_dispatch` (allows choosing environment and image tag)

**What it does**:
1. Waits for the build workflow to complete successfully
2. Determines the target environment (prod/dev)
3. Authenticates to Google Cloud
4. Connects to the appropriate GKE cluster
5. Installs/upgrades the Helm release with the new image
6. Waits for the deployment to complete
7. Verifies the deployment status

**Deployment Strategy**:
- Uses `helm upgrade --install` for idempotent deployments
- `--atomic` flag ensures rollback on failure
- `--wait` ensures Helm waits for all resources to be ready
- 10-minute timeout for deployment

**Environments**:
- **Production** (`master` branch):
  - Project: `urania-economy-indices-prod`
  - Cluster: `prod-gke-cluster`
  - Values: `values-prod.yaml`
  - Image tag: `latest`

- **Development** (`develop` branch):
  - Project: `urania-economy-indices-dev`
  - Cluster: `dev-gke-cluster`
  - Values: `values-dev.yaml`
  - Image tag: `develop`

## Required GitHub Secrets

You need to configure the following secrets in your GitHub repository:

### Workload Identity Federation (WIF) - Development
- `WIF_PROVIDER_DEV`: Workload Identity Provider for dev environment
  - Format: `projects/{PROJECT_NUMBER}/locations/global/workloadIdentityPools/{POOL_ID}/providers/{PROVIDER_ID}`
- `WIF_SERVICE_ACCOUNT_DEV`: Service account email for dev
  - Format: `github-actions@urania-economy-indices-dev.iam.gserviceaccount.com`

### Workload Identity Federation (WIF) - Production
- `WIF_PROVIDER_PROD`: Workload Identity Provider for prod environment
  - Format: `projects/{PROJECT_NUMBER}/locations/global/workloadIdentityPools/{POOL_ID}/providers/{PROVIDER_ID}`
- `WIF_SERVICE_ACCOUNT_PROD`: Service account email for prod
  - Format: `github-actions@urania-economy-indices-prod.iam.gserviceaccount.com`

## Setting Up Workload Identity Federation

### 1. Create Workload Identity Pool

```bash
# For Development
gcloud iam workload-identity-pools create github-actions-pool \
  --project=urania-economy-indices-dev \
  --location=global \
  --display-name="GitHub Actions Pool"

# For Production
gcloud iam workload-identity-pools create github-actions-pool \
  --project=urania-economy-indices-prod \
  --location=global \
  --display-name="GitHub Actions Pool"
```

### 2. Create Workload Identity Provider

```bash
# For Development
gcloud iam workload-identity-pools providers create-oidc github-provider \
  --project=urania-economy-indices-dev \
  --location=global \
  --workload-identity-pool=github-actions-pool \
  --display-name="GitHub Provider" \
  --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
  --issuer-uri="https://token.actions.githubusercontent.com"

# For Production
gcloud iam workload-identity-pools providers create-oidc github-provider \
  --project=urania-economy-indices-prod \
  --location=global \
  --workload-identity-pool=github-actions-pool \
  --display-name="GitHub Provider" \
  --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
  --issuer-uri="https://token.actions.githubusercontent.com"
```

### 3. Create Service Account

```bash
# For Development
gcloud iam service-accounts create github-actions \
  --project=urania-economy-indices-dev \
  --display-name="GitHub Actions Service Account"

# For Production
gcloud iam service-accounts create github-actions \
  --project=urania-economy-indices-prod \
  --display-name="GitHub Actions Service Account"
```

### 4. Grant Permissions

```bash
# For Development
PROJECT_ID="urania-economy-indices-dev"
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/container.developer"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/storage.admin"

# For Production
PROJECT_ID="urania-economy-indices-prod"
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/container.developer"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/storage.admin"
```

### 5. Allow GitHub to Impersonate Service Account

```bash
# For Development
PROJECT_ID="urania-economy-indices-dev"
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
REPO="daemon4d-us/future-economy-indices"

gcloud iam service-accounts add-iam-policy-binding \
  github-actions@${PROJECT_ID}.iam.gserviceaccount.com \
  --project=$PROJECT_ID \
  --role="roles/iam.workloadIdentityUser" \
  --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/attribute.repository/${REPO}"

# For Production
PROJECT_ID="urania-economy-indices-prod"
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
REPO="daemon4d-us/future-economy-indices"

gcloud iam service-accounts add-iam-policy-binding \
  github-actions@${PROJECT_ID}.iam.gserviceaccount.com \
  --project=$PROJECT_ID \
  --role="roles/iam.workloadIdentityUser" \
  --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/attribute.repository/${REPO}"
```

### 6. Get WIF Provider Resource Names

```bash
# For Development
PROJECT_NUMBER=$(gcloud projects describe urania-economy-indices-dev --format="value(projectNumber)")
echo "WIF_PROVIDER_DEV: projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/providers/github-provider"
echo "WIF_SERVICE_ACCOUNT_DEV: github-actions@urania-economy-indices-dev.iam.gserviceaccount.com"

# For Production
PROJECT_NUMBER=$(gcloud projects describe urania-economy-indices-prod --format="value(projectNumber)")
echo "WIF_PROVIDER_PROD: projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/providers/github-provider"
echo "WIF_SERVICE_ACCOUNT_PROD: github-actions@urania-economy-indices-prod.iam.gserviceaccount.com"
```

## Deployment Flow

### Automatic Deployment

1. Developer pushes code to `develop` or `master` branch
2. **Build workflow** triggers:
   - Builds Docker image
   - Pushes to appropriate GCR repository
3. **Deploy workflow** triggers automatically:
   - Waits for build to complete
   - Deploys to appropriate GKE cluster using Helm
4. Verification steps ensure deployment is successful

### Manual Deployment

You can manually trigger a deployment from the GitHub Actions tab:

1. Go to "Actions" tab in GitHub
2. Select "Deploy to GKE with Helm"
3. Click "Run workflow"
4. Choose:
   - Environment (dev or prod)
   - Image tag (optional, defaults to latest/develop)
5. Click "Run workflow"

## Local Development

### Build Docker Image Locally

```bash
# Build for development
docker build -t api-server:local .

# Tag for GCR
docker tag api-server:local gcr.io/urania-economy-indices-dev/api-server:local

# Push to GCR (requires authentication)
gcloud auth configure-docker
docker push gcr.io/urania-economy-indices-dev/api-server:local
```

### Deploy Locally with Helm

```bash
# Development
helm upgrade --install future-economy-indices \
  k8s/helm/future-economy-indices \
  --values k8s/helm/future-economy-indices/values-dev.yaml \
  --set image.tag=local \
  --namespace future-economy-indices \
  --create-namespace

# Production (use with caution!)
helm upgrade --install future-economy-indices \
  k8s/helm/future-economy-indices \
  --values k8s/helm/future-economy-indices/values-prod.yaml \
  --namespace future-economy-indices \
  --create-namespace
```

## Monitoring Deployments

### View Workflow Runs

- Go to the "Actions" tab in your GitHub repository
- Click on a workflow run to see detailed logs

### View GKE Deployment Status

```bash
# Connect to cluster
gcloud container clusters get-credentials dev-gke-cluster \
  --region us-central1 \
  --project urania-economy-indices-dev

# Check deployment status
kubectl get pods -n future-economy-indices
kubectl get deployment -n future-economy-indices
kubectl describe deployment future-economy-indices -n future-economy-indices

# View logs
kubectl logs -n future-economy-indices -l app=future-economy-indices --tail=100

# Check Helm release
helm list -n future-economy-indices
helm status future-economy-indices -n future-economy-indices
```

## Troubleshooting

### Build Failures

1. Check the build logs in GitHub Actions
2. Verify Dockerfile syntax
3. Ensure all dependencies are available
4. Check GCP authentication

### Deployment Failures

1. Check deployment logs in GitHub Actions
2. Verify Helm chart syntax: `helm lint k8s/helm/future-economy-indices`
3. Check GKE cluster status
4. Verify secrets exist: `kubectl get secrets -n future-economy-indices`
5. Check pod events: `kubectl describe pod <pod-name> -n future-economy-indices`

### Rollback

```bash
# List Helm releases
helm list -n future-economy-indices

# View release history
helm history future-economy-indices -n future-economy-indices

# Rollback to previous version
helm rollback future-economy-indices -n future-economy-indices

# Rollback to specific revision
helm rollback future-economy-indices 3 -n future-economy-indices
```

## Security Best Practices

1. **Never commit secrets** to the repository
2. **Use Workload Identity Federation** instead of service account keys
3. **Limit service account permissions** to minimum required
4. **Enable binary authorization** on GKE clusters
5. **Use separate projects** for dev and prod environments
6. **Review and approve** production deployments
7. **Enable audit logging** for all GCP resources

## Next Steps

1. Set up monitoring and alerting (Prometheus/Grafana)
2. Implement canary deployments
3. Add integration tests to the pipeline
4. Set up automatic rollback on error rate increase
5. Implement secrets rotation
6. Add Slack/Discord notifications for deployment status
