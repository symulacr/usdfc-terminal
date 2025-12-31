# Build stage
FROM rust:1.82-slim as builder

# Force cache bust: 2025-12-31-02
WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add wasm32-unknown-unknown

RUN cargo install --locked cargo-leptos

RUN cargo leptos build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/usdfc-analytics-terminal /app/usdfc-analytics-terminal

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["/app/usdfc-analytics-terminal"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1
