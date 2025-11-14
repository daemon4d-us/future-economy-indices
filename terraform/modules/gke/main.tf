# GKE cluster module

resource "google_container_cluster" "primary" {
  name     = var.cluster_name
  location = var.region

  # Regional cluster for high availability
  # node_locations managed automatically by GCP for regional clusters

  # Remove default node pool
  remove_default_node_pool = true
  initial_node_count       = 1

  # Network configuration
  network    = var.network_name
  subnetwork = var.subnet_name

  # IP allocation for pods and services
  ip_allocation_policy {
    cluster_secondary_range_name  = "gke-pods"
    services_secondary_range_name = "gke-services"
  }

  # Workload Identity
  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  # Enable VPC-native cluster
  networking_mode = "VPC_NATIVE"

  # Maintenance window
  maintenance_policy {
    daily_maintenance_window {
      start_time = "03:00"
    }
  }

  # Release channel for automatic upgrades
  release_channel {
    channel = "REGULAR"
  }

  # Cluster addons
  addons_config {
    http_load_balancing {
      disabled = false
    }
    horizontal_pod_autoscaling {
      disabled = false
    }
    network_policy_config {
      disabled = false
    }
    gce_persistent_disk_csi_driver_config {
      enabled = true
    }
  }

  # Network policy
  network_policy {
    enabled  = true
    provider = "PROVIDER_UNSPECIFIED"
  }

  # Enable binary authorization
  binary_authorization {
    evaluation_mode = "PROJECT_SINGLETON_POLICY_ENFORCE"
  }

  # Enable shielded nodes
  enable_shielded_nodes = true

  # Private cluster configuration
  private_cluster_config {
    enable_private_nodes    = true
    enable_private_endpoint = false
    master_ipv4_cidr_block  = "172.16.0.0/28"
  }

  # Master authorized networks (allow access from anywhere, adjust for production)
  master_authorized_networks_config {
    cidr_blocks {
      cidr_block   = "0.0.0.0/0"
      display_name = "All networks"
    }
  }

  # Resource labels
  resource_labels = {
    environment = var.environment
    managed_by  = "terraform"
  }
}

# Node pool
resource "google_container_node_pool" "primary_nodes" {
  name       = "${var.cluster_name}-node-pool"
  location   = var.region
  cluster    = google_container_cluster.primary.name
  node_count = var.min_node_count

  # Autoscaling configuration
  autoscaling {
    min_node_count = var.min_node_count
    max_node_count = var.max_node_count
  }

  # Management
  management {
    auto_repair  = true
    auto_upgrade = true
  }

  # Node configuration
  node_config {
    preemptible  = var.environment == "dev" ? true : false
    machine_type = var.node_machine_type

    # Service account with minimal permissions
    service_account = google_service_account.gke_nodes.email
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]

    # Workload Identity
    workload_metadata_config {
      mode = "GKE_METADATA"
    }

    # Security
    shielded_instance_config {
      enable_secure_boot          = true
      enable_integrity_monitoring = true
    }

    # Metadata
    metadata = {
      disable-legacy-endpoints = "true"
    }

    # Labels
    labels = {
      environment = var.environment
    }

    # Tags for firewall rules
    tags = ["gke-node", "${var.environment}-gke"]

    # Disk
    disk_size_gb = 50
    disk_type    = "pd-standard"
  }

  # Upgrade settings
  upgrade_settings {
    max_surge       = 1
    max_unavailable = 0
  }
}

# Service account for GKE nodes
resource "google_service_account" "gke_nodes" {
  account_id   = "${var.environment}-gke-node-sa"
  display_name = "GKE Node Service Account for ${var.environment}"
}

# IAM bindings for node service account
resource "google_project_iam_member" "gke_node_permissions" {
  for_each = toset([
    "roles/logging.logWriter",
    "roles/monitoring.metricWriter",
    "roles/monitoring.viewer",
    "roles/stackdriver.resourceMetadata.writer"
  ])

  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.gke_nodes.email}"
}
