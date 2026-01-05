#!/bin/bash
# Test Script: Holders, Volume, Transfers Historical Data
# Purpose: Validate data sources for these metrics will show curves over time

set -e

echo "========================================================================"
echo "TEST: Holders / Volume / Transfers Historical Data"
echo "========================================================================"
echo ""

# Configuration
USDFC_TOKEN="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"
POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"

# Get actual values from database
DB_SUPPLY=$(sqlite3 data/metrics_history.db "SELECT supply FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "232964.5")
DB_TCR=$(sqlite3 data/metrics_history.db "SELECT tcr FROM metric_snapshots ORDER BY timestamp DESC LIMIT 1" 2>/dev/null || echo "198.87")

echo "Known values from database:"
echo "  Supply: $DB_SUPPLY USDFC"
echo "  TCR: $DB_TCR%"
echo ""

# Calculate collateral from TCR
FIL_PRICE=$(curl -s "https://api.coingecko.com/api/v3/simple/price?ids=filecoin&vs_currencies=usd" | jq -r '.filecoin.usd // "4.5"')
COLLATERAL=$(echo "scale=2; ($DB_TCR * $DB_SUPPLY / 100) / $FIL_PRICE" | bc)

echo "Calculated:"
echo "  FIL Price: \$$FIL_PRICE"
echo "  Collateral: $COLLATERAL FIL"
echo ""

echo "========================================================================"
echo "TEST 1/3: DEX Volume Over Time"
echo "========================================================================"
echo ""

echo "Fetching volume data from GeckoTerminal OHLCV..."
VOLUME_DATA=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/${POOL_ADDRESS}/ohlcv/hour?aggregate=1&limit=168")

VOLUME_COUNT=$(echo "$VOLUME_DATA" | jq '.data.attributes.ohlcv_list | length')

echo "Volume data received: $VOLUME_COUNT candles"
echo ""

if [ "$VOLUME_COUNT" -gt 100 ]; then
    echo "Sample volume data (every 24 hours):"
    echo "$VOLUME_DATA" | jq -r '
    .data.attributes.ohlcv_list |
    to_entries |
    map(select(.key % 24 == 0)) |
    .[] |
    .value |
    "  \(.[0] | strftime("%Y-%m-%d %H:%M")) | Volume: $\(.[5] | tostring | .[0:10])"
    ' | head -7

    # Calculate statistics
    STATS=$(echo "$VOLUME_DATA" | jq '
    .data.attributes.ohlcv_list |
    map(.[5]) |
    {
        min: min,
        max: max,
        avg: (add / length),
        variation: ((max - min) / (add / length) * 100)
    }
    ')

    MIN_VOL=$(echo "$STATS" | jq -r '.min')
    MAX_VOL=$(echo "$STATS" | jq -r '.max')
    VARIATION=$(echo "$STATS" | jq -r '.variation')

    echo ""
    echo "Statistics:"
    echo "  Min volume: \$${MIN_VOL:0:10}"
    echo "  Max volume: \$${MAX_VOL:0:10}"
    echo "  Variation: ${VARIATION:0:6}%"

    echo ""
    if [ "$(echo "$VARIATION > 50" | bc)" -eq 1 ]; then
        echo "✅ VOLUME: High variation (${VARIATION:0:4}%) - WILL SHOW CURVES"
    elif [ "$(echo "$VARIATION > 10" | bc)" -eq 1 ]; then
        echo "✅ VOLUME: Moderate variation (${VARIATION:0:4}%) - WILL SHOW VISIBLE CHANGES"
    else
        echo "⚠️  VOLUME: Low variation (${VARIATION:0:4}%) - May appear relatively stable"
    fi
else
    echo "❌ VOLUME: Insufficient data ($VOLUME_COUNT candles)"
fi

echo ""
echo "========================================================================"
echo "TEST 2/3: Holders Count Over Time"
echo "========================================================================"
echo ""

echo "Checking Blockscout API for current holder count..."
HOLDERS_CURRENT=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN" | jq -r '.holders // "0"')

echo "Current holders: $HOLDERS_CURRENT"
echo ""

echo "Checking for historical holder data..."
echo "Note: Blockscout typically provides current count only"
echo ""

# Try GraphQL for historical data
GRAPHQL_TEST=$(curl -s -X POST "https://explorer.filecoin.io/api/v2/graphql" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ token(contractAddressHash: \"'$USDFC_TOKEN'\") { holderCount } }"
  }' 2>/dev/null || echo '{}')

HOLDER_COUNT_GQL=$(echo "$GRAPHQL_TEST" | jq -r '.data.token.holderCount // "null"')

if [ "$HOLDER_COUNT_GQL" != "null" ]; then
    echo "✅ GraphQL holder count: $HOLDER_COUNT_GQL"
else
    echo "⚠️  GraphQL doesn't provide historical holder data"
fi

echo ""
echo "Checking database snapshots for holder variation..."
HOLDER_STATS=$(sqlite3 data/metrics_history.db "
SELECT
    MIN(holders),
    MAX(holders),
    COUNT(*)
FROM metric_snapshots
" 2>/dev/null || echo "0|0|0")

IFS='|' read -r MIN_HOLDERS MAX_HOLDERS SNAPSHOT_COUNT <<< "$HOLDER_STATS"

echo "  Snapshots collected: $SNAPSHOT_COUNT"
echo "  Min holders: $MIN_HOLDERS"
echo "  Max holders: $MAX_HOLDERS"
echo "  Variation: $((MAX_HOLDERS - MIN_HOLDERS)) holders"

if [ "$((MAX_HOLDERS - MIN_HOLDERS))" -gt 10 ]; then
    echo ""
    echo "✅ HOLDERS: Variation of $((MAX_HOLDERS - MIN_HOLDERS)) - WILL SHOW CURVES"
elif [ "$((MAX_HOLDERS - MIN_HOLDERS))" -gt 0 ]; then
    echo ""
    echo "⚠️  HOLDERS: Small variation ($((MAX_HOLDERS - MIN_HOLDERS))) - Will show subtle changes"
else
    echo ""
    echo "❌ HOLDERS: No variation yet - WILL BE FLAT (need more time)"
    echo "   Recommendation: Continue collecting snapshots, check again in 1 week"
fi

echo ""
echo "========================================================================"
echo "TEST 3/3: Transfers Over Time"
echo "========================================================================"
echo ""

echo "Testing Blockscout GraphQL for transfer data..."

# Get recent transfers
TRANSFERS_QUERY='{
  "query": "query { token(contractAddressHash: \"'$USDFC_TOKEN'\") { transfersCount } }"
}'

TRANSFERS_RESULT=$(curl -s -X POST "https://explorer.filecoin.io/api/v2/graphql" \
  -H "Content-Type: application/json" \
  -d "$TRANSFERS_QUERY" 2>/dev/null || echo '{}')

TOTAL_TRANSFERS=$(echo "$TRANSFERS_RESULT" | jq -r '.data.token.transfersCount // "0"')

echo "Total transfers (all time): $TOTAL_TRANSFERS"
echo ""

if [ "$TOTAL_TRANSFERS" -gt 0 ]; then
    echo "Fetching recent transfer events for time-series analysis..."

    # Get token transfers
    RECENT_TRANSFERS=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20" | jq '.items | length')

    echo "Recent transfers fetched: $RECENT_TRANSFERS"
    echo ""

    if [ "$RECENT_TRANSFERS" -gt 10 ]; then
        echo "✅ TRANSFERS: Blockscout API working, has transfer data"
        echo "   Implementation: Group transfers by time period (hourly/daily)"
        echo "   Result: WILL SHOW BARS/CURVES based on transfer activity"
    else
        echo "⚠️  TRANSFERS: Limited recent data, may need pagination"
    fi
else
    echo "❌ TRANSFERS: No transfer data available"
fi

echo ""
echo "========================================================================"
echo "SUMMARY & RECOMMENDATIONS"
echo "========================================================================"
echo ""

echo "Metric      | Data Points | Variation | Will Show Curves?"
echo "------------|-------------|-----------|------------------"

# Volume
if [ "$VOLUME_COUNT" -gt 100 ]; then
    echo "Volume      | $VOLUME_COUNT       | High      | ✅ YES"
else
    echo "Volume      | $VOLUME_COUNT         | Unknown   | ❌ NO"
fi

# Holders
if [ "$((MAX_HOLDERS - MIN_HOLDERS))" -gt 5 ]; then
    echo "Holders     | $SNAPSHOT_COUNT         | $((MAX_HOLDERS - MIN_HOLDERS)) holders  | ✅ YES"
elif [ "$((MAX_HOLDERS - MIN_HOLDERS))" -gt 0 ]; then
    echo "Holders     | $SNAPSHOT_COUNT         | $((MAX_HOLDERS - MIN_HOLDERS)) holders  | ⚠️  SUBTLE"
else
    echo "Holders     | $SNAPSHOT_COUNT         | 0         | ❌ FLAT (wait)"
fi

# Transfers
if [ "$TOTAL_TRANSFERS" -gt 100 ]; then
    echo "Transfers   | Many        | Unknown   | ✅ YES (needs aggregation)"
else
    echo "Transfers   | Few         | Unknown   | ⚠️  LIMITED"
fi

echo ""
echo "Implementation Priority:"
echo ""
echo "1. ✅ Volume - Use existing GeckoTerminal OHLCV data (already working!)"
echo "2. ✅ Transfers - Implement Blockscout time-period aggregation"
echo "3. ⏳ Holders - Continue snapshot collection (will improve over time)"
echo ""

echo "Completed at: $(date)"
echo "========================================================================"
