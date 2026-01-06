# =============================================================================
# USDFC Analytics Terminal - Production Dockerfile
# Based on proven leptos-railway template for Railway deployment
# =============================================================================

# Build stage with Rust nightly
FROM rust:1.85-bookworm AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    npm \
    binaryen && \
    rm -rf /var/lib/apt/lists/*

# Install Sass for CSS processing
RUN npm install -g sass

# Install Rust nightly and WASM target
RUN rustup toolchain install nightly && \
    rustup default nightly && \
    rustup target add wasm32-unknown-unknown

# Install cargo-leptos (use pre-compiled binary - much faster than cargo install)
# cargo-leptos 0.2.47 includes wasm-bindgen-cli 0.2.105 (project downgraded to match)
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.47/cargo-leptos-installer.sh | sh

WORKDIR /app

# Copy source code
COPY . .

# Set CC for ring crate C compilation (fix sccache wrapper issue)
ENV CC=/usr/bin/gcc

# Build with release flag (uses standard release profile)
# Matches proven working Railway template approach
# Week 1-3 optimizations applied: mold linker + optimized dependencies
RUN cargo leptos build --release -vv

# =============================================================================
# Runtime stage - minimal Debian image
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create app user for security
RUN useradd --create-home --shell /bin/bash appuser

# Copy binary and assets from builder
COPY --from=builder /app/target/release/usdfc-analytics-terminal /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/public /app/public
COPY --from=builder /app/Cargo.toml /app/
COPY --from=builder /app/crates/terminal/Cargo.toml /app/crates/terminal/

# Create data directory and set ownership
RUN mkdir -p /app/data /app/crates/terminal && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE ${PORT:-3000}

# Leptos environment configuration
ENV LEPTOS_OUTPUT_NAME="usdfc-terminal"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_ENV="PROD"
ENV RUST_LOG="info"
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_PATH=/app/data/analytics.db

CMD ["/app/usdfc-analytics-terminal"]
