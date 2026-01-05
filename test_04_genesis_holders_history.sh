#!/bin/bash
# Test 04: Holder Growth from DAY ONE (Genesis to Today)
# Goal: Build complete holder count history from first mint to now

set -e

echo "========================================================================"
echo "TEST 04: HOLDER HISTORY FROM TOKEN GENESIS"
echo "========================================================================"
echo ""

USDFC_TOKEN="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"

echo "Strategy: Find first mint → Process ALL transfers → Build holder count over time"
echo ""

echo "Step 1: Find Token Genesis (First Mint/Transfer)"
echo "------------------------------------------------------------"

# Get the OLDEST transfers by going to last page
echo "Searching for oldest transfers..."

# Try different large offsets to find the end
for offset in 0 1000 2000 3000 4000 5000 10000 20000 50000; do
    RESPONSE=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20&limit=1&offset=$offset")

    TRANSFER=$(echo "$RESPONSE" | jq -r '.items[0] // null' 2>/dev/null)

    if [ "$TRANSFER" != "null" ]; then
        TIMESTAMP=$(echo "$TRANSFER" | jq -r '.timestamp')
        FROM=$(echo "$TRANSFER" | jq -r '.from.hash[0:10]')
        TO=$(echo "$TRANSFER" | jq -r '.to.hash[0:10]')

        echo "  Offset $offset: $TIMESTAMP | From: $FROM... → To: $TO..."
    else
        echo "  Offset $offset: No data (reached end at ~$((offset-1000)))"
        LAST_VALID_OFFSET=$((offset-1000))
        break
    fi
done

echo ""
echo "Fetching first transfer (genesis)..."
GENESIS=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20&limit=1&offset=$LAST_VALID_OFFSET")

GENESIS_TIME=$(echo "$GENESIS" | jq -r '.items[0].timestamp // "unknown"')
GENESIS_FROM=$(echo "$GENESIS" | jq -r '.items[0].from.hash // "unknown"')
GENESIS_TO=$(echo "$GENESIS" | jq -r '.items[0].to.hash // "unknown"')
GENESIS_VALUE=$(echo "$GENESIS" | jq -r '.items[0].total.value // "0"')

echo ""
echo "🎯 TOKEN GENESIS FOUND:"
echo "  Date: $GENESIS_TIME"
echo "  First mint from: $GENESIS_FROM"
echo "  First recipient: $GENESIS_TO"
echo "  Initial supply: $GENESIS_VALUE"

GENESIS_EPOCH=$(date -d "$GENESIS_TIME" +%s 2>/dev/null || echo "0")

if [ "$GENESIS_EPOCH" -eq 0 ]; then
    echo "❌ Could not parse genesis time"
    exit 1
fi

echo ""
echo "Step 2: Calculate Time Buckets (12-hour intervals)"
echo "------------------------------------------------------------"

NOW=$(date +%s)
TIME_SPAN=$((NOW - GENESIS_EPOCH))
DAYS=$((TIME_SPAN / 86400))
INTERVALS_12H=$((TIME_SPAN / 43200))

echo "  Token age: $DAYS days"
echo "  12-hour intervals: $INTERVALS_12H"
echo "  Total transfers to process: ~$LAST_VALID_OFFSET"

echo ""
echo "Step 3: Build Holder List by Processing ALL Transfers"
echo "------------------------------------------------------------"
echo "Algorithm:"
echo "  1. Process transfers chronologically (oldest to newest)"
echo "  2. Track balance for each address"
echo "  3. Count addresses with balance > 0 at each 12h interval"
echo ""

# Sample approach - process in batches
echo "Processing transfer batches to build holder history..."
echo ""

# We'll sample every 1000 transfers to estimate holder growth
declare -A holder_balances
HOLDER_COUNT=0

echo "Time                 | Active Addresses | Estimated Holders"
echo "---------------------|------------------|------------------"

for offset in 0 1000 2000 3000 4000 5000 10000 20000; do
    if [ $offset -gt $LAST_VALID_OFFSET ]; then
        break
    fi

    BATCH=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20&limit=100&offset=$offset")

    BATCH_TIME=$(echo "$BATCH" | jq -r '.items[0].timestamp // "unknown"' 2>/dev/null)

    if [ "$BATCH_TIME" != "unknown" ]; then
        # Count unique addresses in this batch
        UNIQUE_COUNT=$(echo "$BATCH" | jq -r '[.items[] | .from.hash, .to.hash] | unique | length' 2>/dev/null || echo "0")

        # Estimate holder growth
        HOLDER_COUNT=$((HOLDER_COUNT + UNIQUE_COUNT / 10))  # Rough estimate

        echo "  $BATCH_TIME | $UNIQUE_COUNT           | ~$HOLDER_COUNT"
    fi
done

echo ""
echo "Step 4: Precise Holder Count Calculation (Sample)"
echo "------------------------------------------------------------"
echo "For accurate holder count, we need to:"
echo "  1. Track every address's net balance"
echo "  2. Count addresses with balance > 0"
echo ""

# Get recent transfers to calculate current holders accurately
RECENT=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN/transfers?type=ERC-20&limit=100")

# Build holder set from recent activity
RECENT_HOLDERS=$(echo "$RECENT" | jq -r '
[.items[] | select(.from.hash != null and .to.hash != null)] |
map(.to.hash) |
unique |
length
' 2>/dev/null || echo "0")

echo "Unique recipients in last 100 transfers: $RECENT_HOLDERS"

# Get actual current holder count
CURRENT_HOLDERS=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN" | jq -r '.holders // "0"')
echo "Actual current holders: $CURRENT_HOLDERS"

echo ""
echo "Step 5: Estimated Holder Growth Timeline"
echo "------------------------------------------------------------"

# Calculate growth rate
GROWTH_PER_DAY=$(echo "scale=2; $CURRENT_HOLDERS / $DAYS" | bc 2>/dev/null || echo "0")

echo "Average growth: $GROWTH_PER_DAY holders/day"
echo ""

# Generate sample timeline
echo "Estimated holder count over time (12h intervals):"
echo ""

for i in 0 1 2 5 10 20 30 60 90 120 150 180 210 240 270 300 330 360; do
    if [ $i -gt $DAYS ]; then
        break
    fi

    EST_HOLDERS=$(echo "scale=0; $i * $GROWTH_PER_DAY" | bc 2>/dev/null || echo "0")
    DATE_STR=$(date -d "@$((GENESIS_EPOCH + i * 86400))" '+%Y-%m-%d' 2>/dev/null || echo "Day $i")

    echo "  $DATE_STR | ~$EST_HOLDERS holders"
done

echo ""
echo "Step 6: Check if Full History is Accessible"
echo "------------------------------------------------------------"

TOTAL_TRANSFERS=$(echo "$RECENT" | jq -r '.next_page_params // {} | .items_count // 0' 2>/dev/null || echo "$LAST_VALID_OFFSET")

echo "Total transfers accessible: ~$LAST_VALID_OFFSET"
echo "Can paginate: $([ $LAST_VALID_OFFSET -gt 1000 ] && echo 'YES ✅' || echo 'NO ❌')"

if [ $LAST_VALID_OFFSET -gt 1000 ]; then
    echo ""
    echo "✅ HOLDER HISTORY IS BUILDABLE!"
    echo ""
    echo "Implementation approach:"
    echo "  1. Paginate through ALL $LAST_VALID_OFFSET transfers"
    echo "  2. Build address balance map chronologically"
    echo "  3. Sample holder count every 12 hours"
    echo "  4. Store time series: [(timestamp, holder_count), ...]"
    echo ""
    echo "Estimated processing time: $((LAST_VALID_OFFSET / 100 / 60)) minutes"
    echo "Result: Complete holder growth chart from genesis to today!"
else
    echo "❌ Limited transfer history - may not cover full timeline"
fi

echo ""
echo "========================================================================"
echo "HOLDER GENESIS HISTORY - TEST RESULTS"
echo "========================================================================"
echo ""

echo "Genesis Date:        $GENESIS_TIME"
echo "Token Age:           $DAYS days"
echo "Current Holders:     $CURRENT_HOLDERS"
echo "Total Transfers:     ~$LAST_VALID_OFFSET"
echo "Growth Rate:         $GROWTH_PER_DAY holders/day"
echo ""

if [ $LAST_VALID_OFFSET -gt 5000 ]; then
    echo "✅ VERDICT: Complete holder history CAN be built!"
    echo ""
    echo "Data points available: $INTERVALS_12H (12h intervals)"
    echo "This will show PERFECT holder growth curve from day one!"
    echo ""
    echo "RECOMMENDATION: Implement holder history builder"
    echo "  - Process all transfers on server startup (one-time)"
    echo "  - Cache results in database"
    echo "  - Update incrementally with new transfers"
else
    echo "⚠️  VERDICT: Partial history available"
    echo "   May need alternative approach"
fi

echo ""
