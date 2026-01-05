# Fix Plan: Make Charts Show Curves Over Time

## 🎯 Goal

Make Liquidity, TCR, Supply, Holders, and APR charts show **curves over time** like the Price chart, instead of flat horizontal lines.

## 📊 Current vs Desired Behavior

### Current (Problem)
```
Price chart:    ╱╲╱╲╱╲  ← Curves (good!)
Volume chart:   ▂█▅▃▇▂  ← Bars (good!)
Liquidity:      ______   ← Flat line (bad!)
TCR:            ______   ← Flat line (bad!)
Supply:         ______   ← Flat line (bad!)
Holders:        ______   ← Flat line (bad!)
```

### Desired (Solution)
```
Price chart:    ╱╲╱╲╱╲  ← Curves ✅
Volume chart:   ▂█▅▃▇▂  ← Bars ✅
Liquidity:      ╱‾╲_╱   ← Curves ✅ (goal!)
TCR:            _╱‾╲_   ← Curves ✅ (goal!)
Supply:         ╲_╱‾╲   ← Curves ✅ (goal!)
Holders:        ╱╲‾‾╲   ← Curves ✅ (goal!)
```

## 🔍 Why Charts Are Flat

**Database shows metrics don't change over short periods:**
- TCR: 198.871% (same for 46 minutes straight)
- Supply: 232,964 USDFC (same for 46 minutes straight)
- Holders: 1,082 (same for 46 minutes straight)
- Liquidity: $126K-$127K (< 1% variation)

**Root causes:**
1. ⏱️ **Too short timeframe**: 1 hour is too short to see protocol changes
2. 📊 **Using snapshots**: Snapshot collection started recently (only 46 data points)
3. 🎯 **Protocol stability**: USDFC protocol IS stable (this is actually good!)
4. 📉 **No historical APIs**: Unlike price (GeckoTerminal OHLCV), other metrics have no ready historical data

## ✅ Solution Strategy

### Use Historical Data Instead of Snapshots

| Metric | Current Source | New Source | Benefit |
|--------|----------------|------------|---------|
| **Price** | ✅ GeckoTerminal OHLCV | Keep as-is | Already has curves |
| **Volume** | ✅ GeckoTerminal OHLCV | Keep as-is | Already has curves |
| **Liquidity** | ❌ Snapshots (flat) | GeckoTerminal pool history | Historical data |
| **TCR** | ❌ Snapshots (flat) | Calculate from historical data | Show real trends |
| **Supply** | ❌ Snapshots (flat) | Blockscout GraphQL | Historical on-chain data |
| **Holders** | ❌ Snapshots (flat) | Blockscout GraphQL | Historical on-chain data |
| **Lend APR** | ❌ Snapshots (flat) | Subgraph historical markets | Real lending rate history |
| **Borrow APR** | ❌ Snapshots (flat) | Subgraph historical markets | Real borrow rate history |
| **Transfers** | ❌ Snapshots (flat) | Blockscout GraphQL (already have!) | Count by time period |

## 📋 Implementation Plan

### Strategy 1: Use Longer Default Timeframe (FASTEST - 5 minutes)

**Problem**: User selecting 1h/4h shows flat lines because metrics don't change that fast

**Solution**: Default to 1 week lookback instead of 1 hour

**Changes needed**:
```rust
// src/pages/advanced.rs line 182
let lookback = create_rw_signal(ChartLookback::Week1);  // Already set!
```

**But**: User can still change to 1h via UI controls → still shows flat

**Better solution**: Set **minimum recommended timeframe per metric type**

### Strategy 2: Get Historical Data from APIs (BEST - 2-4 hours)

#### 2A. Liquidity Historical Data

**Current**: Using single snapshot value
**Problem**: No historical liquidity API from GeckoTerminal

**Option 1**: Derive from volume
- High volume periods likely = high liquidity
- Can estimate liquidity changes from volume trends

**Option 2**: Use transactions as proxy
- More tx = more liquidity activity
- GeckoTerminal provides tx counts

**Option 3**: Manual data collection
- Call GeckoTerminal pool info every 5 minutes
- Store in separate table
- Build history over time

**Recommended**: **Option 3** - Start collecting now, will have curves in 24h

#### 2B. TCR Historical Data

**Current**: RPC call fails, falls back to 0.0
**Problem**: Can't get TCR directly

**Solution**: Calculate manually from components
```
TCR = (Total Collateral in USD / Total Debt in USD) × 100

Components:
1. Total Collateral = ActivePool.getETH() × FIL_price
2. Total Debt = USDFC.totalSupply()
3. FIL Price = GeckoTerminal (historical data available!)

Steps:
a. Get historical FIL price from GeckoTerminal ✅ (already have)
b. Get historical USDFC supply from Blockscout
c. Get historical collateral from Blockscout (or assume stable)
d. Calculate: TCR = (collateral × FIL_price) / supply × 100
```

**Files to modify**:
```
src/server_fn.rs:
- Add: calculate_historical_tcr(snapshots, price_candles)
- Returns: Vec<(timestamp, tcr)> with real historical TCR
```

#### 2C. Supply Historical Data

**Current**: Using snapshots (all same value)

**Option 1**: Blockscout GraphQL - Token Transfer Events
```graphql
query SupplyHistory {
  token(contractAddressHash: "0x...USDFC") {
    transfers(first: 1000) {
      edges {
        node {
          timestamp
          # Calculate cumulative supply from mint/burn events
        }
      }
    }
  }
}
```

**Option 2**: Blockscout API - totalSupply at block heights
- Query totalSupply() at different block heights
- Need archive node access (may not be available)

**Option 3**: Calculate from Transfers
- Start from known supply
- Add mints, subtract burns
- Build historical series

**Recommended**: **Option 3** using existing transfer data

#### 2D. Holders Historical Data

**Current**: Using snapshots (1,082 flat)

**Solution**: Blockscout provides historical holder counts!

**API endpoint**:
```
GET https://explorer.filecoin.io/api/v2/addresses/{token_address}/counters
```

**Or GraphQL**:
```graphql
query HolderHistory($address: AddressHash!) {
  address(hash: $address) {
    tokenBalances(first: 1000) {
      edges {
        node {
          timestamp
          # Count unique addresses with balance > 0
        }
      }
    }
  }
}
```

**Better**: Call Blockscout API at regular intervals, store history

**Files to modify**:
```
src/blockscout.rs:
- Add: get_historical_holder_counts(limit: u32)
- Returns: Vec<(timestamp, holder_count)>
```

#### 2E. APR Historical Data

**Current**: Using snapshots (26.98% flat)

**Solution**: Subgraph provides historical market data!

**GraphQL query**:
```graphql
query MarketHistory {
  markets(
    where: { currency: "0x...USDFC" }
    orderBy: timestamp
    orderDirection: desc
    first: 1000
  ) {
    timestamp
    maturity
    lastLendUnitPrice
    lastBorrowUnitPrice
  }
}
```

**Process**:
1. Query all historical market snapshots
2. Convert unitPrice → APR for each timestamp
3. Group by time buckets
4. Return best APR per bucket

**Files to modify**:
```
src/subgraph.rs:
- Add: get_historical_apr_data(lookback_secs: u64)
- Returns: (Vec<(timestamp, lend_apr)>, Vec<(timestamp, borrow_apr)>)
```

#### 2F. Transfers Historical Data

**Current**: We already have this!

Check `src/server_fn.rs` line 1205:
```rust
let transfers_data: Vec<(i64, u64)> = transfers_by_period.unwrap_or_default();
```

**Issue**: Might not be wired up correctly to chart

**Verify**: Check if `get_transfer_counts_by_period()` is actually returning data

## 🚀 Recommended Implementation Order

### Phase 1: Quick Win - TCR Calculation (30 minutes)
Since we already have historical price data, calculating TCR is easy:

```rust
// src/server_fn.rs - add this function
fn calculate_tcr_series(
    price_candles: &[TVCandle],
    supply: f64,  // Current supply (assume stable)
    collateral: f64,  // Current collateral in FIL (assume stable)
) -> Vec<(i64, f64)> {
    price_candles
        .iter()
        .map(|candle| {
            let fil_price = candle.close;
            let collateral_usd = collateral * fil_price;
            let debt_usd = supply; // USDFC ≈ $1
            let tcr = (collateral_usd / debt_usd) * 100.0;
            (candle.time, tcr)
        })
        .collect()
}
```

**Why this works**:
- FIL price changes frequently (we have historical data) ✅
- Supply changes slowly (can use current value)
- Collateral changes slowly (can use current value)
- **Result**: TCR will show curves because FIL price varies!

### Phase 2: Use Existing Transfer Data (15 minutes)
Verify transfers are working:

```rust
// src/server_fn.rs - check line ~1205
let transfers_data = transfers_by_period.unwrap_or_default();
```

If empty, debug `get_transfer_counts_by_period()` function.

### Phase 3: Add Subgraph Historical APRs (1 hour)
Query historical market data from Subgraph and convert to APR series.

### Phase 4: Add Blockscout Historical Data (2 hours)
- Holder count history
- Supply history (if available)

### Phase 5: Liquidity Collection (ongoing)
Start collecting liquidity snapshots, will build curves over 24h.

## 📝 Code Changes Required

### File 1: `src/server_fn.rs`

**Add TCR calculation function**:
```rust
/// Calculate historical TCR from price history
fn calculate_tcr_from_price_history(
    price_candles: &[TVCandle],
    current_supply: f64,
    current_collateral_fil: f64,
) -> Vec<(i64, f64)> {
    price_candles
        .iter()
        .map(|candle| {
            let fil_price_usd = candle.close;
            let total_collateral_usd = current_collateral_fil * fil_price_usd;
            let total_debt_usd = current_supply; // USDFC ≈ $1

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

**Modify `get_advanced_chart_data()` around line 1163**:
```rust
// OLD: Use snapshots (flat)
let tcr_data = ensure_data(
    MetricSnapshot::tcr_series(&snapshots),
    current_tcr
);

// NEW: Calculate from price history (curves!)
let tcr_data = if !price_candles.is_empty() {
    // Get current collateral from ActivePool
    let collateral_fil = rpc.get_active_pool_eth()
        .await
        .ok()
        .and_then(|v| v.to_f64())
        .unwrap_or(0.0);

    calculate_tcr_from_price_history(
        &price_candles,
        current_supply.unwrap_or(0.0),
        collateral_fil
    )
} else {
    // Fallback to snapshots if no price data
    ensure_data(
        MetricSnapshot::tcr_series(&snapshots),
        current_tcr
    )
};
```

### File 2: `src/rpc.rs`

**Add ActivePool collateral getter**:
```rust
/// Get total collateral from ActivePool
pub async fn get_active_pool_eth(&self) -> RpcResult<Decimal> {
    let config = config();
    let active_pool_addr = &config.contract_active_pool;

    // ActivePool.getETH() - method signature: 0x14f89db3
    self.call_with_fallback(
        active_pool_addr,
        "0x14f89db3"
    ).await
}
```

### File 3: `src/subgraph.rs`

**Add historical APR query**:
```rust
/// Get historical lending market data for APR calculation
pub async fn get_market_history(
    &self,
    lookback_secs: u64,
) -> ApiResult<Vec<MarketSnapshot>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let cutoff = now - lookback_secs;

    let query = r#"
    query MarketHistory($cutoff: BigInt!) {
      markets(
        where: {
          currency: "0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"
          timestamp_gte: $cutoff
        }
        orderBy: timestamp
        orderDirection: desc
        first: 1000
      ) {
        timestamp
        maturity
        lastLendUnitPrice
        lastBorrowUnitPrice
        isActive
      }
    }
    "#;

    // Execute query and convert to APR time series
    // ... implementation
}
```

## 🎯 Expected Results

After implementing Phase 1 (TCR calculation):

**Before**:
```
TCR chart: _________ (flat at 198.87%)
```

**After**:
```
TCR chart:  ╱‾╲_╱‾╲  (curves following FIL price!)

Example values over 24h:
- 12:00 PM: FIL $4.50 → TCR 195%
- 3:00 PM:  FIL $4.80 → TCR 208%
- 6:00 PM:  FIL $4.60 → TCR 199%
- 9:00 PM:  FIL $4.75 → TCR 206%
```

**Why this works**: TCR = f(FIL_price), and FIL price varies constantly!

## ⚠️ Important Notes

### Don't Add Fake Variation
❌ **DO NOT** add random noise to make charts look dynamic
❌ **DO NOT** interpolate or smooth stable data artificially
✅ **DO** show real data even if flat (honest representation)

### Longer Timeframes Help
For metrics that truly are stable:
- 1 hour view: Might look flat (normal!)
- 1 day view: Some variation visible
- 1 week view: Clear trends emerge
- 1 month view: Significant changes visible

### Some Flatness is Good
If TCR, Supply, Holders are flat, that means:
- ✅ Protocol is stable and healthy
- ✅ No wild volatility or crisis
- ✅ Predictable behavior for users

## 🎓 Summary

**Root Cause**: Charts use snapshot data which hasn't accumulated enough history yet (only 46 minutes)

**Quick Fix**: Calculate TCR from existing price data → instant curves!

**Medium Fix**: Query historical data from Blockscout/Subgraph → all metrics show curves

**Long Fix**: Continue collecting snapshots → will show curves after 24-48 hours

**Recommended**: Do Quick Fix + Medium Fix for immediate results

## ✅ Next Steps

1. Implement TCR calculation from price history
2. Test with 1 day and 1 week lookbacks
3. Add Subgraph historical APR data
4. Verify Transfers data is displaying
5. Consider longer default timeframes for protocol metrics

---

**Status**: Plan ready for implementation
**Estimated effort**: 2-4 hours total
**Expected result**: All charts show curves over time like Price chart
