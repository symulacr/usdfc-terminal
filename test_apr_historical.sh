#!/bin/bash
# Test Script: APR Historical Data from Subgraph
# Purpose: Verify Subgraph has historical market data that will show curves
# Run this BEFORE implementing code to confirm approach works

set -e

echo "================================================"
echo "TEST: APR Historical Data from Subgraph"
echo "================================================"
echo ""

# Configuration
SUBGRAPH_URL="https://api.goldsky.com/api/public/project_cm3qlkcss000701s5cfy5e68a/subgraphs/secured-finance-filecoin/v1.0/gn"
USDFC_ADDRESS="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"

# Calculate lookback (1 week = 604800 seconds)
NOW=$(date +%s)
ONE_WEEK_AGO=$((NOW - 604800))

echo "Step 1: Querying Subgraph for historical market data..."
echo "  Lookback: 1 week (since $(date -d @$ONE_WEEK_AGO '+%Y-%m-%d %H:%M'))"
echo "  Currency: USDFC ($USDFC_ADDRESS)"
echo ""

# Query subgraph
QUERY='{
  "query": "query MarketHistory($cutoff: BigInt!) { markets(where: { currency: \"'$USDFC_ADDRESS'\", timestamp_gte: $cutoff }, orderBy: timestamp, orderDirection: asc, first: 1000) { id timestamp maturity lastLendUnitPrice lastBorrowUnitPrice isActive } }",
  "variables": {
    "cutoff": "'$ONE_WEEK_AGO'"
  }
}'

MARKET_DATA=$(curl -s -X POST "$SUBGRAPH_URL" \
  -H "Content-Type: application/json" \
  -d "$QUERY")

# Check if we got data
MARKET_COUNT=$(echo "$MARKET_DATA" | jq '.data.markets | length')

echo "Step 2: Market data received..."
echo "  Total markets: $MARKET_COUNT"

if [ "$MARKET_COUNT" -eq 0 ]; then
    echo "  ❌ No historical market data available!"
    echo "  ❌ Subgraph may be empty or query failed"
    echo ""
    echo "Response: $(echo "$MARKET_DATA" | jq -r '.errors // .data')"
    exit 1
fi

echo "  ✅ Market data available!"
echo ""

echo "Step 3: Sample market snapshots..."
echo "------------------------------------------------------------"

echo "$MARKET_DATA" | jq -r '
.data.markets |
to_entries |
map(select(.key % 100 == 0 or .key == (length - 1))) |  # Sample every 100th + last
.[] |
.value |
"  \(.timestamp | tonumber | strftime("%Y-%m-%d %H:%M")) | Maturity: \(.maturity | tonumber | strftime("%Y-%m-%d")) | Active: \(.isActive) | Lend: \(.lastLendUnitPrice // "null") | Borrow: \(.lastBorrowUnitPrice // "null")"
'

echo ""
echo "Step 4: Convert unitPrice to APR..."
echo "Formula: APR = ((10^18 / unitPrice) - 1) × (365 × 86400 / timeToMaturity) × 100"
echo ""

# Count markets with valid APR data
VALID_LEND=$(echo "$MARKET_DATA" | jq '[.data.markets[] | select(.lastLendUnitPrice != null and .isActive == true)] | length')
VALID_BORROW=$(echo "$MARKET_DATA" | jq '[.data.markets[] | select(.lastBorrowUnitPrice != null and .isActive == true)] | length')

echo "  Markets with lend APR: $VALID_LEND"
echo "  Markets with borrow APR: $VALID_BORROW"

if [ "$VALID_LEND" -eq 0 ] && [ "$VALID_BORROW" -eq 0 ]; then
    echo "  ❌ No markets have valid APR data!"
    exit 1
fi

echo ""
echo "Step 5: Calculate APR time series..."
echo "------------------------------------------------------------"

# Calculate APR for each market
APR_SERIES=$(echo "$MARKET_DATA" | jq -r --arg now "$NOW" '
def calculate_apr(unit_price; maturity; now):
    if unit_price == null or unit_price == "0" then
        null
    else
        ((pow(10; 18) / (unit_price | tonumber)) - 1) *
        (31536000 / ((maturity | tonumber) - (now | tonumber))) * 100
    end;

.data.markets |
map(
    select(.isActive == true) |
    {
        timestamp: (.timestamp | tonumber),
        time: (.timestamp | tonumber | strftime("%Y-%m-%d %H:%M")),
        lend_apr: calculate_apr(.lastLendUnitPrice; .maturity; $now),
        borrow_apr: calculate_apr(.lastBorrowUnitPrice; .maturity; $now)
    }
) |
map(select(.lend_apr != null or .borrow_apr != null)) |
to_entries |
map(select(.key % 10 == 0 or .key == (length - 1))) |  # Sample every 10th
.[] |
"  \(.value.time) | Lend APR: \(.value.lend_apr // "N/A" | if type == "number" then tostring | .[0:6] + "%" else . end) | Borrow APR: \(.value.borrow_apr // "N/A" | if type == "number" then tostring | .[0:6] + "%" else . end)"
')

echo "$APR_SERIES"

echo ""
echo "Step 6: Statistical Analysis..."

# Calculate stats
STATS=$(echo "$MARKET_DATA" | jq -r --arg now "$NOW" '
def calculate_apr(unit_price; maturity; now):
    if unit_price == null or unit_price == "0" then
        null
    else
        ((pow(10; 18) / (unit_price | tonumber)) - 1) *
        (31536000 / ((maturity | tonumber) - (now | tonumber))) * 100
    end;

.data.markets |
map(
    select(.isActive == true) |
    {
        lend_apr: calculate_apr(.lastLendUnitPrice; .maturity; $now),
        borrow_apr: calculate_apr(.lastBorrowUnitPrice; .maturity; $now)
    }
) |
{
    lend: [.[] | select(.lend_apr != null) | .lend_apr],
    borrow: [.[] | select(.borrow_apr != null) | .borrow_apr]
} |
{
    lend_count: (.lend | length),
    lend_min: (if .lend | length > 0 then .lend | min else 0 end),
    lend_max: (if .lend | length > 0 then .lend | max else 0 end),
    lend_avg: (if .lend | length > 0 then (.lend | add / length) else 0 end),
    borrow_count: (.borrow | length),
    borrow_min: (if .borrow | length > 0 then .borrow | min else 0 end),
    borrow_max: (if .borrow | length > 0 then .borrow | max else 0 end),
    borrow_avg: (if .borrow | length > 0 then (.borrow | add / length) else 0 end)
}
')

echo "Lend APR:"
echo "  Data points: $(echo "$STATS" | jq -r '.lend_count')"
echo "  Min: $(echo "$STATS" | jq -r '.lend_min | tostring | .[0:6]')%"
echo "  Max: $(echo "$STATS" | jq -r '.lend_max | tostring | .[0:6]')%"
echo "  Avg: $(echo "$STATS" | jq -r '.lend_avg | tostring | .[0:6]')%"
LEND_RANGE=$(echo "$STATS" | jq -r '(.lend_max - .lend_min)')
echo "  Range: ${LEND_RANGE:0:6} percentage points"

echo ""
echo "Borrow APR:"
echo "  Data points: $(echo "$STATS" | jq -r '.borrow_count')"
echo "  Min: $(echo "$STATS" | jq -r '.borrow_min | tostring | .[0:6]')%"
echo "  Max: $(echo "$STATS" | jq -r '.borrow_max | tostring | .[0:6]')%"
echo "  Avg: $(echo "$STATS" | jq -r '.borrow_avg | tostring | .[0:6]')%"
BORROW_RANGE=$(echo "$STATS" | jq -r '(.borrow_max - .borrow_min)')
echo "  Range: ${BORROW_RANGE:0:6} percentage points"

echo ""
echo "Step 7: Verdict - Will APRs show curves?"
echo "------------------------------------------------------------"

LEND_COUNT=$(echo "$STATS" | jq -r '.lend_count')
BORROW_COUNT=$(echo "$STATS" | jq -r '.borrow_count')

if [ "$LEND_COUNT" -gt 10 ] && [ "$BORROW_COUNT" -gt 10 ]; then
    echo "  ✅ YES! Both lend and borrow have $LEND_COUNT / $BORROW_COUNT data points"
    echo "  ✅ Sufficient historical data for smooth curves"
elif [ "$LEND_COUNT" -gt 5 ] || [ "$BORROW_COUNT" -gt 5 ]; then
    echo "  ⚠️  PARTIAL. Lend: $LEND_COUNT points, Borrow: $BORROW_COUNT points"
    echo "  ⚠️  Some curves visible but may be sparse"
else
    echo "  ❌ NO. Insufficient data points (Lend: $LEND_COUNT, Borrow: $BORROW_COUNT)"
    echo "  ❌ Will appear mostly flat or empty"
fi

echo ""
echo "================================================"
echo "TEST COMPLETE"
echo "================================================"
echo ""
echo "Summary:"
echo "  • Total markets found: $MARKET_COUNT"
echo "  • Lend APR data points: $LEND_COUNT"
echo "  • Borrow APR data points: $BORROW_COUNT"
echo "  • Lend APR range: ${LEND_RANGE:0:6}%"
echo "  • Borrow APR range: ${BORROW_RANGE:0:6}%"
echo ""
echo "Next steps:"
if [ "$LEND_COUNT" -gt 5 ]; then
    echo "  ✅ Proceed with Subgraph historical APR implementation"
    echo "  ✅ Add get_market_history() function"
    echo "  ✅ Add market_history_to_apr_series() converter"
else
    echo "  ⚠️  Subgraph may not have enough historical data yet"
    echo "  ⚠️  Consider using snapshots for now, add Subgraph later"
fi
echo ""
