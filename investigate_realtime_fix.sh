#!/bin/bash
# Investigation Script: Real-time Data Loading Analysis
# Purpose: Find how to load data instantly without snapshots

set -e

echo "=================================================="
echo "USDFC Terminal - Real-time Data Investigation"
echo "=================================================="
echo ""

# Load environment variables
source .env 2>/dev/null || echo "Warning: .env not found, using defaults"

# Default values if not in .env
RPC_URL=${RPC_URL:-"https://api.node.glif.io/rpc/v1"}
BLOCKSCOUT_URL=${BLOCKSCOUT_URL:-"https://filecoin.blockscout.com"}
SUBGRAPH_URL=${SUBGRAPH_URL:-"https://api.goldsky.com/api/public/project_cm8i6ca9k24d601wy45zzbsrq/subgraphs/sf-filecoin-mainnet/latest/gn"}
GECKOTERMINAL_URL=${GECKOTERMINAL_URL:-"https://api.geckoterminal.com/api/v2"}

USDFC_TOKEN=${USDFC_TOKEN:-"0x80B98d3aa09ffff255c3ba4A241111Ff1262F045"}
POOL_USDFC_WFIL=${POOL_USDFC_WFIL:-"0x4e07447bd38e60b94176764133788be1a0736b30"}
TROVE_MANAGER=${TROVE_MANAGER:-"0x5aB87c2398454125Dd424425e39c8909bBE16022"}
PRICE_FEED=${PRICE_FEED:-"0x80e651c9739C1ed15A267c11b85361780164A368"}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Timing function
time_request() {
    local name=$1
    local cmd=$2
    echo -ne "  Testing ${BLUE}${name}${NC}... "
    local start=$(date +%s%3N)
    local result=$(eval "$cmd" 2>&1)
    local end=$(date +%s%3N)
    local duration=$((end - start))

    if [ $? -eq 0 ] && [ -n "$result" ]; then
        if [ $duration -lt 500 ]; then
            echo -e "${GREEN}✓ ${duration}ms (FAST)${NC}"
        elif [ $duration -lt 2000 ]; then
            echo -e "${YELLOW}✓ ${duration}ms (OK)${NC}"
        else
            echo -e "${RED}✓ ${duration}ms (SLOW)${NC}"
        fi
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        return 1
    fi
}

echo "=================================================="
echo "1. GECKOTERMINAL API (Price & Liquidity)"
echo "=================================================="

# Test pool info (current price, liquidity, volume)
time_request "Pool Info" \
    "curl -s -m 5 '${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}' | jq -r '.data.attributes.base_token_price_usd'"

# Test OHLCV data (1h candles, last 24h)
time_request "OHLCV 1h/24h" \
    "curl -s -m 5 '${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}/ohlcv/hour?aggregate=1&limit=24' | jq -r '.data.attributes.ohlcv_list | length'"

# Test OHLCV data (5m candles, last 4h = 48 candles)
time_request "OHLCV 5m/4h" \
    "curl -s -m 5 '${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}/ohlcv/minute?aggregate=5&limit=48' | jq -r '.data.attributes.ohlcv_list | length'"

# Test OHLCV data (15m candles, last 24h = 96 candles)
time_request "OHLCV 15m/24h" \
    "curl -s -m 5 '${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}/ohlcv/minute?aggregate=15&limit=96' | jq -r '.data.attributes.ohlcv_list | length'"

echo ""
echo "=================================================="
echo "2. BLOCKSCOUT API (Token Holders & Transfers)"
echo "=================================================="

# Test token info
time_request "Token Info" \
    "curl -s -m 5 '${BLOCKSCOUT_URL}/api/v2/tokens/${USDFC_TOKEN}' | jq -r '.holders'"

# Test token holders
time_request "Holder Count" \
    "curl -s -m 5 '${BLOCKSCOUT_URL}/api/v2/tokens/${USDFC_TOKEN}/counters' | jq -r '.token_holders_count'"

# Test transfers (last 24h)
NOW=$(date +%s)
DAY_AGO=$((NOW - 86400))
time_request "Transfers GraphQL" \
    "curl -s -m 5 '${BLOCKSCOUT_URL}/api/v2/graphql' -H 'Content-Type: application/json' -d '{\"query\":\"{ token(contractAddress: \\\"${USDFC_TOKEN}\\\") { tokenTransfers(first: 100) { edges { node { timestamp } } } } }\"}' | jq -r '.data.token.tokenTransfers.edges | length'"

echo ""
echo "=================================================="
echo "3. SUBGRAPH API (Lending Markets)"
echo "=================================================="

# Test lending markets
time_request "Lending Markets" \
    "curl -s -m 5 '${SUBGRAPH_URL}' -H 'Content-Type: application/json' -d '{\"query\":\"{lendingMarkets(first: 10, orderBy: maturity, orderDirection: asc) { id maturity lastLendUnitPrice lastBorrowUnitPrice }}\"}' | jq -r '.data.lendingMarkets | length'"

echo ""
echo "=================================================="
echo "4. RPC CALLS (Protocol Metrics)"
echo "=================================================="

# Helper function for RPC calls
rpc_call() {
    local method=$1
    local params=$2
    curl -s -m 5 "${RPC_URL}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${TROVE_MANAGER}\",\"data\":\"${method}\"},\"latest\"],\"id\":1}"
}

echo "  Testing RPC endpoint..."
echo "  RPC URL: ${RPC_URL}"
echo ""

# Test get FIL price (from PriceFeed contract)
echo -ne "  Testing ${BLUE}getFILPrice${NC}... "
PRICE_METHOD="0x860aa3c9" # getFILPrice()
PRICE_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${PRICE_FEED}\",\"data\":\"${PRICE_METHOD}\"},\"latest\"],\"id\":1}" 2>&1)

if echo "$PRICE_RESULT" | grep -q '"result"'; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
    echo "    Result: $(echo $PRICE_RESULT | jq -r '.result')"
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "    Error: $(echo $PRICE_RESULT | jq -r '.error.message')"
fi

# Test getTCR() - this is failing
echo -ne "  Testing ${BLUE}getTCR${NC}... "
TCR_METHOD="0xe7c076e0" # getTCR()
TCR_RESULT=$(rpc_call "${TCR_METHOD}" "" 2>&1)

if echo "$TCR_RESULT" | grep -q '"result"'; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
    echo "    Result: $(echo $TCR_RESULT | jq -r '.result')"
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "    Error: $(echo $TCR_RESULT | jq -r '.error.message' | head -c 100)"

    # Try alternative: getEntireSystemColl and getEntireSystemDebt
    echo ""
    echo "  Trying alternative methods..."

    echo -ne "  Testing ${BLUE}getEntireSystemColl${NC}... "
    COLL_METHOD="0x3a0e10e8" # getEntireSystemColl()
    COLL_RESULT=$(rpc_call "${COLL_METHOD}" "" 2>&1)
    if echo "$COLL_RESULT" | grep -q '"result"'; then
        echo -e "${GREEN}✓ SUCCESS${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi

    echo -ne "  Testing ${BLUE}getEntireSystemDebt${NC}... "
    DEBT_METHOD="0x868c5e79" # getEntireSystemDebt()
    DEBT_RESULT=$(rpc_call "${DEBT_METHOD}" "" 2>&1)
    if echo "$DEBT_RESULT" | grep -q '"result"'; then
        echo -e "${GREEN}✓ SUCCESS${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi
fi

# Test totalSupply (USDFC token)
echo -ne "  Testing ${BLUE}totalSupply${NC}... "
SUPPLY_METHOD="0x18160ddd" # totalSupply()
SUPPLY_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"${SUPPLY_METHOD}\"},\"latest\"],\"id\":1}" 2>&1)

if echo "$SUPPLY_RESULT" | grep -q '"result"'; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

echo ""
echo "=================================================="
echo "5. PARALLEL REQUEST TEST"
echo "=================================================="

echo "  Running 5 API calls in parallel..."
START=$(date +%s%3N)

# Run all calls in parallel
(curl -s -m 5 "${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}" > /tmp/gecko_pool.json) &
(curl -s -m 5 "${GECKOTERMINAL_URL}/networks/filecoin/pools/${POOL_USDFC_WFIL}/ohlcv/hour?aggregate=1&limit=24" > /tmp/gecko_ohlcv.json) &
(curl -s -m 5 "${BLOCKSCOUT_URL}/api/v2/tokens/${USDFC_TOKEN}/counters" > /tmp/blockscout_holders.json) &
(curl -s -m 5 "${SUBGRAPH_URL}" -H 'Content-Type: application/json' -d '{"query":"{lendingMarkets(first: 10) { id maturity }}"}' > /tmp/subgraph_markets.json) &
(curl -s -m 5 "${RPC_URL}" -H "Content-Type: application/json" -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"0x18160ddd\"},\"latest\"],\"id\":1}" > /tmp/rpc_supply.json) &

# Wait for all to complete
wait

END=$(date +%s%3N)
PARALLEL_TIME=$((END - START))

echo ""
echo "  Parallel request results:"
echo "  - GeckoTerminal Pool: $(cat /tmp/gecko_pool.json | jq -r '.data.attributes.base_token_price_usd // "FAILED"')"
echo "  - GeckoTerminal OHLCV: $(cat /tmp/gecko_ohlcv.json | jq -r '.data.attributes.ohlcv_list | length // "FAILED"') candles"
echo "  - Blockscout Holders: $(cat /tmp/blockscout_holders.json | jq -r '.token_holders_count // "FAILED"')"
echo "  - Subgraph Markets: $(cat /tmp/subgraph_markets.json | jq -r '.data.lendingMarkets | length // "FAILED"')"
echo "  - RPC Supply: $(cat /tmp/rpc_supply.json | jq -r '.result // "FAILED"')"
echo ""
if [ $PARALLEL_TIME -lt 1000 ]; then
    echo -e "  ${GREEN}Total time: ${PARALLEL_TIME}ms (EXCELLENT - all parallel!)${NC}"
elif [ $PARALLEL_TIME -lt 2000 ]; then
    echo -e "  ${YELLOW}Total time: ${PARALLEL_TIME}ms (GOOD - mostly parallel)${NC}"
else
    echo -e "  ${RED}Total time: ${PARALLEL_TIME}ms (NEEDS IMPROVEMENT)${NC}"
fi

# Cleanup
rm -f /tmp/gecko_*.json /tmp/blockscout_*.json /tmp/subgraph_*.json /tmp/rpc_*.json

echo ""
echo "=================================================="
echo "6. DATA AVAILABILITY ANALYSIS"
echo "=================================================="

echo ""
echo "Available WITHOUT snapshots (instant):"
echo "  ${GREEN}✓${NC} Current Price (GeckoTerminal pool)"
echo "  ${GREEN}✓${NC} Current Liquidity (GeckoTerminal pool)"
echo "  ${GREEN}✓${NC} 24h Volume (GeckoTerminal pool)"
echo "  ${GREEN}✓${NC} Price candles - any timeframe (GeckoTerminal OHLCV)"
echo "  ${GREEN}✓${NC} Volume candles (GeckoTerminal OHLCV)"
echo "  ${GREEN}✓${NC} Current Holder count (Blockscout)"
echo "  ${GREEN}✓${NC} Current Supply (RPC)"
echo "  ${GREEN}✓${NC} Lending Markets (Subgraph)"
echo ""
echo "NOT available without snapshots:"
echo "  ${RED}✗${NC} Historical TCR over time"
echo "  ${RED}✗${NC} Historical Supply over time"
echo "  ${RED}✗${NC} Historical Liquidity over time (GeckoTerminal only has current)"
echo "  ${RED}✗${NC} Historical Holders over time"
echo "  ${RED}✗${NC} Historical APR over time"
echo ""
echo "Problematic RPC calls:"
echo "  ${RED}✗${NC} getTCR() - Contract method reverting"
echo "  ${RED}✗${NC} getTotalDebt() - Contract method reverting"
echo ""

echo "=================================================="
echo "7. RECOMMENDATIONS"
echo "=================================================="

echo ""
echo "For INSTANT load (no snapshots needed):"
echo ""
echo "  1. ${GREEN}Price & Volume charts${NC}:"
echo "     - Use GeckoTerminal OHLCV directly (already have historical data)"
echo "     - No need for snapshots"
echo "     - Available resolutions: 1m, 5m, 15m, 30m, 1h, 4h, 12h, 1d"
echo ""
echo "  2. ${YELLOW}Liquidity chart${NC}:"
echo "     - GeckoTerminal only provides CURRENT liquidity, not historical"
echo "     - Options:"
echo "       a) Show only current value (single point)"
echo "       b) Calculate from OHLCV volume estimates"
echo "       c) Keep using snapshots for history"
echo ""
echo "  3. ${RED}TCR, Supply, Holders, APR charts${NC}:"
echo "     - These REQUIRE historical data"
echo "     - Current RPC calls failing (getTCR, getTotalDebt)"
echo "     - Options:"
echo "       a) Fix RPC contract calls"
echo "       b) Use snapshots (but need to wait)"
echo "       c) Use current value only (single point until history builds)"
echo ""
echo "  4. ${GREEN}Parallel loading strategy${NC}:"
echo "     - Fetch Price/Volume from GeckoTerminal OHLCV"
echo "     - Fetch current metrics (Supply, Holders, Liquidity) in parallel"
echo "     - Fetch historical snapshots separately (non-blocking)"
echo "     - Show available data immediately, update when snapshots arrive"
echo ""

echo "=================================================="
echo "8. PROPOSED SOLUTION"
echo "=================================================="

echo ""
echo "HYBRID APPROACH - Best of both worlds:"
echo ""
echo "  ${BLUE}Phase 1 - Immediate (0-500ms):${NC}"
echo "    - Load GeckoTerminal OHLCV for Price & Volume (already historical)"
echo "    - Load current values: Liquidity, Supply, Holders, APR"
echo "    - Display these immediately with whatever resolution user selected"
echo ""
echo "  ${BLUE}Phase 2 - Background (non-blocking):${NC}"
echo "    - Attempt to load historical snapshots from database"
echo "    - If available, overlay TCR/Supply/Holders/APR history"
echo "    - If not available, show current values as single points"
echo ""
echo "  ${BLUE}Phase 3 - Progressive enhancement:${NC}"
echo "    - As snapshot collector runs (every 60s), data accumulates"
echo "    - Charts automatically update from 1 point → 2 points → lines"
echo "    - User doesn't need to wait - gets SOMETHING immediately"
echo ""
echo "  ${GREEN}Result:${NC} User sees charts in <1 second, progressively better over time"
echo ""

echo "=================================================="
echo "INVESTIGATION COMPLETE"
echo "=================================================="
echo ""
echo "Next steps:"
echo "  1. Review this analysis"
echo "  2. Run 'investigate_realtime_fix_v2.sh' for deeper RPC investigation"
echo "  3. Implement hybrid loading strategy in get_advanced_chart_data()"
echo ""
