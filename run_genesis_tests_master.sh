#!/bin/bash
# MASTER GENESIS TEST RUNNER
# Systematically test if we can build complete history from DAY ONE

set -e

RESULTS_DIR="/tmp/genesis_tests"
mkdir -p "$RESULTS_DIR"

echo "========================================================================"
echo "GENESIS-TO-TODAY TESTING - COMPLETE HISTORICAL DATA"
echo "========================================================================"
echo ""
echo "Goal: Build COMPLETE history for all metrics from token genesis"
echo "      No snapshots needed - use blockchain data from day one!"
echo ""
echo "Started: $(date)"
echo ""

# Make all scripts executable
chmod +x test_04_genesis_holders_history.sh 2>/dev/null
chmod +x test_05_genesis_supply_history.sh 2>/dev/null
chmod +x test_06_genesis_tcr_history.sh 2>/dev/null

echo "========================================================================"
echo "TEST 04: Holder Count from Genesis"
echo "========================================================================"
echo ""

if [ -f "test_04_genesis_holders_history.sh" ]; then
    ./test_04_genesis_holders_history.sh > "$RESULTS_DIR/holders_genesis.log" 2>&1

    # Extract key findings
    echo "Key Findings:"
    grep "TOKEN GENESIS FOUND" "$RESULTS_DIR/holders_genesis.log" || echo "  Genesis: (check log)"
    grep "Token age:" "$RESULTS_DIR/holders_genesis.log" || true
    grep "Current Holders:" "$RESULTS_DIR/holders_genesis.log" || true
    grep "Total transfers:" "$RESULTS_DIR/holders_genesis.log" || true

    echo ""
    if grep -q "✅.*Complete holder history CAN be built" "$RESULTS_DIR/holders_genesis.log"; then
        HOLDERS_RESULT="✅ BUILDABLE - Can reconstruct from transfers"
        HOLDERS_DATAPOINTS=$(grep "12-hour intervals:" "$RESULTS_DIR/holders_genesis.log" | grep -oP '\d+' | head -1)
    else
        HOLDERS_RESULT="⚠️  LIMITED - Partial history available"
        HOLDERS_DATAPOINTS="unknown"
    fi
else
    HOLDERS_RESULT="❌ SKIP - Test not found"
    HOLDERS_DATAPOINTS="0"
fi

echo "Result: $HOLDERS_RESULT"
echo "Data points: $HOLDERS_DATAPOINTS"
echo ""

echo "========================================================================"
echo "TEST 05: Supply from Genesis"
echo "========================================================================"
echo ""

if [ -f "test_05_genesis_supply_history.sh" ]; then
    ./test_05_genesis_supply_history.sh > "$RESULTS_DIR/supply_genesis.log" 2>&1

    echo "Key Findings:"
    grep "Mint events found:" "$RESULTS_DIR/supply_genesis.log" || true
    grep "Burn events found:" "$RESULTS_DIR/supply_genesis.log" || true
    grep "Historical variation:" "$RESULTS_DIR/supply_genesis.log" || true

    echo ""
    if grep -q "Supply is CONSTANT from genesis" "$RESULTS_DIR/supply_genesis.log"; then
        SUPPLY_RESULT="✅ CONSTANT - No variation (correct)"
        SUPPLY_VARIATION="0%"
    elif grep -q "Supply MAY vary" "$RESULTS_DIR/supply_genesis.log"; then
        SUPPLY_RESULT="⚠️  VARIABLE - Needs event processing"
        SUPPLY_VARIATION="TBD"
    else
        SUPPLY_RESULT="⚠️  UNKNOWN - Check logs"
        SUPPLY_VARIATION="unknown"
    fi
else
    SUPPLY_RESULT="❌ SKIP - Test not found"
    SUPPLY_VARIATION="unknown"
fi

echo "Result: $SUPPLY_RESULT"
echo "Variation: $SUPPLY_VARIATION"
echo ""

echo "========================================================================"
echo "TEST 06: TCR from Genesis"
echo "========================================================================"
echo ""

if [ -f "test_06_genesis_tcr_history.sh" ]; then
    ./test_06_genesis_tcr_history.sh > "$RESULTS_DIR/tcr_genesis.log" 2>&1

    echo "Key Findings:"
    grep "Price data available:" "$RESULTS_DIR/tcr_genesis.log" || true
    grep "Coverage:" "$RESULTS_DIR/tcr_genesis.log" || true
    grep "Variation:" "$RESULTS_DIR/tcr_genesis.log" | tail -1 || true

    echo ""
    if grep -q "✅.*Can build TCR history" "$RESULTS_DIR/tcr_genesis.log"; then
        TCR_RESULT="✅ BUILDABLE - From price history"
        TCR_DAYS=$(grep "Price data available:" "$RESULTS_DIR/tcr_genesis.log" | grep -oP '\d+\.\d+' | head -1)
        TCR_VARIATION=$(grep "Variation:" "$RESULTS_DIR/tcr_genesis.log" | tail -1 | grep -oP '\d+\.\d+' | head -1)
    else
        TCR_RESULT="⚠️  LIMITED - Partial history"
        TCR_DAYS="unknown"
        TCR_VARIATION="unknown"
    fi
else
    TCR_RESULT="❌ SKIP - Test not found"
    TCR_DAYS="0"
    TCR_VARIATION="0"
fi

echo "Result: $TCR_RESULT"
echo "Days available: $TCR_DAYS"
echo "Variation: $TCR_VARIATION%"
echo ""

echo "========================================================================"
echo "CONSOLIDATED GENESIS RESULTS"
echo "========================================================================"
echo ""

echo "Metric    | Genesis History | Data Points/Coverage | Solution"
echo "----------|-----------------|----------------------|------------------"
echo "Holders   | $HOLDERS_RESULT | $HOLDERS_DATAPOINTS intervals     |"
echo "Supply    | $SUPPLY_RESULT | $SUPPLY_VARIATION variation      |"
echo "TCR       | $TCR_RESULT | $TCR_DAYS days       |"
echo ""

echo "========================================================================"
echo "IMPLEMENTATION FEASIBILITY"
echo "========================================================================"
echo ""

# Count buildable metrics
BUILDABLE=0
CONSTANT=0
LIMITED=0

[[ "$HOLDERS_RESULT" == *"BUILDABLE"* ]] && ((BUILDABLE++))
[[ "$SUPPLY_RESULT" == *"CONSTANT"* ]] && ((CONSTANT++))
[[ "$TCR_RESULT" == *"BUILDABLE"* ]] && ((BUILDABLE++))

[[ "$HOLDERS_RESULT" == *"LIMITED"* ]] && ((LIMITED++))
[[ "$SUPPLY_RESULT" == *"VARIABLE"* ]] && ((LIMITED++))
[[ "$TCR_RESULT" == *"LIMITED"* ]] && ((LIMITED++))

TOTAL=3
SUCCESS=$((BUILDABLE + CONSTANT))

echo "Metrics with genesis history: $BUILDABLE"
echo "Metrics correctly constant:   $CONSTANT"
echo "Metrics needing more work:    $LIMITED"
echo ""

SUCCESS_RATE=$(echo "scale=0; $SUCCESS * 100 / $TOTAL" | bc)

echo "Success rate: $SUCCESS_RATE%"
echo ""

if [ $BUILDABLE -gt 0 ]; then
    echo "✅ CAN BUILD GENESIS HISTORY FOR:"
    [[ "$HOLDERS_RESULT" == *"BUILDABLE"* ]] && echo "   - Holders (from transfer events)"
    [[ "$TCR_RESULT" == *"BUILDABLE"* ]] && echo "   - TCR (from price history)"
fi

if [ $CONSTANT -gt 0 ]; then
    echo ""
    echo "✅ CORRECTLY CONSTANT (no history needed):"
    [[ "$SUPPLY_RESULT" == *"CONSTANT"* ]] && echo "   - Supply (flat from genesis)"
fi

if [ $LIMITED -gt 0 ]; then
    echo ""
    echo "⚠️  NEEDS MORE ANALYSIS:"
    [[ "$HOLDERS_RESULT" == *"LIMITED"* ]] && echo "   - Holders"
    [[ "$SUPPLY_RESULT" == *"VARIABLE"* ]] && echo "   - Supply"
    [[ "$TCR_RESULT" == *"LIMITED"* ]] && echo "   - TCR"
fi

echo ""
echo "========================================================================"
echo "RECOMMENDED IMPLEMENTATION STRATEGY"
echo "========================================================================"
echo ""

if [ $BUILDABLE -gt 0 ] || [ $CONSTANT -gt 0 ]; then
    echo "✅ Genesis-to-today history IS POSSIBLE!"
    echo ""

    if [[ "$TCR_RESULT" == *"BUILDABLE"* ]]; then
        echo "1. TCR from Price History"
        echo "   - Use GeckoTerminal OHLCV ($TCR_DAYS days available)"
        echo "   - Calculate: TCR = (Collateral × FIL_Price) / Supply × 100"
        echo "   - Variation: $TCR_VARIATION%"
        echo "   - Implementation: 30 minutes"
        echo ""
    fi

    if [[ "$HOLDERS_RESULT" == *"BUILDABLE"* ]]; then
        echo "2. Holders from Transfer Events"
        echo "   - Process all transfers chronologically"
        echo "   - Track balance for each address"
        echo "   - Count holders at 12h intervals"
        echo "   - Implementation: 2-3 hours (one-time processing)"
        echo "   - Cache results for instant access"
        echo ""
    fi

    if [[ "$SUPPLY_RESULT" == *"CONSTANT"* ]]; then
        echo "3. Supply - Accept Constant Value"
        echo "   - No mint/burn events"
        echo "   - Supply set at genesis, never changed"
        echo "   - Flat line is CORRECT"
        echo "   - Implementation: None needed (already accurate)"
        echo ""
    fi
fi

echo "========================================================================"
echo "NEXT STEPS"
echo "========================================================================"
echo ""

echo "1. Review detailed logs in: $RESULTS_DIR/"
echo "2. Decide which metrics to implement"
echo "3. For buildable metrics: Implement history builders"
echo "4. For constant metrics: Accept current behavior"
echo ""

echo "Detailed logs:"
echo "  - Holders: $RESULTS_DIR/holders_genesis.log"
echo "  - Supply:  $RESULTS_DIR/supply_genesis.log"
echo "  - TCR:     $RESULTS_DIR/tcr_genesis.log"
echo ""

echo "Completed: $(date)"
echo "========================================================================"
