# Cloud SQL PostgreSQL module

resource "random_password" "db_password" {
  length  = 32
  special = true
}

resource "google_sql_database_instance" "postgres" {
  name             = "${var.environment}-postgres-instance"
  database_version = "POSTGRES_15"
  region           = var.region

  settings {
    tier              = var.database_tier
    availability_type = var.environment == "prod" ? "REGIONAL" : "ZONAL"
    disk_type         = "PD_SSD"
    disk_size         = var.disk_size_gb
    disk_autoresize   = true

    # Backup configuration
    backup_configuration {
      enabled                        = true
      start_time                     = "02:00"
      point_in_time_recovery_enabled = var.environment == "prod" ? true : false
      transaction_log_retention_days = 7
      backup_retention_settings {
        retained_backups = 7
        retention_unit   = "COUNT"
      }
    }

    # IP configuration - private IP only
    ip_configuration {
      ipv4_enabled    = false
      private_network = var.network_self_link
      ssl_mode        = "ENCRYPTED_ONLY"
    }

    # Maintenance window
    maintenance_window {
      day          = 7 # Sunday
      hour         = 3
      update_track = "stable"
    }

    # Database flags (conditional based on tier)
    database_flags {
      name  = "max_connections"
      value = var.database_tier == "db-f1-micro" ? "25" : "100"
    }

    database_flags {
      name  = "shared_buffers"
      # db-f1-micro (1GB RAM): valid range 13107-78643 (set to 16384 = ~16MB)
      # db-n1-standard-1 (3.75GB RAM): can use 256000 (~256MB)
      value = var.database_tier == "db-f1-micro" ? "16384" : "256000"
    }

    database_flags {
      name  = "work_mem"
      # Smaller work_mem for micro instances
      value = var.database_tier == "db-f1-micro" ? "2048" : "4096"
    }

    # Insights configuration
    insights_config {
      query_insights_enabled  = true
      query_string_length     = 1024
      record_application_tags = true
      record_client_address   = true
    }
  }

  deletion_protection = var.environment == "prod" ? true : false

  depends_on = [var.network_self_link]
}

# Database
resource "google_sql_database" "database" {
  name     = var.database_name
  instance = google_sql_database_instance.postgres.name
}

# Default user
resource "google_sql_user" "default_user" {
  name     = "postgres"
  instance = google_sql_database_instance.postgres.name
  password = random_password.db_password.result
}

# Application user
resource "google_sql_user" "app_user" {
  name     = "api_server"
  instance = google_sql_database_instance.postgres.name
  password = random_password.db_password.result
}

# Store password in Secret Manager
resource "google_secret_manager_secret" "db_password" {
  secret_id = "${var.environment}-db-password"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "db_password" {
  secret      = google_secret_manager_secret.db_password.id
  secret_data = random_password.db_password.result
}
