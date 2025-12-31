# =============================================================================
# USDFC Analytics Terminal - Production Dockerfile
# Multi-stage build for Leptos SSR application
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# Compiles the Rust application and WASM client
# -----------------------------------------------------------------------------
FROM rust:1.83-slim AS builder

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

# Install cargo-leptos build tool
RUN cargo install --locked cargo-leptos

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

# Set ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user for security
USER appuser

# Expose the application port
EXPOSE 3000

# Environment variables for Leptos
ENV LEPTOS_OUTPUT_NAME="usdfc-terminal"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_ENV="PROD"
ENV RUST_LOG="info"

# Legacy environment variables (for compatibility)
ENV HOST=0.0.0.0
ENV PORT=3000

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# Run the application
CMD ["/app/usdfc-analytics-terminal"]
