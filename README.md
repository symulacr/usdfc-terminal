# USDFC Analytics Terminal

Real-time analytics for USDFC stablecoin on Filecoin. Unified REST API aggregating 4 data sources into 10 developer-friendly endpoints.

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
# Get price
curl http://localhost:3000/api/v1/price

# Get protocol metrics
curl http://localhost:3000/api/v1/metrics

# Get troves
curl http://localhost:3000/api/v1/troves?limit=50
```

See [API.md](./API.md) for all 10 endpoints.

## Screenshots
<!-- Add after deployment -->
| Dashboard | Protocol | Lending |
|-----------|----------|---------|
| ![Dashboard](docs/screenshots/dashboard.png) | ![Protocol](docs/screenshots/protocol.png) | ![Lending](docs/screenshots/lending.png) |

## Architecture

```mermaid
%%{init: {'theme': 'neutral'}}%%
block-beta
    columns 1

    block:terminal["USDFC Analytics Terminal"]:1
        api["REST API · 10 endpoints"]
        sf["Server Functions · 15 async"]
        cache["Cache · TTL 30-300s"]
    end

    space

    block:sources:1
        columns 4
        rpc["Filecoin RPC"]
        block["Blockscout"]
        gold["Goldsky"]
        gecko["Gecko"]
    end

    cache --> sources
```

## Data Flow

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart LR
    subgraph src[" "]
        direction TB
        s1[Blockscout]
        s2[Gecko]
        s3[Goldsky]
        s4[RPC]
    end

    subgraph srv["Server"]
        direction TB
        c[Cache]
        f[Functions]
        r[SSR]
    end

    subgraph cli["Client"]
        direction TB
        w[WASM]
        b[Browser]
    end

    src --> c
    c --> f --> r
    r --> w --> b
    b -.-> f
```

## API Endpoints

```mermaid
%%{init: {'theme': 'neutral'}}%%
flowchart LR
    api["/api/v1"]

    api --- h[health]
    api --- p[price]
    api --- m[metrics]
    api --- t[troves]
    api --- ho[holders]
    api --- tx[transactions]
    api --- l[lending]
    api --- hi[history]
```

## Tech Stack
- **Leptos 0.6** - Full-stack Rust (SSR + WASM)
- **Axum** - HTTP server with security headers
- **SQLite** - Metrics persistence (7-day history)
- **Tokio** - Async runtime

## Performance
- SSR render: <5ms
- API response: <100ms (p99)
- Cache hit: >90%

## Documentation
- [API Reference](./API.md)
- [Installation](./INSTALL.md)
- [Environment Variables](./.env.example)

## License
MIT + Apache 2.0 dual license. See [LICENSE](./LICENSE).

## Contributing
Issues and PRs welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md).

---

Built for Filecoin ecosystem. Data sources: Filecoin RPC, Blockscout, Secured Finance, GeckoTerminal.
