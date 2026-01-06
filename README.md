# USDFC Analytics Terminal

Real-time analytics for USDFC stablecoin on Filecoin. Unified REST API aggregating 4 data sources into 10 developer-friendly endpoints.

**Production**: https://usdfc-terminal-cleaned-production.up.railway.app/

## Problem
USDFC data is fragmented across Filecoin RPC, Blockscout, Secured Finance subgraph, and GeckoTerminal—each with different auth, rate limits, and formats.

## Solution
Single REST API with caching, consistent JSON responses, and <100ms latency.

## Features
- **Protocol Metrics** - TCR, supply, collateral, stability pool
- **Price & Volume** - Real-time from DEX pools
- **Trove Explorer** - All positions with ICR health
- **Lending Markets** - Secured Finance rates
- **Transaction History** - Recent USDFC transfers
- **Address Lookup** - Full balance sheet per address

## Quick Start
```bash
# Prerequisites: Rust 1.75+, cargo-leptos
git clone https://github.com/symulacr/usdfc-terminal.git
cd usdfc-terminal
cp .env.example .env
cargo leptos build --release
./target/release/usdfc-analytics-terminal
```

## API Usage
```bash
# Production
curl https://usdfc-terminal-cleaned-production.up.railway.app/api/v1/price
curl https://usdfc-terminal-cleaned-production.up.railway.app/api/v1/metrics

# Local development
curl http://localhost:3000/api/v1/price
curl http://localhost:3000/api/v1/metrics
```

See [API.md](./API.md) for all 10 endpoints.

## Live Demo

👉 **[Try the live application](https://usdfc-terminal-cleaned-production.up.railway.app/)**

## Architecture

4-crate Cargo workspace with SSR + WASM hydration. See [Architecture Documentation](./docs/ARCHITECTURE.md) for system diagrams, data flow, and workspace structure.

## Tech Stack

**Core Framework**
- Leptos 0.6 - Full-stack (SSR + WASM hydration, 16 server functions)
- Axum 0.7 - HTTP server with Tower middleware
- Tokio - Multi-threaded async runtime

**Data & Resilience**
- RuSQLite - 7-day historical metrics (bundled)
- Custom TTL Cache - In-memory (10s-300s TTL per endpoint)
- Circuit Breaker - Auto-recovery for external APIs

**External Integrations**
- Filecoin RPC - MultiTroveGetter aggregation
- Blockscout API - Token transfers, holders
- Secured Finance Subgraph - GraphQL lending data
- GeckoTerminal API - DEX prices, volume

**Build & Deploy**
- cargo-leptos 0.2.47 - Dual compilation (server + WASM)
- wasm-bindgen 0.2.105 - Rust/JS interop
- 4 Custom profiles - Railway/CI/production optimized

[Complete tech stack →](./docs/ARCHITECTURE.md)

## Performance
- SSR render: <5ms
- API response: <100ms (p99)
- Cache hit: >90%
- Railway hosting: ~$0.0002 per deployment
- Resources: 0.58 GB RAM, 0.03 vCPU

## Documentation
- [API Reference](./API.md)
- [Installation](./INSTALL.md)
- [Architecture](./docs/ARCHITECTURE.md)
- [Monitoring](./docs/MONITORING.md)
- [Environment Variables](./.env.example)

## License
MIT + Apache 2.0 dual license. See [LICENSE](./LICENSE).

## Contributing
Issues and PRs welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md).

---

Built for Filecoin ecosystem. Data sources: Filecoin RPC, Blockscout, Secured Finance, GeckoTerminal.
