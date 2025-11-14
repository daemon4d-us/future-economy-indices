# CI/CD Pipeline Setup Guide

Quick setup guide for the Future Economy Indices CI/CD pipeline.

## Prerequisites

- GCP projects created (`urania-economy-indices-dev` and `urania-economy-indices-prod`)
- GKE clusters running in both projects
- GitHub repository set up
- `gcloud` CLI installed and authenticated

## Quick Setup Steps

### 1. Enable Required APIs

```bash
# For both dev and prod projects
for PROJECT in urania-economy-indices-dev urania-economy-indices-prod; do
  gcloud services enable \
    container.googleapis.com \
    containerregistry.googleapis.com \
    iamcredentials.googleapis.com \
    --project=$PROJECT
done
```

### 2. Set Up Workload Identity Federation

Run this script to set up WIF for both environments:

```bash
#!/bin/bash

setup_wif() {
  PROJECT_ID=$1
  REPO="daemon4d-us/future-economy-indices"

  echo "Setting up WIF for $PROJECT_ID..."

  # Create workload identity pool
  gcloud iam workload-identity-pools create github-actions-pool \
    --project=$PROJECT_ID \
    --location=global \
    --display-name="GitHub Actions Pool" || true

  # Create OIDC provider
  gcloud iam workload-identity-pools providers create-oidc github-provider \
    --project=$PROJECT_ID \
    --location=global \
    --workload-identity-pool=github-actions-pool \
    --display-name="GitHub Provider" \
    --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
    --issuer-uri="https://token.actions.githubusercontent.com" || true

  # Create service account
  gcloud iam service-accounts create github-actions \
    --project=$PROJECT_ID \
    --display-name="GitHub Actions Service Account" || true

  # Grant permissions
  gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
    --role="roles/container.developer"

  gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
    --role="roles/storage.admin"

  gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
    --role="roles/artifactregistry.writer"

  # Get project number
  PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

  # Allow GitHub to impersonate the service account
  gcloud iam service-accounts add-iam-policy-binding \
    github-actions@${PROJECT_ID}.iam.gserviceaccount.com \
    --project=$PROJECT_ID \
    --role="roles/iam.workloadIdentityUser" \
    --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/attribute.repository/${REPO}"

  # Output secrets for GitHub
  echo ""
  echo "GitHub Secrets for $PROJECT_ID:"
  echo "================================"
  echo "WIF_PROVIDER: projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions-pool/providers/github-provider"
  echo "WIF_SERVICE_ACCOUNT: github-actions@${PROJECT_ID}.iam.gserviceaccount.com"
  echo ""
}

# Setup for both projects
setup_wif "urania-economy-indices-dev"
setup_wif "urania-economy-indices-prod"
```

### 3. Add GitHub Secrets

Go to your GitHub repository → Settings → Secrets and variables → Actions

Add these secrets:

**For Development:**
- `WIF_PROVIDER_DEV`
- `WIF_SERVICE_ACCOUNT_DEV`

**For Production:**
- `WIF_PROVIDER_PROD`
- `WIF_SERVICE_ACCOUNT_PROD`

### 4. Test the Pipeline

```bash
# Create a test change
echo "# Test" >> README.md

# Commit and push to develop branch
git checkout -b develop
git add README.md
git commit -m "test: trigger CI/CD pipeline"
git push origin develop

# Watch the build in GitHub Actions
# https://github.com/daemon4d-us/future-economy-indices/actions
```

## Verification

After setup, verify everything works:

### 1. Check Workload Identity Pool

```bash
PROJECT_ID="urania-economy-indices-dev"
gcloud iam workload-identity-pools describe github-actions-pool \
  --project=$PROJECT_ID \
  --location=global
```

### 2. Check Service Account Permissions

```bash
PROJECT_ID="urania-economy-indices-dev"
gcloud projects get-iam-policy $PROJECT_ID \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com"
```

### 3. Test Docker Image Build

```bash
# Build locally
docker build -t test-build .

# If successful, the CI/CD pipeline should work
```

## Workflow Overview

### Branch Strategy

- `master` → Production deployment
- `develop` → Development deployment
- `feature/*` → Build only (no deployment)
- `pr-*` → Build only (no deployment)

### Automatic Flow

1. **Push to develop/master**
   ↓
2. **Build workflow runs**
   - Builds Docker image
   - Pushes to GCR
   ↓
3. **Deploy workflow runs**
   - Deploys to GKE via Helm
   - Verifies deployment

### Manual Deployment

Use workflow_dispatch in GitHub Actions UI to:
- Deploy specific image tag
- Choose environment (dev/prod)
- Useful for hotfixes or rollbacks

## Troubleshooting

### "Permission denied" errors

Check service account has correct roles:
```bash
PROJECT_ID="urania-economy-indices-dev"
gcloud projects get-iam-policy $PROJECT_ID \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:github-actions@"
```

### "Image not found" errors

Check if image was pushed to GCR:
```bash
gcloud container images list --repository=gcr.io/urania-economy-indices-dev
```

### Deployment hangs or fails

Check pod status and logs:
```bash
kubectl get pods -n future-economy-indices
kubectl logs -n future-economy-indices -l app=future-economy-indices
kubectl describe pod <pod-name> -n future-economy-indices
```

## Security Checklist

- [ ] Workload Identity Federation configured (no service account keys)
- [ ] GitHub secrets added and verified
- [ ] Service account has minimal required permissions
- [ ] GKE cluster has Workload Identity enabled
- [ ] Binary authorization enabled on GKE
- [ ] Network policies configured
- [ ] Secrets stored in Kubernetes secrets (not in code)
- [ ] Image scanning enabled in GCR

## Cost Optimization

- Development cluster uses preemptible nodes
- Development uses smaller machine types
- Docker layer caching reduces build times
- GCR lifecycle policies for old images

## Monitoring

Set up monitoring for:
- GitHub Actions workflow success/failure
- GCR storage usage
- GKE pod health
- Application logs and metrics

## Next Steps

1. Set up branch protection rules
2. Require PR reviews for master branch
3. Add integration tests to pipeline
4. Set up Slack/Discord notifications
5. Configure automatic rollback on errors
6. Implement canary deployments
7. Add performance testing to pipeline

## Useful Commands

```bash
# View workflow runs
gh workflow list
gh run list --workflow=build.yml

# View recent deployments
helm list -n future-economy-indices -a

# Check cluster connectivity
gcloud container clusters get-credentials dev-gke-cluster \
  --region us-central1 \
  --project urania-economy-indices-dev
kubectl cluster-info

# Force rebuild and redeploy
git commit --allow-empty -m "trigger: rebuild"
git push origin develop
```

## Support

For detailed documentation, see:
- [GitHub Workflows README](./.github/workflows/README.md)
- [Kubernetes README](./k8s/README.md)
- [Deployment Guide](./DEPLOYMENT.md)
