# Production environment variables

project_id  = "urania-economy-indices-prod"
region      = "us-central1"
environment = "prod"

# Database configuration
database_tier = "db-n1-standard-1"

# GKE configuration
node_machine_type = "e2-standard-2"
min_node_count    = 2
max_node_count    = 10

# Kubernetes
k8s_namespace = "future-economy-indices"
