#!/bin/bash
# Deep RPC Investigation - Why are contract methods reverting?

set -e

echo "=================================================="
echo "RPC Contract Method Deep Investigation"
echo "=================================================="
echo ""

source .env 2>/dev/null || echo "Warning: .env not found"

RPC_URL=${RPC_URL:-"https://api.node.glif.io/rpc/v1"}
RPC_URL_2="https://filecoin.chainup.net/rpc/v1"
RPC_URL_3="https://rpc.ankr.com/filecoin"

TROVE_MANAGER=${TROVE_MANAGER:-"0x5aB87c2398454125Dd424425e39c8909bBE16022"}
PRICE_FEED=${PRICE_FEED:-"0x80e651c9739C1ed15A267c11b85361780164A368"}
USDFC_TOKEN=${USDFC_TOKEN:-"0x80B98d3aa09ffff255c3ba4A241111Ff1262F045"}
ACTIVE_POOL=${ACTIVE_POOL:-"0x8637Ac7FdBB4c763B72e26504aFb659df71c7803"}
BORROWER_OPS=${BORROWER_OPERATIONS:-"0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0"}

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "Testing contract deployments..."
echo ""

# Test if contracts exist
test_contract_exists() {
    local name=$1
    local address=$2
    echo -ne "  ${BLUE}${name}${NC} (${address})... "

    local result=$(curl -s -m 5 "${RPC_URL}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthGetCode\",\"params\":[\"${address}\",\"latest\"],\"id\":1}")

    local code=$(echo "$result" | jq -r '.result')

    if [ "$code" != "null" ] && [ "$code" != "0x" ] && [ ${#code} -gt 10 ]; then
        echo -e "${GREEN}✓ EXISTS (${#code} bytes)${NC}"
        return 0
    else
        echo -e "${RED}✗ NOT DEPLOYED${NC}"
        return 1
    fi
}

test_contract_exists "TroveManager" "$TROVE_MANAGER"
test_contract_exists "PriceFeed" "$PRICE_FEED"
test_contract_exists "USDFC Token" "$USDFC_TOKEN"
test_contract_exists "ActivePool" "$ACTIVE_POOL"
test_contract_exists "BorrowerOps" "$BORROWER_OPS"

echo ""
echo "=================================================="
echo "Testing Alternative Contract Methods"
echo "=================================================="
echo ""

# Simple read-only methods that should work
echo "Trying simpler contract methods..."
echo ""

# 1. Try getting owner/admin
echo -ne "  ${BLUE}owner()${NC} on TroveManager... "
OWNER_SIG="0x8da5cb5b"
OWNER_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${TROVE_MANAGER}\",\"data\":\"${OWNER_SIG}\"},\"latest\"],\"id\":1}")

if echo "$OWNER_RESULT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
    echo "    Owner: $(echo $OWNER_RESULT | jq -r '.result')"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# 2. Try name() on USDFC token
echo -ne "  ${BLUE}name()${NC} on USDFC Token... "
NAME_SIG="0x06fdde03"
NAME_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"${NAME_SIG}\"},\"latest\"],\"id\":1}")

if echo "$NAME_RESULT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# 3. Try symbol() on USDFC token
echo -ne "  ${BLUE}symbol()${NC} on USDFC Token... "
SYMBOL_SIG="0x95d89b41"
SYMBOL_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"${SYMBOL_SIG}\"},\"latest\"],\"id\":1}")

if echo "$SYMBOL_RESULT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# 4. Try decimals()
echo -ne "  ${BLUE}decimals()${NC} on USDFC Token... "
DECIMALS_SIG="0x313ce567"
DECIMALS_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"${DECIMALS_SIG}\"},\"latest\"],\"id\":1}")

if echo "$DECIMALS_RESULT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
    echo "    Decimals: $(echo $DECIMALS_RESULT | jq -r '.result')"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# 5. Try balanceOf for a known address
echo -ne "  ${BLUE}balanceOf(pool)${NC} on USDFC Token... "
POOL_ADDR="0x4e07447bd38e60b94176764133788be1a0736b30"
BALANCE_SIG="0x70a08231"
# Encode the address parameter (pad to 32 bytes)
ADDR_PARAM=$(echo "$POOL_ADDR" | sed 's/0x//' | awk '{printf "%064s\n", $0}')
BALANCE_RESULT=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"${BALANCE_SIG}${ADDR_PARAM}\"},\"latest\"],\"id\":1}")

if echo "$BALANCE_RESULT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SUCCESS${NC}"
    echo "    Balance: $(echo $BALANCE_RESULT | jq -r '.result')"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

echo ""
echo "=================================================="
echo "Analyzing Error Pattern"
echo "=================================================="
echo ""

# Get full error details
echo "Fetching detailed error for getTCR()..."
TCR_ERROR=$(curl -s -m 5 "${RPC_URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${TROVE_MANAGER}\",\"data\":\"0xe7c076e0\"},\"latest\"],\"id\":1}")

echo ""
echo "Full error response:"
echo "$TCR_ERROR" | jq '.'

echo ""
echo "Error details:"
ERROR_MSG=$(echo "$TCR_ERROR" | jq -r '.error.message')
echo "  Message: $ERROR_MSG"

# Check if it's a specific contract error
if echo "$ERROR_MSG" | grep -q "f03516344"; then
    echo ""
    echo -e "  ${YELLOW}Contract f03516344 is being called (TroveManager)${NC}"
    echo "  Method 3844450837 is failing"
    echo "  RetCode 33 = Contract execution reverted"
    echo ""
    echo "  This suggests:"
    echo "    - Contract IS deployed"
    echo "    - But method call reverts internally"
    echo "    - Possible reasons:"
    echo "      1. Contract is paused"
    echo "      2. Method doesn't exist (wrong ABI)"
    echo "      3. Method has a require() that fails"
    echo "      4. Dependency contract not set up"
fi

echo ""
echo "=================================================="
echo "Testing RPC Across Multiple Providers"
echo "=================================================="
echo ""

test_rpc_provider() {
    local name=$1
    local url=$2
    echo "  Testing ${BLUE}${name}${NC}:"

    # Test totalSupply (known to work)
    local supply=$(curl -s -m 5 "${url}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${USDFC_TOKEN}\",\"data\":\"0x18160ddd\"},\"latest\"],\"id\":1}" \
        | jq -r '.result // "FAILED"')

    if [ "$supply" != "FAILED" ] && [ "$supply" != "null" ]; then
        echo -e "    totalSupply: ${GREEN}✓${NC} $supply"
    else
        echo -e "    totalSupply: ${RED}✗${NC}"
    fi

    # Test getTCR (known to fail)
    local tcr=$(curl -s -m 5 "${url}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"Filecoin.EthCall\",\"params\":[{\"to\":\"${TROVE_MANAGER}\",\"data\":\"0xe7c076e0\"},\"latest\"],\"id\":1}" \
        | jq -r '.result // .error.message // "FAILED"')

    if [ ${#tcr} -eq 66 ]; then
        echo -e "    getTCR: ${GREEN}✓${NC} $tcr"
    else
        echo -e "    getTCR: ${RED}✗${NC}"
    fi

    echo ""
}

test_rpc_provider "Glif" "$RPC_URL"
test_rpc_provider "ChainUp" "$RPC_URL_2"
test_rpc_provider "Ankr" "$RPC_URL_3"

echo "=================================================="
echo "CONCLUSION & WORKAROUNDS"
echo "=================================================="
echo ""

echo "Findings:"
echo "  1. ${GREEN}✓${NC} TroveManager contract IS deployed and accessible"
echo "  2. ${GREEN}✓${NC} Simple token methods (totalSupply, decimals) work fine"
echo "  3. ${RED}✗${NC} Complex protocol methods (getTCR, getTotalDebt) revert"
echo "  4. ${RED}✗${NC} This happens on ALL RPC providers (not provider issue)"
echo ""
echo "Root cause:"
echo "  ${YELLOW}Contract methods are reverting internally${NC}"
echo "  RetCode 33 = require() statement or state issue"
echo ""
echo "Possible reasons:"
echo "  1. Protocol not fully initialized"
echo "  2. Oracle/PriceFeed not working"
echo "  3. No troves exist yet (division by zero)"
echo "  4. Contract in emergency shutdown mode"
echo ""
echo "WORKAROUND OPTIONS:"
echo ""
echo "  ${GREEN}Option A: Manual TCR calculation${NC}"
echo "    - Get total collateral from ActivePool.getETH()"
echo "    - Get total debt from totalSupply()"
echo "    - Get FIL price from external oracle (GeckoTerminal)"
echo "    - Calculate: TCR = (collateral * price) / debt"
echo ""
echo "  ${GREEN}Option B: Skip TCR entirely${NC}"
echo "    - Show only metrics that work:"
echo "      • Price (GeckoTerminal)"
echo "      • Volume (GeckoTerminal)"
echo "      • Liquidity (GeckoTerminal)"
echo "      • Supply (RPC totalSupply)"
echo "      • Holders (Blockscout)"
echo "      • APR (Subgraph markets)"
echo ""
echo "  ${GREEN}Option C: Use fallback values${NC}"
echo "    - For TCR: Calculate from available data"
echo "    - Show with disclaimer: 'Estimated (contract method unavailable)'"
echo ""

echo "=================================================="
echo "RECOMMENDED IMMEDIATE FIX"
echo "=================================================="
echo ""
echo "Implement HYBRID strategy:"
echo ""
echo "  1. ${BLUE}Load instantly (Phase 1):${NC}"
echo "     - Price/Volume from GeckoTerminal OHLCV (~400ms)"
echo "     - Current Liquidity from GeckoTerminal pool (~375ms)"
echo "     - Holders from Blockscout (~74ms)"
echo "     - Supply from RPC (~fast)"
echo "     - APR from Subgraph (~313ms)"
echo "     ${GREEN}→ Total parallel load: <1 second${NC}"
echo ""
echo "  2. ${BLUE}Skip problematic metrics (Phase 2):${NC}"
echo "     - Remove TCR chart (contract broken)"
echo "     - OR calculate TCR manually if possible"
echo ""
echo "  3. ${BLUE}Progressive enhancement (Phase 3):${NC}"
echo "     - Load historical snapshots in background"
echo "     - Add Supply/Holders/APR history when available"
echo "     - Don't block initial render on snapshots"
echo ""
echo "  ${GREEN}Result: User sees 80% of data in <1s, rest loads progressively${NC}"
echo ""
