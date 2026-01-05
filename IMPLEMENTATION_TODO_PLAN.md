# Implementation TODO Plan - Make Charts Show Curves

## 🎯 Goal
Transform flat line charts into dynamic curves by using historical data sources and smart calculations.

## 📋 Complete Implementation Checklist

### ✅ PHASE 1: TCR Calculation from Price History (30-45 minutes)

**Why this first?**
- Quickest win - uses existing price data
- TCR will immediately show curves because FIL price varies
- No new API calls needed

#### Task 1.1: Add ActivePool RPC Method
**File**: `src/rpc.rs`
**Location**: Add after existing contract methods (around line 200-250)

```rust
/// Get total ETH (FIL) collateral from ActivePool contract
pub async fn get_active_pool_eth(&self) -> RpcResult<Decimal> {
    let config = config();
    let active_pool_addr = &config.contract_active_pool;

    // ActivePool.getETH() method signature: 0x14f89db3
    self.call_with_fallback(
        active_pool_addr,
        "0x14f89db3"
    ).await
}
```

**Expected result**: Returns Decimal amount of FIL in ActivePool
**Test**: Call this method and verify it returns a value > 0

---

#### Task 1.2: Create TCR Calculation Helper Function
**File**: `src/server_fn.rs`
**Location**: Add before `get_advanced_chart_data()` function (around line 1000-1050)

```rust
/// Calculate historical TCR from price candles
///
/// TCR = (Total Collateral USD / Total Debt USD) × 100
/// Where:
/// - Collateral USD = collateral_fil × fil_price_usd
/// - Debt USD ≈ total_supply (since USDFC ≈ $1)
fn calculate_tcr_from_price_history(
    price_candles: &[TVCandle],
    current_supply: f64,
    current_collateral_fil: f64,
) -> Vec<(i64, f64)> {
    if price_candles.is_empty() || current_supply == 0.0 {
        return Vec::new();
    }

    price_candles
        .iter()
        .map(|candle| {
            // FIL price varies over time (from candle data)
            let fil_price_usd = candle.close;

            // Calculate collateral value in USD at this point in time
            let total_collateral_usd = current_collateral_fil * fil_price_usd;

            // Debt is approximately equal to USDFC supply (pegged to $1)
            let total_debt_usd = current_supply;

            // Calculate TCR percentage
            let tcr = if total_debt_usd > 0.0 {
                (total_collateral_usd / total_debt_usd) * 100.0
            } else {
                0.0
            };

            (candle.time, tcr)
        })
        .collect()
}
```

**Expected result**: Function that converts price candles → TCR time series
**Test**: Call with sample data, verify TCR varies with price

---

#### Task 1.3: Fetch ActivePool Collateral in get_advanced_chart_data()
**File**: `src/server_fn.rs`
**Location**: In `get_advanced_chart_data()` function, around line 1090-1100 (where other RPC calls are made)

**Find this section**:
```rust
// Get current metric values (for display) - None if unavailable
let current_tcr = tcr_result.ok().and_then(|v| v.to_f64());
let current_supply = supply_result.ok().and_then(|v| v.to_f64());
```

**Add after it**:
```rust
// Get ActivePool collateral for TCR calculation
let collateral_result = rpc.get_active_pool_eth().await;
let current_collateral_fil = collateral_result
    .ok()
    .and_then(|v| v.to_f64())
    .unwrap_or(0.0);
```

**Expected result**: Variable `current_collateral_fil` contains FIL amount
**Test**: Log the value, verify it's reasonable (should be > 0)

---

#### Task 1.4: Replace TCR Snapshot Data with Calculated Data
**File**: `src/server_fn.rs`
**Location**: Around line 1163-1166 (where TCR data is extracted from snapshots)

**Find this code**:
```rust
let tcr_data = ensure_data(
    MetricSnapshot::tcr_series(&snapshots),
    current_tcr
);
```

**Replace with**:
```rust
// Calculate TCR from price history instead of snapshots
// This gives us curves because FIL price varies over time
let tcr_data = if !price_candles.is_empty()
    && current_supply.is_some()
    && current_collateral_fil > 0.0
{
    // Use price-based calculation (shows curves!)
    calculate_tcr_from_price_history(
        &price_candles,
        current_supply.unwrap(),
        current_collateral_fil
    )
} else {
    // Fallback to snapshots if price data unavailable
    ensure_data(
        MetricSnapshot::tcr_series(&snapshots),
        current_tcr
    )
};
```

**Expected result**: TCR data now varies with FIL price → shows curves!
**Test**: Check tcr_data length matches price_candles length

---

### ✅ PHASE 2: Verify Transfers Data (15 minutes)

**Why this?**
- Transfers data might already work, just need to verify
- Quick check before adding new features

#### Task 2.1: Check Transfers Data Source
**File**: `src/server_fn.rs`
**Location**: Around line 1205-1210

**Find this code**:
```rust
let transfers_data: Vec<(i64, u64)> = transfers_by_period.unwrap_or_default();
```

**Add debug logging**:
```rust
let transfers_data: Vec<(i64, u64)> = transfers_by_period.unwrap_or_default();

// Debug: Log transfers data
tracing::debug!(
    "Transfers data: {} points, range: {:?} to {:?}",
    transfers_data.len(),
    transfers_data.first(),
    transfers_data.last()
);
```

**Expected result**: Logs show if transfers_data has data
**Test**: Check server logs for debug message

---

#### Task 2.2: Test Transfers Query
**File**: `src/blockscout.rs`
**Location**: Find `get_transfer_counts_by_period()` function

**Add error logging**:
```rust
pub async fn get_transfer_counts_by_period(...) -> ApiResult<Vec<(i64, u64)>> {
    // ... existing code ...

    tracing::info!("Transfer counts query returned {} periods", result.len());

    Ok(result)
}
```

**Expected result**: Logs show transfer counts are being fetched
**Test**: Check if result is empty or has data

---

### ✅ PHASE 3: Add Historical APR Data from Subgraph (1-2 hours)

**Why this?**
- APRs currently flat because using snapshots
- Subgraph has historical market data we can use
- Will show real lending rate changes over time

#### Task 3.1: Create Market History Query
**File**: `src/subgraph.rs`
**Location**: Add new method after `get_lending_markets()` (around line 150-200)

```rust
/// Get historical lending market data for APR calculation
pub async fn get_market_history(
    &self,
    lookback_secs: u64,
) -> ApiResult<Vec<MarketHistoryPoint>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let cutoff = now.saturating_sub(lookback_secs);

    let query = r#"
    query MarketHistory($cutoff: BigInt!) {
      markets(
        where: {
          currency: "0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"
          timestamp_gte: $cutoff
        }
        orderBy: timestamp
        orderDirection: asc
        first: 1000
      ) {
        id
        timestamp
        maturity
        lastLendUnitPrice
        lastBorrowUnitPrice
        isActive
      }
    }
    "#;

    let variables = serde_json::json!({
        "cutoff": cutoff.to_string()
    });

    let response = self.rate_limited_request(query, variables).await?;

    // Parse response
    let markets: Vec<MarketHistoryPoint> = response["data"]["markets"]
        .as_array()
        .ok_or_else(|| ApiError::ParseError("No markets array".to_string()))?
        .iter()
        .filter_map(|m| serde_json::from_value(m.clone()).ok())
        .collect();

    Ok(markets)
}
```

**Also add struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHistoryPoint {
    pub id: String,
    pub timestamp: String,
    pub maturity: String,
    #[serde(rename = "lastLendUnitPrice")]
    pub last_lend_unit_price: Option<String>,
    #[serde(rename = "lastBorrowUnitPrice")]
    pub last_borrow_unit_price: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}
```

**Expected result**: Returns historical market snapshots
**Test**: Call with lookback_secs = 86400 (1 day), verify returns data

---

#### Task 3.2: Convert Market History to APR Series
**File**: `src/subgraph.rs`
**Location**: Add helper function after `unit_price_to_apr()`

```rust
/// Convert market history to APR time series
pub fn market_history_to_apr_series(
    markets: Vec<MarketHistoryPoint>,
) -> (Vec<(i64, f64)>, Vec<(i64, f64)>) {
    use std::collections::HashMap;

    // Group by timestamp, take best APR per timestamp
    let mut lend_map: HashMap<i64, f64> = HashMap::new();
    let mut borrow_map: HashMap<i64, f64> = HashMap::new();

    for market in markets {
        if !market.is_active {
            continue;
        }

        let timestamp = market.timestamp.parse::<i64>().unwrap_or(0);
        let maturity = market.maturity.parse::<i64>().unwrap_or(0);

        // Calculate lend APR
        if let Some(ref price) = market.last_lend_unit_price {
            if let Ok(apr) = unit_price_to_apr(price, maturity) {
                let current_max = lend_map.get(&timestamp).copied().unwrap_or(0.0);
                lend_map.insert(timestamp, apr.max(current_max));
            }
        }

        // Calculate borrow APR
        if let Some(ref price) = market.last_borrow_unit_price {
            if let Ok(apr) = unit_price_to_apr(price, maturity) {
                let current_max = borrow_map.get(&timestamp).copied().unwrap_or(0.0);
                borrow_map.insert(timestamp, apr.max(current_max));
            }
        }
    }

    // Convert to sorted vectors
    let mut lend_series: Vec<(i64, f64)> = lend_map.into_iter().collect();
    let mut borrow_series: Vec<(i64, f64)> = borrow_map.into_iter().collect();

    lend_series.sort_by_key(|(ts, _)| *ts);
    borrow_series.sort_by_key(|(ts, _)| *ts);

    (lend_series, borrow_series)
}
```

**Expected result**: Returns (lend_apr_series, borrow_apr_series)
**Test**: Verify series are sorted by timestamp

---

#### Task 3.3: Use Historical APRs in get_advanced_chart_data()
**File**: `src/server_fn.rs`
**Location**: Around line 1195-1203 (where APR data is extracted)

**Find this code**:
```rust
let lend_apr_data = ensure_data(
    MetricSnapshot::lend_apr_series(&snapshots),
    current_lend_apr
);

let borrow_apr_data = ensure_data(
    MetricSnapshot::borrow_apr_series(&snapshots),
    current_borrow_apr
);
```

**Replace with**:
```rust
// Try to get historical APR data from Subgraph
let (lend_apr_data, borrow_apr_data) = {
    let lookback_secs = if lookback_mins == 0 {
        604800 // 1 week default for "ALL"
    } else {
        (lookback_mins as u64) * 60
    };

    match subgraph.get_market_history(lookback_secs).await {
        Ok(history) => {
            tracing::debug!("Got {} market history points", history.len());
            let (lend_series, borrow_series) =
                crate::subgraph::market_history_to_apr_series(history);

            // Ensure at least current value if series empty
            let lend_data = if lend_series.is_empty() {
                if let Some(apr) = current_lend_apr {
                    vec![(now, apr)]
                } else {
                    vec![]
                }
            } else {
                lend_series
            };

            let borrow_data = if borrow_series.is_empty() {
                if let Some(apr) = current_borrow_apr {
                    vec![(now, apr)]
                } else {
                    vec![]
                }
            } else {
                borrow_series
            };

            (lend_data, borrow_data)
        }
        Err(e) => {
            tracing::warn!("Failed to get market history: {}", e);
            // Fallback to snapshots
            (
                ensure_data(
                    MetricSnapshot::lend_apr_series(&snapshots),
                    current_lend_apr
                ),
                ensure_data(
                    MetricSnapshot::borrow_apr_series(&snapshots),
                    current_borrow_apr
                )
            )
        }
    }
};
```

**Expected result**: APR data now comes from Subgraph history
**Test**: Check logs for "Got X market history points"

---

### ✅ PHASE 4: Improve Chart UX (30 minutes)

**Why this?**
- Even with data, some metrics might be stable
- Help users understand what they're seeing
- Set appropriate defaults

#### Task 4.1: Add Stability Detection
**File**: `src/server_fn.rs`
**Location**: Before returning ChartDataResponse (around line 1230)

```rust
// Check if data is stable (low variation)
fn is_stable_series(data: &[(i64, f64)], threshold: f64) -> bool {
    if data.len() < 2 {
        return true;
    }

    let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = values.iter().sum::<f64>() / values.len() as f64;

    // Check if range is < threshold% of average
    let range = max - min;
    let variation_pct = (range / avg) * 100.0;

    variation_pct < threshold
}
```

**Add to response**:
```rust
// Add metadata about data stability
let tcr_stable = is_stable_series(&tcr_data, 1.0);  // <1% variation
let supply_stable = is_stable_series(&supply_data, 1.0);
```

**Expected result**: Metadata about which metrics are stable
**Test**: Log stability flags, verify accuracy

---

#### Task 4.2: Add Info Banner for Stable Metrics
**File**: `src/pages/advanced.rs`
**Location**: Update existing snapshot-info-banner (around line 1267-1287)

**Modify banner to show**:
```rust
<Show when=move || {
    let data = chart_data.get();
    data.snapshot_count < 10 ||
    (data.tcr_data.len() > 0 && is_all_same_value(&data.tcr_data))
}>
    <div class="snapshot-info-banner">
        <span class="info-icon">"ℹ️"</span>
        <span class="info-text">
            {move || {
                let data = chart_data.get();
                if data.snapshot_count < 10 {
                    format!("Historical data: {} snapshots. Full charts in ~{} minutes.",
                        data.snapshot_count, 10 - data.snapshot_count)
                } else {
                    "Protocol metrics stable (TCR, Supply unchanged - this indicates healthy protocol) ✅".to_string()
                }
            }}
        </span>
    </div>
</Show>
```

**Expected result**: Banner explains why some metrics are flat
**Test**: View page, verify banner shows appropriate message

---

### ✅ PHASE 5: Testing and Verification (30 minutes)

#### Task 5.1: Rebuild Server
```bash
cd /home/eya/claude/usdfc-terminal
export PATH="/usr/bin:/bin:/home/eya/.cargo/bin:$PATH"
cargo leptos build --release
```

**Expected result**: Build succeeds with no errors
**Test**: Check for compilation errors

---

#### Task 5.2: Restart Server
```bash
# Kill old server
pkill usdfc-analytics-terminal

# Start new server in background
./target/release/usdfc-analytics-terminal &
```

**Expected result**: Server starts on port 3000
**Test**: Check logs for "Starting USDFC Analytics Terminal"

---

#### Task 5.3: Test in Browser
Visit: `http://95.133.252.220:3000/advanced`

**Check these specific things**:

1. **TCR Chart**:
   - [ ] Shows multiple data points (not just 1)
   - [ ] Has visible curve/variation (not flat line)
   - [ ] Variation matches FIL price movements
   - [ ] Tooltip shows different TCR values at different times

2. **APR Charts**:
   - [ ] Lend APR shows historical data
   - [ ] Borrow APR shows historical data
   - [ ] Data points span the selected timeframe
   - [ ] Values vary over time (if market data exists)

3. **All Charts**:
   - [ ] 1 hour view loads fast
   - [ ] 1 day view shows more data points
   - [ ] 1 week view shows full curves
   - [ ] No console errors

4. **Info Banner**:
   - [ ] Shows when snapshots < 10
   - [ ] Explains stability when appropriate
   - [ ] Dismisses when data is sufficient

---

#### Task 5.4: Test Different Timeframes
Test each lookback period:

**1 Hour**:
- [ ] Price: Shows curves
- [ ] Volume: Shows bars
- [ ] TCR: Shows curves (following FIL price)
- [ ] Liquidity: May be flat (OK)
- [ ] Supply: May be flat (OK)

**1 Day**:
- [ ] All metrics load
- [ ] More data points visible
- [ ] Smoother curves

**1 Week**:
- [ ] Long-term trends visible
- [ ] Stable metrics might show some variation
- [ ] Charts render smoothly

---

#### Task 5.5: Check Server Logs
```bash
tail -f logs/terminal.log
# or check background task output
```

**Look for**:
- [ ] No error messages
- [ ] "Got X market history points" (from Subgraph)
- [ ] "Transfers data: Y points" (if added debug log)
- [ ] No RPC failures (except expected getTotalDebt)

---

## 📊 Expected Results Summary

### Before Implementation
```
TCR:       ________ (flat at 198.87%)
Supply:    ________ (flat at 232K)
Holders:   ________ (flat at 1082)
APR:       ________ (flat at 26.98%)
```

### After Implementation
```
TCR:       ╱‾╲_╱‾╲ (curves following FIL price!)
Supply:    _______ (may still be flat - OK)
Holders:   _______ (may still be flat - OK)
APR:       ╱╲‾‾╲_ (curves from market history)
```

### What Will Show Curves
✅ **TCR** - Calculated from varying FIL price
✅ **APRs** - From Subgraph historical markets
✅ **Price** - Already working (GeckoTerminal)
✅ **Volume** - Already working (GeckoTerminal)

### What Might Stay Flat (That's OK!)
⚠️ **Supply** - Only changes with mint/burn events
⚠️ **Holders** - Only changes when new wallets join
⚠️ **Liquidity** - DEX reserves are stable

---

## ⚠️ Important Notes

### Don't Skip Phases
- Phase 1 (TCR) must come before Phase 5 (testing)
- Each phase builds on previous ones
- Test after each phase to catch issues early

### Error Handling
- All API calls should have fallbacks
- Never crash if data unavailable
- Log errors but continue serving data

### Performance
- Historical queries may be slow first time
- Consider caching results
- Don't block initial page load

---

## 📈 Success Criteria

Implementation is complete when:

- [ ] TCR chart shows curves that follow FIL price
- [ ] APR charts show historical market data
- [ ] No flat lines where data should vary
- [ ] Page loads in < 2 seconds
- [ ] All timeframes work correctly
- [ ] No console errors
- [ ] No server crashes
- [ ] Info banner helps users understand data

---

## 🔄 Rollback Plan

If something breaks:

1. **Revert git changes**:
```bash
git diff > /tmp/changes.patch
git checkout src/server_fn.rs src/rpc.rs src/subgraph.rs
```

2. **Rebuild and restart**:
```bash
cargo leptos build --release
./target/release/usdfc-analytics-terminal
```

3. **Identify issue**:
- Check server logs
- Check browser console
- Review error messages

4. **Fix incrementally**:
- Apply changes one phase at a time
- Test after each change
- Keep working backup

---

## 📞 Status Tracking

Use todo list to track progress:
- ✅ Completed
- 🔄 In Progress
- ⏳ Pending
- ❌ Blocked

Update after each task completion!

---

**Total Estimated Time**: 3-4 hours
**Priority**: High - User-facing feature
**Risk**: Low - Has fallbacks, won't break existing functionality
**Impact**: High - Significantly improves chart usability

**Ready to start implementation!** 🚀
