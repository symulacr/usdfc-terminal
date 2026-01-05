# USDFC Terminal - Quick Analysis Summary

**Date:** January 4, 2026
**Project:** /home/eya/claude/usdfc-terminal

## Tech Stack
- **Backend:** Rust + Axum + Leptos 0.6 (SSR)
- **Frontend:** Leptos + WebAssembly + ECharts
- **Data:** Filecoin RPC, Blockscout, GeckoTerminal, Secured Finance Subgraph
- **Database:** SQLite (historical snapshots)

## Architecture
```
Client (WASM) ←→ Server Functions ←→ API Clients (RPC/Blockscout/Subgraph/Gecko)
                      ↓
               SQLite Cache
```

## Critical Issues (5)

| # | Issue | Location | Severity |
|---|-------|----------|----------|
| 1 | Silent error swallowing | pages/advanced.rs:283 | High |
| 2 | Unused offset param | server_fn.rs:123 | Medium |
| 3 | Large inline JS (~1000 lines) | pages/advanced.rs | High |
| 4 | Option<f64> everywhere | types.rs | Medium |
| 5 | No pagination in get_troves | server_fn.rs | Medium |

## Code Quality Concerns

- **Duplication:** Formatting functions, chart components
- **Error Handling:** String-based errors, silent unwraps
- **Documentation:** Missing module/function docs
- **Naming:** Inconsistent metric naming (LendAPR vs lend_apr)

## Performance Concerns

1. Large chart data payloads (no pagination)
2. No lazy loading for ECharts
3. Multiple concurrent resources (could be combined)
4. SVG chart recalculation without memoization

## Frontend Analysis

**Strengths:**
- Clean component hierarchy
- Good context usage (GlobalMetrics)
- Proper resource patterns

**Weaknesses:**
- Direct DOM manipulation (js_sys::eval)
- Window globals pollution
- No ARIA labels/accessibility
- Inconsistent resource patterns

## Recommendations (Priority)

**High:**
1. Add error boundary retry UI
2. Implement pagination for troves
3. Extract inline JS to separate files
4. Create MaybeUnavailable<T> wrapper

**Medium:**
1. Combine multiple resources
2. Implement automatic retry with backoff
3. Add context to error logs

## Data Sources

| Source | Module | Reliability |
|--------|--------|-------------|
| Filecoin RPC | rpc.rs | Good |
| Blockscout | blockscout.rs | Good |
| GeckoTerminal | gecko.rs | Rate limited |
| Subgraph | subgraph.rs | Good |

## Server Functions (15 total)

1. GetProtocolMetrics
2. GetRecentTransactions
3. GetTroves
4. GetLendingMarkets
5. GetDailyVolumes
6. GetAddressInfo
7. GetNormalizedAddress
8. GetTopHolders
9. GetStabilityPoolTransfers
10. GetUSDFCPriceData
11. CheckApiHealth
12. GetHolderCount
13. GetOrderBook
14. GetRecentLendingTrades
15. GetAdvancedChartData

## Pages (28 total)

Main: dashboard, advanced, protocol, lending, transactions, entities, analytics, infrastructure, tools

Legacy redirects: supply, collateral, stability, flow, network, sankey, contracts, architecture, api, export, alerts

## File Statistics

| File | Lines |
|------|-------|
| server_fn.rs | 1541 |
| types.rs | 1247 |
| blockscout.rs | 1252 |
| advanced.rs | 2000+ |
| main.rs | 395 |
| app.rs | 193 |

## Summary

Production-ready full-stack Rust app with good architecture. Main improvements needed in:
- Error handling (surfacing errors to users)
- Type safety (reducing Option<f64> proliferation)
- Code organization (extracting inline JS)
- Testing coverage (no visible tests)

Good patterns: Reactivity, context usage, API aggregation
