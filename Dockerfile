# =============================================================================
# USDFC Analytics Terminal - Optimized Production Dockerfile
# Multi-stage build with dependency caching for faster CI builds
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Dependency Cache Layer
# Compiles dependencies separately for better Docker layer caching
# -----------------------------------------------------------------------------
FROM rustlang/rust:nightly-slim AS deps

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    perl \
    && rm -rf /var/lib/apt/lists/*

# Install WASM target for client-side compilation
RUN rustup target add wasm32-unknown-unknown

# Install cargo-leptos from pre-built binary (NOT source - saves ~3 minutes)
RUN curl -L https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.2/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xz -C /usr/local/cargo/bin

# Copy only dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs and lib.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies only (cached layer - only rebuilds when Cargo.toml changes)
# Use railway profile for faster CI builds
RUN cargo build --profile railway --features ssr --lib

# -----------------------------------------------------------------------------
# Stage 2: Application Build
# Builds actual application code using cached dependencies
# -----------------------------------------------------------------------------
FROM deps AS builder

# Remove dummy files
RUN rm -rf src

# Copy real source code
COPY src ./src
COPY public ./public

# Build the application with railway profile
# This uses cached dependencies from previous layer
RUN cargo leptos build --profile railway

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

# Copy the compiled binary (railway profile builds to target/railway/)
COPY --from=builder /app/target/railway/usdfc-analytics-terminal /app/usdfc-analytics-terminal

# Copy the site assets (WASM, CSS, JS, static files)
COPY --from=builder /app/target/site /app/site

# Copy public assets
COPY --from=builder /app/public /app/public

# Copy Cargo.toml for Leptos configuration
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

# Create data directory for SQLite database
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Switch to non-root user for security
USER appuser

# Expose the application port (Railway sets this dynamically)
EXPOSE ${PORT:-3000}

# Environment variables for Leptos
ENV LEPTOS_OUTPUT_NAME="usdfc-terminal"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_ENV="PROD"
ENV RUST_LOG="info"

# Server binding (Railway sets PORT automatically)
ENV HOST=0.0.0.0
ENV PORT=3000

# SQLite database path (for Railway Volume)
ENV DATABASE_PATH=/app/data/analytics.db

# Health check endpoint (uses PORT from environment)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-3000}/health || exit 1

# Run the application
CMD ["/app/usdfc-analytics-terminal"]
