# =============================================================================
# USDFC Analytics Terminal - Optimized Workspace Dockerfile
# Multi-stage build with workspace dependency caching
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Dependency Cache Layer
# Compiles workspace dependencies separately for better Docker layer caching
# -----------------------------------------------------------------------------
FROM rust:1.85-bookworm AS deps

# Install Rust nightly (needed for Leptos)
RUN rustup toolchain install nightly && rustup default nightly

WORKDIR /app

# Note: rust:1.85-bookworm already includes build-essential, curl, pkg-config, and libssl-dev
# No additional apt packages needed

# Install WASM target for client-side compilation
RUN rustup target add wasm32-unknown-unknown

# Install cargo-leptos from pre-built binary (saves ~3 minutes)
RUN curl -L https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.2/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xz -C /usr/local/cargo/bin

# Copy workspace manifest and crate manifests for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
COPY crates/terminal/Cargo.toml crates/terminal/Cargo.toml

# Create dummy source files to build dependencies only
RUN mkdir -p crates/core/src crates/api/src crates/backend/src crates/terminal/src && \
    echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/api/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/backend/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/terminal/src/lib.rs && \
    echo "fn main() {}" > crates/terminal/src/main.rs

# Build dependencies only (cached layer - rebuilds only when Cargo.toml changes)
# Use railway profile for faster CI builds
RUN cargo build --profile railway -p usdfc-core && \
    cargo build --profile railway -p usdfc-api --features ssr && \
    cargo build --profile railway -p usdfc-backend

# -----------------------------------------------------------------------------
# Stage 2: Application Build
# Builds actual application code using cached dependencies
# -----------------------------------------------------------------------------
FROM deps AS builder

# Remove dummy files
RUN rm -rf crates/*/src

# Copy real source code
COPY crates/core/src crates/core/src
COPY crates/api/src crates/api/src
COPY crates/backend/src crates/backend/src
COPY crates/terminal/src crates/terminal/src
COPY public ./public

# Build the application with railway profile
# This reuses cached dependencies from previous layer
# Use --release with --bin-cargo-args to specify custom profile
RUN cargo leptos build --release -p usdfc-analytics-terminal \
    --bin-cargo-args="--profile=railway"

# -----------------------------------------------------------------------------
# Stage 3: Runtime
# Minimal image with only the compiled binary and assets
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash appuser

# Copy the compiled binary (railway profile)
COPY --from=builder /app/target/railway/usdfc-analytics-terminal /app/usdfc-analytics-terminal

# Copy the site assets (WASM, CSS, JS, static files)
COPY --from=builder /app/target/site /app/site

# Copy public assets
COPY --from=builder /app/public /app/public

# Copy Cargo.toml files for Leptos configuration
COPY --from=builder /app/Cargo.toml /app/Cargo.toml
COPY --from=builder /app/crates/terminal/Cargo.toml /app/crates/terminal/Cargo.toml

# Create data directory for SQLite database
RUN mkdir -p /app/data /app/crates/terminal && chown -R appuser:appuser /app

# Switch to non-root user for security
USER appuser

# Expose the application port
EXPOSE ${PORT:-3000}

# Environment variables for Leptos
ENV LEPTOS_OUTPUT_NAME="usdfc-terminal"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_ENV="PROD"
ENV RUST_LOG="info"

# Server binding
ENV HOST=0.0.0.0
ENV PORT=3000

# SQLite database path
ENV DATABASE_PATH=/app/data/analytics.db

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-3000}/health || exit 1

# Run the application
CMD ["/app/usdfc-analytics-terminal"]
