# =============================================================================
# USDFC Analytics Terminal - Production Dockerfile
# Multi-stage build for Leptos SSR application
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# Compiles the Rust application and WASM client
# -----------------------------------------------------------------------------
FROM rustlang/rust:nightly-slim AS builder

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

# Install cargo-leptos build tool (latest version, compatible with nightly and wasm-bindgen 0.2.106)
RUN cargo install --locked cargo-leptos

# Cache bust for rebuilds - updated 2026-01-05 17:30 UTC
ARG CACHE_BUST=2026-01-05-17:30-cursor-pagination
RUN echo "Cache bust: ${CACHE_BUST}"

# Copy source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY public ./public

# Build the release binary and WASM client
RUN cargo leptos build --release

# -----------------------------------------------------------------------------
# Stage 2: Runtime
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

# Copy the compiled binary
COPY --from=builder /app/target/release/usdfc-analytics-terminal /app/usdfc-analytics-terminal

# Copy the site assets (WASM, CSS, JS, static files)
COPY --from=builder /app/target/site /app/site

# Copy public assets if they exist separately
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
    CMD curl -f http://localhost:${PORT:-3000}/api/health || exit 1

# Run the application
CMD ["/app/usdfc-analytics-terminal"]
