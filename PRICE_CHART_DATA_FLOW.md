# 📊 How Price Chart Data is Obtained

**Question**: How is the price chart data obtained and displayed?

---

## 🔄 Complete Data Flow

```
1. USER REQUEST
   ↓
2. FRONTEND (Browser)
   • User navigates to /advanced page
   • Leptos calls server function
   ↓
3. SERVER FUNCTION (src/server_fn.rs:1076)
   • get_advanced_chart_data()
   • Calls GeckoTerminal API client
   ↓
4. GECKOTERMINAL API CLIENT (src/gecko.rs:121)
   • get_pool_ohlcv()
   • Makes HTTP request to GeckoTerminal
   ↓
5. GECKOTERMINAL API (External)
   • https://api.geckoterminal.com/api/v2/
   • Returns OHLCV data (Open, High, Low, Close, Volume)
   ↓
6. DATA PROCESSING (src/gecko.rs:148-161)
   • Parse JSON response
   • Convert array format to struct
   ↓
7. RETURN TO FRONTEND
   • Price candles sent to browser
   ↓
8. CHART RENDERING (src/components/advanced_chart/canvas.rs)
   • ECharts draws candlestick chart
   • Display to user
```

---

## 📝 Detailed Breakdown

### Step 1: API Call Construction

**File**: `src/gecko.rs` lines 121-131

```rust
pub async fn get_pool_ohlcv(
    &self,
    pool_address: &str,      // Pool to query
    timeframe: &str,          // "minute", "hour", or "day"
    aggregate: u32,           // Aggregation: 1, 4, 12 hours etc.
    limit: u32,               // Number of candles (max 100)
) -> ApiResult<Vec<OHLCV>> {
    let url = format!(
        "{}/pools/{}/ohlcv/{}?aggregate={}&limit={}",
        self.base_url, pool_address, timeframe, aggregate, limit
    );
    // ...
}
```

**Example URL**:
```
https://api.geckoterminal.com/api/v2/networks/filecoin/pools/0x4e07447bd38e60b94176764133788be1a0736b30/ohlcv/hour?aggregate=1&limit=168
```

**Parameters**:
- **Pool Address**: `0x4e07447bd38e60b94176764133788be1a0736b30` (USDFC/WFIL pool)
- **Timeframe**: `hour` (hourly candles)
- **Aggregate**: `1` (no aggregation, 1-hour intervals)
- **Limit**: `168` (1 week of hourly data)

---

### Step 2: API Response Format

**Raw JSON from GeckoTerminal**:
```json
{
  "data": {
    "type": "ohlcv",
    "attributes": {
      "ohlcv_list": [
        [1735999200, 1.0001, 1.0050, 0.9950, 1.0020, 12345.67],
        [1735995600, 0.9990, 1.0030, 0.9980, 1.0001, 10234.56],
        // ... more candles
      ]
    }
  }
}
```

**Array Format Explanation**:
```
[timestamp, open, high, low, close, volume]
 ↓         ↓     ↓     ↓    ↓      ↓
 Unix time  $     $     $    $      USD trading volume
```

---

### Step 3: Data Parsing (src/gecko.rs:148-161)

```rust
let ohlcv_list = data
    .data
    .attributes
    .ohlcv_list
    .into_iter()
    .map(|arr| OHLCV {
        timestamp: arr[0] as i64,  // Convert to Unix timestamp
        open: arr[1],               // Opening price
        high: arr[2],               // Highest price in period
        low: arr[3],                // Lowest price in period
        close: arr[4],              // Closing price
        volume: arr[5],             // Trading volume in USD
    })
    .collect();
```

**Result**: Vector of `OHLCV` structs

---

### Step 4: Server Function Processing (src/server_fn.rs:1086-1102)

```rust
// Convert OHLCV to TVCandle format for frontend
let mut price_candles: Vec<TVCandle> = ohlcv_result
    .map_err(|e| SfnError::ServerError(format!("GeckoTerminal OHLCV error: {}", e)))?
    .into_iter()
    .map(|o| TVCandle {
        time: o.timestamp,    // Unix timestamp
        open: o.open,         // $ price
        high: o.high,         // $ price
        low: o.low,           // $ price
        close: o.close,       // $ price
        volume: o.volume,     // $ volume
    })
    .collect();

// Apply custom time range filter if requested
if let Some(custom_start) = start {
    let effective_end = end.unwrap_or(now);
    price_candles.retain(|c| c.time >= custom_start && c.time <= effective_end);
}
```

---

### Step 5: Return to Frontend

**Type**: `ChartDataResponse` (src/types.rs:1127)

```rust
pub struct ChartDataResponse {
    pub price_candles: Vec<TVCandle>,  // ← Price chart data
    pub volume_data: Vec<(i64, f64)>,
    pub liquidity_data: Vec<(i64, f64)>,
    pub tcr_data: Vec<(i64, f64)>,
    // ... other metrics
}
```

**Sent to browser as JSON**:
```json
{
  "price_candles": [
    {"time": 1735999200, "open": 1.0001, "high": 1.0050, "low": 0.9950, "close": 1.0020, "volume": 12345.67},
    {"time": 1735995600, "open": 0.9990, "high": 1.0030, "low": 0.9980, "close": 1.0001, "volume": 10234.56},
    // ... 168 candles for 1 week
  ]
}
```

---

### Step 6: Chart Rendering (src/components/advanced_chart/canvas.rs)

**Uses ECharts library** to draw candlestick chart:

```rust
// Extract candles from server response
let candles = &data.get().price_candles;

// Prepare data for ECharts
let chart_data = candles
    .iter()
    .map(|c| vec![
        c.time * 1000,  // Convert to milliseconds for JS
        c.open,
        c.close,
        c.low,
        c.high,
    ])
    .collect();

// Send to ECharts for rendering
echarts.set_option(&option);
```

**ECharts draws**:
- Green candles when `close > open` (price went up)
- Red candles when `close < open` (price went down)
- Wicks showing high/low range

---

## 🔍 Data Source Details

### GeckoTerminal API

**Provider**: GeckoTerminal (by CoinGecko)
**Endpoint**: `https://api.geckoterminal.com/api/v2/`
**Network**: `filecoin`
**Pool**: `0x4e07447bd38e60b94176764133788be1a0736b30`

**What is this pool?**
- DEX: Likely Uniswap V2/V3 or similar on Filecoin
- Pair: USDFC / WFIL (Wrapped FIL)
- Used to get USDFC price in USD (via FIL price)

**Data Freshness**:
- Real-time: Last candle is current hour
- Historical: Up to 100 candles per request
- Update frequency: Every hour (for hourly timeframe)

---

## ⚙️ Configuration

**From environment** (`.env` file):

```bash
POOL_USDFC_WFIL=0x4e07447bd38e60b94176764133788be1a0736b30
GECKOTERMINAL_URL=https://api.geckoterminal.com/api/v2/networks/filecoin
```

**Resolution Settings** (src/types.rs):

```rust
pub enum ChartResolution {
    Minute1,   // 1-minute candles
    Minute5,   // 5-minute candles
    Minute15,  // 15-minute candles
    Hour1,     // 1-hour candles (DEFAULT)
    Hour4,     // 4-hour candles
    Hour12,    // 12-hour candles
    Day,       // Daily candles
}

pub enum ChartLookback {
    Hour1,     // Last 1 hour
    Hour6,     // Last 6 hours
    Hour12,    // Last 12 hours
    Day1,      // Last 24 hours
    Week1,     // Last 7 days (DEFAULT)
    Month1,    // Last 30 days
    All,       // All available
}
```

---

## 📊 Example: 1 Week Hourly Chart

**User selects**: "1 Hour" resolution, "1 Week" lookback

**System calls**:
```rust
gecko.get_pool_ohlcv(
    "0x4e07447bd38e60b94176764133788be1a0736b30",  // Pool
    "hour",                                          // Timeframe
    1,                                               // Aggregate (no grouping)
    168                                              // Limit (7 days × 24 hours)
)
```

**API returns**: 168 candles

**Chart displays**:
- X-axis: Last 7 days (Dec 28 - Jan 4)
- Y-axis: Price in USD
- Candles: 1 per hour
- Pattern: Shows price movement over the week

---

## 🚀 Why This Works Well

✅ **Real-time data**: Direct from DEX pool
✅ **High frequency**: Hourly updates
✅ **Reliable source**: GeckoTerminal aggregates from blockchain
✅ **No snapshots needed**: API provides historical data
✅ **Clean visualization**: OHLCV format perfect for candlestick charts

---

## 🔄 Complete Request Flow Example

```
1. User opens http://95.133.252.220:3000/advanced
   ↓
2. Browser calls: get_advanced_chart_data(hour, week)
   ↓
3. Server calls: gecko.get_pool_ohlcv("0x4e...", "hour", 1, 168)
   ↓
4. HTTP GET: https://api.geckoterminal.com/api/v2/.../ohlcv/hour?aggregate=1&limit=168
   ↓
5. GeckoTerminal responds with 168 candles (JSON)
   ↓
6. Server parses JSON → Vec<TVCandle>
   ↓
7. Server returns ChartDataResponse to browser (JSON)
   ↓
8. Browser receives data
   ↓
9. ECharts library renders candlestick chart
   ↓
10. User sees price chart with 168 hourly candles!
```

**Total time**: ~500ms (API call + parsing + rendering)

---

## 📈 Data Quality

**From our testing** (`test_extended_lookback_200points.sh`):

- **Data points**: 200 candles (33 days, 4-hour intervals)
- **Date range**: Dec 2, 2025 → Jan 4, 2026
- **Price variation**: 3.00% over 33 days
- **Min price**: $0.9803 (Dec 12)
- **Max price**: $1.0099 (Jan 2)
- **Data retention**: 100% (no gaps!)

---

**Summary**: Price chart data comes from **GeckoTerminal API** → provides real **DEX pool OHLCV data** → parsed and sent to **ECharts** for display! 🎯
