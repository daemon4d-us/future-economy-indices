variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "network_self_link" {
  description = "VPC network self link"
  type        = string
}

variable "database_tier" {
  description = "Database instance tier"
  type        = string
  default     = "db-f1-micro"
}

variable "database_name" {
  description = "Database name"
  type        = string
  default     = "future_economy_indices"
}

variable "disk_size_gb" {
  description = "Disk size in GB"
  type        = number
  default     = 10
}
