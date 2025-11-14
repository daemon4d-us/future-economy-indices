# Makefile for Future Economy Indices deployment

.PHONY: help build push deploy clean test

# Configuration
PROJECT_ID ?= urania-economy-indices-prod
REGION ?= us-central1
CLUSTER_NAME ?= prod-gke-cluster
IMAGE_NAME ?= api-server
IMAGE_TAG ?= latest
REGISTRY ?= gcr.io

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development
test: ## Run all tests
	cargo test --all-features --workspace

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test ## Run all checks (fmt, lint, test)

# Docker
build: ## Build Docker image
	docker build -t $(REGISTRY)/$(PROJECT_ID)/$(IMAGE_NAME):$(IMAGE_TAG) .

push: build ## Build and push Docker image to GCR
	docker push $(REGISTRY)/$(PROJECT_ID)/$(IMAGE_NAME):$(IMAGE_TAG)

# GCP Authentication
gcp-auth: ## Authenticate with GCP
	gcloud auth login
	gcloud config set project $(PROJECT_ID)

# GCS Backend Setup
gcs-bucket-create: ## Create GCS bucket for Terraform state
	@echo "Creating GCS bucket for Terraform state..."
	gsutil mb -p $(PROJECT_ID) -l $(REGION) gs://future-economy-terraform-state || true
	gsutil versioning set on gs://future-economy-terraform-state
	@echo "✅ GCS bucket created and versioning enabled"

gcs-bucket-check: ## Check if GCS bucket exists
	@gsutil ls -b gs://future-economy-terraform-state > /dev/null 2>&1 && echo "✅ Bucket exists" || echo "❌ Bucket does not exist - run 'make gcs-bucket-create'"

gcs-bucket-info: ## Show GCS bucket information
	gsutil ls -b gs://future-economy-terraform-state
	gsutil versioning get gs://future-economy-terraform-state

# Terraform
tf-init: ## Initialize Terraform
	cd terraform && terraform init

tf-plan-dev: ## Plan Terraform changes for dev environment
	cd terraform && terraform plan -var-file=environments/dev/terraform.tfvars

tf-plan-prod: ## Plan Terraform changes for prod environment
	cd terraform && terraform plan -var-file=environments/prod/terraform.tfvars

tf-apply-dev: ## Apply Terraform for dev environment
	cd terraform && terraform apply -var-file=environments/dev/terraform.tfvars

tf-apply-prod: ## Apply Terraform for prod environment
	cd terraform && terraform apply -var-file=environments/prod/terraform.tfvars

tf-destroy-dev: ## Destroy dev environment
	cd terraform && terraform destroy -var-file=environments/dev/terraform.tfvars

tf-destroy-prod: ## Destroy prod environment (use with caution!)
	cd terraform && terraform destroy -var-file=environments/prod/terraform.tfvars

tf-output: ## Show Terraform outputs
	cd terraform && terraform output

# Kubernetes
k8s-credentials: ## Get GKE cluster credentials
	gcloud container clusters get-credentials $(CLUSTER_NAME) \
		--region $(REGION) \
		--project $(PROJECT_ID)

helm-deploy-dev: ## Deploy to Kubernetes (dev environment) using Helm
	helm upgrade --install future-economy-indices ./k8s/helm/future-economy-indices \
		-f k8s/helm/future-economy-indices/values-dev.yaml \
		--set image.repository=$(REGISTRY)/$(PROJECT_ID)/$(IMAGE_NAME) \
		--set image.tag=$(IMAGE_TAG) \
		--namespace future-economy-indices \
		--create-namespace

helm-deploy-prod: ## Deploy to Kubernetes (prod environment) using Helm
	helm upgrade --install future-economy-indices ./k8s/helm/future-economy-indices \
		-f k8s/helm/future-economy-indices/values-prod.yaml \
		--set image.repository=$(REGISTRY)/$(PROJECT_ID)/$(IMAGE_NAME) \
		--set image.tag=$(IMAGE_TAG) \
		--namespace future-economy-indices \
		--create-namespace

k8s-deploy-dev: helm-deploy-dev ## Alias for helm-deploy-dev

k8s-deploy-prod: helm-deploy-prod ## Alias for helm-deploy-prod

k8s-deploy: k8s-deploy-prod ## Deploy to Kubernetes (defaults to prod)

k8s-status: ## Check deployment status
	kubectl get all -n future-economy-indices
	kubectl get ingress -n future-economy-indices

k8s-logs: ## View API server logs
	kubectl logs -f deployment/api-server -n future-economy-indices

k8s-exec: ## Execute shell in API server pod
	kubectl exec -it deployment/api-server -n future-economy-indices -- /bin/sh

k8s-rollback: ## Rollback deployment
	kubectl rollout undo deployment/api-server -n future-economy-indices

# Database
db-migrate: ## Run database migrations in production
	$(eval DB_POD=$(shell kubectl get pod -n future-economy-indices -l app=api-server -o jsonpath='{.items[0].metadata.name}'))
	kubectl exec -it $(DB_POD) -n future-economy-indices -- /app/api-server db init

db-status: ## Check database status
	$(eval DB_POD=$(shell kubectl get pod -n future-economy-indices -l app=api-server -o jsonpath='{.items[0].metadata.name}'))
	kubectl exec -it $(DB_POD) -n future-economy-indices -- /app/api-server db status

# Full Deployment Pipeline
deploy-dev: tf-apply-dev k8s-credentials push k8s-deploy-dev k8s-status ## Full deployment to dev environment

deploy-prod: tf-apply-prod k8s-credentials push k8s-deploy-prod k8s-status ## Full deployment to prod environment

# Monitoring
logs-cloudlog: ## View logs in Cloud Logging
	gcloud logging read "resource.type=k8s_container AND resource.labels.namespace_name=future-economy-indices" \
		--limit 50 \
		--format json

metrics: ## Show pod metrics
	kubectl top pods -n future-economy-indices
	kubectl top nodes

hpa-status: ## Show HPA status
	kubectl get hpa -n future-economy-indices
	kubectl describe hpa api-server -n future-economy-indices

# Cleanup
clean: ## Clean local build artifacts
	cargo clean
	rm -rf target/

# Secrets Management
secrets-create: ## Create Kubernetes secrets (interactive)
	@echo "Enter DATABASE_URL:"
	@read DATABASE_URL && \
	echo "Enter POLYGON_API_KEY:" && \
	read POLYGON_API_KEY && \
	echo "Enter ANTHROPIC_API_KEY:" && \
	read ANTHROPIC_API_KEY && \
	kubectl create secret generic api-server-secrets \
		--namespace=future-economy-indices \
		--from-literal=DATABASE_URL=$$DATABASE_URL \
		--from-literal=POLYGON_API_KEY=$$POLYGON_API_KEY \
		--from-literal=ANTHROPIC_API_KEY=$$ANTHROPIC_API_KEY \
		--dry-run=client -o yaml | kubectl apply -f -

secrets-update: secrets-create ## Update Kubernetes secrets

# Utilities
port-forward: ## Forward port 3000 to local machine
	kubectl port-forward -n future-economy-indices deployment/api-server 3000:3000

scale: ## Scale deployment (usage: make scale REPLICAS=5)
	kubectl scale deployment api-server --replicas=$(REPLICAS) -n future-economy-indices

restart: ## Restart deployment
	kubectl rollout restart deployment/future-economy-indices -n future-economy-indices

# Helm Commands
helm-lint: ## Lint Helm chart
	helm lint k8s/helm/future-economy-indices

helm-template-dev: ## Preview dev Helm templates
	helm template future-economy-indices ./k8s/helm/future-economy-indices \
		-f k8s/helm/future-economy-indices/values-dev.yaml

helm-template-prod: ## Preview prod Helm templates
	helm template future-economy-indices ./k8s/helm/future-economy-indices \
		-f k8s/helm/future-economy-indices/values-prod.yaml

helm-uninstall: ## Uninstall Helm release
	helm uninstall future-economy-indices --namespace future-economy-indices

helm-rollback: ## Rollback to previous Helm release
	helm rollback future-economy-indices --namespace future-economy-indices
