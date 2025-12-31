# USDFC Analytics Terminal - Complete Codebase Overview

**Last Updated:** December 27, 2025
**Version:** 0.1.0
**Network:** Filecoin Mainnet

---

## What's Being Built

**USDFC Analytics Terminal** is a real-time analytics dashboard for the **USDFC stablecoin protocol** on **Filecoin mainnet**. It's a Bloomberg Terminal-style monitoring tool for tracking:

- **Protocol health** (collateral ratios, trove positions, stability pool)
- **Token metrics** (supply, circulation, holders, transfers)
- **Lending markets** (Secured Finance fixed-rate lending integration)
- **DEX activity** (via GeckoTerminal API)

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| **Frontend** | Leptos 0.6 (Rust → WASM) |
| **Backend** | Axum (SSR + API) |
| **Build** | cargo-leptos |
| **Styling** | Custom CSS (monochrome + data colors) |
| **Data Sources** | 4 APIs (RPC, Blockscout, Subgraph, GeckoTerminal) |

---

## Project Structure

```
usdfc-terminal/
├── Cargo.toml           # Dependencies and leptos metadata
├── .env                 # Environment configuration
├── src/
│   ├── app.rs           # Main app component, page router
│   ├── lib.rs           # Module exports, WASM hydration entry
│   ├── main.rs          # SSR server (Axum)
│   ├── config.rs        # Environment-based configuration
│   ├── types.rs         # Core data types (ProtocolMetrics, Trove, Transaction)
│   ├── error.rs         # Error handling (ApiError, ValidationError)
│   ├── format.rs        # Formatting utilities
│   ├── server_fn.rs     # Server functions (callable from client)
│   ├── state.rs         # Application state (SSR only)
│   ├── fileserv.rs      # Static file serving (SSR only)
│   ├── address_conv.rs  # Filecoin address format conversions
│   │
│   ├── # API Clients (SSR only)
│   ├── rpc.rs           # Filecoin RPC client (JSON-RPC)
│   ├── blockscout.rs    # Blockscout API client
│   ├── subgraph.rs      # Secured Finance GraphQL client
│   ├── gecko.rs         # GeckoTerminal DEX data client
│   │
│   ├── components/      # Reusable UI components
│   │   ├── mod.rs
│   │   ├── header.rs    # Top navigation bar
│   │   ├── sidebar.rs   # Left navigation menu
│   │   ├── footer.rs    # Status bar
│   │   ├── metric_card.rs # Dashboard metric cards
│   │   ├── data_table.rs  # Generic data tables
│   │   ├── charts.rs    # Chart components (line, bar, area)
│   │   ├── gauge.rs     # TCR gauge visualization
│   │   └── icons.rs     # SVG icon components
│   │
│   └── pages/           # 16 page components
│       ├── mod.rs
│       ├── dashboard.rs     # Main overview page
│       ├── supply.rs        # Token supply metrics
│       ├── collateral.rs    # Trove health monitoring
│       ├── stability.rs     # Stability pool data
│       ├── lending.rs       # Fixed-rate lending markets
│       ├── transactions.rs  # Token transfer search
│       ├── address.rs       # Address lookup
│       ├── contracts.rs     # Smart contract registry
│       ├── entities.rs      # DEX/bridge integrations
│       ├── flow.rs          # Operation flow diagrams
│       ├── network.rs       # Network graph visualization
│       ├── sankey.rs        # Token flow Sankey chart
│       ├── architecture.rs  # Protocol architecture diagram
│       ├── api_reference.rs # API documentation
│       ├── export.rs        # Data export
│       └── alerts.rs        # Alert configuration
│
├── docs/                # Documentation
├── backup/              # Source backups
└── target/              # Build artifacts
```

---

## Data Sources (4 APIs)

### 1. Glif RPC (Filecoin Mainnet)
**URL:** `https://api.node.glif.io/rpc/v1`
**Purpose:** Protocol state from smart contracts

| Method | Contract | Data |
|--------|----------|------|
| `totalSupply()` | USDFC Token | Total USDFC minted |
| `getEntireSystemColl()` | TroveManager | Total FIL collateral |
| `getTroveOwnersCount()` | TroveManager | Active trove count |
| `lastGoodPrice()` | PriceFeed | FIL/USD price |
| `getTotalDebtTokenDeposits()` | StabilityPool | Pool balance |
| `getMultipleSortedTroves()` | MultiTroveGetter | All trove data |

**Limitation:** Only ~15 hours of event history available (archival constraint)

### 2. Blockscout API
**URL:** `https://filecoin.blockscout.com/api/v2`
**Purpose:** Token transfers, holders, address info

| Endpoint | Data |
|----------|------|
| `/tokens/{addr}/transfers` | Recent USDFC transfers |
| `/tokens/{addr}/holders` | Top token holders |
| `/tokens/{addr}/counters` | Holder count, transfer count |
| `/addresses/{addr}/token-balances` | Address token holdings |

### 3. Secured Finance Subgraph
**URL:** `https://api.goldsky.com/.../sf-filecoin-mainnet/latest/gn`
**Purpose:** Fixed-rate lending markets

| Query | Data |
|-------|------|
| `lendingMarkets` | Active markets, APRs, volumes |
| `orders` | Open order book |
| `transactions` | Recent trades |
| `dailyVolumes` | Trading volume history |

### 4. GeckoTerminal API
**URL:** `https://api.geckoterminal.com/api/v2/networks/filecoin`
**Purpose:** DEX prices, liquidity, historical data

| Endpoint | Data |
|----------|------|
| `/tokens/{addr}` | Price, market cap, supply |
| `/pools/{addr}` | Pool liquidity, volume |
| `/pools/{addr}/ohlcv/{timeframe}` | Historical OHLCV candles |
| `/pools/{addr}/trades` | Recent DEX trades |

**Rate Limit:** 30 requests/minute

---

## Key Contract Addresses (Mainnet)

| Contract | Address | Purpose |
|----------|---------|---------|
| **USDFC Token** | `0x80B98d3aa09ffff255c3ba4A241111Ff1262F045` | ERC20 stablecoin |
| **TroveManager** | `0x5aB87c2398454125Dd424425e39c8909bBE16022` | Manages collateralized positions |
| **SortedTroves** | `0x2C32e48e358d5b893C46906b69044D342d8DDd5F` | Ordered trove list |
| **PriceFeed** | `0x80e651c9739C1ed15A267c11b85361780164A368` | FIL/USD oracle |
| **StabilityPool** | `0x791Ad78bBc58324089D3E0A8689E7D045B9592b5` | Liquidation buffer |
| **ActivePool** | `0x8637Ac7FdBB4c763B72e26504aFb659df71c7803` | Holds system collateral |
| **BorrowerOperations** | `0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0` | User-facing operations |
| **MultiTroveGetter** | `0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F` | Batch trove queries |

### DEX Pool Addresses
| Pool | Address |
|------|---------|
| USDFC/WFIL | `0x4e07447bd38e60b94176764133788be1a0736b30` |
| USDFC/axlUSDC | `0x21ca72fe39095db9642ca9cc694fa056f906037f` |
| USDFC/USDC | `0xc8f38dbaf661b897b6a2ee5721aac5a8766ffa13` |

---

## Core Data Types

### ProtocolMetrics
```rust
pub struct ProtocolMetrics {
    pub total_supply: Decimal,        // Total USDFC minted
    pub circulating_supply: Decimal,  // In circulation
    pub total_collateral: Decimal,    // Total FIL locked
    pub active_troves: u64,           // Number of positions
    pub tcr: Decimal,                 // Total Collateral Ratio %
    pub stability_pool_balance: Decimal,
    pub treasury_balance: Decimal,
}
```

### Trove
```rust
pub struct Trove {
    pub address: String,      // Owner address
    pub collateral: Decimal,  // FIL amount
    pub debt: Decimal,        // USDFC owed
    pub icr: Decimal,         // Individual Collateral Ratio %
    pub status: TroveStatus,  // Active/AtRisk/Critical/Closed
}
```

### Transaction
```rust
pub struct Transaction {
    pub hash: String,
    pub tx_type: TransactionType,  // Mint/Burn/Transfer
    pub amount: Decimal,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub block: u64,
    pub status: TransactionStatus,
}
```

---

## Key Formulas

### Total Collateral Ratio (TCR)
```
TCR = (Total_Collateral × FIL_Price) / Total_Debt × 100%
```

### Individual Collateral Ratio (ICR)
```
ICR = (Trove_Collateral × FIL_Price) / Trove_Debt × 100%
```

### APR from Unit Price (Lending)
```rust
fn unit_price_to_apr(unit_price: f64, maturity_timestamp: i64) -> f64 {
    let bond_price = unit_price / 10000.0;  // 9500 → 0.95
    let days_to_maturity = (maturity_timestamp - now) / 86400;
    let discount = (1.0 / bond_price) - 1.0;
    discount * 365.0 / days_to_maturity * 100.0
}
```

### Risk Classification
| ICR Range | Status | Color |
|-----------|--------|-------|
| < 110% | Liquidatable | Red |
| 110-125% | Critical | Orange |
| 125-150% | At Risk | Yellow |
| 150-200% | Moderate | Blue |
| > 200% | Safe | Green |

---

## Current Implementation Status

### ✅ Working (Real Data)

| Page | Data Source | Features |
|------|-------------|----------|
| **Dashboard** | RPC + Blockscout | Total supply, collateral, TCR, troves, stability pool, recent transactions |
| **Collateral Health** | RPC | All troves with ICR, risk distribution chart |
| **Transactions** | Blockscout | Real-time USDFC transfers, search/filter |
| **Lending Markets** | Subgraph | Active markets, APRs, volumes |
| **Contracts** | Static | All 8 contract addresses with links |

### ⚠️ Partially Working

| Page | Working | Mock |
|------|---------|------|
| **Supply** | Total supply | Circulating/treasury split, top holders |
| **Stability Pool** | Pool balance | Depositor count, APR, depositor list |
| **Address Lookup** | Structure | Full implementation needed |

### ❌ Placeholder/Mock

| Page | Status |
|------|--------|
| **Entity Registry** | Hardcoded DEX list |
| **Flow Diagrams** | Placeholder text |
| **Network Graph** | Placeholder text |
| **Sankey Charts** | Placeholder text |
| **Architecture** | Placeholder text |
| **API Reference** | Placeholder text |
| **Data Export** | Placeholder text |
| **Alerts** | UI only, no backend |

---

## Server Functions

All data fetching happens through Leptos server functions in `src/server_fn.rs`:

| Function | Returns | Source |
|----------|---------|--------|
| `get_protocol_metrics()` | ProtocolMetrics | RPC |
| `get_recent_transactions(limit)` | Vec<Transaction> | Blockscout |
| `get_troves(limit, offset)` | Vec<Trove> | RPC |
| `get_icr_distribution()` | Vec<ChartDataPoint> | RPC (calculated) |
| `get_lending_markets()` | Vec<LendingMarketData> | Subgraph |
| `get_address_info(address)` | AddressInfo | Blockscout |
| `get_top_holders(limit)` | Vec<TokenHolderInfo> | Blockscout |
| `get_stability_pool_transfers(limit)` | Vec<Transaction> | Blockscout |

---

## Build & Run

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install cargo-leptos
cargo install cargo-leptos
```

### Development
```bash
cd usdfc-terminal

# Watch mode (auto-rebuild)
cargo leptos watch

# Runs on http://localhost:3000
```

### Production Build
```bash
# Build release
cargo leptos build --release

# Run server
./target/release/usdfc-analytics-terminal
```

### Check Compilation
```bash
# SSR build
cargo check --features ssr

# WASM build
cargo check --target wasm32-unknown-unknown --features hydrate
```

---

## Environment Configuration

All configuration via `.env` file:

```bash
# RPC Endpoint
RPC_URL=https://api.node.glif.io/rpc/v1

# APIs
SUBGRAPH_URL=https://api.goldsky.com/.../sf-filecoin-mainnet/latest/gn
BLOCKSCOUT_URL=https://filecoin.blockscout.com/api/v2
GECKOTERMINAL_URL=https://api.geckoterminal.com/api/v2/networks/filecoin

# Contracts (all required)
USDFC_TOKEN=0x80B98d3aa09ffff255c3ba4A241111Ff1262F045
TROVE_MANAGER=0x5aB87c2398454125Dd424425e39c8909bBE16022
# ... (see .env for full list)

# Server
HOST=0.0.0.0
PORT=3000

# Refresh intervals (seconds)
REFRESH_INTERVAL_FAST=30
REFRESH_INTERVAL_MEDIUM=60
REFRESH_INTERVAL_SLOW=300
```

---

## What Remains To Be Done

### Phase 1: Fix Build/Code Issues
From `IMPLEMENTATION_PLAN.md`:
1. Fix `.expect()` panics (24 instances in pages)
2. Fix Option<View> type mismatches
3. Fix hydration in lib.rs
4. Consolidate format functions
5. Update deprecated chrono APIs

### Phase 2: Replace Mock Data
From `FULL_INTEGRATION_PLAN.md`:
1. **Dashboard charts** - Use GeckoTerminal OHLCV for price/supply history
2. **Supply breakdown** - Get real circulating vs treasury from holders API
3. **Stability depositors** - Query recent deposit events
4. **Address lookup** - Full implementation with Blockscout + RPC

### Phase 3: UI Redesign
From `LAYERZERO_REDESIGN_PLAN.md`:
1. Monochrome UI chrome (only data gets color)
2. Remove all border-radius (0px everywhere)
3. Remove shadows and gradients
4. Standardize typography

### Phase 4: Visualizations
- Network graph (D3.js or SVG)
- Sankey flow diagram (Plotly)
- Architecture diagram (Mermaid or static SVG)
- Yield curve chart for lending

---

## Key Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| **RPC Archival (~15h)** | No historical events beyond 15h | Use GeckoTerminal for price history |
| **No Database** | All data fetched fresh | Add caching layer (future) |
| **GeckoTerminal Rate Limit** | 30 req/min | Cache responses, batch requests |
| **No WebSocket** | No real-time updates | Polling on page load |

---

## Related Documentation

| Document | Purpose |
|----------|---------|
| `FULL_INTEGRATION_PLAN.md` | Complete data integration roadmap |
| `IMPLEMENTATION_PLAN.md` | Build fixes and code quality |
| `LAYERZERO_REDESIGN_PLAN.md` | UI/UX redesign specification |
| `TERMINAL_DATA_INVENTORY.md` | Page-by-page data audit |
| `USDFC_QUICK_REFERENCE_V3.md` | API endpoints and formulas |
| `DAY1_PROGRESS_REPORT.md` | Infrastructure completion status |

---

## Summary

This is a **well-architected Leptos full-stack Rust application** with:
- Clean separation between server (SSR, API clients) and client (WASM hydration)
- Multiple data source integrations (RPC, REST, GraphQL)
- Modular page-based routing
- ~50% real data, ~35% mock, ~15% not implemented

**Primary work remaining:**
1. Fixing build issues and code quality
2. Completing data integration (replacing mock data)
3. UI polish (LayerZero-style monochrome redesign)
4. Visualization implementations

---

**Server URL:** http://localhost:3000 (dev) or configured HOST:PORT
**GitHub:** (if applicable)
**Live Demo:** http://5.180.182.231:53111
