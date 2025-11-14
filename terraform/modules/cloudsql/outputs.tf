output "instance_name" {
  description = "Cloud SQL instance name"
  value       = google_sql_database_instance.postgres.name
}

output "connection_name" {
  description = "Cloud SQL connection name"
  value       = google_sql_database_instance.postgres.connection_name
}

output "private_ip_address" {
  description = "Private IP address"
  value       = google_sql_database_instance.postgres.private_ip_address
}

output "database_name" {
  description = "Database name"
  value       = google_sql_database.database.name
}

output "db_password_secret" {
  description = "Secret Manager secret name for DB password"
  value       = google_secret_manager_secret.db_password.secret_id
}
