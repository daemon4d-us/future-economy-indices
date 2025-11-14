# Variables for Terraform configuration

variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP region for resources"
  type        = string
  default     = "us-central1"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

# Database configuration
variable "database_tier" {
  description = "Cloud SQL instance tier"
  type        = string
  default     = "db-f1-micro" # For dev/staging. Use db-n1-standard-1 for prod
}

# GKE configuration
variable "node_machine_type" {
  description = "GKE node machine type"
  type        = string
  default     = "e2-medium"
}

variable "min_node_count" {
  description = "Minimum number of nodes in the cluster"
  type        = number
  default     = 1
}

variable "max_node_count" {
  description = "Maximum number of nodes in the cluster"
  type        = number
  default     = 5
}

# Kubernetes configuration
variable "k8s_namespace" {
  description = "Kubernetes namespace for the application"
  type        = string
  default     = "future-economy-indices"
}
