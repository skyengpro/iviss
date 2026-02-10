# Dockerfile for IVISS Rust Backend
# Multi-stage build for development with live reloading

# Development Stage - with cargo-watch for auto-reload
FROM rust:latest AS development

# Install cargo-watch for live reloading
RUN cargo install cargo-watch

# Create app directory
WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code (will be overridden by volume mount in docker-compose)
COPY src ./src

# Expose backend port
EXPOSE 3000

# Use cargo-watch to auto-rebuild on file changes
# -x run: execute 'cargo run' on changes
# -w src: watch the src directory
CMD ["cargo", "watch", "-x", "run", "-w", "src"]

# ============================================
# Production Stage (optional - for future use)
# ============================================
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS production

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libpq5 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/iviss-backend /app/iviss-backend

EXPOSE 3000

CMD ["/app/iviss-backend"]
