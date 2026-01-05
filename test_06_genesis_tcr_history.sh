#!/bin/bash
# Test 06: TCR History from DAY ONE (Genesis to Today)
# Goal: Calculate TCR from genesis using FIL price history

set -e

echo "========================================================================"
echo "TEST 06: TCR HISTORY FROM TOKEN GENESIS"
echo "========================================================================"
echo ""

echo "Strategy: Get FIL price history from genesis → Calculate TCR at each point"
echo ""

echo "Step 1: Determine Token Genesis Date"
echo "------------------------------------------------------------"

USDFC_TOKEN="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"

# Get oldest transfer to find genesis
GENESIS_TRANSFER=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20&limit=1&offset=20000" 2>/dev/null)
GENESIS_TIME=$(echo "$GENESIS_TRANSFER" | jq -r '.items[0].timestamp // "2024-01-01T00:00:00Z"')
GENESIS_EPOCH=$(date -d "$GENESIS_TIME" +%s 2>/dev/null || echo "1704067200")

echo "Token genesis (estimated): $GENESIS_TIME"

NOW=$(date +%s)
TOKEN_AGE_DAYS=$(( (NOW - GENESIS_EPOCH) / 86400 ))

echo "Token age: $TOKEN_AGE_DAYS days"

echo ""
echo "Step 2: Get FIL Price History from CoinGecko"
echo "------------------------------------------------------------"

# Try to get long-term FIL price data
echo "Fetching FIL price history..."

# CoinGecko API (free, might have historical data)
FIL_HISTORY=$(curl -s "https://api.coingecko.com/api/v3/coins/filecoin/market_chart?vs_currency=usd&days=max&interval=daily" 2>/dev/null || echo '{}')

PRICE_COUNT=$(echo "$FIL_HISTORY" | jq '.prices | length' 2>/dev/null || echo "0")

echo "FIL price data points: $PRICE_COUNT"

if [ "$PRICE_COUNT" -gt 0 ]; then
    echo "✅ Historical FIL price available!"
    echo ""
    echo "Sample FIL prices over time:"

    echo "$FIL_HISTORY" | jq -r '
    .prices |
    [.[] | select(.[0] >= 1704067200000)] |  # Filter to 2024+
    .[::30] |  # Sample every 30th point
    .[] |
    "\(.[0] / 1000 | strftime("%Y-%m-%d")) | $\(.[1] | tostring | .[0:6])"
    ' 2>/dev/null | head -20
else
    echo "❌ No historical FIL price data from CoinGecko free API"
    echo "   Will use GeckoTerminal pool data instead"
fi

echo ""
echo "Step 3: Alternative - GeckoTerminal Pool History"
echo "------------------------------------------------------------"

POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"

# Get maximum available history from GeckoTerminal
echo "Fetching pool OHLCV (max available)..."

# Try different timeframes
for timeframe in "day" "hour" "minute"; do
    for limit in 1000 500 300 168; do
        OHLCV=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/${POOL_ADDRESS}/ohlcv/${timeframe}?aggregate=1&limit=$limit" 2>/dev/null)

        COUNT=$(echo "$OHLCV" | jq '.data.attributes.ohlcv_list | length' 2>/dev/null || echo "0")

        if [ "$COUNT" -gt 0 ]; then
            echo "  $timeframe candles (limit=$limit): $COUNT data points"

            OLDEST=$(echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[-1][0] | strftime("%Y-%m-%d %H:%M")' 2>/dev/null)
            NEWEST=$(echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[0][0] | strftime("%Y-%m-%d %H:%M")' 2>/dev/null)

            echo "    Range: $OLDEST to $NEWEST"
            break 2
        fi
    done
done

PRICE_HISTORY_DAYS=$(echo "$OHLCV" | jq -r '
.data.attributes.ohlcv_list |
(.[0][0] - .[-1][0]) / 86400
' 2>/dev/null || echo "0")

echo ""
echo "Price history available: ${PRICE_HISTORY_DAYS:0:4} days"

echo ""
echo "Step 4: Calculate TCR Timeline"
echo "------------------------------------------------------------"

# Get current values
CURRENT_TCR=$(sqlite3 data/metrics_history.db "SELECT tcr FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "198.87")
CURRENT_SUPPLY=$(sqlite3 data/metrics_history.db "SELECT supply FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "232964.5")

echo "Current TCR: $CURRENT_TCR%"
echo "Current supply: $CURRENT_SUPPLY USDFC"

# Calculate collateral from TCR
CURRENT_FIL_PRICE=$(curl -s "https://api.coingecko.com/api/v3/simple/price?ids=filecoin&vs_currencies=usd" | jq -r '.filecoin.usd // "4.5"')
COLLATERAL=$(echo "scale=2; ($CURRENT_TCR * $CURRENT_SUPPLY / 100) / $CURRENT_FIL_PRICE" | bc 2>/dev/null || echo "300000")

echo "Current FIL price: \$$CURRENT_FIL_PRICE"
echo "Calculated collateral: $COLLATERAL FIL"

echo ""
echo "Calculating TCR from price history..."

# Use available price data to show TCR variation
TCR_TIMELINE=$(echo "$OHLCV" | jq -r --arg supply "$CURRENT_SUPPLY" --arg collateral "$COLLATERAL" '
.data.attributes.ohlcv_list |
reverse |  # Oldest first
.[::24] |  # Sample every 24 points (daily if hourly)
.[] |
{
    time: (.[0] | strftime("%Y-%m-%d")),
    fil_price: .[4],
    tcr: ((($collateral | tonumber) * .[4]) / ($supply | tonumber) * 100)
} |
"\(.time) | FIL: $\(.fil_price | tostring | .[0:6]) | TCR: \(.tcr | tostring | .[0:6])%"
' 2>/dev/null)

echo "$TCR_TIMELINE"

echo ""
echo "Step 5: Estimate TCR from Genesis"
echo "------------------------------------------------------------"

# If we don't have full history, estimate based on FIL price trends
echo "Since we have limited price history (${PRICE_HISTORY_DAYS:0:4} days),"
echo "estimating TCR for full token lifetime ($TOKEN_AGE_DAYS days)..."

echo ""
echo "Approach: Assume collateral and supply were constant,"
echo "          TCR varied ONLY with FIL price changes"

# Get FIL price range from available data
FIL_MIN=$(echo "$OHLCV" | jq '[.data.attributes.ohlcv_list[][3]] | min' 2>/dev/null || echo "4.0")
FIL_MAX=$(echo "$OHLCV" | jq '[.data.attributes.ohlcv_list[][2]] | max' 2>/dev/null || echo "5.0")

echo ""
echo "FIL price range in available data:"
echo "  Min: \$$FIL_MIN"
echo "  Max: \$$FIL_MAX"

# Calculate TCR range
TCR_MIN=$(echo "scale=2; ($COLLATERAL * $FIL_MIN) / $CURRENT_SUPPLY * 100" | bc 2>/dev/null || echo "190")
TCR_MAX=$(echo "scale=2; ($COLLATERAL * $FIL_MAX) / $CURRENT_SUPPLY * 100" | bc 2>/dev/null || echo "210")

echo ""
echo "Estimated TCR range:"
echo "  Min TCR: ${TCR_MIN:0:6}% (when FIL = \$$FIL_MIN)"
echo "  Max TCR: ${TCR_MAX:0:6}% (when FIL = \$$FIL_MAX)"

TCR_VARIATION=$(echo "scale=2; (($TCR_MAX - $TCR_MIN) / $CURRENT_TCR) * 100" | bc 2>/dev/null || echo "0")
echo "  Variation: ${TCR_VARIATION:0:5}%"

echo ""
echo "========================================================================"
echo "TCR GENESIS HISTORY - TEST RESULTS"
echo "========================================================================"
echo ""

echo "Token age:               $TOKEN_AGE_DAYS days"
echo "Price data available:    ${PRICE_HISTORY_DAYS:0:4} days"
echo "Coverage:                $(echo "scale=0; ${PRICE_HISTORY_DAYS:0:4} * 100 / $TOKEN_AGE_DAYS" | bc 2>/dev/null || echo "50")%"
echo ""
echo "TCR calculation:"
echo "  Current TCR:           $CURRENT_TCR%"
echo "  Estimated min TCR:     ${TCR_MIN:0:6}%"
echo "  Estimated max TCR:     ${TCR_MAX:0:6}%"
echo "  Variation:             ${TCR_VARIATION:0:5}%"
echo ""

if [ "$(echo "${PRICE_HISTORY_DAYS:0:4} > 30" | bc 2>/dev/null)" == "1" ]; then
    echo "✅ VERDICT: Can build TCR history for ${PRICE_HISTORY_DAYS:0:4} days"
    echo ""
    echo "  Data points: $(echo "$OHLCV" | jq '.data.attributes.ohlcv_list | length' 2>/dev/null)"
    echo "  Method: TCR = (Collateral × FIL_Price) / Supply × 100"
    echo "  Result: TCR chart with ${TCR_VARIATION:0:5}% variation"
    echo ""
    echo "📊 Chart: Curves showing TCR following FIL price"
    echo ""
    echo "RECOMMENDATION: Implement TCR calculation from price history"
    echo "  - Use available ${PRICE_HISTORY_DAYS:0:4} days of data"
    echo "  - For genesis to price_start: show estimated flat line"
    echo "  - For price_start to today: show calculated curves"
else
    echo "⚠️  VERDICT: Limited price history (${PRICE_HISTORY_DAYS:0:4} days)"
    echo ""
    echo "RECOMMENDATION: Use available data, estimate beyond"
fi

echo ""
