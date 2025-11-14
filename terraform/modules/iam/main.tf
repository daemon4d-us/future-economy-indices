# IAM module - Service accounts and permissions

# Service account for API server workload identity
resource "google_service_account" "api_server" {
  account_id   = "${var.environment}-api-server"
  display_name = "API Server Service Account for ${var.environment}"
}

# Grant API server SA access to Secret Manager
resource "google_project_iam_member" "api_server_secret_accessor" {
  project = var.project_id
  role    = "roles/secretmanager.secretAccessor"
  member  = "serviceAccount:${google_service_account.api_server.email}"
}

# Grant API server SA access to Cloud SQL
resource "google_project_iam_member" "api_server_cloudsql_client" {
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.api_server.email}"
}

# Workload Identity binding
resource "google_service_account_iam_binding" "api_server_workload_identity" {
  service_account_id = google_service_account.api_server.name
  role               = "roles/iam.workloadIdentityUser"

  members = [
    "serviceAccount:${var.project_id}.svc.id.goog[future-economy-indices/api-server]"
  ]
}
