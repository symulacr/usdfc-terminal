#!/bin/bash

# Test Extended Lookback: 200 data points at 4-hour intervals
# This covers ~33 days of historical data
# Purpose: Find patterns, peaks, and lows for ALL metrics

echo "================================================"
echo "TEST: Extended Lookback - 200 Points × 4 Hours"
echo "================================================"
echo "Coverage: ~33 days (800 hours)"
echo "Resolution: 4-hour intervals"
echo "Purpose: Find peaks, lows, and patterns for ALL metrics"
echo ""

# Configuration
POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"
USDFC_TOKEN="0x80B98d3aa09ffff255c3ba4A241111Ff1262F045"
DATA_POINTS=200
TIMEFRAME="hour"
AGGREGATE="4"

# Step 1: Fetch OHLCV data
echo "Step 1: Fetching OHLCV data (200 points, 4h intervals)..."
OHLCV=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/$POOL_ADDRESS/ohlcv/$TIMEFRAME?aggregate=$AGGREGATE&limit=$DATA_POINTS")

if [ $? -ne 0 ] || [ -z "$OHLCV" ]; then
    echo "❌ Failed to fetch OHLCV data"
    exit 1
fi

CANDLE_COUNT=$(echo "$OHLCV" | jq '.data.attributes.ohlcv_list | length')
echo "  ✅ Received $CANDLE_COUNT candles"

if [ "$CANDLE_COUNT" -eq 0 ]; then
    echo "❌ No candles received"
    exit 1
fi

# Extract first and last timestamps
FIRST_TIME=$(echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[-1][0]')
LAST_TIME=$(echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[0][0]')
FIRST_DATE=$(date -d @$FIRST_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$FIRST_TIME")
LAST_DATE=$(date -d @$LAST_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$LAST_TIME")

echo "  Date Range:"
echo "    Oldest: $FIRST_DATE"
echo "    Newest: $LAST_DATE"
echo ""

# Step 2: Get current supply and collateral for TCR calculation
echo "Step 2: Fetching current supply and collateral..."
SUPPLY=$(sqlite3 data/metrics_history.db "SELECT supply FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null)
if [ -z "$SUPPLY" ] || [ "$SUPPLY" = "0" ]; then
    SUPPLY=232964.516863532  # Fallback
fi

# Get collateral via RPC
RPC_URL="https://api.node.glif.io/rpc/v1"
ACTIVE_POOL="0x8637Ac7FdBB4c763B72e26504aFb659df71c7803"
COLLATERAL_HEX=$(curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "'"$ACTIVE_POOL"'",
      "data": "0x4a59ff51"
    }, "latest"],
    "id": 1
  }' | jq -r '.result')

if [ "$COLLATERAL_HEX" != "null" ] && [ -n "$COLLATERAL_HEX" ]; then
    COLLATERAL_WEI=$(echo "ibase=16; ${COLLATERAL_HEX#0x}" | bc 2>/dev/null || echo "0")
    COLLATERAL=$(echo "scale=2; $COLLATERAL_WEI / 1000000000000000000" | bc -l)
else
    COLLATERAL=466139  # Fallback
fi

echo "  Supply: $SUPPLY USDFC"
echo "  Collateral: $COLLATERAL FIL"
echo ""

# Step 3: Calculate all metrics for each candle
echo "Step 3: Calculating metrics for all $CANDLE_COUNT candles..."
echo "  Processing..."

# Create temporary files for analysis
PRICE_FILE=$(mktemp)
VOLUME_FILE=$(mktemp)
TCR_FILE=$(mktemp)
LIQUIDITY_FILE=$(mktemp)

# Process each candle
echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[] | @json' | while read -r candle; do
    TIME=$(echo "$candle" | jq -r '.[0]')
    OPEN=$(echo "$candle" | jq -r '.[1]')
    HIGH=$(echo "$candle" | jq -r '.[2]')
    LOW=$(echo "$candle" | jq -r '.[3]')
    CLOSE=$(echo "$candle" | jq -r '.[4]')
    VOLUME=$(echo "$candle" | jq -r '.[5]')

    # Calculate TCR
    TCR=$(echo "scale=2; ($COLLATERAL * $CLOSE) / $SUPPLY * 100" | bc -l)

    # Calculate Liquidity (Volume / Price Impact)
    IMPACT=$(echo "scale=10; ($HIGH - $LOW) / $CLOSE" | bc -l)
    if (( $(echo "$IMPACT > 0.0001" | bc -l) )); then
        LIQUIDITY=$(echo "scale=2; $VOLUME / $IMPACT" | bc -l)
    else
        LIQUIDITY="0"
    fi

    # Store in temp files
    echo "$TIME $CLOSE" >> $PRICE_FILE
    echo "$TIME $VOLUME" >> $VOLUME_FILE
    echo "$TIME $TCR" >> $TCR_FILE
    if [ "$LIQUIDITY" != "0" ]; then
        echo "$TIME $LIQUIDITY" >> $LIQUIDITY_FILE
    fi
done

echo "  ✅ Calculations complete"
echo ""

# Step 4: Analyze Price
echo "================================================"
echo "METRIC 1: PRICE (FIL/USD)"
echo "================================================"

PRICE_MIN=$(awk '{print $2}' $PRICE_FILE | sort -n | head -1)
PRICE_MAX=$(awk '{print $2}' $PRICE_FILE | sort -n | tail -1)
PRICE_AVG=$(awk '{sum+=$2; count++} END {printf "%.6f", sum/count}' $PRICE_FILE)
PRICE_MIN_TIME=$(awk -v min="$PRICE_MIN" '$2==min {print $1; exit}' $PRICE_FILE)
PRICE_MAX_TIME=$(awk -v max="$PRICE_MAX" '$2==max {print $1; exit}' $PRICE_FILE)
PRICE_MIN_DATE=$(date -d @$PRICE_MIN_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$PRICE_MIN_TIME")
PRICE_MAX_DATE=$(date -d @$PRICE_MAX_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$PRICE_MAX_TIME")
PRICE_RANGE=$(echo "scale=6; $PRICE_MAX - $PRICE_MIN" | bc)
PRICE_VAR=$(echo "scale=2; ($PRICE_RANGE / $PRICE_MIN) * 100" | bc)

echo "  Minimum: \$$PRICE_MIN"
echo "    Date: $PRICE_MIN_DATE"
echo "  Maximum: \$$PRICE_MAX"
echo "    Date: $PRICE_MAX_DATE"
echo "  Average: \$$PRICE_AVG"
echo "  Range: \$$PRICE_RANGE"
echo "  Variation: $PRICE_VAR%"
echo ""

# Step 5: Analyze Volume
echo "================================================"
echo "METRIC 2: VOLUME (24h Trading)"
echo "================================================"

VOLUME_MIN=$(awk '{print $2}' $VOLUME_FILE | sort -n | head -1)
VOLUME_MAX=$(awk '{print $2}' $VOLUME_FILE | sort -n | tail -1)
VOLUME_AVG=$(awk '{sum+=$2; count++} END {printf "%.2f", sum/count}' $VOLUME_FILE)
VOLUME_MIN_TIME=$(awk -v min="$VOLUME_MIN" '$2==min {print $1; exit}' $VOLUME_FILE)
VOLUME_MAX_TIME=$(awk -v max="$VOLUME_MAX" '$2==max {print $1; exit}' $VOLUME_FILE)
VOLUME_MIN_DATE=$(date -d @$VOLUME_MIN_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$VOLUME_MIN_TIME")
VOLUME_MAX_DATE=$(date -d @$VOLUME_MAX_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$VOLUME_MAX_TIME")
VOLUME_RANGE=$(echo "scale=2; $VOLUME_MAX - $VOLUME_MIN" | bc)
VOLUME_VAR=$(echo "scale=2; ($VOLUME_RANGE / $VOLUME_MIN) * 100" | bc)

echo "  Minimum: \$$VOLUME_MIN"
echo "    Date: $VOLUME_MIN_DATE"
echo "  Maximum: \$$VOLUME_MAX"
echo "    Date: $VOLUME_MAX_DATE"
echo "  Average: \$$VOLUME_AVG"
echo "  Range: \$$VOLUME_RANGE"
echo "  Variation: $VOLUME_VAR%"
echo "  📊 PEAK VOLUME EVENT: $VOLUME_MAX_DATE"
echo ""

# Step 6: Analyze TCR
echo "================================================"
echo "METRIC 3: TCR (Total Collateral Ratio)"
echo "================================================"

TCR_MIN=$(awk '{print $2}' $TCR_FILE | sort -n | head -1)
TCR_MAX=$(awk '{print $2}' $TCR_FILE | sort -n | tail -1)
TCR_AVG=$(awk '{sum+=$2; count++} END {printf "%.2f", sum/count}' $TCR_FILE)
TCR_MIN_TIME=$(awk -v min="$TCR_MIN" '$2==min {print $1; exit}' $TCR_FILE)
TCR_MAX_TIME=$(awk -v max="$TCR_MAX" '$2==max {print $1; exit}' $TCR_FILE)
TCR_MIN_DATE=$(date -d @$TCR_MIN_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$TCR_MIN_TIME")
TCR_MAX_DATE=$(date -d @$TCR_MAX_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$TCR_MAX_TIME")
TCR_RANGE=$(echo "scale=2; $TCR_MAX - $TCR_MIN" | bc)
TCR_VAR=$(echo "scale=2; ($TCR_RANGE / $TCR_MIN) * 100" | bc)

echo "  Minimum: $TCR_MIN%"
echo "    Date: $TCR_MIN_DATE"
echo "  Maximum: $TCR_MAX%"
echo "    Date: $TCR_MAX_DATE"
echo "  Average: $TCR_AVG%"
echo "  Range: $TCR_RANGE percentage points"
echo "  Variation: $TCR_VAR%"
echo "  🔒 SAFEST MOMENT: $TCR_MAX_DATE (highest collateral)"
echo "  ⚠️  RISKIEST MOMENT: $TCR_MIN_DATE (lowest collateral)"
echo ""

# Step 7: Analyze Liquidity
echo "================================================"
echo "METRIC 4: LIQUIDITY (Estimated from Volume/Impact)"
echo "================================================"

if [ -s $LIQUIDITY_FILE ]; then
    LIQ_MIN=$(awk '{print $2}' $LIQUIDITY_FILE | sort -n | head -1)
    LIQ_MAX=$(awk '{print $2}' $LIQUIDITY_FILE | sort -n | tail -1)
    LIQ_AVG=$(awk '{sum+=$2; count++} END {printf "%.2f", sum/count}' $LIQUIDITY_FILE)
    LIQ_MIN_TIME=$(awk -v min="$LIQ_MIN" '$2==min {print $1; exit}' $LIQUIDITY_FILE)
    LIQ_MAX_TIME=$(awk -v max="$LIQ_MAX" '$2==max {print $1; exit}' $LIQUIDITY_FILE)
    LIQ_MIN_DATE=$(date -d @$LIQ_MIN_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$LIQ_MIN_TIME")
    LIQ_MAX_DATE=$(date -d @$LIQ_MAX_TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$LIQ_MAX_TIME")
    LIQ_RANGE=$(echo "scale=2; $LIQ_MAX - $LIQ_MIN" | bc)
    LIQ_VAR=$(echo "scale=2; ($LIQ_RANGE / $LIQ_MIN) * 100" | bc)

    echo "  Minimum: \$$LIQ_MIN"
    echo "    Date: $LIQ_MIN_DATE"
    echo "  Maximum: \$$LIQ_MAX"
    echo "    Date: $LIQ_MAX_DATE"
    echo "  Average: \$$LIQ_AVG"
    echo "  Range: \$$LIQ_RANGE"
    echo "  Variation: $LIQ_VAR%"
    echo "  💧 DEEPEST LIQUIDITY: $LIQ_MAX_DATE"
    echo "  📉 SHALLOWEST LIQUIDITY: $LIQ_MIN_DATE"
else
    echo "  ❌ No liquidity data calculated (insufficient price movements)"
fi
echo ""

# Step 8: Get holder count (check if stable or varies)
echo "================================================"
echo "METRIC 5: HOLDERS (Token Balance Addresses)"
echo "================================================"

CURRENT_HOLDERS=$(curl -s "https://filecoin.blockscout.com/api/v2/tokens/$USDFC_TOKEN" | jq -r '.holders')
echo "  Current Holders: $CURRENT_HOLDERS"
echo "  Note: Holder count typically doesn't vary much in short term"
echo "  For historical variation, need to process transfer events from genesis"
echo ""

# Step 9: Summary
echo "================================================"
echo "SUMMARY: Extended Lookback Analysis"
echo "================================================"

echo "Data Coverage:"
echo "  Time Range: $FIRST_DATE to $LAST_DATE"
echo "  Data Points: $CANDLE_COUNT × 4-hour intervals"
echo "  Total Days: ~33 days"
echo ""

echo "Metric Variations (over 33 days):"
echo "  Price:     $PRICE_VAR% variation"
echo "  Volume:    $VOLUME_VAR% variation 📊"
echo "  TCR:       $TCR_VAR% variation 🔒"
if [ -s $LIQUIDITY_FILE ]; then
    echo "  Liquidity: $LIQ_VAR% variation 💧"
fi
echo ""

echo "Key Moments:"
echo "  📊 PEAK Volume:      $VOLUME_MAX_DATE (\$$VOLUME_MAX)"
echo "  🔒 SAFEST TCR:       $TCR_MAX_DATE ($TCR_MAX%)"
echo "  ⚠️  RISKIEST TCR:     $TCR_MIN_DATE ($TCR_MIN%)"
if [ -s $LIQUIDITY_FILE ]; then
    echo "  💧 DEEPEST Liquidity: $LIQ_MAX_DATE (\$$LIQ_MAX)"
fi
echo ""

echo "Chart Rendering Verdict:"
if (( $(echo "$PRICE_VAR > 1" | bc -l) )); then
    echo "  ✅ Price: Will show clear curves ($PRICE_VAR% variation)"
else
    echo "  ⚠️  Price: Subtle curves ($PRICE_VAR% variation)"
fi

if (( $(echo "$VOLUME_VAR > 100" | bc -l) )); then
    echo "  ✅ Volume: Will show DRAMATIC curves ($VOLUME_VAR% variation)"
else
    echo "  ⚠️  Volume: Moderate curves ($VOLUME_VAR% variation)"
fi

if (( $(echo "$TCR_VAR > 5" | bc -l) )); then
    echo "  ✅ TCR: Will show clear curves ($TCR_VAR% variation)"
elif (( $(echo "$TCR_VAR > 2" | bc -l) )); then
    echo "  ✅ TCR: Will show visible curves ($TCR_VAR% variation)"
else
    echo "  ⚠️  TCR: Subtle curves ($TCR_VAR% variation)"
fi

if [ -s $LIQUIDITY_FILE ]; then
    if (( $(echo "$LIQ_VAR > 300" | bc -l) )); then
        echo "  ✅ Liquidity: Will show DRAMATIC curves ($LIQ_VAR% variation) 🔥"
    elif (( $(echo "$LIQ_VAR > 100" | bc -l) )); then
        echo "  ✅ Liquidity: Will show clear curves ($LIQ_VAR% variation)"
    else
        echo "  ⚠️  Liquidity: Moderate curves ($LIQ_VAR% variation)"
    fi
fi

echo ""
echo "================================================"
echo "TEST COMPLETE"
echo "================================================"

# Cleanup
rm -f $PRICE_FILE $VOLUME_FILE $TCR_FILE $LIQUIDITY_FILE
