# USDFC Terminal - Implementation Plan Review & Validation

## Plan Status: APPROVED with Refinements

The combined Phase 1 + Phase 2 audit findings have been validated. This document provides the complete execution plan with refinements.

---

## Section 1: Build Blockers - VALIDATED

| Fix | Location | Status | Notes |
|-----|----------|--------|-------|
| Option<View> mismatch | transactions.rs:77 | ✓ Correct | Add `.unwrap_or_else(\|\| view! { }.into_view())` |
| Vec<View> type | alerts.rs:45+ | ✓ Correct | Add `.into_view()` to each push |
| Hydration broken | lib.rs:50 | ⚠ Needs revision | See refinement below |
| 30+ .expect() calls | 8 files | ✓ Correct | Replace with safe handling |

**Refinement for lib.rs hydration:**
```rust
// Current (BROKEN):
body.set_inner_html("");
leptos::mount_to_body(App);

// Plan suggests:
leptos::mount::hydrate_body(App);

// CORRECT fix (Leptos 0.6 API):
leptos::mount::hydrate_body(App);
// Note: hydrate_body already handles the body element
```

**Complete .expect() locations from audit:**
```
dashboard.rs:    197, 201
transactions.rs: 193, 201, 210
collateral.rs:   94, 181, 197
stability.rs:    31, 32, 152, 173
supply.rs:       130, 150
lending.rs:      249, 252, 253, 264
flow.rs:         57, 58, 59, 136, 145
entities.rs:     80
types.rs:        151
alerts.rs:       43, 74, 118
```
**Total: 24 instances** (not 30+, but still critical)

---

## Section 2: DataSource Architecture - REFINEMENT NEEDED

**Issue:** The `DataSource` trait in `data.rs` uses `async_trait(?Send)` but the plan doesn't address this.

**Current data.rs signature:**
```rust
#[async_trait(?Send)]
pub trait DataSource: Send + Sync {
    async fn get_protocol_metrics(&self) -> ApiResult<ProtocolMetrics>;
    // ...
}
```

**Refinement:** The trait is already well-defined. Implementation order:

| Client | Trait Methods to Implement | Existing Methods |
|--------|---------------------------|------------------|
| RpcClient | `get_protocol_metrics`, `get_troves`, `get_tcr` | All exist, need wrapper |
| BlockscoutClient | `get_transactions`, `get_address_info` | All exist |
| SubgraphClient | `get_lending_markets` | Exists |
| GeckoClient | `get_price_data` (NEW) | `get_pool_ohlcv` exists |

**Missing from plan:** Need to add `PricePoint` type to `types.rs`:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: u64,
    pub price: f64,
    pub volume: Option<f64>,
}
```

---

## Section 3: GeckoClient Integration - VALIDATED

**Current gecko.rs methods available but unused:**
- `get_token_info()` - token price, supply, market cap
- `get_pool_ohlcv()` - OHLCV candle data
- `get_pool_info()` - pool liquidity, volume
- `get_pool_trades()` - recent trades
- `get_token_pools()` - all pools for a token

**Page integration plan:**

| Page | GeckoClient Method | Data Display |
|------|-------------------|--------------|
| dashboard.rs | `get_pool_ohlcv("hour", 24)` | 24h price sparkline |
| supply.rs | None needed | Already has supply data |
| lending.rs | None - uses SubgraphClient | Already complete |
| NEW: price.rs | All methods | Full price analysis |

**Refinement:** Skip new `price.rs` page initially - integrate into existing pages first.

---

## Section 4: Format Consolidation - VALIDATED

**Current format.rs functions:**
```rust
pub fn format_currency(value: f64) -> String  // exists
pub fn format_number(value: f64) -> String    // exists
pub fn format_percentage(value: f64) -> String // exists
pub fn truncate_address(address: &str) -> String // exists
pub fn truncate_tx_hash(hash: &str) -> String // exists
pub fn format_relative_time(seconds_ago: u64) -> String // exists
```

**Missing functions to add:**
```rust
pub fn format_timestamp(seconds: u64) -> String  // NEW - replace 6 duplicates
pub fn format_amount(value: Decimal) -> String   // NEW - replace 6 duplicates
pub fn decimal_to_f64(value: Decimal) -> f64     // NEW - safe conversion
```

**Files requiring import updates:**
- dashboard.rs (remove lines 183-210)
- transactions.rs (remove lines 192-226)
- collateral.rs (remove lines 181-210)
- stability.rs (remove lines 152-186)
- supply.rs (remove lines 129-163)
- flow.rs (remove lines 135-158)
- entities.rs (remove lines 79-96)
- alerts.rs (remove lines 117-134)

---

## Section 5: Visualizations - NEEDS ARCHITECTURE

**Network Graph Implementation Plan:**

```rust
// src/pages/network.rs - Architecture
#[component]
pub fn NetworkGraph() -> impl IntoView {
    // 1. Fetch transaction data
    let transactions = create_resource(|| (), |_| get_recent_transactions(100));

    // 2. Build adjacency map
    // Map<address, Vec<(connected_addr, tx_count, total_value)>>

    // 3. Render SVG with force-directed positioning
    // Use web_sys Canvas for performance, or SVG for simplicity
}
```

**Sankey Diagram Implementation Plan:**

```rust
// src/pages/sankey.rs - Architecture
#[component]
pub fn SankeyCharts() -> impl IntoView {
    // 1. Aggregate flows by category
    // Sources: Mints, Transfers In
    // Destinations: Burns, Transfers Out, Stability Pool

    // 2. Calculate path widths proportional to value

    // 3. Render curved SVG paths between nodes
}
```

**Refinement:** Both require significant Canvas/SVG work. Suggest Phase 1 as tables with visualization in Phase 2.

---

## Section 6: Monochrome Design - VALIDATED with CLARIFICATION

**Color rules confirmed:**

| Context | Color Allowed | Example |
|---------|--------------|---------|
| UI Chrome | NO | Header, sidebar, buttons, cards |
| Data Values | YES (status) | +5% green, -3% red |
| Charts | YES (full palette) | Bar colors, line colors |
| Status Badges | YES | Success/Failed/Pending |
| Transaction Types | YES | Mint/Burn/Transfer badges |

**CSS variable final mapping:**
```css
:root {
    /* UI - Monochrome only */
    --bg-primary: #000000;
    --bg-secondary: #0a0a0a;
    --bg-tertiary: #111111;
    --text-primary: #ffffff;
    --text-secondary: #888888;
    --text-muted: #555555;
    --border-color: #222222;
    --accent-primary: #ffffff;
    --accent-secondary: #888888;

    /* Data visualization - Colorful */
    --data-positive: #22c55e;
    --data-negative: #ef4444;
    --chart-1: #00d4ff;
    --chart-2: #a855f7;
    --chart-3: #22c55e;
    --chart-4: #f59e0b;
    --chart-5: #ec4899;
}
```

---

## Section 7-9: Number Standards, Plain Language, Components - VALIDATED

No major refinements needed. These are polish tasks for Week 3-4.

---

## Revised Execution Order

| Day | Task | Files | Blocking |
|-----|------|-------|----------|
| 1a | Fix transactions.rs build error | transactions.rs | YES |
| 1b | Fix alerts.rs build error | alerts.rs | YES |
| 1c | Fix lib.rs hydration | lib.rs | YES |
| 1d | Replace all .expect() (24 instances) | 10 files | YES |
| 2 | Run `cargo check --features ssr` | - | VERIFY |
| 3 | Consolidate format functions | format.rs + 8 pages | NO |
| 4 | Update deprecated chrono API | 6 files | NO |
| 5 | CSS monochrome variables | styles.css | NO |
| 6 | CSS border-radius removal | styles.css | NO |
| 7 | Component updates (header, sidebar) | 3 files | NO |
| 8 | Page inline style cleanup | 16 pages | NO |
| 9 | GeckoClient integration | server_fn.rs, dashboard.rs | NO |
| 10 | Network/Sankey visualization | network.rs, sankey.rs | NO |

---

## Day 1 Fixes - Detailed

### Fix 1a: transactions.rs:77

```rust
// BEFORE (line 77):
address_info.get().map(|info_opt| {
    match info_opt {
        // ...
    }
})

// AFTER:
address_info.get().map(|info_opt| {
    match info_opt {
        // ...
    }
}).unwrap_or_else(|| view! { <div></div> }.into_view())
```

### Fix 1b: alerts.rs:45+

```rust
// BEFORE:
alerts.push(view! {
    <div class="alert-card danger">...</div>
});

// AFTER:
alerts.push(view! {
    <div class="alert-card danger">...</div>
}.into_view());
```

Apply `.into_view()` to ALL view! pushes in alerts.rs (lines 45, 56, 78, 87).

### Fix 1c: lib.rs hydration

```rust
// BEFORE:
pub fn hydrate() {
    use wasm_bindgen::JsCast;
    console_error_panic_hook::set_once();
    let document = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let body = document.body().expect("body");
    body.set_inner_html("");  // WRONG!
    leptos::mount_to_body(App);
}

// AFTER:
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

### Fix 1d: All .expect() replacements

Pattern to apply across all 24 instances:

```rust
// Decimal parsing
// BEFORE:
let val: f64 = amount.to_string().parse().expect("decimal parse");
// AFTER:
let val: f64 = amount.to_string().parse().unwrap_or(0.0);

// Timestamp parsing
// BEFORE:
chrono::NaiveDateTime::from_timestamp_opt(seconds as i64, 0)
    .expect("timestamp out of range")
// AFTER:
chrono::DateTime::from_timestamp(seconds as i64, 0)
    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    .unwrap_or_else(|| "Invalid time".to_string())

// Comparison expect
// BEFORE:
b_val.partial_cmp(&a_val).expect("compare")
// AFTER:
b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
```

---

## Format Functions to Add

```rust
// Add to src/format.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Safe decimal to f64 conversion
pub fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

/// Format timestamp from unix seconds
pub fn format_timestamp(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(seconds as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Invalid time".to_string())
}

/// Format token amount with appropriate suffix
pub fn format_amount(value: Decimal) -> String {
    let v = decimal_to_f64(value);
    if v >= 1_000_000.0 {
        format!("{:.2}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.2}K", v / 1_000.0)
    } else {
        format!("{:.2}", v)
    }
}

/// Shorten hash/address for display
pub fn shorten_hash(value: &str) -> String {
    if value.len() > 12 {
        format!("{}...{}", &value[0..6], &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}
```

---

## Plain Language Glossary

| Technical Term | Plain Language | Tooltip Text |
|----------------|----------------|--------------|
| TCR | Total Collateral Ratio | How much backing exists for all USDFC tokens combined |
| ICR | Individual Collateral Ratio | How much backing one user's loan position has |
| Trove | Loan Position | A user's borrowed amount and the collateral securing it |
| Stability Pool | Reserve Fund | Tokens set aside to cover loans that get closed |
| Liquidation | Position Closure | When undercollateralized loans are automatically closed |
| Mint | Create | New USDFC tokens created when borrowing |
| Burn | Remove | USDFC tokens destroyed when repaying |
| Collateral | Backing | FIL assets securing a loan |
| APR | Annual Return | Yearly percentage earned or paid |
| TVL | Total Value | All assets locked in the protocol |

---

## Verification Commands

After all fixes, run:

```bash
# Check SSR build
cargo check --features ssr

# Check WASM/hydrate build
cargo check --target wasm32-unknown-unknown --features hydrate

# Run clippy
cargo clippy --features ssr -- -W clippy::all

# Search for remaining .expect() on user data
grep -rn "\.expect(" src/pages/ src/components/
```

---

## Success Criteria

- [ ] `cargo check --features ssr` passes with 0 errors
- [ ] `cargo check --features hydrate` passes (WASM target)
- [ ] No `.expect()` calls on user/API data
- [ ] No deprecated chrono API usage
- [ ] All format functions consolidated in format.rs
- [ ] Monochrome UI with colored data visualization only
- [ ] Network page shows actual graph
- [ ] Sankey page shows actual flow diagram
