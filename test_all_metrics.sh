#!/bin/bash
# Master Test Script: Test All Metrics Data Availability
# Purpose: Run all metric tests and generate comprehensive report
# This validates the ENTIRE implementation approach before coding

set -e

echo "========================================================================"
echo "COMPREHENSIVE METRICS TEST - Data Validation Before Implementation"
echo "========================================================================"
echo ""
echo "This script will test ALL metrics to confirm which will show curves"
echo "vs flat lines, helping us prioritize implementation work."
echo ""
echo "Testing period: 1 week lookback"
echo "Started at: $(date)"
echo ""

# Make scripts executable
chmod +x test_tcr_calculation.sh 2>/dev/null || true
chmod +x test_apr_historical.sh 2>/dev/null || true

# Results tracking
RESULTS_FILE="/tmp/metrics_test_results.txt"
> "$RESULTS_FILE"

echo "========================================================================"
echo "TEST 1/7: Price Data (GeckoTerminal OHLCV)"
echo "========================================================================"
echo ""

# Test price data availability
POOL_ADDRESS="0x4e07447bd38e60b94176764133788be1a0736b30"

PRICE_TEST=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/${POOL_ADDRESS}/ohlcv/hour?aggregate=1&limit=168")
PRICE_COUNT=$(echo "$PRICE_TEST" | jq '.data.attributes.ohlcv_list | length' 2>/dev/null || echo "0")

if [ "$PRICE_COUNT" -gt 100 ]; then
    echo "✅ PRICE: $PRICE_COUNT candles (WILL SHOW CURVES)"
    echo "PRICE|PASS|$PRICE_COUNT|GeckoTerminal OHLCV working" >> "$RESULTS_FILE"
else
    echo "❌ PRICE: Only $PRICE_COUNT candles (INSUFFICIENT)"
    echo "PRICE|FAIL|$PRICE_COUNT|Insufficient data" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 2/7: Volume Data (from Price Candles)"
echo "========================================================================"
echo ""

# Volume is in the same OHLCV data
if [ "$PRICE_COUNT" -gt 100 ]; then
    echo "✅ VOLUME: $PRICE_COUNT data points (WILL SHOW BARS)"
    echo "VOLUME|PASS|$PRICE_COUNT|Extracted from OHLCV" >> "$RESULTS_FILE"
else
    echo "❌ VOLUME: Insufficient data"
    echo "VOLUME|FAIL|0|No OHLCV data" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 3/7: Liquidity Data"
echo "========================================================================"
echo ""

# Check current liquidity
POOL_INFO=$(curl -s "https://api.geckoterminal.com/api/v2/networks/filecoin/pools/${POOL_ADDRESS}")
LIQUIDITY=$(echo "$POOL_INFO" | jq -r '.data.attributes.reserve_in_usd // "0"')

echo "Current liquidity: \$$LIQUIDITY"
echo "Note: GeckoTerminal only provides current value, not historical"
echo ""
echo "⚠️  LIQUIDITY: Only current value available (WILL BE FLAT unless we collect snapshots)"
echo "LIQUIDITY|PARTIAL|1|Only current value, needs snapshot collection" >> "$RESULTS_FILE"

echo ""
echo "========================================================================"
echo "TEST 4/7: TCR Calculation from Price History"
echo "========================================================================"
echo ""

echo "Running dedicated TCR test script..."
if [ -f "test_tcr_calculation.sh" ]; then
    bash test_tcr_calculation.sh > /tmp/tcr_test_output.txt 2>&1

    # Extract verdict
    if grep -q "✅.*Proceed with implementation" /tmp/tcr_test_output.txt; then
        VARIATION=$(grep "Variation:" /tmp/tcr_test_output.txt | grep -oP '\d+\.\d+' | head -1)
        echo "✅ TCR: Variation ${VARIATION}% (WILL SHOW CURVES)"
        echo "TCR|PASS|168|Calculated from FIL price, variation ${VARIATION}%" >> "$RESULTS_FILE"

        # Show sample output
        echo ""
        echo "Sample TCR values:"
        grep "| FIL:" /tmp/tcr_test_output.txt | head -5
    else
        echo "❌ TCR: Insufficient variation"
        echo "TCR|FAIL|0|Calculation possible but data too flat" >> "$RESULTS_FILE"
    fi
else
    echo "⚠️  TCR test script not found, skipping detailed test"
    echo "TCR|SKIP|0|Test script not available" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 5/7: Supply Historical Data"
echo "========================================================================"
echo ""

# Check if we can get current supply
USDFC_TOKEN="0x0D0a84DA0cedE940A7E5028E52D13f0Beb5442f6"
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

SUPPLY_HEX=$(echo "$SUPPLY_RESULT" | jq -r '.result // "0x0"')

if [ "$SUPPLY_HEX" != "0x0" ] && [ -n "$SUPPLY_HEX" ]; then
    echo "✅ Current supply: Available via RPC"
    echo "Note: Historical supply requires archive node or transfer analysis"
    echo ""
    echo "⚠️  SUPPLY: Only current value (WILL BE FLAT unless calculated from transfers)"
    echo "SUPPLY|PARTIAL|1|Current value only, needs transfer analysis for history" >> "$RESULTS_FILE"
else
    echo "❌ SUPPLY: RPC call failed"
    echo "SUPPLY|FAIL|0|RPC unavailable" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 6/7: Holders Count Historical Data"
echo "========================================================================"
echo ""

# Check Blockscout holder count
HOLDERS_RESULT=$(curl -s "https://explorer.filecoin.io/api/v2/tokens/$USDFC_TOKEN")
HOLDERS_COUNT=$(echo "$HOLDERS_RESULT" | jq -r '.holders // "0"')

if [ "$HOLDERS_COUNT" != "0" ]; then
    echo "✅ Current holders: $HOLDERS_COUNT"
    echo "Note: Blockscout provides current count, historical requires time-series API"
    echo ""
    echo "⚠️  HOLDERS: Only current value (WILL BE FLAT unless Blockscout has historical API)"
    echo "HOLDERS|PARTIAL|1|Current value only, check if Blockscout has historical endpoint" >> "$RESULTS_FILE"
else
    echo "❌ HOLDERS: Blockscout API unavailable or failed"
    echo "HOLDERS|FAIL|0|API error" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 7/7: APR Historical Data from Subgraph"
echo "========================================================================"
echo ""

echo "Running dedicated APR test script..."
if [ -f "test_apr_historical.sh" ]; then
    bash test_apr_historical.sh > /tmp/apr_test_output.txt 2>&1

    # Extract verdict
    if grep -q "✅.*Proceed with Subgraph" /tmp/apr_test_output.txt; then
        LEND_COUNT=$(grep "Lend APR data points:" /tmp/apr_test_output.txt | grep -oP '\d+' | head -1)
        BORROW_COUNT=$(grep "Borrow APR data points:" /tmp/apr_test_output.txt | grep -oP '\d+' | head -1)
        echo "✅ APR: Lend $LEND_COUNT points, Borrow $BORROW_COUNT points (WILL SHOW CURVES)"
        echo "APR|PASS|$LEND_COUNT|Subgraph has historical market data" >> "$RESULTS_FILE"

        # Show sample
        echo ""
        echo "Sample APR values:"
        grep "Lend APR:" /tmp/apr_test_output.txt | head -5
    else
        echo "⚠️  APR: Limited historical data"
        echo "APR|PARTIAL|0|Subgraph may have insufficient history" >> "$RESULTS_FILE"
    fi
else
    echo "⚠️  APR test script not found, skipping detailed test"
    echo "APR|SKIP|0|Test script not available" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "TEST 8/7 (BONUS): Transfers Historical Data"
echo "========================================================================"
echo ""

# Test Blockscout GraphQL for transfer data
TRANSFERS_QUERY='{
  "query": "query { address(hash: \"'$USDFC_TOKEN'\") { transactions(first: 10) { edges { node { timestamp } } } } }"
}'

TRANSFERS_TEST=$(curl -s -X POST "https://explorer.filecoin.io/api/v2/graphql" \
  -H "Content-Type: application/json" \
  -d "$TRANSFERS_QUERY" 2>/dev/null || echo '{}')

TRANSFER_COUNT=$(echo "$TRANSFERS_TEST" | jq '.data.address.transactions.edges | length' 2>/dev/null || echo "0")

if [ "$TRANSFER_COUNT" -gt 0 ]; then
    echo "✅ TRANSFERS: Blockscout GraphQL working ($TRANSFER_COUNT sample transactions)"
    echo "Note: Can aggregate by time period for historical chart"
    echo "TRANSFERS|PASS|many|Blockscout GraphQL working" >> "$RESULTS_FILE"
else
    echo "⚠️  TRANSFERS: GraphQL query needs refinement"
    echo "TRANSFERS|PARTIAL|0|May need different query structure" >> "$RESULTS_FILE"
fi

echo ""
echo "========================================================================"
echo "FINAL RESULTS SUMMARY"
echo "========================================================================"
echo ""

echo "Metric          | Status | Data Points | Notes"
echo "----------------|--------|-------------|--------------------------------"

while IFS='|' read -r metric status count notes; do
    printf "%-15s | %-6s | %-11s | %s\n" "$metric" "$status" "$count" "$notes"
done < "$RESULTS_FILE"

echo ""
echo "========================================================================"
echo "IMPLEMENTATION PRIORITY RECOMMENDATIONS"
echo "========================================================================"
echo ""

# Count passes
PASS_COUNT=$(grep -c "|PASS|" "$RESULTS_FILE" || echo "0")
PARTIAL_COUNT=$(grep -c "|PARTIAL|" "$RESULTS_FILE" || echo "0")
FAIL_COUNT=$(grep -c "|FAIL|" "$RESULTS_FILE" || echo "0")

echo "Test Results: ✅ $PASS_COUNT passed | ⚠️  $PARTIAL_COUNT partial | ❌ $FAIL_COUNT failed"
echo ""

echo "PHASE 1 (High Priority - Will show curves immediately):"
if grep -q "^PRICE|PASS" "$RESULTS_FILE"; then
    echo "  ✅ 1. Price chart - Already working via GeckoTerminal"
fi
if grep -q "^VOLUME|PASS" "$RESULTS_FILE"; then
    echo "  ✅ 2. Volume chart - Already working via GeckoTerminal"
fi
if grep -q "^TCR|PASS" "$RESULTS_FILE"; then
    echo "  ✅ 3. TCR calculation - Implement price-based calculation"
fi

echo ""
echo "PHASE 2 (Medium Priority - Has historical data):"
if grep -q "^APR|PASS" "$RESULTS_FILE"; then
    echo "  ✅ 4. APR charts - Implement Subgraph historical query"
fi
if grep -q "^TRANSFERS|PASS" "$RESULTS_FILE"; then
    echo "  ✅ 5. Transfers - Implement Blockscout time-series aggregation"
fi

echo ""
echo "PHASE 3 (Low Priority - Requires snapshot collection):"
if grep -q "^LIQUIDITY|PARTIAL" "$RESULTS_FILE"; then
    echo "  ⏳ 6. Liquidity - Continue collecting snapshots (will improve over time)"
fi
if grep -q "^SUPPLY|PARTIAL" "$RESULTS_FILE"; then
    echo "  ⏳ 7. Supply - Continue collecting snapshots OR analyze transfers"
fi
if grep -q "^HOLDERS|PARTIAL" "$RESULTS_FILE"; then
    echo "  ⏳ 8. Holders - Continue collecting snapshots OR find Blockscout historical API"
fi

echo ""
echo "========================================================================"
echo "RECOMMENDED IMPLEMENTATION ORDER"
echo "========================================================================"
echo ""
echo "Based on test results, implement in this order:"
echo ""
echo "✅ WEEK 1 (2-3 hours):"
echo "   1. TCR from price history (30 min) - Immediate curves!"
echo "   2. APR from Subgraph (1-2 hours) - Historical market data"
echo "   3. Test and verify (30 min)"
echo ""
echo "⚠️  WEEK 2 (Optional, 2-4 hours):"
echo "   4. Transfers time-series from Blockscout (1-2 hours)"
echo "   5. Supply historical analysis (2 hours)"
echo "   6. Holders historical API (if available)"
echo ""
echo "⏳ ONGOING:"
echo "   7. Let snapshots collect for 1-2 weeks"
echo "   8. Review snapshot data quality"
echo "   9. Implement long-term archival storage"
echo ""

echo "Full test output saved to:"
echo "  - TCR test: /tmp/tcr_test_output.txt"
echo "  - APR test: /tmp/apr_test_output.txt"
echo "  - Summary: $RESULTS_FILE"
echo ""

echo "Completed at: $(date)"
echo "========================================================================"
