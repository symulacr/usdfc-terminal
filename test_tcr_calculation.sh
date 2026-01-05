#!/bin/bash
# Test Script: TCR Calculation from Price History
# Purpose: Verify TCR will show curves when calculated from FIL price changes
# Run this BEFORE implementing code to confirm approach works

set -e

echo "================================================"
echo "TEST: TCR Calculation from Price History"
echo "================================================"
echo ""

# Configuration
POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"
USDFC_TOKEN="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"
ACTIVE_POOL="0xf03516344"  # Filecoin address format
TROVE_MANAGER="0xf03516344"

# Step 1: Get FIL Price History (1 week, 1 hour resolution)
echo "Step 1: Fetching FIL price history (1 week, hourly)..."
echo "API: GeckoTerminal OHLCV"
echo ""

PRICE_DATA=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/${POOL_ADDRESS}/ohlcv/hour?aggregate=1&limit=168")

# Parse and show price data
echo "Price candles received:"
CANDLES=$(echo "$PRICE_DATA" | jq -r '.data.attributes.ohlcv_list | length')
echo "  Total candles: $CANDLES"

if [ "$CANDLES" -gt 0 ]; then
    echo "  First 5 price points:"
    echo "$PRICE_DATA" | jq -r '.data.attributes.ohlcv_list[:5] | .[] | "    Time: \(.[0] | strftime("%Y-%m-%d %H:%M")) | Close: $\(.[4])"'

    echo ""
    echo "  Last 5 price points:"
    echo "$PRICE_DATA" | jq -r '.data.attributes.ohlcv_list[-5:] | .[] | "    Time: \(.[0] | strftime("%Y-%m-%d %H:%M")) | Close: $\(.[4])"'
else
    echo "  ❌ No price data received!"
    exit 1
fi

echo ""
echo "Step 2: Get current USDFC supply..."

# Try to get from database first (more reliable)
SUPPLY=$(sqlite3 data/metrics_history.db "SELECT supply FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "0")

if [ "$SUPPLY" == "0" ] || [ -z "$SUPPLY" ]; then
    # Fallback to RPC
    SUPPLY_RESULT=$(curl -s -X POST https://api.node.glif.io/rpc/v1 \
      -H "Content-Type: application/json" \
      -d '{
        "jsonrpc": "2.0",
        "method": "Filecoin.EthCall",
        "params": [{
          "to": "'"$USDFC_TOKEN"'",
          "data": "0x18160ddd"
        }, "latest"],
        "id": 1
      }')

    SUPPLY_HEX=$(echo "$SUPPLY_RESULT" | jq -r '.result')
    SUPPLY_DEC=$(printf "%d" "$SUPPLY_HEX" 2>/dev/null || echo "0")
    SUPPLY=$(echo "scale=2; $SUPPLY_DEC / 1000000000000000000" | bc)
fi

echo "  Current USDFC Supply: $SUPPLY USDFC"

echo ""
echo "Step 3: Get current collateral from ActivePool..."

# Get TCR from database to calculate collateral
DB_TCR=$(sqlite3 data/metrics_history.db "SELECT tcr FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "198.87")

# Get current FIL price from price candles
CURRENT_PRICE=$(echo "$PRICE_DATA" | jq -r '.data.attributes.ohlcv_list[-1][4]')

# Calculate collateral from TCR formula: TCR = (Collateral * Price) / Supply * 100
# Therefore: Collateral = (TCR * Supply) / (Price * 100)
COLLATERAL=$(echo "scale=2; ($DB_TCR * $SUPPLY) / ($CURRENT_PRICE * 100)" | bc)

echo "  Using database TCR: $DB_TCR%"
echo "  Current FIL price: \$$CURRENT_PRICE"
echo "  Calculated Collateral: $COLLATERAL FIL"

echo ""
echo "Step 4: Calculate TCR for each price point..."
echo "Formula: TCR = (Collateral × FIL_Price) / Debt × 100"
echo "         Where Debt ≈ USDFC Supply (pegged to \$1)"
echo ""

# Calculate TCR for each candle
echo "TCR Time Series (showing 10 samples across the week):"
echo "------------------------------------------------------------"

TCR_VALUES=$(echo "$PRICE_DATA" | jq -r --arg supply "$SUPPLY" --arg collateral "$COLLATERAL" '
.data.attributes.ohlcv_list |
to_entries |
map(
    select(.key % 17 == 0 or .key == (length - 1)) |  # Sample every ~17 hours + last
    {
        time: (.value[0] | strftime("%Y-%m-%d %H:%M")),
        price: .value[4],
        tcr: ((($collateral | tonumber) * .value[4]) / ($supply | tonumber) * 100)
    }
) |
.[] |
"  \(.time) | FIL: $\(.price | tostring | .[0:6]) | TCR: \(.tcr | tostring | .[0:6])%"
')

echo "$TCR_VALUES"

echo ""
echo "Step 5: Statistical Analysis..."

# Calculate min/max/variation
STATS=$(echo "$PRICE_DATA" | jq -r --arg supply "$SUPPLY" --arg collateral "$COLLATERAL" '
.data.attributes.ohlcv_list |
map(((($collateral | tonumber) * .[4]) / ($supply | tonumber) * 100)) |
{
    min: min,
    max: max,
    avg: (add / length),
    range: (max - min),
    variation_pct: ((max - min) / (add / length) * 100)
}
')

MIN_TCR=$(echo "$STATS" | jq -r '.min')
MAX_TCR=$(echo "$STATS" | jq -r '.max')
AVG_TCR=$(echo "$STATS" | jq -r '.avg')
RANGE=$(echo "$STATS" | jq -r '.range')
VARIATION=$(echo "$STATS" | jq -r '.variation_pct')

echo "  Minimum TCR: ${MIN_TCR:0:6}%"
echo "  Maximum TCR: ${MAX_TCR:0:6}%"
echo "  Average TCR: ${AVG_TCR:0:6}%"
echo "  Range: ${RANGE:0:6} percentage points"
echo "  Variation: ${VARIATION:0:4}%"

echo ""
echo "Step 6: Verdict - Will TCR show curves?"
echo "------------------------------------------------------------"

VARIATION_INT=$(echo "$VARIATION" | awk '{print int($1)}')

if [ "$VARIATION_INT" -gt 5 ]; then
    echo "  ✅ YES! TCR varies by ${VARIATION:0:4}% over the week"
    echo "  ✅ This will show clear curves on the chart"
    echo "  ✅ Variation is significant enough to visualize"
elif [ "$VARIATION_INT" -gt 1 ]; then
    echo "  ⚠️  MAYBE. TCR varies by ${VARIATION:0:4}% over the week"
    echo "  ⚠️  Curves will be subtle but visible"
    echo "  ⚠️  Consider longer timeframe for better curves"
else
    echo "  ❌ NO. TCR varies by only ${VARIATION:0:4}% over the week"
    echo "  ❌ Will appear mostly flat"
    echo "  ❌ Need different approach or longer timeframe"
fi

echo ""
echo "Step 7: Sample Chart Preview (ASCII approximation)"
echo "------------------------------------------------------------"

# Create simple ASCII chart
echo "$PRICE_DATA" | jq -r --arg supply "$SUPPLY" --arg collateral "$COLLATERAL" '
def normalize(val; min; max):
    ((val - min) / (max - min) * 20) | floor;

.data.attributes.ohlcv_list as $candles |
($candles | map(((($collateral | tonumber) * .[4]) / ($supply | tonumber) * 100))) as $tcrs |
($tcrs | min) as $min |
($tcrs | max) as $max |

# Sample 40 points across the week
($candles | to_entries | map(select(.key % 4 == 0)) | .[0:40]) as $samples |

$samples | map({
    time: (.value[0] | strftime("%m/%d %H:%M")),
    tcr: ((($collateral | tonumber) * .value[4]) / ($supply | tonumber) * 100),
    normalized: normalize(((($collateral | tonumber) * .value[4]) / ($supply | tonumber) * 100); $min; $max)
}) |
map(
    .time + " | " +
    ("█" * (.normalized | floor)) +
    " " + (.tcr | tostring | .[0:6]) + "%"
) |
.[]
' 2>/dev/null || echo "  (Chart rendering requires jq 1.6+)"

echo ""
echo "================================================"
echo "TEST COMPLETE"
echo "================================================"
echo ""
echo "Summary:"
echo "  • Price data points: $CANDLES candles"
echo "  • TCR variation: ${VARIATION:0:4}%"
echo "  • Current supply: $SUPPLY USDFC"
echo "  • Estimated collateral: $COLLATERAL FIL"
echo ""
echo "Next steps:"
if [ "$VARIATION_INT" -gt 1 ]; then
    echo "  ✅ Proceed with implementation - TCR calculation will work!"
    echo "  ✅ Add calculate_tcr_from_price_history() function"
    echo "  ✅ Add get_active_pool_eth() RPC method"
else
    echo "  ⚠️  Consider alternative approaches"
    echo "  ⚠️  May need longer timeframes or different metrics"
fi
echo ""
