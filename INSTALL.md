# Installation Guide

## Prerequisites
- Rust 1.75+ (`rustc --version`)
- cargo-leptos (`cargo install cargo-leptos`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

## Setup

### 1. Clone
```bash
git clone https://github.com/symulacr/usdfc-terminal.git
cd usdfc-terminal
```

### 2. Configure
```bash
cp .env.example .env
# Edit .env if needed (defaults work for Filecoin mainnet)
```

### 3. Build & Run

**Development:**
```bash
cargo leptos watch
# http://localhost:3000
```

**Production:**
```bash
cargo leptos build --release
HOST=0.0.0.0 PORT=3000 ./target/release/usdfc-analytics-terminal
```

### 4. Docker (Optional)
```dockerfile
FROM rust:1.75-slim
WORKDIR /app
COPY . .
RUN cargo install cargo-leptos
RUN rustup target add wasm32-unknown-unknown
RUN cargo leptos build --release
EXPOSE 3000
CMD ["./target/release/usdfc-analytics-terminal"]
```

```bash
docker build -t usdfc-terminal .
docker run -p 3000:3000 usdfc-terminal
```

## Environment Variables

See [.env.example](./.env.example) for all options.

**Required:** All have working defaults for Filecoin mainnet.

**Key settings:**
| Variable | Default | Description |
|----------|---------|-------------|
| HOST | 0.0.0.0 | Bind address |
| PORT | 3000 | HTTP port |
| RPC_URL | Glif API | Filecoin RPC |

## Health Check
```bash
curl http://localhost:3000/api/v1/health
```

## Troubleshooting

**WASM error:** `rustup target add wasm32-unknown-unknown`

**Port in use:** `PORT=3001 cargo leptos watch`

**API 502:** Check RPC_URL is accessible
