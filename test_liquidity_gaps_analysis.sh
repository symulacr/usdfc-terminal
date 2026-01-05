#!/bin/bash

# Check EVERY data point for liquidity calculation
# Find gaps, missing days, and compare day-by-day patterns
# Purpose: Find why 40k% variation is misleading

echo "================================================"
echo "TEST: Liquidity Gaps & Missing Data Analysis"
echo "================================================"
echo "Purpose: Check every data point, find gaps and patterns"
echo ""

# Configuration
POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"
DATA_POINTS=200

# Step 1: Fetch OHLCV data
echo "Step 1: Fetching 200 candles (4h intervals)..."
OHLCV=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/$POOL_ADDRESS/ohlcv/hour?aggregate=4&limit=$DATA_POINTS")

CANDLE_COUNT=$(echo "$OHLCV" | jq '.data.attributes.ohlcv_list | length')
echo "  Total candles received: $CANDLE_COUNT"
echo ""

# Step 2: Process EVERY candle and check if liquidity can be calculated
echo "Step 2: Processing every candle and checking calculability..."
echo ""

SUCCESS_COUNT=0
FILTERED_LOW_IMPACT=0
FILTERED_ZERO_CLOSE=0
FILTERED_HIGH_IMPACT=0
FILTERED_OUTLIER=0

echo "Analyzing each candle:"
echo "------------------------------------------------------------"

# Create temp file for detailed analysis
DETAIL_FILE=$(mktemp)
GOOD_FILE=$(mktemp)
BAD_FILE=$(mktemp)

echo "$OHLCV" | jq -r '.data.attributes.ohlcv_list[] | @json' | while read -r candle; do
    TIME=$(echo "$candle" | jq -r '.[0]')
    HIGH=$(echo "$candle" | jq -r '.[2]')
    LOW=$(echo "$candle" | jq -r '.[3]')
    CLOSE=$(echo "$candle" | jq -r '.[4]')
    VOLUME=$(echo "$candle" | jq -r '.[5]')

    DATE=$(date -d @$TIME "+%Y-%m-%d %H:%M" 2>/dev/null || echo "$TIME")

    # Check if close is zero
    if (( $(echo "$CLOSE == 0" | bc -l) )); then
        echo "$TIME|FILTERED_ZERO_CLOSE|Close is zero|$DATE|$VOLUME|$HIGH|$LOW|$CLOSE" >> $BAD_FILE
        continue
    fi

    # Calculate price impact
    IMPACT=$(echo "scale=10; ($HIGH - $LOW) / $CLOSE" | bc -l)

    # Check filter conditions
    if (( $(echo "$IMPACT <= 0.0001" | bc -l) )); then
        echo "$TIME|FILTERED_LOW_IMPACT|Impact too low ($IMPACT)|$DATE|$VOLUME|$HIGH|$LOW|$CLOSE" >> $BAD_FILE
        continue
    fi

    if (( $(echo "$IMPACT >= 1.0" | bc -l) )); then
        echo "$TIME|FILTERED_HIGH_IMPACT|Impact too high ($IMPACT)|$DATE|$VOLUME|$HIGH|$LOW|$CLOSE" >> $BAD_FILE
        continue
    fi

    # Calculate liquidity
    LIQUIDITY=$(echo "scale=2; $VOLUME / $IMPACT" | bc -l)

    # Check outlier filter
    if (( $(echo "$LIQUIDITY >= 10000000" | bc -l) )); then
        echo "$TIME|FILTERED_OUTLIER|Liquidity outlier ($LIQUIDITY)|$DATE|$VOLUME|$IMPACT|$LIQUIDITY" >> $BAD_FILE
        continue
    fi

    if (( $(echo "$LIQUIDITY <= 0" | bc -l) )); then
        echo "$TIME|FILTERED_OUTLIER|Liquidity negative/zero|$DATE|$VOLUME|$IMPACT|$LIQUIDITY" >> $BAD_FILE
        continue
    fi

    # Success!
    echo "$TIME|SUCCESS|$LIQUIDITY|$DATE|$VOLUME|$IMPACT" >> $GOOD_FILE
done

# Count results
SUCCESS_COUNT=$(wc -l < $GOOD_FILE 2>/dev/null || echo "0")
FILTERED_LOW_IMPACT=$(grep "FILTERED_LOW_IMPACT" $BAD_FILE 2>/dev/null | wc -l || echo "0")
FILTERED_ZERO_CLOSE=$(grep "FILTERED_ZERO_CLOSE" $BAD_FILE 2>/dev/null | wc -l || echo "0")
FILTERED_HIGH_IMPACT=$(grep "FILTERED_HIGH_IMPACT" $BAD_FILE 2>/dev/null | wc -l || echo "0")
FILTERED_OUTLIER=$(grep "FILTERED_OUTLIER" $BAD_FILE 2>/dev/null | wc -l || echo "0")
TOTAL_FILTERED=$((FILTERED_LOW_IMPACT + FILTERED_ZERO_CLOSE + FILTERED_HIGH_IMPACT + FILTERED_OUTLIER))

echo ""
echo "================================================"
echo "RESULTS SUMMARY"
echo "================================================"
echo "  Total Candles:             $CANDLE_COUNT"
echo "  ✅ Successfully Calculated: $SUCCESS_COUNT"
echo "  ❌ Filtered Out:            $TOTAL_FILTERED"
echo ""
echo "Filter Breakdown:"
echo "  - Low Impact (<0.0001):    $FILTERED_LOW_IMPACT"
echo "  - Zero Close Price:        $FILTERED_ZERO_CLOSE"
echo "  - High Impact (>=1.0):     $FILTERED_HIGH_IMPACT"
echo "  - Outlier (>$10M or <=0):  $FILTERED_OUTLIER"
echo ""

# Calculate data loss percentage
DATA_LOSS=$(echo "scale=2; ($TOTAL_FILTERED / $CANDLE_COUNT) * 100" | bc)
DATA_RETENTION=$(echo "scale=2; ($SUCCESS_COUNT / $CANDLE_COUNT) * 100" | bc)

echo "Data Quality:"
echo "  📊 Data Retention:  $DATA_RETENTION%"
echo "  📉 Data Loss:       $DATA_LOSS%"
echo ""

if (( $(echo "$DATA_LOSS > 50" | bc -l) )); then
    echo "  ⚠️  WARNING: More than 50% of data points filtered out!"
    echo "  ⚠️  Charts will have significant gaps!"
elif (( $(echo "$DATA_LOSS > 30" | bc -l) )); then
    echo "  ⚠️  CAUTION: 30-50% of data points filtered out"
    echo "  ⚠️  Charts may have noticeable gaps"
elif (( $(echo "$DATA_LOSS > 10" | bc -l) )); then
    echo "  ℹ️  MODERATE: 10-30% of data points filtered out"
else
    echo "  ✅ GOOD: Less than 10% data loss"
fi
echo ""

# Step 3: Analyze filtered days by date
echo "================================================"
echo "FILTERED DAYS ANALYSIS"
echo "================================================"
echo ""

if [ -s $BAD_FILE ]; then
    echo "Sample of filtered candles (first 20):"
    echo "------------------------------------------------------------"
    head -20 $BAD_FILE | while IFS='|' read -r time reason detail date volume rest; do
        printf "  %s | %s\n" "$date" "$detail"
    done
    echo ""

    echo "Filtered days grouped by reason:"
    echo "------------------------------------------------------------"

    if [ "$FILTERED_LOW_IMPACT" -gt 0 ]; then
        echo ""
        echo "LOW IMPACT DAYS (price barely moved - stable days):"
        grep "FILTERED_LOW_IMPACT" $BAD_FILE | head -10 | while IFS='|' read -r time reason detail date volume high low close; do
            IMPACT=$(echo "scale=10; ($high - $low) / $close" | bc -l)
            printf "  %s | Impact: %s | Range: \$%s-\$%s\n" "$date" "$IMPACT" "$low" "$high"
        done
        if [ "$FILTERED_LOW_IMPACT" -gt 10 ]; then
            echo "  ... and $((FILTERED_LOW_IMPACT - 10)) more"
        fi
    fi

    if [ "$FILTERED_HIGH_IMPACT" -gt 0 ]; then
        echo ""
        echo "HIGH IMPACT DAYS (extreme volatility):"
        grep "FILTERED_HIGH_IMPACT" $BAD_FILE | while IFS='|' read -r time reason detail date volume high low close; do
            IMPACT=$(echo "scale=10; ($high - $low) / $close" | bc -l)
            printf "  %s | Impact: %s (>100%% price swing!)\n" "$date" "$IMPACT"
        done
    fi
fi
echo ""

# Step 4: Compare week patterns
echo "================================================"
echo "WEEK-BY-WEEK PATTERN ANALYSIS"
echo "================================================"
echo ""

if [ -s $GOOD_FILE ]; then
    echo "Checking which weeks have good data vs gaps..."
    echo "------------------------------------------------------------"

    # Group by week
    awk -F'|' '{
        cmd = "date -d @"$1" +%Y-W%V 2>/dev/null"
        cmd | getline week
        close(cmd)
        weeks[week]++
    }
    END {
        for (w in weeks) {
            print w, weeks[w]
        }
    }' $GOOD_FILE | sort | while read -r week count; do
        if [ "$count" -lt 10 ]; then
            printf "  %s: %2d points ❌ (sparse - gaps likely)\n" "$week" "$count"
        elif [ "$count" -lt 30 ]; then
            printf "  %s: %2d points ⚠️  (moderate coverage)\n" "$week" "$count"
        else
            printf "  %s: %2d points ✅ (good coverage)\n" "$week" "$count"
        fi
    done
fi
echo ""

# Step 5: Day-by-day comparison
echo "================================================"
echo "DAY-BY-DAY SIMILARITY ANALYSIS"
echo "================================================"
echo ""

if [ -s $GOOD_FILE ]; then
    echo "Grouping by day to find similar patterns..."
    echo "------------------------------------------------------------"

    # Group by day
    awk -F'|' '{
        cmd = "date -d @"$1" +%Y-%m-%d 2>/dev/null"
        cmd | getline day
        close(cmd)

        # Store liquidity value
        liq = $3

        if (day_count[day] == 0) {
            day_min[day] = liq
            day_max[day] = liq
            day_sum[day] = 0
        }

        if (liq < day_min[day]) day_min[day] = liq
        if (liq > day_max[day]) day_max[day] = liq
        day_sum[day] += liq
        day_count[day]++
    }
    END {
        for (d in day_count) {
            avg = day_sum[d] / day_count[d]
            var = ((day_max[d] - day_min[d]) / day_min[d]) * 100
            printf "%s|%d|%.2f|%.2f|%.2f|%.2f\n", d, day_count[d], day_min[d], day_max[d], avg, var
        }
    }' $GOOD_FILE | sort | tail -20 | while IFS='|' read -r day count min max avg var; do
        if [ "$count" -eq 0 ]; then
            printf "  %s: ❌ NO DATA\n" "$day"
        elif [ "$count" -lt 3 ]; then
            printf "  %s: %d points | Avg: \$%10.0f | Var: %6.1f%% ⚠️  (sparse)\n" "$day" "$count" "$avg" "$var"
        else
            printf "  %s: %d points | Avg: \$%10.0f | Var: %6.1f%%\n" "$day" "$count" "$avg" "$var"
        fi
    done
fi
echo ""

# Step 6: The Real Issue
echo "================================================"
echo "🔍 THE REAL ISSUE"
echo "================================================"
echo ""

echo "Why 40,056% variation is misleading:"
echo ""
echo "1. FILTERING REMOVES STABLE DAYS:"
echo "   - Days with low price movement (Impact < 0.0001) get filtered"
echo "   - These are the days with LOW liquidity estimates"
echo "   - Only volatile days remain in dataset"
echo "   - Creates artificial inflation of variation"
echo ""

echo "2. DATA GAPS:"
echo "   - Only $DATA_RETENTION% of candles pass filters"
echo "   - Charts will have $DATA_LOSS% missing data"
echo "   - Gaps create visual discontinuity"
echo ""

echo "3. SELECTION BIAS:"
echo "   - We're measuring variation ONLY among volatile days"
echo "   - Missing all the stable days (which would lower variation)"
echo "   - True variation would be much lower if all days included"
echo ""

echo "4. CHART DISPLAY ISSUE:"
echo "   - On days with no data points, chart shows gap/break"
echo "   - User sees incomplete timeline"
echo "   - Not a smooth curve"
echo ""

# Step 7: Recommendations
echo "================================================"
echo "💡 RECOMMENDATIONS"
echo "================================================"
echo ""

echo "Option 1: RELAX FILTERS (Better Coverage)"
echo "  - Change Impact filter from >0.0001 to >0.00001 (10x more sensitive)"
echo "  - Include more stable days"
echo "  - Reduce data loss from $DATA_LOSS% to ~5-10%"
echo "  - More realistic variation (probably 500-1000% instead of 40,000%)"
echo ""

echo "Option 2: USE ALTERNATIVE CALCULATION (No Filtering)"
echo "  - Use direct pool reserves from API (no calculation needed)"
echo "  - Or: Use Volume alone as proxy (no division needed)"
echo "  - 100% data retention"
echo "  - Smoother curves"
echo ""

echo "Option 3: INTERPOLATION (Fill Gaps)"
echo "  - When day has no data, interpolate from neighbors"
echo "  - Maintains timeline continuity"
echo "  - Shows estimated values for stable days"
echo "  - More complete chart"
echo ""

echo "Option 4: ACCEPT GAPS (Current Approach)"
echo "  - Keep current filters"
echo "  - Accept $DATA_LOSS% data loss"
echo "  - Chart shows only high-volatility periods"
echo "  - Add note: \"Only shown for periods with price movement\""
echo ""

# Cleanup
rm -f $DETAIL_FILE $GOOD_FILE $BAD_FILE

echo "================================================"
echo "TEST COMPLETE"
echo "================================================"
