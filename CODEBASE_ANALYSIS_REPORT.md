# USDFC Analytics Terminal - Comprehensive Codebase Analysis Report

**Generated:** January 4, 2026
**Project Path:** /home/eya/claude/usdfc-terminal
**Analysis Tool:** Manual code review

---

## Table of Contents

1. [Complete Code Map](#1-complete-code-map)
2. [Architecture Overview](#2-architecture-overview)
3. [Issues and Errors Identified](#3-issues-and-errors-identified)
4. [Code Quality Concerns](#4-code-quality-concerns)
5. [Improvement Opportunities](#5-improvement-opportunities)
6. [Frontend-Specific Analysis](#6-frontend-specific-analysis)
7. [Summary](#7-summary)

---

## 1. COMPLETE CODE MAP

### 1.1 Project Structure

```
/home/eya/claude/usdfc-terminal/
├── src/
│   ├── main.rs                 # Entry point with SSR/CSR/hydrate modes
│   ├── lib.rs                  # Library root with module exports
│   ├── app.rs                  # Main App component with routing
│   ├── api/                    # REST API layer
│   │   ├── mod.rs
│   │   ├── handlers.rs         # Axum HTTP handlers
│   │   └── models.rs           # API response types
│   ├── server_fn.rs            # Leptos server functions (1541 lines)
│   ├── rpc.rs                  # Filecoin RPC client
│   ├── blockscout.rs           # Blockscout API client (1252 lines)
│   ├── subgraph.rs             # Secured Finance subgraph client
│   ├── gecko.rs                # GeckoTerminal DEX client
│   ├── config.rs               # Unified configuration from env
│   ├── cache.rs                # SSR caching layer
│   ├── circuit_breaker.rs      # Circuit breaker pattern
│   ├── historical.rs           # SQLite metric snapshots
│   ├── error.rs                # Error types
│   ├── types.rs                # Core data types (1247 lines)
│   ├── format.rs               # Formatting utilities
│   ├── data.rs                 # Data source abstraction
│   ├── global_metrics.rs       # Shared metrics context
│   ├── fileserv.rs             # Static file serving
│   ├── state.rs                # Server state
│   ├── address_conv.rs         # EVM/f4 address conversion
│   ├── components/             # 16 Leptos components
│   │   ├── mod.rs
│   │   ├── header.rs
│   │   ├── sidebar.rs
│   │   ├── footer.rs
│   │   ├── charts.rs           # SVG chart components
│   │   ├── gauge.rs
│   │   ├── metric_card.rs
│   │   ├── data_table.rs
│   │   ├── error_boundary.rs
│   │   ├── loading.rs
│   │   ├── memo.rs
│   │   ├── tabs.rs
│   │   ├── controls.rs         # UI controls
│   │   ├── advanced_chart/     # ECharts wrapper
│   │   └── pagination.rs
│   ├── pages/                  # 28 page components
│   │   ├── mod.rs
│   │   ├── dashboard.rs        # Main dashboard
│   │   ├── advanced.rs         # Advanced analytics (2000+ lines)
│   │   ├── protocol.rs
│   │   ├── lending.rs
│   │   ├── transactions.rs
│   │   ├── entities.rs
│   │   ├── analytics.rs
│   │   ├── infrastructure.rs
│   │   ├── tools.rs
│   │   ├── address.rs
│   │   └── legacy/             # Redirect stubs
│   └── styles.css
├── Cargo.toml                  # Dependencies & features
├── Makefile.toml               # cargo-make tasks
└── target/                     # Build outputs
```

### 1.2 Entry Points & Startup Flow

**main.rs (395 lines):**
- SSR: `main()` → Axum server with routing, CORS, compression, security headers
- CSR/Hydrate: `main()` → Leptos `mount_to_body(App)`
- Registers 15 server functions explicitly
- Health check endpoints: `/health`, `/ready`
- REST API routes: `/api/v1/*`
- Background tasks: snapshot collector, cache cleanup

**lib.rs (64 lines):**
- Exports all modules
- Provides `hydrate()` WASM entry point
- Conditional SSR-only modules

### 1.3 API Aggregation Layer

#### External Data Sources

| Source | Module | Purpose |
|--------|--------|---------|
| Filecoin RPC | `rpc.rs` | Smart contract calls, on-chain metrics |
| Blockscout | `blockscout.rs` | Token transfers, holders, addresses |
| GeckoTerminal | `gecko.rs` | DEX prices, OHLCV, liquidity |
| Secured Finance Subgraph | `subgraph.rs` | Lending markets, orders, trades |
| SQLite | `historical.rs` | Historical snapshots |

#### Server Functions (server_fn.rs)

| Function | Purpose |
|----------|---------|
| `GetProtocolMetrics` | Supply, collateral, TCR, troves |
| `GetRecentTransactions` | Transfer history |
| `GetTroves` | Collateralized positions |
| `GetLendingMarkets` | Yield curve data |
| `GetDailyVolumes` | Historical volume |
| `GetAddressInfo` | Wallet analysis |
| `GetNormalizedAddress` | Address conversion |
| `GetTopHolders` | Token holder list |
| `GetStabilityPoolTransfers` | Stability pool activity |
| `GetUSDFCPriceData` | DEX pricing |
| `CheckApiHealth` | Service status |
| `GetHolderCount` | Total holders |
| `GetOrderBook` | Lending orders |
| `GetRecentLendingTrades` | Lending activity |
| `GetAdvancedChartData` | Multi-metric charts |

#### REST API Endpoints (handlers.rs)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/price` | GET | Current prices |
| `/api/v1/metrics` | GET | Protocol metrics |
| `/api/v1/history` | GET | Historical data |
| `/api/v1/troves` | GET | Trove list |
| `/api/v1/transactions` | GET | Recent transfers |
| `/api/v1/address/:addr` | GET | Address info |
| `/api/v1/lending` | GET | Lending markets |
| `/api/v1/holders` | GET | Top holders |

### 1.4 Frontend Architecture

#### App Component (app.rs)

- Provides `AppState` context (sidebar, theme, network status)
- Provides `GlobalMetrics` context
- Router with 17 routes + redirects
- Error boundary with fallback
- Theme support (dark/light)

#### GlobalMetrics Context

- Shared resources for protocol, price, holders
- Uses `create_local_resource` to avoid hydration issues
- Manual refresh trigger

#### Page Components

| Page | Purpose |
|------|---------|
| Dashboard | Viewport-locked layout, volume charts, key metrics, recent activity |
| Advanced | ECharts multi-series chart, 9 toggleable metrics, URL state sync |
| Protocol | Protocol overview with trove management |
| Lending | Secured Finance markets display |
| Transactions | Token transfer history |
| Entities | Known protocol entities |
| Analytics | Flow visualization |
| Infrastructure | Contract registry |
| Tools | Export, alerts, API docs |

### 1.5 Key Data Structures

#### ProtocolMetrics (types.rs)

```rust
pub struct ProtocolMetrics {
    pub total_supply: Decimal,
    pub circulating_supply: Decimal,
    pub total_collateral: Decimal,
    pub active_troves: u64,
    pub tcr: Decimal,
    pub stability_pool_balance: Decimal,
    pub treasury_balance: Decimal,
}
```

#### Transaction (types.rs)

```rust
pub struct Transaction {
    pub hash: String,
    pub tx_type: TransactionType,
    pub amount: Decimal,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub block: u64,
    pub status: TransactionStatus,
}
```

#### Trove (types.rs)

```rust
pub struct Trove {
    pub address: String,
    pub collateral: Decimal,
    pub debt: Decimal,
    pub icr: Decimal,
    pub status: TroveStatus,
}
```

#### ChartDataResponse (types.rs)

```rust
pub struct ChartDataResponse {
    pub resolution: ChartResolution,
    pub lookback: ChartLookback,
    pub generated_at: i64,
    pub fetch_time_ms: u32,
    pub price_candles: Vec<TVCandle>,
    pub volume_data: Vec<(i64, f64)>,
    pub liquidity_data: Vec<(i64, f64)>,
    pub tcr_data: Vec<(i64, f64)>,
    pub supply_data: Vec<(i64, f64)>,
    pub holders_data: Vec<(i64, u64)>,
    pub lend_apr_data: Vec<(i64, f64)>,
    pub borrow_apr_data: Vec<(i64, f64)>,
    pub transfers_data: Vec<(i64, u64)>,
    pub current_price: Option<f64>,
    pub current_volume_24h: Option<f64>,
    pub current_liquidity: Option<f64>,
    pub current_tcr: Option<f64>,
    pub current_supply: Option<f64>,
    pub current_holders: Option<u64>,
    pub current_lend_apr: Option<f64>,
    pub current_borrow_apr: Option<f64>,
    pub snapshot_count: usize,
    pub oldest_snapshot_time: Option<i64>,
}
```

---

## 2. ARCHITECTURE OVERVIEW

### 2.1 Backend Architecture

```
                    ┌─────────────────────────────────────┐
                    │         main.rs (Axum)              │
                    ├─────────────────────────────────────┤
                    │  ┌─────────┐  ┌─────────────────┐   │
                    │  │ REST API│  │  Leptos Routes  │   │
                    │  │ /api/v1 │  │  (SSR + hydrate)│   │
                    │  └────┬────┘  └────────┬────────┘   │
                    │       │                │            │
                    └───────┼────────────────┼────────────┘
                            │                │
                    ┌───────▼────────────────▼────────────┐
                    │        Server Functions             │
                    │  (leptos server_fn macro)           │
                    └──────────────┬──────────────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
   ┌────▼────┐               ┌─────▼─────┐              ┌─────▼─────┐
   │ RPC     │               │Blockscout │              │Subgraph   │
   │Client   │               │Client     │              │Client     │
   └────┬────┘               └─────┬─────┘              └─────┬─────┘
        │                          │                          │
   ┌────▼──────────────────────────▼──────────────────────────▼─────┐
   │                    Filecoin RPC                              │
   │  • get_total_supply()  • get_tcr()  • get_troves()           │
   └───────────────────────────────────────────────────────────────┘
```

### 2.2 Frontend Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Client (WASM)                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐                                               │
│  │   App        │◄── Context: AppState, GlobalMetrics           │
│  │  Component   │                                               │
│  └──────┬───────┘                                               │
│         │ Router                                                │
│  ┌──────┼───────────────────────────────────────────┐          │
│  │      │                   │                       │          │
│  ▼      ▼                   ▼                       ▼          │
│ ┌─────────────┐    ┌──────────────┐    ┌──────────────────┐   │
│ │ Dashboard   │    │ Advanced     │    │ Other Pages      │   │
│ │ Page        │    │ Analytics    │    │ (protocol, etc)  │   │
│ └──────┬──────┘    └──────┬───────┘    └──────────────────┘   │
│        │                  │                                       │
│        ▼                  ▼                                       │
│ ┌─────────────┐    ┌──────────────┐                               │
│ │ Components  │    │ ECharts      │                               │
│ │ (charts.rs) │    │ Wrapper      │                               │
│ └─────────────┘    └──────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
         ▲                         ▲
         │ Server Functions        │ REST API
         └─────────────────────────┘
                    │
           ┌───────▼───────┐
           │   Server      │
           │   (SSR)       │
           └───────────────┘
```

### 2.3 Data Flow

1. **User Request** → Leptos Router
2. **Page Component** → `create_resource()` or `create_local_resource()`
3. **Server Function Call** → Serialized to server
4. **Server** → Calls appropriate API client (RPC/Blockscout/Subgraph/Gecko)
5. **API Response** → Parsed, cached, returned
6. **Client** → Updates reactive state, triggers re-render

---

## 3. ISSUES AND ERRORS IDENTIFIED

### 3.1 Bugs and Logical Errors

#### Issue 1: Unused Parameter in get_troves

**Location:** `server_fn.rs:123`

```rust
pub async fn get_troves(limit: Option<u32>, _offset: Option<u32>) -> Result<Vec<Trove>...
```

**Description:** The `offset` parameter is ignored despite being part of the function signature.

**Severity:** Medium

**Impact:** Clients cannot paginate through troves; all results are returned at once.

**Recommendation:** Implement offset-based pagination or remove the parameter.

---

#### Issue 2: Silent Error Swallowing in Advanced Chart

**Location:** `pages/advanced.rs:283-284`

```rust
Some(Err(_)) => {
    is_loading.set(false);
}
```

**Description:** Errors are silently swallowed; user sees no feedback when chart data fails to load.

**Severity:** High

**Impact:** Users see empty charts with no explanation when API calls fail.

**Recommendation:** Display error state with retry button.

---

#### Issue 3: Potentially Redundant Server Function Registration

**Location:** `main.rs:63-80`

**Description:** Leptos 0.6 auto-registers server functions; explicit registration may be redundant.

**Severity:** Low

**Impact:** Minor code redundancy; no functional impact.

**Recommendation:** Remove explicit registration if auto-registration works.

---

#### Issue 4: Artificial TCR Value for Zero Debt

**Location:** `rpc.rs:304`

```rust
if total_debt.is_zero() {
    return Ok(Decimal::new(999999, 0)); // Returns 999999% TCR
}
```

**Description:** Returns an artificially high value instead of proper error handling.

**Severity:** Medium

**Impact:** Misleading TCR display when no debt exists.

**Recommendation:** Return `None` or use `Decimal::MAX` with proper documentation.

---

#### Issue 5: Inconsistent -0.0 Handling

**Location:** `format.rs` multiple locations

**Description:** The -0.0 normalization is handled in some functions but not consistently.

**Severity:** Low

**Impact:** Minor UI inconsistency.

**Recommendation:** Centralize normalization logic.

---

### 3.2 Potential Runtime Failures

#### Issue 6: SQLite Mutex Poison Handling

**Location:** `historical.rs:127-130`

```rust
let db_lock = DB_CONN.lock().map_err(|e| {
    tracing::error!("Mutex poison error in save_to_db: {}", e);
    rusqlite::Error::InvalidQuery
})?;
```

**Description:** Mutex poison could cause database writes to fail silently.

**Severity:** Medium

**Impact:** Historical data not persisted if thread panics while holding lock.

**Recommendation:** Implement proper recovery mechanism.

---

#### Issue 7: Rate Limiting Without Retry

**Location:** `gecko.rs:79-81`

```rust
return Err(ApiError::RateLimit {
    retry_after: Duration::from_secs(60),
});
```

**Description:** When rate-limited, returns error without automatic retry mechanism in callers.

**Severity:** Medium

**Impact:** User sees errors instead of automatic retries.

**Recommendation:** Implement retry logic in callers with exponential backoff.

---

#### Issue 8: Silent Overflow Handling

**Location:** `format.rs:15-17`

```rust
pub fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}
```

**Description:** Silently returns 0.0 for values that overflow f64.

**Severity:** Medium

**Impact:** Large numbers display as 0.0 without warning.

**Recommendation:** Log warning or return error for overflow.

---

### 3.3 Type Safety Gaps

#### Issue 9: Ubiquitous Option<f64> in Chart Data

**Description:** Using `Option<f64>` for all metrics means callers must handle `None` extensively.

**Severity:** Medium

**Impact:** Verbose code with frequent `.unwrap_or()` calls.

**Recommendation:** Create `MaybeUnavailable<T>` wrapper with better ergonomics.

---

#### Issue 10: String-Based Error Messages

**Description:** Many errors are converted to strings before being propagated.

**Severity:** Low

**Impact:** Loss of type information for error handling.

**Recommendation:** Use structured error types with `thiserror`.

---

#### Issue 11: Unvalidated Wallet Address

**Location:** `pages/advanced.rs:184`

```rust
let wallet_address = create_rw_signal(None::<String>);
```

**Description:** Address not validated until used.

**Severity:** Low

**Impact:** Potentially invalid address passed to API.

**Recommendation:** Validate on input, not on use.

---

### 3.4 WASM/Hydration Issues

#### Issue 12: Large Inline JavaScript

**Location:** `pages/advanced.rs`

**Description:** The ECharts configuration is ~1000 lines of inline JavaScript string.

**Severity:** High

**Impact:** Hard to maintain, error-prone, no syntax highlighting.

**Recommendation:** Extract to separate .js files.

---

#### Issue 13: Direct DOM Manipulation

**Location:** `pages/advanced.rs:1071`

```rust
let _ = js_sys::eval(&js_code_clone);
```

**Description:** Bypasses Leptos reactivity for chart updates.

**Severity:** Medium

**Impact:** Chart state may become out of sync with Leptos state.

**Recommendation:** Use Leptos effects for chart updates.

---

#### Issue 14: Inconsistent Resource Patterns

**Description:** Using `create_local_resource` in some places and `create_resource` in others.

**Severity:** Medium

**Impact:** Inconsistent hydration behavior.

**Recommendation:** Standardize on one pattern.

---

#### Issue 15: Window Global Pollution

**Location:** `pages/advanced.rs:1040`

```rust
window.__usdfc_echarts = chart;
```

**Description:** Pollutes global namespace with chart instance.

**Severity:** Low

**Impact:** Potential conflicts with other code.

**Recommendation:** Use closures or WeakMap for storage.

---

### 3.5 Performance Concerns

#### Issue 16: Large Chart Data Payloads

**Description:** Sending all historical data to client for large lookback periods.

**Severity:** Medium

**Impact:** Slow page loads for long time ranges.

**Recommendation:** Implement server-side pagination or streaming.

---

#### Issue 17: No Pagination in get_troves

**Description:** Fetches up to 500 troves unconditionally.

**Severity:** Medium

**Impact:** Memory usage grows with trove count.

**Recommendation:** Implement cursor-based pagination.

---

#### Issue 18: SVG Chart Recalculation

**Location:** `components/charts.rs`

**Description:** Recalculates all points on every render without memoization.

**Severity:** Low

**Impact:** Minor performance impact for small datasets.

**Recommendation:** Use `create_memo` for point calculations.

---

#### Issue 19: Multiple Concurrent Resources

**Location:** `pages/dashboard.rs`

**Description:** Dashboard creates 5 separate resources that could be combined.

**Severity:** Low

**Impact:** Multiple HTTP requests instead of one.

**Recommendation:** Combine into single endpoint.

---

## 4. CODE QUALITY CONCERNS

### 4.1 Non-Idiomatic Rust Patterns

| Issue | Location | Description |
|-------|----------|-------------|
| Excessive unwrap | Throughout | Using `.unwrap()`, `.expect()`, `.unwrap_or()` extensively |
| String errors | Throughout | Converting errors to strings before propagation |
| Unnecessary clone | Throughout | Cloning RcSignal/RwSignal values on read |
| Missing From impls | error.rs | No From implementations for error conversions |

### 4.2 Code Duplication

| Duplicated Code | Locations | Description |
|-----------------|-----------|-------------|
| Formatting functions | format.rs | `format_volume`, `format_usd_compact`, `format_compact` overlap |
| Chart components | charts.rs vs advanced_chart/ | SVG and ECharts implementations |
| Address validation | Multiple | Same validation logic repeated |

### 4.3 Documentation Gaps

| Missing Documentation | Impact |
|----------------------|--------|
| Module-level docs (//!) | Hard to understand module purpose |
| Function docs | Unknown function behavior |
| No examples | No usage guidance for complex functions |

### 4.4 Naming Inconsistencies

| Inconsistent Pattern | Examples |
|---------------------|----------|
| Chart metric naming | `LendAPR` vs `lend_apr` |
| Resource naming | `lending` vs `LendingMarkets` |
| URL params | `res` vs `resolution` |

---

## 5. IMPROVEMENT OPPORTUNITIES

### 5.1 Type Safety Enhancements

| Enhancement | Description | Priority |
|-------------|-------------|----------|
| MaybeUnavailable<T> wrapper | Replace `Option<f64>` with typed wrapper | High |
| Structured errors | Use `thiserror` for all error types | High |
| From conversions | Implement From for error type conversions | Medium |
| Const generics | Use where applicable for performance | Low |

### 5.2 Performance Optimizations

| Optimization | Description | Priority |
|--------------|-------------|----------|
| Combined resources | Merge multiple resources into single API calls | Medium |
| Cursor pagination | Implement pagination for troves | High |
| Web Workers | Offload heavy data processing | Medium |
| Lazy ECharts | Load ECharts only when needed | Low |

### 5.3 Code Organization

| Organization | Description | Priority |
|--------------|-------------|----------|
| Extract JS | Move inline JS to separate .js files | High |
| Shared types | Create shared types module | Medium |
| Chart config | Factor out common chart configuration | Medium |
| Leptos actions | Use for form submissions | Low |

### 5.4 Error Handling Improvements

| Improvement | Description | Priority |
|-------------|-------------|----------|
| Error boundary retry | Add retry UI in error boundaries | High |
| Context logging | Add context to error logs | Medium |
| User feedback | Surface actionable error messages | High |
| Auto-retry | Implement automatic retry with backoff | Medium |

---

## 6. FRONTEND-SPECIFIC ANALYSIS

### 6.1 Component Architecture

#### Strengths

- Clean component hierarchy with proper parent-child relationships
- Good use of context for global state (AppState, GlobalMetrics)
- Proper use of `create_resource` and `Suspense` for async data
- Type-safe props with Leptos macros

#### Weaknesses

- Large inline JavaScript in Advanced page (~1000 lines)
- Direct DOM manipulation bypassing Leptos reactivity
- Inconsistent resource patterns (local vs global)
- Window globals for chart state management

### 6.2 Reactivity Patterns

| Pattern | Usage | Assessment |
|---------|-------|------------|
| GlobalMetrics context | Shared data across pages | Good pattern |
| Local resources | Page-specific data | Good pattern |
| Derived signals | Computed values | Good pattern |
| Effects | Side effects, chart updates | Needs improvement |

### 6.3 Hydration Considerations

| Issue | Description | Impact |
|-------|-------------|--------|
| create_local_resource | Avoids hydration mismatches | Good |
| Window globals | May cause hydration issues | Risk |
| Direct JS eval | Bypasses Leptos | Risk |

### 6.4 Bundle Size Concerns

| Concern | Impact | Mitigation |
|---------|--------|------------|
| ECharts CDN load | ~700KB minified | Already CDN, consider lazy load |
| Inline JS strings | Increases WASM size | Extract to files |
| SVG components | Multiple components | Tree-shaking |
| No lazy loading | All code loaded upfront | Consider code splitting |

### 6.5 Accessibility Issues

| Issue | Description | Severity |
|-------|-------------|----------|
| ARIA labels | Missing on interactive elements | High |
| Color-only indicators | Status not accessible | Medium |
| Keyboard navigation | Not implemented for chart controls | Medium |
| Focus management | No focus handling | Low |

---

## 7. SUMMARY

### Overall Assessment

The USDFC Analytics Terminal is a well-architected full-stack Rust application using Leptos and Axum. The code demonstrates good understanding of Rust patterns and Leptos reactivity. The application successfully aggregates data from 5 different APIs and presents it through a modern, responsive dashboard.

### Key Strengths

1. **Clean Architecture:** Clear separation between backend API clients, server functions, and frontend components
2. **Good Reactivity:** Proper use of Leptos signals, resources, and context
3. **API Integration:** Successful aggregation from multiple sources (RPC, Blockscout, GeckoTerminal, Subgraph)
4. **Type Safety:** Use of Rust's type system for protocol data structures
5. **Caching:** TTL-based caching for performance optimization
6. **Error Handling:** Circuit breaker pattern for resilience

### Critical Issues Requiring Attention

1. **Error Handling:** Silent error swallowing in advanced chart page
2. **Type Safety:** Ubiquitous `Option<f64>` with inconsistent unwrap patterns
3. **Performance:** Large chart payloads and missing pagination
4. **Maintainability:** Large inline JavaScript strings
5. **Testing:** No visible test coverage

### Recommendations Priority Matrix

| Priority | Items | Effort | Impact |
|----------|-------|--------|--------|
| High | Error boundary improvements, pagination, Type safety | Medium | High |
| Medium | Performance optimizations, error handling structure | Medium | Medium |
| Low | Code organization, accessibility | Low | Low |

### Conclusion

This codebase is production-ready but would benefit from refactoring around error handling, type safety, and code organization. The frontend architecture is sound but would benefit from extracting inline JavaScript and improving accessibility. The backend API aggregation layer is well-designed and demonstrates good Rust practices.

---

## File Reference

| File | Lines | Purpose |
|------|-------|---------|
| main.rs | 395 | Entry point, server setup |
| lib.rs | 64 | Library root, exports |
| app.rs | 193 | Main app component |
| server_fn.rs | 1541 | Server functions |
| rpc.rs | 433 | Filecoin RPC client |
| blockscout.rs | 1252 | Blockscout API client |
| subgraph.rs | 495 | Subgraph client |
| gecko.rs | 415 | GeckoTerminal client |
| config.rs | 217 | Configuration |
| cache.rs | 178 | Caching layer |
| circuit_breaker.rs | 271 | Resilience pattern |
| historical.rs | 392 | SQLite snapshots |
| error.rs | 142 | Error types |
| types.rs | 1247 | Data types |
| format.rs | 384 | Formatting utilities |
| advanced.rs | 2000+ | Advanced chart page |

---

*Report generated by comprehensive manual code analysis*
