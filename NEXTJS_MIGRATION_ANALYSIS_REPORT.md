# Next.js Frontend Migration Analysis Report

**Source Project:** USDFC Analytics Terminal (Leptos/Rust)
**Target Framework:** Next.js 14+ (App Router)
**Analysis Date:** January 4, 2026
**Report Version:** 1.0

---

## 1. FRONTEND FILE INVENTORY

### 1.1 Complete Directory Structure

```
src/
├── app.rs                          # Main app component, routing, context providers (193 lines)
├── main.rs                         # Entry point, server setup (395 lines)
├── lib.rs                          # Library root, exports (64 lines)
├── global_metrics.rs               # Shared metrics context (78 lines)
├── error.rs                        # Error types (142 lines)
├── types.rs                        # Core data types (1247 lines)
├── format.rs                       # Formatting utilities (384 lines)
├── config.rs                       # Configuration (217 lines)
├── cache.rs                        # SSR caching (178 lines)
├── server_fn.rs                    # Server functions (1541 lines)
├── state.rs                        # Server state
├── data.rs                         # Data source abstraction (77 lines)
├── address_conv.rs                 # Address conversion
├── fileserv.rs                     # Static file serving (74 lines)
├── rpc.rs                          # Filecoin RPC client (433 lines)
├── blockscout.rs                   # Blockscout API client (1252 lines)
├── subgraph.rs                     # Secured Finance subgraph (495 lines)
├── gecko.rs                        # GeckoTerminal client (415 lines)
├── circuit_breaker.rs              # Resilience pattern (271 lines)
├── historical.rs                   # SQLite snapshots (392 lines)
├── api/
│   ├── mod.rs                      # API module
│   ├── handlers.rs                 # Axum HTTP handlers
│   └── models.rs                   # API response types
├── components/
│   ├── mod.rs                      # Component exports (27 lines)
│   ├── header.rs                   # Header component (269 lines)
│   ├── sidebar.rs                  # Sidebar component (137 lines)
│   ├── footer.rs                   # Footer component (62 lines)
│   ├── icons.rs                    # SVG icon components
│   ├── charts.rs                   # SVG chart components (285 lines)
│   ├── gauge.rs                    # Gauge chart component
│   ├── metric_card.rs              # Metric card (105 lines)
│   ├── data_table.rs               # Data table component
│   ├── error_boundary.rs           # Error boundary
│   ├── loading.rs                  # Loading skeletons (236 lines)
│   ├── memo.rs                     # Memo component
│   ├── tabs.rs                     # Tab navigation (85 lines)
│   ├── controls.rs                 # UI controls (417 lines)
│   ├── pagination.rs               # Pagination
│   ├── advanced_chart/             # ECharts wrapper
│   │   ├── mod.rs
│   │   ├── container.rs
│   │   ├── series.rs
│   │   ├── legend.rs
│   │   ├── tooltip.rs
│   │   ├── canvas.rs
│   │   └── header.rs
├── pages/
│   ├── mod.rs                      # Page exports (28 lines)
│   ├── dashboard.rs                # Main dashboard (480 lines)
│   ├── advanced.rs                 # Advanced analytics (2100+ lines)
│   ├── protocol.rs                 # Protocol page (843 lines)
│   ├── lending.rs                  # Lending markets (541 lines)
│   ├── transactions.rs             # Transaction explorer (1023 lines)
│   ├── entities.rs                 # Entity registry (374 lines)
│   ├── analytics.rs                # Analytics page
│   ├── infrastructure.rs           # Infrastructure page
│   ├── tools.rs                    # Tools page
│   ├── address.rs                  # Address detail page
│   └── legacy/                     # Redirect stubs
└── styles.css                      # Main stylesheet (2200+ lines)
```

### 1.2 Component Count Summary

| Category | Count | Description |
|----------|-------|-------------|
| Page Components | 17 | Full pages with routes |
| UI Components | 16 | Reusable UI elements |
| Chart Components | 4+ | SVG and ECharts |
| Icon Components | 20+ | SVG icons |
| Total Files | 70+ | All source files |

---

## 2. COMPONENT DOCUMENTATION

### 2.1 Root App Component (`app.rs`)

**Props/Signals:**
- `AppState` context: `sidebar_expanded`, `theme_mode`, `network_status`, `unread_alerts`, `mobile_menu_open`
- `GlobalMetrics` context: `protocol`, `price`, `holders`, `refresh_trigger`

**Routing:**
```rust
Routes:
  / → Dashboard
  /dashboard → Dashboard
  /protocol → Protocol
  /transactions → TransactionSearch
  /address → TransactionSearch
  /address/:addr → AddressDetail
  /lending → LendingMarkets
  /entities → EntityRegistry
  /analytics → Analytics
  /advanced → AdvancedAnalytics
  /infrastructure → Infrastructure
  /tools → Tools
  /*any → NotFoundPage
```

**Styling Classes:** `app-container`, `sidebar-collapsed`, `theme-light`, `main-wrapper`, `main-content`

**Next.js Equivalent:**
```tsx
// app/layout.tsx
// app/page.tsx (Dashboard)
// app/[...not-found]/page.tsx
```

---

### 2.2 Dashboard Page (`pages/dashboard.rs`)

**Signals:**
- `time_range: RwSignal<TimeRange>` - Master state for all data
- `chart_type: RwSignal<String>` - "area", "line", or "bars"

**Resources:**
- `protocol: Resource<(), Result<ProtocolMetrics>>`
- `price: Resource<(), Result<USDFCPriceData>>`
- `volumes: Resource<Option<i32>, Result<Vec<Volume>>>`
- `transactions: Resource<TimeRange, Result<Vec<Transaction>>>`
- `health: Resource<(), Result<ApiHealth>>`

**Derived Signals:**
- `price_display` - Formatted price with fallback
- `price_change` - 24h change percentage
- `tcr_display` - TCR with status color
- `tcr_status` - "negative", "warning", "positive", or ""
- `source_status` - Array of (name, StatusLevel)

**Components:**
- `TimeRangeSelector` - Time range buttons
- `ChartTypeSelector` - Chart type toggle
- `DashboardChart` - SVG chart component
- `MetricRow` - Key metrics display
- `ActivityItem` - Transaction list items

**Layout Structure:**
```
page-viewport
├── page-header-bar
│   ├── page-header-left (title, inline stats, status dots)
│   └── page-header-right (TimeRangeSelector, refresh btn)
└── page-primary-zone
    ├── page-main-content (split-panel with chart)
    └── page-sidebar-panel (Key Metrics + Recent Activity)
```

**Next.js Equivalent:**
```tsx
// app/page.tsx
// useSWR for data fetching
// useState for time range and chart type
// Memo for chart calculations
```

---

### 2.3 Advanced Analytics Page (`pages/advanced.rs`)

**Signals:**
- `resolution: RwSignal<ChartResolution>` - M1, M5, M15, M30, H1, H4, D1
- `lookback: RwSignal<ChartLookback>` - 1h, 6h, 24h, 7d, 30d, 90d, all
- `chart_type: RwSignal<ChartType>` - Area, Line, Candle
- `wallet_address: RwSignal<Option<String>>`
- `custom_start/custom_end: Signal<Option<i64>>`
- `visible_metrics: RwSignal<HashSet<ChartMetric>>`
- `is_loading: RwSignal<bool>`
- `chart_data: RwSignal<ChartDataResponse>`

**URL State:**
- State serialized to query params: `?metrics=price,volume&res=1h&lookback=1w&type=area`
- `ChartUrlState::from_url()` parses on load
- `update_browser_url()` updates without reload

**Chart Data (ECharts):**
```typescript
interface ChartDataResponse {
  price_candles: TVCandle[];       // [time, open, close, low, high]
  volume_data: TVCandle[];
  liquidity_data: [time, value][];
  tcr_data: [time, value][];
  supply_data: [time, value][];
  holders_data: [time, value][];
  lend_apr_data: [time, value][];
  borrow_apr_data: [time, value][];
  transfers_data: [time, value][];
  current_price: number;
  current_volume_24h: number;
  current_liquidity: number;
  current_tcr: number;
  current_supply: number;
  current_holders: number;
  current_lend_apr: number;
  current_borrow_apr: number;
  fetch_time_ms: number;
}
```

**Chart Configuration:**
- 8 y-axes (Price, Volume, Liquidity, TCR, Supply, Holders, APR, Transfers)
- DataZoom slider and inside zoom
- Click-to-lock crosshair feature
- Export PNG button

**Components:**
- `DateRangePicker` - Custom date selection
- `LzStatCard` - Stat cards with loading/error states
- `LzSource` - Data source status indicator

**Next.js Equivalent:**
```tsx
// app/advanced/page.tsx
// React ECharts wrapper
// URLSearchParams for state
// SWR for auto-refresh
```

---

### 2.4 Header Component (`components/header.rs`)

**Resources:**
- `price_data` - USDFC price and 24h change
- `holder_count` - Total token holders
- `protocol_metrics` - TCR and other metrics

**Display:**
- USDFC price with change percentage
- 24h volume
- Liquidity
- Holders count
- TCR
- Network status indicator

**Next.js Equivalent:**
```tsx
// components/Header.tsx
// useSWR for live data
// Fixed position header
```

---

### 2.5 Sidebar Component (`components/sidebar.rs`)

**Navigation Structure:**
```
Overview
  Dashboard (/dashboard)
Protocol  
  Protocol (/protocol)
Markets
  Lending (/lending)
Data
  Transactions (/transactions)
  Entities (/entities)
Analytics
  Analytics (/analytics)
  Advanced (/advanced)
Reference
  Infrastructure (/infrastructure)
  Tools (/tools)
```

**Features:**
- Collapse/expand state
- Mobile menu overlay
- Active item highlighting
- Badge support

**Next.js Equivalent:**
```tsx
// components/Sidebar.tsx
// Next.js Link components
// usePathname for active state
// Collapsible state with useState
```

---

### 2.6 Footer Component (`components/footer.rs`)

**Display:**
- Version info
- API health status (RPC, Blockscout, Subgraph)
- Status dots with connection indicators

**Next.js Equivalent:**
```tsx
// components/Footer.tsx
// Fixed position footer
```

---

### 2.7 Chart Components (`components/charts.rs`)

**AreaChart:**
```tsx
props: {
  data: Vec<(String, f64)>,
  color?: "#00d4ff",
  height?: 200
}
```

**BarChart:**
```tsx
props: {
  data: Vec<(String, f64)>,
  color?: "#00d4ff", 
  height?: 200
}
```

**DonutChart:**
```tsx
props: {
  data: Vec<(String, f64, &str)>,  // label, value, color
  size?: 120
}
```

**SparklineChart:**
```tsx
props: {
  data: Vec<f64>,
  color?: "#00d4ff",
  width?: 60,
  height?: 24
}
```

**Next.js Equivalent:**
```tsx
// components/charts/AreaChart.tsx
// components/charts/BarChart.tsx
// components/charts/DonutChart.tsx
// SVG-based or use recharts
```

---

### 2.8 Controls Component (`components/controls.rs`)

**TimeRangeSelector:**
```tsx
props: {
  selected: RwSignal<TimeRange>,
  options?: TimeRange[],
  compact?: boolean
}

enum TimeRange {
  Hour1, Hour6, Hour24, Day7, Day30, Day90, All
}
```

**StatusDots:**
```tsx
props: {
  sources: Signal<Vec<(&str, StatusLevel)>>
}

enum StatusLevel {
  Online, Warning, Offline, Unknown
}
```

**MetricBar:**
```tsx
props: {
  metrics: Signal<Vec<MetricItem>>,
  maxVisible?: 5
}

interface MetricItem {
  label: &str;
  value: String;
  change?: f64;
}
```

**ChartTypeSelector:**
- Options: "area", "line", "bars"

**InlineStat:**
```tsx
props: {
  label: &str,
  value: Signal<String>,
  change?: Signal<f64>,
  status?: &str
}
```

**Next.js Equivalent:**
```tsx
// components/controls/TimeRangeSelector.tsx
// components/controls/StatusDots.tsx
// components/controls/MetricBar.tsx
```

---

### 2.9 Loading Components (`components/loading.rs`)

**Skeleton Components:**
- `Skeleton` - Generic skeleton
- `CardSkeleton` - Card placeholder
- `ActivityItemSkeleton` - Activity item
- `MetricRowSkeleton` - Metric row
- `MetricCardSkeleton` - Metric card
- `TableRowSkeleton` - Table row
- `TableSkeleton` - Full table
- `ChartSkeleton` - Chart placeholder

**Other Components:**
- `LoadingSpinner` - Full-page spinner
- `InlineSpinner` - Inline loading
- `LoadingOverlay` - Section overlay
- `EmptyState` - No data state
- `LiveIndicator` - Live data badge
- `ProgressBar` - Progress indicator

**Next.js Equivalent:**
```tsx
// components/loading/Skeleton.tsx
// components/loading/LoadingSpinner.tsx
// components/loading/EmptyState.tsx
// shadcn/ui skeletons
```

---

### 2.10 Protocol Page (`pages/protocol.rs`)

**Tabs:**
1. SupplyDynamicsTab - Supply concentration, mint/burn analysis, top holders table
2. RiskAnalysisTab - TCR gauge, liquidation risk summary, troves table
3. PoolActivityTab - Stability pool flows, deposits/withdrawals, activity table

**Data:**
- `get_top_holders()` - Top 10 holders
- `get_troves()` - Active troves with ICR
- `get_recent_transactions()` - Mint/burn transactions
- `get_stability_pool_transfers()` - Pool activity

**Next.js Equivalent:**
```tsx
// app/protocol/page.tsx
// Tab components with client state
// Tables with sorting
```

---

### 2.11 Lending Page (`pages/lending.rs`)

**Resources:**
- `get_lending_markets()` - Market pairs by maturity
- `get_order_book()` - Lend/borrow orders
- `get_recent_lending_trades()` - Trade history
- `get_daily_volumes()` - Volume chart data

**Features:**
- Best APR summary cards
- USDFC/FIL market pairs table
- Expandable order books
- Recent trades table
- Daily volume bar chart

**Next.js Equivalent:**
```tsx
// app/lending/page.tsx
// TanStack Table for markets
// Bar chart for volumes
```

---

### 2.12 Transactions Page (`pages/transactions.rs`)

**State:**
- `search_address` - Address lookup
- `filter_type` - Transaction type filter
- `filter_address` - Address filter
- `min_amount/max_amount` - Amount range
- `current_page/page_size` - Pagination

**Features:**
- Address search with validation
- Filter chips with remove buttons
- Pagination with size selector
- Transaction detail modal
- Address detail modal
- External blockscout links

**Next.js Equivalent:**
```tsx
// app/explorer/page.tsx
// TanStack Table for transactions
// Modal components for details
// Address validation hook
```

---

### 2.13 Entities Page (`pages/entities.rs`)

**Sections:**
1. Protocol Contracts - Core contract addresses
2. DEX Liquidity Pools - GeckoTerminal pool data
3. Top Holders - Paginated holder list

**Features:**
- Copy-to-clipboard buttons
- Entity identification (contracts, pools, unknown)
- Entity badges (protocol, whale, exchange, contract)
- Pagination for holders

**Next.js Equivalent:**
```tsx
// app/entities/page.tsx
// Tables with copy buttons
// Badge components
```

---

## 3. COMPLETE CSS EXTRACTION

### 3.1 Color Palette

```css
:root {
  /* Backgrounds - Monochrome scale */
  --bg-primary: #000000;
  --bg-secondary: #080808;
  --bg-tertiary: #0f0f0f;
  --bg-card: #0f0f0f;
  --bg-elevated: #161616;

  /* Borders */
  --border-color: #1a1a1a;
  --border-subtle: #141414;
  --border-strong: #2a2a2a;

  /* Text - White/gray hierarchy */
  --text-primary: #ffffff;
  --text-secondary: #b0b0b0;
  --text-muted: #888888;
  --text-disabled: #666666;

  /* Status colors - ONLY for data visualization */
  --color-positive: #22c55e;
  --color-negative: #ef4444;
  --color-warning: #f59e0b;
  --color-info: #3b82f6;

  /* Chart palette */
  --chart-1: #ffffff;
  --chart-2: #a0a0a0;
  --chart-3: #606060;
  --chart-4: #22c55e;
  --chart-5: #3b82f6;

  /* Legacy accent colors */
  --accent-cyan: #ffffff;
  --accent-green: #22c55e;
  --accent-red: #ef4444;
  --accent-yellow: #f59e0b;
  --accent-purple: #a0a0a0;
  --accent-blue: #ffffff;

  /* Gradients */
  --gradient-cyan: linear-gradient(135deg, #ffffff 0%, #e0e0e0 100%);
  --gradient-green: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  --gradient-red: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
}
```

### 3.2 Typography System

```css
:root {
  --font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: "JetBrains Mono", "SF Mono", "Consolas", monospace;
}

/* Font sizes */
.page-title { font-size: 18px; font-weight: 700; }
.page-subtitle { font-size: 14px; color: var(--text-secondary); }
.card-title { font-size: 14px; font-weight: 600; }
.metric-label { font-size: 12px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.metric-value { font-size: clamp(16px, 3vw, 24px); font-weight: 700; font-family: var(--font-mono); }
.stat-label { font-size: 10px; color: var(--text-muted); text-transform: uppercase; }
.stat-value { font-size: 13px; font-weight: 600; font-family: var(--font-mono); }
.table th { font-size: 12px; font-weight: 600; }
.table td { font-size: 14px; }
```

### 3.3 Spacing System

```css
:root {
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
}

/* Usage examples */
.card { padding: 20px; margin-bottom: 20px; }
.page-header { margin-bottom: 24px; }
.grid-2 { gap: 20px; }
.table th, .table td { padding: 12px 16px; }
```

### 3.4 Z-Index Scale

```css
:root {
  --z-sidebar: 100;
  --z-header: 200;
  --z-dropdown: 300;
  --z-modal: 400;
  --z-tooltip: 500;
}
```

### 3.5 Animation Keyframes

```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes skeleton-loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

@keyframes pulse-green {
  0%, 100% { opacity: 1; box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4); }
  50% { opacity: 0.8; box-shadow: 0 0 8px 2px rgba(34, 197, 94, 0.3); }
}

@keyframes pulse-red {
  0%, 100% { opacity: 1; box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { opacity: 0.6; box-shadow: 0 0 8px 2px rgba(239, 68, 68, 0.5); }
}
```

### 3.6 Component CSS Classes

**Header:** `.header`, `.header-left`, `.header-stats`, `.header-right`, `.logo`, `.logo-icon`, `.sidebar-toggle`

**Sidebar:** `.sidebar`, `.sidebar-section`, `.sidebar-section-title`, `.sidebar-item`, `.sidebar-badge`

**Main:** `.main-wrapper`, `.main-content`, `.page-container`, `.page-header`

**Cards:** `.card`, `.card-header`, `.card-title`, `.card-subtitle`

**Metrics:** `.metric-card`, `.metric-label`, `.metric-value`, `.metric-change`, `.metric-row`

**Tables:** `.table`, `.table th`, `.table td`, `.table-responsive`, `.data-table`

**Buttons:** `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.refresh-btn`, `.time-btn`

**Inputs:** `.input`, `.input-group`, `.input-with-icon`, `.select`, `.filter-chip`

**Status:** `.status-badge`, `.type-badge`, `.status-dot`, `.live-indicator`

**Charts:** `.chart-container`, `.chart-svg`, `.gauge-container`, `.donut-container`

**Loading:** `.skeleton`, `.loading-spinner`, `.loading-overlay`, `.empty-state`

**Advanced (LZ):** `.lz-page`, `.lz-header`, `.lz-chart-section`, `.lz-metrics-row`, `.lz-stat-card`, `.lz-legend`, `.lz-type-btn`, `.res-btn`, `.lb-btn`, `.date-picker-btn`

**Modal:** `.address-modal-backdrop`, `.address-modal-content`, `.address-modal-header`, `.tx-modal-content`

**Misc:** `.progress-bar`, `.progress-fill`, `.tabs`, `.tab`, `.filter-panel`, `.entity-badge`, `.icon-circle`

### 3.7 Responsive Breakpoints

No explicit media queries visible. Uses:
- `clamp()` for fluid typography
- `.hide-mobile` class for mobile hiding
- CSS Grid with `min-width: 0` for overflow handling

**Next.js Tailwind Recommendation:**
```tsx
// tailwind.config.ts
theme: {
  colors: {
    background: { primary: '#000000', secondary: '#080808', tertiary: '#0f0f0f' },
    border: { DEFAULT: '#1a1a1a', subtle: '#141414', strong: '#2a2a2a' },
    text: { primary: '#ffffff', secondary: '#b0b0b0', muted: '#888888' },
    positive: '#22c55e',
    negative: '#ef4444',
    warning: '#f59e0b',
    info: '#3b82f6',
  },
  fontFamily: {
    sans: ['Inter', 'sans-serif'],
    mono: ['JetBrains Mono', 'monospace'],
  },
}
```

---

## 4. API ENDPOINT REFERENCE

### 4.1 Server Functions (Called from Frontend)

All server functions are defined in `server_fn.rs` and called via Leptos server function mechanism.

| Function | Parameters | Returns | Purpose |
|----------|------------|---------|---------|
| `get_protocol_metrics` | () | ProtocolMetrics | Supply, collateral, TCR |
| `get_usdfc_price_data` | () | USDFCPriceData | Price, volume, liquidity |
| `get_recent_transactions` | limit: Option<u32> | Vec<Transaction> | Transfer history |
| `get_troves` | limit, offset | Vec<Trove> | Collateralized positions |
| `get_lending_markets` | () | Vec<Market> | Secured Finance markets |
| `get_daily_volumes` | days: Option<i32> | Vec<Volume> | Historical volume |
| `get_address_info` | address: &str | AddressInfo | Wallet analysis |
| `get_normalized_address` | addr: &str | String | EVM/f4 conversion |
| `get_top_holders` | limit: Option<u32> | Vec<Holder> | Token holder list |
| `get_stability_pool_transfers` | limit: Option<u32> | Vec<Transaction> | Stability pool activity |
| `get_holder_count` | () | u64 | Total holders |
| `get_order_book` | limit: Option<u32> | OrderBook | Lending orders |
| `get_recent_lending_trades` | limit: Option<u32> | Vec<Trade> | Lending activity |
| `check_api_health` | () | ApiHealth | Service status |
| `get_advanced_chart_data` | res, lookback, start, end | ChartDataResponse | Multi-metric charts |

### 4.2 REST API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/price` | GET | Current USDFC price |
| `/api/v1/metrics` | GET | Protocol metrics |
| `/api/v1/history` | GET | Historical data |
| `/api/v1/troves` | GET | Trove list |
| `/api/v1/transactions` | GET | Recent transfers |
| `/api/v1/address/:addr` | GET | Address info |
| `/api/v1/lending` | GET | Lending markets |
| `/api/v1/holders` | GET | Top holders |

### 4.3 External API Sources

**Filecoin RPC:**
- Endpoint: `https://api.node.glif.io/rpc/v1`
- Methods: `eth_call`, `eth_blockNumber`, `eth_getBalance`

**Blockscout:**
- Base: `https://filecoin.blockscout.com/api`
- Endpoints: `/addresses/{addr}/transfers`, `/tokens/{addr}/holders`

**GeckoTerminal:**
- Base: `https://api.geckoterminal.com/api/v2`
- Endpoints: `/networks/fil/pools/{addr}`

**Secured Finance Subgraph:**
- Endpoint: `https://api.goldsky.com/api/v1/graphql`
- Query: Lending markets, orders, trades

### 4.4 Data Types

```typescript
interface ProtocolMetrics {
  total_supply: number;
  circulating_supply: number;
  total_collateral: number;
  active_troves: number;
  tcr: number;
  stability_pool_balance: number;
  treasury_balance: number;
}

interface USDFCPriceData {
  price_usd: number | null;
  price_change_24h: number | null;
  volume_24h: number | null;
  liquidity_usd: number | null;
}

interface Transaction {
  hash: string;
  tx_type: 'mint' | 'burn' | 'transfer' | 'deposit' | 'withdraw' | 'liquidation' | 'redemption';
  amount: number;
  from: string;
  to: string;
  timestamp: number;
  block: number;
  status: 'success' | 'failed' | 'pending';
}

interface Trove {
  address: string;
  collateral: number;
  debt: number;
  icr: number;
  status: 'active' | 'closed' | 'liquidated';
}

interface ChartDataResponse {
  price_candles: Array<[number, number, number, number, number]>; // time, open, close, low, high, volume
  volume_data: Array<[number, number]>;
  liquidity_data: Array<[number, number]>;
  tcr_data: Array<[number, number]>;
  supply_data: Array<[number, number]>;
  holders_data: Array<[number, number]>;
  lend_apr_data: Array<[number, number]>;
  borrow_apr_data: Array<[number, number]>;
  transfers_data: Array<[number, number]>;
  current_price: number | null;
  current_volume_24h: number | null;
  current_liquidity: number | null;
  current_tcr: number | null;
  current_supply: number | null;
  current_holders: number | null;
  current_lend_apr: number | null;
  current_borrow_apr: number | null;
  fetch_time_ms: number;
}
```

---

## 5. NEXT.JS MIGRATION CHECKLIST

### 5.1 Project Setup

- [ ] Initialize Next.js 14+ project with TypeScript
- [ ] Install dependencies:
  - `echarts`, `echarts-for-react`
  - `@tanstack/react-table`
  - `swr` or `@tanstack/react-query`
  - `clsx`, `tailwind-merge`
  - `lucide-react` for icons
  - `date-fns` for formatting
- [ ] Configure Tailwind CSS with exact color palette
- [ ] Set up ESLint and Prettier

### 5.2 State Management Migration

| Leptos Pattern | React Equivalent |
|----------------|------------------|
| `create_signal` | `useState` |
| `create_rw_signal` | `useState` with setter |
| `create_resource` | `useSWR` or `useQuery` |
| `create_local_resource` | `useSWR` (client-only) |
| `create_memo` | `useMemo` |
| `create_effect` | `useEffect` |
| `provide_context` | React Context |
| `use_context` | `useContext` |
| `create_node_ref` | `useRef` |
| `RwSignal<T>` | `State<T>` |
| `Signal<T>` | `T` (derived) |

### 5.3 Component Migration Order

1. **Layout Components**
   - [ ] Layout wrapper (sidebar, header, footer)
   - [ ] Header with live data
   - [ ] Sidebar navigation
   - [ ] Footer with status

2. **UI Components**
   - [ ] Button variants
   - [ ] Input components
   - [ ] Card components
   - [ ] Table components
   - [ ] Modal components
   - [ ] Loading skeletons
   - [ ] Empty states

3. **Chart Components**
   - [ ] AreaChart (SVG)
   - [ ] BarChart (SVG)
   - [ ] DonutChart (SVG)
   - [ ] Sparkline (SVG)
   - [ ] GaugeChart
   - [ ] ECharts wrapper (Advanced page)

4. **Controls**
   - [ ] TimeRangeSelector
   - [ ] ChartTypeSelector
   - [ ] StatusDots
   - [ ] MetricBar
   - [ ] Pagination

5. **Page Components**
   - [ ] Dashboard
   - [ ] Advanced Analytics
   - [ ] Protocol
   - [ ] Lending Markets
   - [ ] Transactions/Explorer
   - [ ] Entities
   - [ ] Analytics
   - [ ] Infrastructure
   - [ ] Tools

### 5.4 Data Fetching Strategy

**Server Components (RSC):**
- Protocol metrics on page load
- Lending markets
- Top holders
- Transaction history initial load

**Client Components:**
- Live price updates (SWR with refresh interval)
- Chart data (lazy loaded)
- Filtered/sorted tables
- Modals and interactive elements

```tsx
// Example: Dashboard data fetching
async function getDashboardData() {
  const [metrics, price, volumes, transactions, health] = await Promise.all([
    fetch('/api/v1/metrics').then(r => r.json()),
    fetch('/api/v1/price').then(r => r.json()),
    fetch('/api/v1/history?days=7').then(r => r.json()),
    fetch('/api/v1/transactions?limit=100').then(r => r.json()),
    fetch('/health').then(r => r.json()),
  ]);
  return { metrics, price, volumes, transactions, health };
}
```

### 5.5 Routing Structure

```
app/
├── layout.tsx              # Root layout with providers
├── page.tsx                # Dashboard
├── advanced/
│   └── page.tsx            # Advanced analytics
├── protocol/
│   └── page.tsx            # Protocol page
├── lending/
│   └── page.tsx            # Lending markets
├── explorer/
│   └── page.tsx            # Transaction search
├── entities/
│   └── page.tsx            # Entity registry
├── analytics/
│   └── page.tsx            # Analytics
├── infrastructure/
│   └── page.tsx            # Infrastructure
├── tools/
│   └── page.tsx            # Tools
├── address/
│   └── [address]/
│       └── page.tsx        # Address detail
└── globals.css             # Tailwind + custom CSS
```

### 5.6 Key Migration Decisions

**Styling:**
- Use Tailwind CSS with custom theme colors
- Maintain exact color values from CSS variables
- Use CSS Modules or Tailwind for component styles

**Charts:**
- Keep SVG charts for simple visualizations (Area, Bar, Donut, Sparkline)
- Use ECharts for advanced analytics page
- Create reusable chart wrapper components

**Tables:**
- Use TanStack Table for all data tables
- Implement sorting, filtering, pagination
- Create reusable Table components

**Icons:**
- Replace inline SVG with lucide-react
- Create icon components map

**Forms:**
- Use controlled inputs with React state
- Implement validation with Zod
- Create reusable Input/Select components

---

## 6. VPS DEPLOYMENT PLAN

### 6.1 Target Environment

- **VPS IP:** 5.180.182.231
- **Current Leptos App:** Port 3000 (assumed)
- **New Next.js App Port:** 3001
- **Deployment Location:** `/home/eya/claude/usdfc-terminal-next` (cloned folder)

### 6.2 Folder Structure

```
/home/eya/claude/
├── usdfc-terminal/          # Original Leptos app (port 3000)
└── usdfc-terminal-next/     # New Next.js app (port 3001)
    ├── app/
    ├── components/
    ├── lib/
    ├── public/
    ├── package.json
    ├── next.config.js
    ├── tailwind.config.ts
    └── tsconfig.json
```

### 6.3 Environment Variables

```bash
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:3000
NEXT_PUBLIC_APP_NAME=USDFC Analytics
NEXT_PUBLIC_NETWORK=filecoin

# Optional: Custom API endpoints
NEXT_PUBLIC_RPC_URL=https://api.node.glif.io/rpc/v1
NEXT_PUBLIC_BLOCKSCOUT_URL=https://filecoin.blockscout.com
NEXT_PUBLIC_GECKO_URL=https://api.geckoterminal.com/api/v2
NEXT_PUBLIC_SUBGRAPH_URL=https://api.goldsky.com/api/v1/graphql
```

### 6.4 Build and Run Commands

```bash
# Development
cd /home/eya/claude/usdfc-terminal-next
npm run dev
# Runs on http://localhost:3001

# Production build
npm run build
npm start
# Runs on http://localhost:3001
```

### 6.5 Process Management (PM2)

```bash
# Install PM2
npm install -g pm2

# Start the app
pm2 start npm --name "usdfc-next" -- run start -- -p 3001

# Save PM2 config
pm2 save

# Setup startup script
pm2 startup

# Monitor
pm2 monit usdfc-next

# Logs
pm2 logs usdfc-next
```

### 6.6 Nginx Configuration (Optional)

```nginx
server {
    listen 80;
    server_name usdfc.5.180.182.231.nip.io;

    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

### 6.7 Firewall Rules

```bash
# Check current rules
sudo ufw status

# Allow port 3001
sudo ufw allow 3001/tcp

# Reload firewall
sudo ufw reload
```

### 6.8 Testing Checklist

- [ ] Verify app loads at `http://5.180.182.231:3001`
- [ ] Test all navigation links
- [ ] Verify data fetching from backend
- [ ] Test responsive layouts
- [ ] Test charts render correctly
- [ ] Test modals and interactions
- [ ] Verify no console errors
- [ ] Test on mobile devices

### 6.9 Rollback Plan

```bash
# If issues occur, stop Next.js
pm2 stop usdfc-next

# Leptos app continues on port 3000
# Fix issues and redeploy
pm2 restart usdfc-next
```

---

## 7. SUMMARY

### 7.1 Complexity Assessment

| Area | Complexity | Notes |
|------|------------|-------|
| Dashboard | Medium | 480 lines, 5 resources, SVG charts |
| Advanced Analytics | High | 2100+ lines, complex ECharts config, URL state |
| Protocol | Medium | 843 lines, tabs, data tables |
| Lending | Medium | 541 lines, expandable tables |
| Transactions | High | 1023 lines, filters, pagination, modals |
| Entities | Low | 374 lines, tables, copy buttons |

### 7.2 Estimated Effort

| Task | Hours |
|------|-------|
| Project setup & config | 4 |
| Layout components | 8 |
| UI component library | 16 |
| Chart components | 12 |
| Controls components | 8 |
| Dashboard page | 8 |
| Advanced page | 16 |
| Protocol page | 8 |
| Lending page | 8 |
| Transactions page | 12 |
| Entities page | 6 |
| Remaining pages | 16 |
| Testing & fixes | 24 |
| **Total** | **~146 hours** |

### 7.3 Risk Areas

1. **ECharts Integration** - Complex inline JS needs proper React wrapper
2. **Real-time Updates** - Need to replicate Leptos resource reactivity
3. **Large State Objects** - Chart data response is complex
4. **URL State Sync** - Advanced page has bidirectional URL sync
5. **Table Sorting/Filtering** - Need robust TanStack Table implementation

### 7.4 Success Criteria

- [ ] All pages render identically to Leptos version
- [ ] Data fetches correctly from backend APIs
- [ ] Charts display with correct data and animations
- [ ] Navigation and routing work correctly
- [ ] Responsive design works on mobile
- [ ] Performance is acceptable (LCP < 2.5s)
- [ ] No console errors in production

---

*Report generated for Next.js migration planning. See CODEBASE_ANALYSIS_REPORT.md for additional backend and architecture details.*
