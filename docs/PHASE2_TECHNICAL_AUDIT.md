# USDFC Terminal - Deep Technical Audit Report (Phase 2)

## Objective
Deep technical analysis of Rust/Leptos codebase focusing on hydration, WASM compilation, function mapping, data formatting, and code quality for production readiness.

---

## 1. HYDRATION & SSR/CSR ANALYSIS

### Critical Hydration Issues

```
CATEGORY: Hydration
FILE: src/lib.rs
LINE: 44-52
FINDING: Hydrate function clears SSR content instead of hydrating
SEVERITY: Error
FIX: Use leptos::mount::hydrate_body() instead of clearing innerHTML
CODE:
  // Current (BROKEN):
  pub fn hydrate() {
      let body = document.body().expect("body element");
      body.set_inner_html("");  // DESTROYS SSR content!
      leptos::mount_to_body(App);
  }

  // Fixed:
  pub fn hydrate() {
      console_error_panic_hook::set_once();
      leptos::mount::hydrate_body(App);
  }
```

```
CATEGORY: Hydration
FILE: src/app.rs
LINE: 19-21
FINDING: Initial page differs between SSR (context) and CSR (URL parse) causing mismatch
SEVERITY: Warning
FIX: Ensure both paths use identical logic or serialize initial state
CODE:
  // SSR provides InitialPage context, CSR parses window.location
  // These can differ if page loads with stale cache
  // Add hydration marker to detect mismatch
```

```
CATEGORY: Hydration
FILE: src/subgraph.rs
LINE: 253-260
FINDING: Different time source between WASM and server
SEVERITY: Warning
FIX: Already correctly handled with #[cfg(target_arch = "wasm32")]
CODE:
  #[cfg(target_arch = "wasm32")]
  let now = (js_sys::Date::now() / 1000.0) as i64;

  #[cfg(not(target_arch = "wasm32"))]
  let now = std::time::SystemTime::now()...
```

```
CATEGORY: Hydration
FILE: src/app.rs
LINE: 105-133
FINDING: get_initial_page() accesses window in WASM only - safe pattern
SEVERITY: Info
FIX: None needed - correctly guarded with #[cfg(target_arch = "wasm32")]
```

---

## 2. WASM BUILD & COMPILATION ERRORS

### Build-Breaking Errors

```
CATEGORY: Build
FILE: src/pages/transactions.rs
LINE: 77
FINDING: Type mismatch - returns Option<View> where View expected
SEVERITY: Error
FIX: Unwrap option or flatten view logic
CODE:
  // Before:
  address_info.get().map(|info_opt| { ... })
  // Returns Option<View> but context expects View

  // After:
  address_info.get().map(|info_opt| { ... }).unwrap_or_else(|| view! { <div></div> }.into_view())
```

```
CATEGORY: Build
FILE: src/pages/alerts.rs
LINE: 45-54
FINDING: alerts.push() receives HtmlElement<Div> but Vec expects View
SEVERITY: Error
FIX: Add .into_view() to view! macro result
CODE:
  // Before:
  alerts.push(view! { <div class="alert-card danger">...</div> });

  // After:
  alerts.push(view! { <div class="alert-card danger">...</div> }.into_view());
```

```
CATEGORY: Build
FILE: Cargo.toml + WASM target
LINE: N/A
FINDING: mio crate doesn't support WASM (from tokio dependency)
SEVERITY: Error
FIX: Disable tokio net feature for WASM builds
CODE:
  # In Cargo.toml, tokio should NOT be included in hydrate/csr features
  # Current config already does this correctly - but ethers pulls it in
  # Solution: Use ethers with default-features = false
```

### Clippy Warnings

```
CATEGORY: Build
FILE: src/pages/stability.rs
LINE: 33
FINDING: Unnecessary parentheses around expression
SEVERITY: Warning
FIX: Remove parentheses
CODE:
  // Before:
  let coverage = if supply_f64 > 0.0 { (pool_f64 / supply_f64 * 100.0) } else { 0.0 };
  // After:
  let coverage = if supply_f64 > 0.0 { pool_f64 / supply_f64 * 100.0 } else { 0.0 };
```

```
CATEGORY: Build
FILE: src/pages/transactions.rs
LINE: 3
FINDING: Unused import AddressInfo
SEVERITY: Warning
FIX: Remove unused import
```

```
CATEGORY: Build
FILE: src/gecko.rs
LINE: 12
FINDING: Unused import std::collections::HashMap
SEVERITY: Warning
FIX: Remove unused import
```

```
CATEGORY: Build
FILE: src/components/sidebar.rs
LINE: 231
FINDING: Unused variable set_active_page
SEVERITY: Warning
FIX: Prefix with underscore or use the variable
```

---

## 3. FUNCTION CONNECTION MAPPING

### Server Function → Page → Component Call Graph

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ SERVER FUNCTIONS (src/server_fn.rs)                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  get_protocol_metrics()                                                     │
│     ├── dashboard.rs (metrics cards, TCR gauge, charts)                    │
│     ├── stability.rs (pool balance, coverage ratio)                        │
│     └── supply.rs (total/circulating supply)                               │
│                                                                             │
│  get_recent_transactions(limit)                                            │
│     ├── dashboard.rs (recent activity table)                               │
│     ├── transactions.rs (full transaction list)                            │
│     ├── supply.rs (mint/burn events)                                       │
│     └── flow.rs (top transfers, flow summary)                              │
│                                                                             │
│  get_troves(limit, offset)                                                 │
│     └── collateral.rs (trove table, health indicators)                     │
│                                                                             │
│  get_icr_distribution()                                                    │
│     └── collateral.rs (ICR distribution chart)                             │
│                                                                             │
│  get_lending_markets()                                                     │
│     └── lending.rs (yield curve table)                                     │
│                                                                             │
│  get_address_info(address)                                                 │
│     ├── transactions.rs (address search results)                           │
│     └── address.rs (redirects to transactions)                             │
│                                                                             │
│  get_top_holders(limit)                                                    │
│     └── entities.rs (holder table)                                         │
│                                                                             │
│  get_stability_pool_transfers(limit)                                       │
│     └── stability.rs (pool transfer table)                                 │
│                                                                             │
│  search_transactions(query) [STUB - just calls get_recent_transactions]    │
│     └── transactions.rs (search functionality)                             │
│                                                                             │
│  get_normalized_address(address)                                           │
│     └── NOT USED (orphan function)                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Orphan/Unused Server Functions

```
CATEGORY: Functions
FILE: src/server_fn.rs
LINE: 331-375
FINDING: get_normalized_address() defined but never called from any page
SEVERITY: Warning
FIX: Either use in address.rs or remove
```

### API Client → Server Function Mapping

```
┌────────────────────────┬─────────────────────────────────────────────────────┐
│ API CLIENT             │ METHODS USED                                        │
├────────────────────────┼─────────────────────────────────────────────────────┤
│ RpcClient              │ get_total_supply, get_total_collateral,             │
│ (src/rpc.rs)           │ get_trove_owners_count, get_tcr, get_fil_price,    │
│                        │ get_stability_pool_balance, get_multiple_sorted_troves │
├────────────────────────┼─────────────────────────────────────────────────────┤
│ BlockscoutClient       │ get_recent_transfers, get_address_usdfc_info,       │
│ (src/blockscout.rs)    │ get_token_holders, get_address_transfers            │
├────────────────────────┼─────────────────────────────────────────────────────┤
│ SubgraphClient         │ get_lending_markets (only)                          │
│ (src/subgraph.rs)      │ get_usdfc_orders, get_recent_transactions UNUSED    │
├────────────────────────┼─────────────────────────────────────────────────────┤
│ GeckoClient            │ NOT USED - all methods orphaned                     │
│ (src/gecko.rs)         │                                                     │
└────────────────────────┴─────────────────────────────────────────────────────┘
```

### Dead Code: GeckoClient

```
CATEGORY: Dead Code
FILE: src/gecko.rs
LINE: 1-398
FINDING: Entire GeckoClient module is never used - no server functions call it
SEVERITY: Warning
FIX: Either integrate into pages or remove
```

---

## 4. NUMBER FORMATTING STANDARDS AUDIT

### Formatting Inconsistencies

```
CATEGORY: Formatting
FILE: Multiple files
FINDING: 6 different format_value() implementations with inconsistent behavior
SEVERITY: Warning
LOCATIONS:
  - dashboard.rs:183-195
  - collateral.rs:181-193
  - stability.rs:152-161
  - supply.rs:129-138
  - entities.rs:79-88
  - flow.rs:135-142

FIX: Use centralized src/format.rs functions
CODE:
  // All files should use:
  use crate::format::format_currency;

  // Instead of local:
  fn format_value(value: Decimal) -> String { ... }
```

### Raw Decimal.to_string() Usage

```
CATEGORY: Formatting
FILE: src/types.rs
LINE: 150-152
FINDING: TroveStatus::from_icr() uses .to_string().parse() instead of Decimal::to_f64()
SEVERITY: Info
FIX: Use rust_decimal's built-in conversion
CODE:
  // Before:
  let icr_f64 = icr.to_string().parse::<f64>().expect("decimal parse");

  // After:
  use rust_decimal::prelude::ToPrimitive;
  let icr_f64 = icr.to_f64().unwrap_or(0.0);
```

### Inconsistent Number Formats

| Location | Format | Expected |
|----------|--------|----------|
| dashboard.rs | `$1.50M` | ✓ Correct |
| stability.rs | `$1.50M` | ✓ Correct |
| entities.rs | `1.50M` (no $) | ✗ Missing currency symbol |
| lending.rs | `12.34%` | ✓ Correct |
| collateral.rs ICR | `150.00%` | ✗ Shows `150.00` without % |

---

## 5. EVENT HANDLER AUDIT

### Missing Debounce on Search

```
CATEGORY: Events
FILE: src/pages/transactions.rs
LINE: 27-50
FINDING: Search button has no debounce - rapid clicks cause multiple fetches
SEVERITY: Warning
FIX: Add loading state disable or debounce
CODE:
  <button
      class="btn btn-primary"
      disabled=move || is_searching.get()  // Add this
      on:click=move |_| { ... }
  >
```

### Missing keyboard support

```
CATEGORY: Events
FILE: src/pages/transactions.rs
LINE: 14-25
FINDING: Search input has no on:keypress for Enter key submit
SEVERITY: Warning
FIX: Add Enter key handler
CODE:
  <input
      on:keypress=move |ev| {
          if ev.key() == "Enter" {
              do_search();
          }
      }
      ...
  />
```

### Missing prevent_default

```
CATEGORY: Events
FILE: src/pages/alerts.rs
LINE: ~100 (form area)
FINDING: Alert form lacks prevent_default on submit
SEVERITY: Info
FIX: No actual form element, so not applicable here
```

---

## 6. DEAD CODE & REDUNDANCY DETECTION

### Unused Imports (from clippy)

```
CATEGORY: Dead Code
FILE: src/pages/transactions.rs:3
FINDING: Unused import AddressInfo
```

```
CATEGORY: Dead Code
FILE: src/gecko.rs:12
FINDING: Unused import std::collections::HashMap
```

### Duplicate Helper Functions

| Function | Files Where Duplicated |
|----------|------------------------|
| `format_value()` | dashboard, collateral, stability, supply, entities, flow |
| `format_amount()` | dashboard, stability, supply, flow |
| `format_timestamp()` | dashboard, transactions, collateral, stability, supply, flow |
| `shorten_hash()` | dashboard, transactions, stability, entities, flow |

**Total: 4 functions × ~6 files = ~24 duplicate implementations**

```
CATEGORY: Redundancy
FILE: Multiple
FINDING: format_timestamp() duplicated 6 times with identical code
SEVERITY: Warning
FIX: Move to src/format.rs and import everywhere
```

### Unused CSS Classes (sample)

```
CATEGORY: Dead Code
FILE: src/styles.css
LINE: 895-919
FINDING: .network-canvas CSS defined but never used (NetworkGraph is a table)
SEVERITY: Info
FIX: Either implement canvas visualization or remove CSS
```

### Orphan Data Types

```
CATEGORY: Dead Code
FILE: src/data.rs
LINE: 1-77
FINDING: DataSource trait defined but never implemented
SEVERITY: Warning
FIX: Either implement trait for API clients or remove
```

---

## 7. SVG & CHART TECHNICAL ISSUES

### AreaChart Issues

```
CATEGORY: SVG
FILE: src/components/charts.rs
LINE: 33-98
FINDING: Fixed viewBox (0 0 400 200) doesn't respond to container resize
SEVERITY: Warning
FIX: Use preserveAspectRatio="xMidYMid meet" and CSS aspect-ratio
```

```
CATEGORY: SVG
FILE: src/components/charts.rs
LINE: 48-58
FINDING: Path generation doesn't handle edge cases (empty data, single point)
SEVERITY: Warning
FIX: Add guards for edge cases
CODE:
  if data.is_empty() {
      return view! { <div class="chart-empty">"No data"</div> };
  }
  if data.len() == 1 {
      // Draw single point instead of path
  }
```

### Gauge Component Issues

```
CATEGORY: SVG
FILE: src/components/gauge.rs
LINE: 56-68
FINDING: Needle rotation has no animation/transition
SEVERITY: Info
FIX: Add CSS transition to needle group
```

```
CATEGORY: SVG
FILE: src/components/gauge.rs
LINE: 40-55
FINDING: Threshold zones (green/yellow/red) hardcoded, not configurable
SEVERITY: Info
FIX: Accept threshold props
```

### Hardcoded Colors (not using CSS variables)

```
CATEGORY: SVG
FILE: src/components/charts.rs
LINE: 72, 85, 92
FINDING: Colors like "#00d4ff" hardcoded instead of using var(--accent-cyan)
SEVERITY: Info
FIX: Use CSS custom properties where possible, or pass as props
```

---

## 8. PAGE WORKFLOW DOCUMENTATION

### Dashboard (dashboard.rs)
```
PURPOSE: Main protocol overview with key metrics
DATA SOURCES: get_protocol_metrics(), get_recent_transactions(10)
USER ACTIONS: Refresh buttons on each card
STATE MANAGEMENT: Two create_resource() for independent data fetching
EDGE CASES:
  - Error: Shows red error message in card
  - Empty: Table shows "No transactions"
  - Loading: "Loading..." text
ISSUES FOUND: Duplicate format functions, no loading skeleton
```

### Transactions (transactions.rs)
```
PURPOSE: Search and browse USDFC transfers
DATA SOURCES: get_recent_transactions(50), get_address_info(query)
USER ACTIONS: Search input, search button, refresh
STATE MANAGEMENT: create_signal for search input, create_resource for results
EDGE CASES:
  - Invalid address: Shows validation error
  - Empty results: "No transactions found"
  - Error: Red error message
ISSUES FOUND: BUILD ERROR at line 77 (Option<View> mismatch)
```

### Collateral (collateral.rs)
```
PURPOSE: Trove health monitoring
DATA SOURCES: get_troves(100, 0), get_icr_distribution()
USER ACTIONS: Refresh button
STATE MANAGEMENT: Two resources for troves and distribution
EDGE CASES: Handled with match Ok/Err
ISSUES FOUND: ICR display missing % suffix
```

### Alerts (alerts.rs)
```
PURPOSE: Protocol alert configuration (TCR thresholds)
DATA SOURCES: get_protocol_metrics() for current values
USER ACTIONS: Toggle switches, threshold inputs
STATE MANAGEMENT: Multiple signals for form state
EDGE CASES: None handled - purely presentational
ISSUES FOUND: BUILD ERROR - view macro returns wrong type
```

### Lending (lending.rs)
```
PURPOSE: Secured Finance yield curve data
DATA SOURCES: get_lending_markets()
USER ACTIONS: Refresh
STATE MANAGEMENT: Single resource
EDGE CASES: Empty markets, error states
ISSUES FOUND: APR calculation depends on system time
```

### Network (network.rs)
```
PURPOSE: "Network Graph" - but actually just a table
DATA SOURCES: Static data only
USER ACTIONS: None
STATE MANAGEMENT: None
EDGE CASES: None
ISSUES FOUND: Name suggests graph but shows table
```

### Sankey (sankey.rs)
```
PURPOSE: "Sankey Charts" - but shows progress bars
DATA SOURCES: Static percentages
USER ACTIONS: None
STATE MANAGEMENT: None
EDGE CASES: None
ISSUES FOUND: Not an actual Sankey diagram
```

---

## 9. DEPRECATED PATTERN DETECTION

### NaiveDateTime::from_timestamp Deprecation

```
CATEGORY: Deprecated
FILE: Multiple (6 files)
LINE: dashboard.rs:197, transactions.rs:193, collateral.rs:197,
      stability.rs:173, supply.rs:150, flow.rs:145
FINDING: NaiveDateTime::from_timestamp_opt is deprecated in chrono 0.4.35+
SEVERITY: Warning
FIX: Use DateTime::from_timestamp
CODE:
  // Before (deprecated):
  chrono::NaiveDateTime::from_timestamp_opt(seconds as i64, 0)
      .expect("timestamp out of range")
  chrono::DateTime::<chrono::Utc>::from_utc(dt, chrono::Utc)

  // After:
  chrono::DateTime::from_timestamp(seconds as i64, 0)
      .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
      .unwrap_or_else(|| "Invalid time".to_string())
```

### .expect() on User Data

```
CATEGORY: Deprecated Pattern
FILE: Multiple
FINDING: 30+ instances of .expect() on user-provided or API data
SEVERITY: Error (can panic in production)
EXAMPLES:
  - collateral.rs:94 - parse().expect("decimal parse")
  - flow.rs:57-59 - expect on sorting comparisons
  - entities.rs:80 - parse().expect()
FIX: Replace all with .unwrap_or_default() or proper error handling
```

### String::from vs .into()

```
CATEGORY: Style
FILE: Multiple
FINDING: Mix of String::from() and .to_string() where .into() would work
SEVERITY: Info
FIX: Consistent style - prefer .to_string() for &str, .into() for generics
```

---

## 10. PRODUCTION READINESS CHECKLIST

### BLOCKERS (Must Fix Before Production)

| # | Issue | File | Severity |
|---|-------|------|----------|
| 1 | Build error: Option<View> mismatch | transactions.rs:77 | **Error** |
| 2 | Build error: HtmlElement vs View | alerts.rs:45 | **Error** |
| 3 | Hydration broken: innerHTML cleared | lib.rs:50 | **Error** |
| 4 | 30+ panic points from .expect() | Multiple | **Error** |
| 5 | Deprecated chrono API usage | 6 files | **Warning** |
| 6 | WASM build fails (mio unsupported) | Cargo.toml | **Error** |

### HIGH PRIORITY (Should Fix)

| # | Issue | File |
|---|-------|------|
| 7 | GeckoClient entirely unused | gecko.rs |
| 8 | DataSource trait never implemented | data.rs |
| 9 | get_normalized_address() orphaned | server_fn.rs |
| 10 | 24 duplicate helper functions | 6 pages |
| 11 | No loading skeletons | components |
| 12 | No pagination in tables | data_table.rs |

### MEDIUM PRIORITY (Polish)

| # | Issue | File |
|---|-------|------|
| 13 | Network page isn't a graph | network.rs |
| 14 | Sankey page isn't Sankey | sankey.rs |
| 15 | Unused CSS classes | styles.css |
| 16 | Search lacks Enter key support | transactions.rs |
| 17 | No debounce on buttons | Multiple |
| 18 | Inconsistent number formats | Multiple |

### SUMMARY STATISTICS

| Category | Count |
|----------|-------|
| **Build Errors** | 3 |
| **Panic Points** | 30+ |
| **Unused Code** | 3 modules |
| **Duplicate Functions** | 24 |
| **Deprecated APIs** | 6 instances |
| **Missing Features** | 12 |

---

## RECOMMENDED FIX ORDER

1. **Day 1**: Fix build errors (transactions.rs, alerts.rs)
2. **Day 2**: Fix hydration (lib.rs), remove all .expect()
3. **Day 3**: Fix deprecated chrono usage
4. **Day 4**: Consolidate duplicate functions into format.rs
5. **Day 5**: Remove dead code (gecko.rs, data.rs, orphan functions)
6. **Week 2**: Add loading states, pagination, keyboard support
7. **Week 3**: Implement actual Network/Sankey visualizations
