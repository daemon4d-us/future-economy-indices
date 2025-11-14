# Main Terraform configuration for Future Economy Indices
# This creates the complete GCP infrastructure including GKE, Cloud SQL, and networking

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
  }

  backend "gcs" {
    bucket = "future-economy-terraform-state"
    prefix = "terraform/state"
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

provider "google-beta" {
  project = var.project_id
  region  = var.region
}

# Enable required APIs
resource "google_project_service" "required_apis" {
  for_each = toset([
    "compute.googleapis.com",
    "container.googleapis.com",
    "sqladmin.googleapis.com",
    "servicenetworking.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "iam.googleapis.com",
    "secretmanager.googleapis.com",
  ])

  service            = each.value
  disable_on_destroy = false
}

# Networking
module "networking" {
  source = "./modules/networking"

  project_id   = var.project_id
  region       = var.region
  network_name = "${var.environment}-vpc"

  depends_on = [google_project_service.required_apis]
}

# Cloud SQL (PostgreSQL)
module "cloudsql" {
  source = "./modules/cloudsql"

  project_id        = var.project_id
  region            = var.region
  environment       = var.environment
  network_self_link = module.networking.network_self_link
  database_tier     = var.database_tier
  database_name     = "future_economy_indices"

  depends_on = [module.networking]
}

# GKE Cluster
module "gke" {
  source = "./modules/gke"

  project_id        = var.project_id
  region            = var.region
  environment       = var.environment
  network_name      = module.networking.network_name
  subnet_name       = module.networking.subnet_name
  cluster_name      = "${var.environment}-gke-cluster"
  node_machine_type = var.node_machine_type
  min_node_count    = var.min_node_count
  max_node_count    = var.max_node_count

  depends_on = [module.networking]
}

# IAM and Service Accounts
module "iam" {
  source = "./modules/iam"

  project_id  = var.project_id
  environment = var.environment

  depends_on = [module.gke]
}

# Static IP for Ingress
resource "google_compute_global_address" "ingress_ip" {
  name         = "future-economy-api-ip"
  address_type = "EXTERNAL"

  depends_on = [google_project_service.required_apis]
}
