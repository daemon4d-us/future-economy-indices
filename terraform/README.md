# Terraform Infrastructure

Infrastructure as Code for Future Economy Indices GCP deployment.

## Directory Structure

```
terraform/
├── main.tf                 # Main configuration
├── variables.tf            # Input variables
├── outputs.tf             # Output values
├── modules/
│   ├── networking/        # VPC, subnets, NAT
│   ├── gke/              # GKE cluster configuration
│   ├── cloudsql/         # Cloud SQL PostgreSQL
│   └── iam/              # Service accounts and permissions
└── environments/
    ├── dev/              # Development environment
    │   └── terraform.tfvars
    └── prod/             # Production environment
        └── terraform.tfvars
```

## Quick Start

### Prerequisites

```bash
# Install Terraform
brew install terraform  # macOS
# or download from https://www.terraform.io/downloads

# Authenticate with GCP
gcloud auth application-default login
```

### Initialize

```bash
terraform init
```

### Deploy Development Environment

```bash
terraform plan -var-file=environments/dev/terraform.tfvars
terraform apply -var-file=environments/dev/terraform.tfvars
```

### Deploy Production Environment

```bash
terraform plan -var-file=environments/prod/terraform.tfvars
terraform apply -var-file=environments/prod/terraform.tfvars
```

## Modules

### Networking Module

Creates:
- VPC network
- Subnets with secondary ranges for GKE
- Cloud Router and Cloud NAT
- Firewall rules
- Private service connection for Cloud SQL

### GKE Module

Creates:
- Regional GKE cluster
- Node pool with autoscaling
- Workload Identity configuration
- Service account for nodes

Features:
- Private nodes (no external IPs)
- Shielded nodes
- Binary authorization
- Automatic upgrades

### Cloud SQL Module

Creates:
- PostgreSQL 15 instance
- Database and users
- Private IP configuration
- Automated backups
- Password stored in Secret Manager

### IAM Module

Creates:
- Service accounts for workloads
- Workload Identity bindings
- IAM role assignments

## Outputs

After applying, get outputs:

```bash
# Get all outputs
terraform output

# Get specific output
terraform output gke_cluster_name
terraform output cloudsql_connection_name

# Get kubectl config command
terraform output kubeconfig_command
```

## State Management

Terraform state is stored in Google Cloud Storage:
- Bucket: `future-economy-terraform-state`
- Versioning enabled
- Encryption at rest

## Environments

### Development
- Smaller instances
- Preemptible nodes
- Single zone
- Minimal backups

### Production
- High availability (regional)
- Standard nodes
- Enhanced backups
- Deletion protection

## Cost Optimization

Development environment (~$30/month):
- 1x e2-medium node
- db-f1-micro database
- Preemptible nodes

Production environment (~$145/month):
- 2-10x e2-standard-2 nodes
- db-n1-standard-1 database
- Regional deployment

## Security

- Private GKE cluster
- Workload Identity (no service account keys)
- Cloud SQL private IP only
- Secrets in Secret Manager
- Network policies enabled

## Maintenance

### Update Terraform

```bash
# Update providers
terraform init -upgrade

# Check what will change
terraform plan -var-file=environments/prod/terraform.tfvars
```

### Import Existing Resources

```bash
terraform import module.gke.google_container_cluster.primary projects/PROJECT_ID/locations/REGION/clusters/CLUSTER_NAME
```

### Destroy Resources

```bash
# Development
terraform destroy -var-file=environments/dev/terraform.tfvars

# Production (with approval)
terraform destroy -var-file=environments/prod/terraform.tfvars
```

## Troubleshooting

### State Lock Issues

```bash
# Force unlock (use carefully)
terraform force-unlock LOCK_ID
```

### API Not Enabled

If you get "API not enabled" errors, manually enable:

```bash
gcloud services enable compute.googleapis.com
gcloud services enable container.googleapis.com
gcloud services enable sqladmin.googleapis.com
```

### Quotas

Check and request quota increases:

```bash
gcloud compute project-info describe --project=PROJECT_ID
```

## Best Practices

1. **Always run `terraform plan` before `apply`**
2. **Use workspaces for multiple environments** (optional)
3. **Tag resources** for cost tracking
4. **Enable deletion protection** for production
5. **Review state file** access permissions
6. **Use remote state locking**

## Variables

Key variables you must set:

```hcl
project_id        # Your GCP project ID
region            # GCP region (default: us-central1)
environment       # dev, staging, or prod
database_tier     # Cloud SQL tier
node_machine_type # GKE node machine type
```

See `variables.tf` for full list.

## Contributing

When adding new resources:
1. Add to appropriate module
2. Update module outputs
3. Document in README
4. Test in dev environment first
