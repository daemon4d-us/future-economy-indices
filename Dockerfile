# Multi-stage build for Rust API server

# Stage 1: Build stage
FROM rust:1.75-slim-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src

# Build the application in release mode
RUN cargo build --release --package api-server

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 app

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api-server /app/api-server

# Copy database migrations
COPY --from=builder /app/crates/database/migrations /app/migrations

# Change ownership
RUN chown -R app:app /app

# Switch to app user
USER app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/bin/curl", "-f", "http://localhost:3000/health", "||", "exit 1"]

# Run the application
CMD ["/app/api-server"]
