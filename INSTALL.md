# USDFC Analytics Terminal - Installation & Deployment Guide

A comprehensive guide for installing, building, and deploying the USDFC Analytics Terminal.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Environment Configuration](#environment-configuration)
4. [Development Build](#development-build)
5. [Production Build](#production-build)
6. [Docker Deployment](#docker-deployment)
7. [VPS Deployment](#vps-deployment)
8. [Verification](#verification)
9. [Troubleshooting](#troubleshooting)
10. [CI/CD Integration](#cicd-integration)
11. [Monitoring & Logging](#monitoring--logging)
12. [Security Hardening](#security-hardening)
13. [Maintenance & Updates](#maintenance--updates)
14. [Performance Tuning](#performance-tuning)

---

## Prerequisites

### Required Software

#### 1. Rust 1.75+ (Recommended: 1.83+)

**Linux/macOS:**
```bash
# Install rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should show 1.75.0 or higher

# Update to latest stable (recommended)
rustup update stable
```

**Windows:**
```powershell
# Download and run rustup-init.exe from https://rustup.rs
# Or use winget:
winget install Rustlang.Rustup

# Verify installation
rustc --version
```

#### 2. cargo-leptos Build Tool

cargo-leptos is essential for building Leptos SSR applications with WASM:

```bash
# Install cargo-leptos
cargo install --locked cargo-leptos

# Verify installation
cargo leptos --version
```

#### 3. WebAssembly Target

The WASM target is required for client-side hydration:

```bash
# Add wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown

# Verify target is installed
rustup target list --installed | grep wasm32
```

#### 4. Build Dependencies

**Debian/Ubuntu:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev curl perl
```

**Fedora/RHEL:**
```bash
sudo dnf install -y gcc pkg-config openssl-devel curl perl
```

**Arch Linux:**
```bash
sudo pacman -S base-devel openssl curl perl
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install OpenSSL via Homebrew (if needed)
brew install openssl pkg-config
```

**Windows:**
```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Select "Desktop development with C++"

# Or use chocolatey:
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
```

### Optional Software

- **Docker**: For containerized deployments
- **Nginx**: For reverse proxy in production
- **Git**: For cloning the repository

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8+ GB |
| Disk | 10 GB | 20+ GB |
| Build RAM | 4 GB | 8+ GB (for release builds with LTO) |

---

## Quick Start

Get up and running in 3 steps:

```bash
# Step 1: Clone the repository
git clone https://github.com/symulacr/usdfc-terminal.git
cd usdfc-terminal

# Step 2: Configure environment
cp .env.example .env
# Edit .env if needed (defaults work for Filecoin mainnet)

# Step 3: Build and run
cargo leptos watch
```

Open http://localhost:3000 in your browser.

### One-Line Install (Linux/macOS)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
source $HOME/.cargo/env && \
cargo install --locked cargo-leptos && \
rustup target add wasm32-unknown-unknown && \
git clone https://github.com/symulacr/usdfc-terminal.git && \
cd usdfc-terminal && \
cp .env.example .env && \
cargo leptos watch
```

---

## Environment Configuration

### Configuration File

Copy the example configuration:

```bash
cp .env.example .env
```

### Environment Variables Reference

#### API Endpoints (Filecoin Mainnet)

| Variable | Default | Description |
|----------|---------|-------------|
| `RPC_URL` | `https://api.node.glif.io/rpc/v1` | Filecoin JSON-RPC endpoint |
| `BLOCKSCOUT_URL` | `https://filecoin.blockscout.com/api/v2` | Blockscout API for transaction data |
| `SUBGRAPH_URL` | Goldsky endpoint | TheGraph subgraph for indexed data |
| `GECKOTERMINAL_URL` | GeckoTerminal API | DEX analytics and price data |

#### Contract Addresses

| Variable | Address | Description |
|----------|---------|-------------|
| `USDFC_TOKEN` | `0x80B98d3aa09ffff255c3ba4A241111Ff1262F045` | USDFC stablecoin token contract |
| `TROVE_MANAGER` | `0x5aB87c2398454125Dd424425e39c8909bBE16022` | Manages individual trove positions |
| `SORTED_TROVES` | `0x2C32e48e358d5b893C46906b69044D342d8DDd5F` | Sorted linked list of troves |
| `PRICE_FEED` | `0x80e651c9739C1ed15A267c11b85361780164A368` | Oracle price feed contract |
| `MULTI_TROVE_GETTER` | `0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F` | Batch getter for trove data |
| `STABILITY_POOL` | `0x791Ad78bBc58324089D3E0A8689E7D045B9592b5` | Stability pool contract |
| `ACTIVE_POOL` | `0x8637Ac7FdBB4c763B72e26504aFb659df71c7803` | Active pool contract |
| `BORROWER_OPERATIONS` | `0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0` | Borrower operations contract |

#### DEX Pool Addresses

| Variable | Address | Description |
|----------|---------|-------------|
| `POOL_USDFC_WFIL` | `0x4e07447bd38e60b94176764133788be1a0736b30` | USDFC/WFIL liquidity pool |
| `POOL_USDFC_AXLUSDC` | `0x21ca72fe39095db9642ca9cc694fa056f906037f` | USDFC/axlUSDC liquidity pool |
| `POOL_USDFC_USDC` | `0xc8f38dbaf661b897b6a2ee5721aac5a8766ffa13` | USDFC/USDC liquidity pool |

#### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3000` | HTTP server port |

#### Refresh Intervals

| Variable | Default | Description |
|----------|---------|-------------|
| `REFRESH_INTERVAL_FAST` | `30` | Fast refresh interval (seconds) |
| `REFRESH_INTERVAL_MEDIUM` | `60` | Medium refresh interval (seconds) |
| `REFRESH_INTERVAL_SLOW` | `300` | Slow refresh interval (seconds) |

#### Optional Settings

Uncomment in `.env` to override defaults:

```bash
# Risk thresholds
TCR_DANGER_THRESHOLD=150.0      # TCR danger zone percentage
TCR_WARNING_THRESHOLD=200.0     # TCR warning zone percentage

# Whale detection
WHALE_THRESHOLD_USD=100000.0    # USD threshold for whale alerts

# Performance tuning
REFRESH_INTERVAL_MS=30000       # UI refresh interval (milliseconds)
HISTORY_RETENTION_SECS=604800   # History retention (7 days default)
RPC_TIMEOUT_SECS=30             # RPC request timeout
RPC_RETRY_COUNT=3               # RPC retry attempts
```

### Environment-Specific Configurations

**Development (.env.development):**
```bash
HOST=127.0.0.1
PORT=3000
RUST_LOG=debug
LEPTOS_ENV=DEV
```

**Staging (.env.staging):**
```bash
HOST=0.0.0.0
PORT=3000
RUST_LOG=info
LEPTOS_ENV=PROD
```

**Production (.env.production):**
```bash
HOST=0.0.0.0
PORT=3000
RUST_LOG=warn
LEPTOS_ENV=PROD
```

---

## Development Build

### Running in Development Mode

cargo-leptos provides hot reload for both server and client:

```bash
cargo leptos watch
```

This command:
- Compiles the server binary with SSR features
- Compiles the WASM client with hydrate features
- Watches for file changes and recompiles automatically
- Serves at http://localhost:3000
- Hot reload port at 3001

### Development Options

```bash
# Custom port
PORT=8080 cargo leptos watch

# Verbose output
RUST_LOG=debug cargo leptos watch

# Skip initial browser open
cargo leptos watch --hot-reload

# Release mode development (for performance testing)
cargo leptos watch --release
```

### Development URLs

- **Application**: http://localhost:3000
- **Hot Reload WebSocket**: ws://localhost:3001

### IDE Setup

**VS Code Extensions:**
- rust-analyzer (Rust language support)
- Even Better TOML (Cargo.toml support)
- crates (Dependency version checking)

**VS Code settings.json:**
```json
{
  "rust-analyzer.cargo.features": ["ssr"],
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.target": null
}
```

---

## Production Build

### Building for Production

```bash
cargo leptos build --release
```

This creates:
- **Server binary**: `target/release/usdfc-analytics-terminal`
- **Site assets**: `target/site/` (WASM, CSS, JS, static files)

### Build Profiles

The project includes optimized profiles in `Cargo.toml`:

| Profile | Description | Use Case |
|---------|-------------|----------|
| `release` | Maximum optimization (LTO, size opt) | Production deployment |
| `release-fast` | Fast compilation, good performance | CI/CD builds |
| `wasm-release` | WASM-specific optimizations | Client bundle |

### Build Time Optimization

```bash
# Parallel compilation (uses all CPU cores by default)
cargo leptos build --release

# Faster builds for CI (less optimization)
cargo leptos build --profile release-fast

# Verbose build output
CARGO_LOG=debug cargo leptos build --release
```

### Running the Production Binary

```bash
# Set required environment variables
export HOST=0.0.0.0
export PORT=3000
export LEPTOS_SITE_ROOT="target/site"
export LEPTOS_OUTPUT_NAME="usdfc-terminal"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_ENV="PROD"

# Run the binary
./target/release/usdfc-analytics-terminal
```

Or in a single command:

```bash
HOST=0.0.0.0 PORT=3000 ./target/release/usdfc-analytics-terminal
```

### Binary Size Optimization

The release build is already optimized for size:
- LTO (Link Time Optimization) enabled
- `opt-level = 'z'` for size optimization
- Binary stripping enabled
- Single codegen unit

Expected binary sizes:
- Server binary: ~15-25 MB
- WASM bundle: ~2-5 MB (gzipped: ~500KB-1MB)

---

## Docker Deployment

### Using the Dockerfile

The project includes a multi-stage Dockerfile for optimized production images:

```bash
# Build the Docker image
docker build -t usdfc-terminal .

# Run the container
docker run -d \
  --name usdfc-terminal \
  -p 3000:3000 \
  --env-file .env \
  usdfc-terminal
```

### Docker Image Details

The Dockerfile uses a multi-stage build:

1. **Builder stage** (`rust:1.83-slim`):
   - Installs build dependencies
   - Compiles the application
   - ~2GB image size

2. **Runtime stage** (`debian:bookworm-slim`):
   - Minimal runtime dependencies
   - Non-root user for security
   - ~100MB final image size

### Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  usdfc-terminal:
    build: .
    container_name: usdfc-terminal
    ports:
      - "3000:3000"
    environment:
      - HOST=0.0.0.0
      - PORT=3000
      - LEPTOS_ENV=PROD
      - RUST_LOG=info
      # Add other environment variables as needed
      - RPC_URL=https://api.node.glif.io/rpc/v1
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M

  # Optional: Nginx reverse proxy
  nginx:
    image: nginx:alpine
    container_name: nginx-proxy
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - usdfc-terminal
    restart: unless-stopped
```

Run with Docker Compose:

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f usdfc-terminal

# Stop services
docker-compose down

# Rebuild and restart
docker-compose up -d --build
```

### Docker Build Arguments

```bash
# Build with specific Rust version
docker build --build-arg RUST_VERSION=1.83 -t usdfc-terminal .

# Build without cache (clean build)
docker build --no-cache -t usdfc-terminal .

# Multi-platform build
docker buildx build --platform linux/amd64,linux/arm64 -t usdfc-terminal .
```

### Docker Registry

```bash
# Tag for registry
docker tag usdfc-terminal your-registry.com/usdfc-terminal:latest

# Push to registry
docker push your-registry.com/usdfc-terminal:latest

# Pull and run
docker pull your-registry.com/usdfc-terminal:latest
docker run -d -p 3000:3000 your-registry.com/usdfc-terminal:latest
```

---

## VPS Deployment

This section covers deploying to a VPS (like the deployment at 5.180.182.231:8080).

### Server Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 1 core | 2+ cores |
| RAM | 1 GB | 2+ GB |
| Disk | 10 GB | 20+ GB |
| OS | Debian 11+ / Ubuntu 22.04+ | Latest LTS |
| Network | 100 Mbps | 1 Gbps |

### Step 1: Server Setup

```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install dependencies
sudo apt-get install -y build-essential pkg-config libssl-dev curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install cargo-leptos and WASM target
cargo install --locked cargo-leptos
rustup target add wasm32-unknown-unknown
```

### Step 2: Clone and Build

```bash
# Clone repository
git clone https://github.com/symulacr/usdfc-terminal.git
cd usdfc-terminal

# Configure environment
cp .env.example .env
nano .env  # Edit as needed

# Build for production
cargo leptos build --release
```

### Step 3: Firewall Configuration (UFW)

```bash
# Enable UFW
sudo ufw enable

# Allow SSH (important - don't lock yourself out!)
sudo ufw allow 22/tcp

# Allow HTTP traffic
sudo ufw allow 80/tcp

# Allow HTTPS traffic
sudo ufw allow 443/tcp

# Allow application port (if exposing directly)
sudo ufw allow 8080/tcp

# Check status
sudo ufw status verbose
```

### Step 4: Running as a Background Service

#### Option A: Using systemd (Recommended)

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/usdfc-terminal.service
```

Add the following content:

```ini
[Unit]
Description=USDFC Analytics Terminal
Documentation=https://github.com/symulacr/usdfc-terminal
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/home/deploy/usdfc-terminal
EnvironmentFile=/home/deploy/usdfc-terminal/.env
Environment=HOST=0.0.0.0
Environment=PORT=8080
Environment=LEPTOS_SITE_ROOT=target/site
Environment=LEPTOS_OUTPUT_NAME=usdfc-terminal
Environment=LEPTOS_SITE_PKG_DIR=pkg
Environment=LEPTOS_ENV=PROD
Environment=RUST_LOG=info
ExecStart=/home/deploy/usdfc-terminal/target/release/usdfc-analytics-terminal
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=usdfc-terminal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictSUIDSGID=true

# Resource limits
LimitNOFILE=65535
MemoryMax=1G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable usdfc-terminal

# Start the service
sudo systemctl start usdfc-terminal

# Check status
sudo systemctl status usdfc-terminal

# View logs
sudo journalctl -u usdfc-terminal -f
```

#### Option B: Using screen/tmux (Quick testing)

```bash
# Using screen
screen -S usdfc
HOST=0.0.0.0 PORT=8080 ./target/release/usdfc-analytics-terminal
# Press Ctrl+A, D to detach

# Reattach later
screen -r usdfc
```

#### Option C: Using nohup

```bash
nohup ./target/release/usdfc-analytics-terminal > /var/log/usdfc-terminal.log 2>&1 &
```

### Step 5: Nginx Reverse Proxy (Optional but Recommended)

Install Nginx:

```bash
sudo apt-get install nginx
```

Create Nginx configuration:

```bash
sudo nano /etc/nginx/sites-available/usdfc-terminal
```

Add the following:

```nginx
# Rate limiting zone
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;

upstream usdfc_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name your-domain.com;  # Or use IP: 5.180.182.231

    # Redirect HTTP to HTTPS (uncomment after SSL setup)
    # return 301 https://$server_name$request_uri;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_proxied any;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/x-javascript application/xml application/wasm;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Static assets caching
    location ~* \.(wasm|js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        proxy_pass http://usdfc_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # API endpoints with rate limiting
    location /api/ {
        limit_req zone=api_limit burst=20 nodelay;
        proxy_pass http://usdfc_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Main application
    location / {
        proxy_pass http://usdfc_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;
    }

    # WebSocket support for live updates
    location /ws {
        proxy_pass http://usdfc_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }

    # Health check endpoint (no rate limit)
    location /api/v1/health {
        proxy_pass http://usdfc_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }
}
```

Enable the site:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/usdfc-terminal /etc/nginx/sites-enabled/

# Remove default site
sudo rm /etc/nginx/sites-enabled/default

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx
```

### Step 6: SSL with Let's Encrypt (Optional)

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal is configured automatically
sudo systemctl status certbot.timer

# Test renewal
sudo certbot renew --dry-run
```

---

## Verification

### Health Check Endpoints

```bash
# Basic health check
curl http://localhost:3000/api/v1/health

# Expected response:
# {"status":"healthy","timestamp":"2024-..."}

# For remote server
curl http://5.180.182.231:8080/api/v1/health

# With timeout
curl --max-time 10 http://localhost:3000/api/v1/health
```

### Testing the Deployment

#### 1. Check Service Status

```bash
# systemd service
sudo systemctl status usdfc-terminal

# Docker container
docker ps | grep usdfc-terminal
docker logs usdfc-terminal
```

#### 2. Check Port Binding

```bash
# Check if port is listening
ss -tlnp | grep 3000

# Or using netstat
netstat -tlnp | grep 3000
```

#### 3. Test HTTP Response

```bash
# Test main page
curl -I http://localhost:3000

# Expected: HTTP/1.1 200 OK

# Test with verbose output
curl -v http://localhost:3000

# Test response time
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000
```

#### 4. Test API Endpoints

```bash
# System metrics
curl http://localhost:3000/api/v1/system

# Protocol stats
curl http://localhost:3000/api/v1/protocol

# Test all endpoints
for endpoint in health system protocol; do
  echo "Testing /api/v1/$endpoint"
  curl -s -o /dev/null -w "%{http_code}\n" "http://localhost:3000/api/v1/$endpoint"
done
```

#### 5. Browser Testing

Open in browser and verify:
- [ ] Page loads without errors
- [ ] Charts and data display correctly
- [ ] Real-time updates are working
- [ ] No console errors (F12 Developer Tools)
- [ ] WASM loads successfully (Network tab)
- [ ] Responsive design works on mobile

#### 6. Load Testing

```bash
# Install wrk (if not available)
sudo apt-get install wrk

# Basic load test
wrk -t4 -c100 -d30s http://localhost:3000/

# API load test
wrk -t4 -c100 -d30s http://localhost:3000/api/v1/health
```

---

## Troubleshooting

### Common Issues and Solutions

#### WASM Build Errors

**Problem**: `error[E0463]: can't find crate for 'std'`

**Solution**:
```bash
rustup target add wasm32-unknown-unknown
```

**Problem**: `wasm-bindgen version mismatch`

**Solution**:
```bash
cargo clean
cargo update
cargo leptos build --release
```

**Problem**: `wasm-opt not found`

**Solution**:
```bash
# Install binaryen (contains wasm-opt)
sudo apt-get install binaryen

# Or via cargo
cargo install wasm-opt
```

#### Port Conflicts

**Problem**: `Address already in use`

**Solution**:
```bash
# Find process using the port
sudo lsof -i :3000

# Kill the process
sudo kill -9 <PID>

# Or use a different port
PORT=3001 cargo leptos watch
```

#### Build Failures

**Problem**: `linker 'cc' not found`

**Solution**:
```bash
sudo apt-get install build-essential
```

**Problem**: `openssl-sys build failed`

**Solution**:
```bash
sudo apt-get install pkg-config libssl-dev
```

**Problem**: Memory exhaustion during build

**Solution**:
```bash
# Add swap space
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make permanent
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# Or use release-fast profile
cargo leptos build --profile release-fast
```

**Problem**: `cargo leptos` not found

**Solution**:
```bash
cargo install --locked cargo-leptos
# Ensure ~/.cargo/bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

#### Runtime Errors

**Problem**: `API 502 Bad Gateway`

**Solution**: Check RPC_URL accessibility:
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://api.node.glif.io/rpc/v1
```

**Problem**: `CORS errors in browser`

**Solution**: The server includes CORS headers. If using Nginx, ensure proxy headers are set correctly.

**Problem**: `WebSocket connection failed`

**Solution**: Check Nginx proxy_set_header for WebSocket upgrade.

**Problem**: `Site assets not found (404)`

**Solution**:
```bash
# Verify LEPTOS_SITE_ROOT is set correctly
export LEPTOS_SITE_ROOT="target/site"

# Check assets exist
ls -la target/site/pkg/
```

#### Docker Issues

**Problem**: Container exits immediately

**Solution**:
```bash
# Check logs
docker logs usdfc-terminal

# Run interactively to debug
docker run -it --rm usdfc-terminal /bin/bash
```

**Problem**: Build takes too long

**Solution**: Use Docker BuildKit and caching:
```bash
DOCKER_BUILDKIT=1 docker build -t usdfc-terminal .
```

**Problem**: Out of disk space during build

**Solution**:
```bash
# Clean Docker resources
docker system prune -a

# Check disk usage
df -h
```

### Getting Help

- Check the [GitHub Issues](https://github.com/symulacr/usdfc-terminal/issues)
- Review logs: `sudo journalctl -u usdfc-terminal -n 100`
- Enable debug logging: `RUST_LOG=debug`

---

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/ci.yml`:

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt
          targets: wasm32-unknown-unknown

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-leptos
        run: cargo install --locked cargo-leptos

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Build
        run: cargo leptos build --release

      - name: Run tests
        run: cargo test --all-features

  docker:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ghcr.io/${{ github.repository }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    needs: docker
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to VPS
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.VPS_HOST }}
          username: ${{ secrets.VPS_USER }}
          key: ${{ secrets.VPS_SSH_KEY }}
          script: |
            cd /home/deploy/usdfc-terminal
            git pull origin main
            docker-compose pull
            docker-compose up -d --force-recreate
```

### GitLab CI

Create `.gitlab-ci.yml`:

```yaml
stages:
  - test
  - build
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/
    - target/

test:
  stage: test
  image: rust:1.83
  before_script:
    - rustup target add wasm32-unknown-unknown
    - cargo install --locked cargo-leptos
  script:
    - cargo fmt --all -- --check
    - cargo clippy --all-features -- -D warnings
    - cargo test --all-features

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main

deploy:
  stage: deploy
  script:
    - ssh deploy@$VPS_HOST "cd /home/deploy/usdfc-terminal && docker-compose pull && docker-compose up -d"
  only:
    - main
  when: manual
```

---

## Monitoring & Logging

### Structured Logging

The application uses `tracing` for structured logging:

```bash
# Set log level via environment
RUST_LOG=info ./target/release/usdfc-analytics-terminal

# Log levels: error, warn, info, debug, trace
RUST_LOG=debug ./target/release/usdfc-analytics-terminal

# Module-specific logging
RUST_LOG=usdfc_analytics_terminal=debug,hyper=warn ./target/release/usdfc-analytics-terminal
```

### Log Aggregation with journald

```bash
# View logs
sudo journalctl -u usdfc-terminal -f

# Filter by time
sudo journalctl -u usdfc-terminal --since "1 hour ago"

# Export logs
sudo journalctl -u usdfc-terminal --since today > /tmp/usdfc-logs.txt
```

### Prometheus Metrics (Optional)

Add metrics endpoint for Prometheus scraping:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'usdfc-terminal'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: /api/v1/metrics
```

### Health Monitoring Script

Create `/usr/local/bin/check-usdfc.sh`:

```bash
#!/bin/bash
HEALTH_URL="http://localhost:3000/api/v1/health"
TIMEOUT=10

response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $TIMEOUT $HEALTH_URL)

if [ "$response" != "200" ]; then
    echo "Health check failed with status: $response"
    systemctl restart usdfc-terminal
    echo "Service restarted at $(date)" >> /var/log/usdfc-restart.log
fi
```

Add to crontab:
```bash
*/5 * * * * /usr/local/bin/check-usdfc.sh
```

---

## Security Hardening

### Server Hardening

```bash
# Disable root SSH login
sudo sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sudo systemctl restart sshd

# Install fail2ban
sudo apt-get install fail2ban
sudo systemctl enable fail2ban

# Configure automatic security updates
sudo apt-get install unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
```

### Application Security

The systemd service file includes security hardening:

- `NoNewPrivileges`: Prevents privilege escalation
- `ProtectSystem`: Read-only filesystem
- `PrivateTmp`: Isolated /tmp
- `ProtectKernelTunables`: Prevents kernel modification

### Network Security

```bash
# Rate limiting with iptables
sudo iptables -A INPUT -p tcp --dport 3000 -m connlimit --connlimit-above 50 -j DROP

# Save iptables rules
sudo apt-get install iptables-persistent
sudo netfilter-persistent save
```

### SSL/TLS Best Practices

```nginx
# Strong SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
ssl_prefer_server_ciphers off;
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 1d;
ssl_session_tickets off;
ssl_stapling on;
ssl_stapling_verify on;
```

---

## Maintenance & Updates

### Updating the Application

```bash
# Stop service
sudo systemctl stop usdfc-terminal

# Pull latest changes
cd /home/deploy/usdfc-terminal
git pull origin main

# Rebuild
cargo leptos build --release

# Restart service
sudo systemctl start usdfc-terminal

# Verify
sudo systemctl status usdfc-terminal
```

### Automated Update Script

Create `/usr/local/bin/update-usdfc.sh`:

```bash
#!/bin/bash
set -e

cd /home/deploy/usdfc-terminal

echo "Pulling latest changes..."
git pull origin main

echo "Building release..."
cargo leptos build --release

echo "Restarting service..."
sudo systemctl restart usdfc-terminal

echo "Update complete!"
```

### Database Maintenance

The application uses SQLite for caching:

```bash
# Backup database
cp /home/deploy/usdfc-terminal/data/cache.db /backup/cache-$(date +%Y%m%d).db

# Vacuum database (reclaim space)
sqlite3 /home/deploy/usdfc-terminal/data/cache.db "VACUUM;"
```

### Log Rotation

Create `/etc/logrotate.d/usdfc-terminal`:

```
/var/log/usdfc-terminal.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 www-data www-data
    sharedscripts
    postrotate
        systemctl reload usdfc-terminal > /dev/null 2>&1 || true
    endscript
}
```

---

## Performance Tuning

### System Tuning

```bash
# Increase file descriptor limits
echo "* soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65535" | sudo tee -a /etc/security/limits.conf

# TCP optimization
sudo sysctl -w net.core.somaxconn=65535
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=65535
```

### Application Tuning

Environment variables for performance:

```bash
# Adjust refresh intervals based on load
REFRESH_INTERVAL_FAST=60      # Increase from 30s
REFRESH_INTERVAL_MEDIUM=120   # Increase from 60s
REFRESH_INTERVAL_SLOW=600     # Increase from 300s

# RPC tuning
RPC_TIMEOUT_SECS=60          # Increase for slow RPCs
RPC_RETRY_COUNT=5            # More retries for reliability
```

### Nginx Caching

```nginx
# Proxy cache configuration
proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=usdfc_cache:10m max_size=1g inactive=60m use_temp_path=off;

server {
    location /api/ {
        proxy_cache usdfc_cache;
        proxy_cache_valid 200 30s;
        proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
        add_header X-Cache-Status $upstream_cache_status;
    }
}
```

### Resource Monitoring

```bash
# Monitor memory usage
watch -n 5 'ps aux | grep usdfc-analytics-terminal'

# Monitor CPU usage
top -p $(pgrep usdfc-analytics)

# Monitor network connections
ss -s
```

---

## Quick Reference

### Essential Commands

```bash
# Development
cargo leptos watch

# Production build
cargo leptos build --release

# Run production
HOST=0.0.0.0 PORT=3000 ./target/release/usdfc-analytics-terminal

# Docker
docker build -t usdfc-terminal .
docker run -d -p 3000:3000 usdfc-terminal

# Service management
sudo systemctl start usdfc-terminal
sudo systemctl stop usdfc-terminal
sudo systemctl restart usdfc-terminal
sudo systemctl status usdfc-terminal

# Logs
sudo journalctl -u usdfc-terminal -f

# Health check
curl http://localhost:3000/api/v1/health
```

### File Locations

| File | Location |
|------|----------|
| Server binary | `target/release/usdfc-analytics-terminal` |
| Site assets | `target/site/` |
| WASM bundle | `target/site/pkg/` |
| Configuration | `.env` |
| Systemd service | `/etc/systemd/system/usdfc-terminal.service` |
| Nginx config | `/etc/nginx/sites-available/usdfc-terminal` |
| Logs (journald) | `journalctl -u usdfc-terminal` |

### Useful Aliases

Add to `~/.bashrc`:

```bash
alias usdfc-status='sudo systemctl status usdfc-terminal'
alias usdfc-logs='sudo journalctl -u usdfc-terminal -f'
alias usdfc-restart='sudo systemctl restart usdfc-terminal'
alias usdfc-build='cd /home/deploy/usdfc-terminal && cargo leptos build --release'
```

---

## Support

For issues and feature requests:
- GitHub Issues: https://github.com/symulacr/usdfc-terminal/issues
- Documentation: https://github.com/symulacr/usdfc-terminal/wiki

**Live Deployment**: http://5.180.182.231:8080
