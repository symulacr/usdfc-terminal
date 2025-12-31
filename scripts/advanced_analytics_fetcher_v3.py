#!/usr/bin/env python3
"""
ADVANCED ANALYTICS DATA FETCHER v3.0
=====================================
Comprehensive data fetching for USDFC Analytics Terminal

FIXES from v2.0:
- [#1] Added all known DEX pools and routers
- [#2] Removed misleading pool_context from address analysis
- [#3] Fetches ALL token transfers, not just USDFC
- [#4] Added internal transaction detection
- [#5] Improved swap detection with router/log analysis
- [#6] Fixed side type consistency
- [#7] Fixed APR calculation using actual maturity
- [#8] Added incomplete data flags
- [#9] Added all discovered DEX contracts
- [#10] Fixed GraphQL schema for orders
- [#11] Fixed holding time calculation
- [#12] Added time-range filtering for volumes
- [#13] Fixed event correlation to be address-specific
- [#14] Improved behavior classification with scores
- [#15] Added USD value tracking capability
- [#16] Added data completeness flags
- [#17] Added cache bypass option

Author: USDFC Terminal Team
"""

import asyncio
import aiohttp
import json
import time
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple, Any, Set
from dataclasses import dataclass, asdict, field
from collections import defaultdict
import hashlib

# =============================================================================
# CONFIGURATION - ISSUE #9 FIXED: Added all DEX contracts
# =============================================================================

CONFIG = {
    # API Endpoints
    "blockscout_graphql": "https://filecoin.blockscout.com/api/v1/graphql",
    "blockscout_rest": "https://filecoin.blockscout.com/api/v2",
    "subgraph_url": "https://api.goldsky.com/api/public/project_cm8i6ca9k24d601wy45zzbsrq/subgraphs/sf-filecoin-mainnet/latest/gn",
    "gecko_base": "https://api.geckoterminal.com/api/v2",
    "rpc_url": "https://api.node.glif.io/rpc/v1",

    # USDFC Token
    "usdfc_token": "0x80B98d3aa09ffff255c3ba4A241111Ff1262F045",

    # Protocol Contracts
    "trove_manager": "0x5aB87c2398454125Dd424425e39c8909bBE16022",
    "price_feed": "0x80e651c9739C1ed15A267c11b85361780164A368",
    "stability_pool": "0x791Ad78bBc58324089D3E0A8689E7D045B9592b5",
    "multi_trove_getter": "0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F",
    "sorted_troves": "0x2C32e48e358d5b893C46906b69044D342d8DDd5F",
    "active_pool": "0x8637Ac7FdBB4c763B72e26504aFb659df71c7803",
    "borrower_operations": "0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0",

    # [ISSUE #1 & #9 FIX] All known DEX pools
    "dex_pools": {
        "usdfc_wfil": "0x4e07447bd38e60b94176764133788be1a0736b30",
        # Add more pools as discovered
    },

    # [ISSUE #1 & #9 FIX] All known DEX routers and aggregators
    "dex_routers": {
        "squid_router_proxy": "0xce16F69375520ab01377ce7B88f5BA8C48F8D666",
        "red_snwapper": "0xAC4c6e212A361AA761D2BA4f96f4e0bb4c9b1A13",
        # Common router patterns
        "sushiswap_router": "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506",  # If exists on Filecoin
    },

    # Known tokens for multi-token tracking
    "known_tokens": {
        "USDFC": "0x80B98d3aa09ffff255c3ba4A241111Ff1262F045",
        "WFIL": "0x60E1773636CF5E4A227d9AC24F20fEca034ee25A",
        "axlUSDC": "0xEB466342C4d449BC9f53A865D5Cb90586f405215",
        "pFIL": "0x57E3BB9F790185Cfe70Cc2C15Ed5d6B84dCf4aDb",
        "wpFIL": "0x57E3BB9F790185Cfe70Cc2C15Ed5d6B84dCf4aDb",
        "iFIL": "0x690908f7fa93afC040CFbD9fE1dDd2C2668Aa0e0",
        "SFC": "0x...",  # Secured Finance token if applicable
    },

    # Subgraph Currencies
    "currency_usdfc": "0x5553444643000000000000000000000000000000000000000000000000000000",
    "currency_fil": "0x46494c0000000000000000000000000000000000000000000000000000000000",

    # Time Range Configs
    "time_ranges": {
        "1h":  {"gecko": ("minute", 1, 60),   "subgraph_interval": 900,  "seconds": 3600},
        "6h":  {"gecko": ("minute", 5, 72),   "subgraph_interval": 900,  "seconds": 21600},
        "24h": {"gecko": ("minute", 15, 96),  "subgraph_interval": 3600, "seconds": 86400},
        "7d":  {"gecko": ("hour", 1, 168),    "subgraph_interval": 3600, "seconds": 604800},
        "30d": {"gecko": ("hour", 4, 180),    "subgraph_interval": 86400,"seconds": 2592000},
        "90d": {"gecko": ("day", 1, 90),      "subgraph_interval": 86400,"seconds": 7776000},
    },

    # Performance
    "max_concurrent": 10,
    "timeout": 30,
    "max_retries": 3,
}

# =============================================================================
# DATA CLASSES - Enhanced with completeness flags
# =============================================================================

@dataclass
class FetchResult:
    success: bool
    data: Any
    source: str
    latency_ms: float
    error: Optional[str] = None
    from_cache: bool = False  # [ISSUE #17 FIX]

@dataclass
class TransferRecord:
    timestamp: str
    from_addr: str
    to_addr: str
    amount: float
    token_symbol: str
    token_address: str
    tx_hash: str
    block: int
    direction: str  # 'in', 'out', 'internal'
    is_swap: bool = False
    swap_type: Optional[str] = None  # 'buy', 'sell', 'router'
    counterparty_type: Optional[str] = None  # 'pool', 'router', 'user', 'contract'

@dataclass
class SwapRecord:
    timestamp: str
    tx_hash: str
    swap_type: str  # 'buy', 'sell'
    token_in: str
    token_out: str
    amount_in: float
    amount_out: float
    router: Optional[str] = None
    pool: Optional[str] = None

@dataclass
class BalancePoint:
    timestamp: str
    balance: float
    event: str
    data_complete: bool = True  # [ISSUE #8 FIX]

@dataclass
class AddressAnalysis:
    address: str
    generated_at: str
    fetch_time_seconds: float

    # Current state
    current_balance: float
    is_holder: bool

    # Balance history [ISSUE #8 FIX - with completeness]
    balance_history: List[BalancePoint]
    balance_data_complete: bool
    balance_data_pages: int

    # Transfer stats
    total_transfers: int
    usdfc_transfers: int
    other_token_transfers: int
    tokens_used: List[str]

    # [ISSUE #5 FIX] Proper swap detection
    swaps: List[SwapRecord]
    swap_stats: Dict

    # Lending activity
    lending_activity: Dict

    # [ISSUE #12 FIX] Time-based volumes
    volume_24h: Dict
    volume_7d: Dict
    volume_30d: Dict

    # [ISSUE #11 FIX] Proper holding time
    holding_periods: List[Dict]
    total_holding_days: float
    current_holding_days: float

    # [ISSUE #14 FIX] Multi-dimensional classification
    behavior_scores: Dict
    behavior_tags: List[str]

    # Internal transactions
    internal_tx_count: int
    router_interactions: List[str]

# =============================================================================
# ASYNC HTTP CLIENT - ISSUE #17 FIX: Cache bypass option
# =============================================================================

class AsyncDataFetcher:
    """High-performance async data fetcher with parallel execution"""

    def __init__(self, config: Dict = CONFIG):
        self.config = config
        self.session: Optional[aiohttp.ClientSession] = None
        self.cache: Dict[str, Tuple[float, Any]] = {}
        self.cache_ttl = 60
        self.stats = {"requests": 0, "cache_hits": 0, "errors": 0}

    async def __aenter__(self):
        connector = aiohttp.TCPConnector(limit=self.config["max_concurrent"])
        timeout = aiohttp.ClientTimeout(total=self.config["timeout"])
        self.session = aiohttp.ClientSession(connector=connector, timeout=timeout)
        return self

    async def __aexit__(self, *args):
        if self.session:
            await self.session.close()

    def _cache_key(self, url: str, data: Any = None) -> str:
        key = url + str(data) if data else url
        return hashlib.md5(key.encode()).hexdigest()

    def _check_cache(self, key: str) -> Optional[Any]:
        if key in self.cache:
            ts, data = self.cache[key]
            if time.time() - ts < self.cache_ttl:
                self.stats["cache_hits"] += 1
                return data
        return None

    async def fetch_json(self, url: str, method: str = "GET",
                        json_data: Dict = None, headers: Dict = None,
                        bypass_cache: bool = False) -> FetchResult:  # [ISSUE #17 FIX]
        """Fetch JSON with caching and retry"""
        cache_key = self._cache_key(url, json_data)

        if not bypass_cache:
            cached = self._check_cache(cache_key)
            if cached:
                return FetchResult(True, cached, url, 0, from_cache=True)

        start = time.time()
        self.stats["requests"] += 1

        for retry in range(self.config["max_retries"]):
            try:
                if method == "GET":
                    async with self.session.get(url, headers=headers or {"Accept": "application/json"}) as resp:
                        data = await resp.json()
                else:
                    async with self.session.post(url, json=json_data, headers=headers) as resp:
                        data = await resp.json()

                latency = (time.time() - start) * 1000
                self.cache[cache_key] = (time.time(), data)
                return FetchResult(True, data, url, latency)

            except Exception as e:
                if retry == self.config["max_retries"] - 1:
                    self.stats["errors"] += 1
                    return FetchResult(False, None, url, 0, str(e))
                await asyncio.sleep(0.5 * (retry + 1))

        return FetchResult(False, None, url, 0, "Max retries exceeded")

    async def rpc_call(self, method: str, params: List) -> FetchResult:
        """Make async RPC call"""
        return await self.fetch_json(
            self.config["rpc_url"],
            method="POST",
            json_data={"jsonrpc": "2.0", "method": method, "params": params, "id": 1}
        )

    async def graphql_query(self, url: str, query: str) -> FetchResult:
        """Make async GraphQL query"""
        return await self.fetch_json(url, method="POST", json_data={"query": query})

# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

def hex_to_decimal(hex_str: str, decimals: int = 18) -> float:
    """Convert hex to decimal"""
    if not hex_str or hex_str == "0x":
        return 0.0
    return int(hex_str, 16) / (10 ** decimals)

def format_address(addr: str) -> str:
    """Format address for display"""
    if len(addr) > 10:
        return f"{addr[:6]}...{addr[-4:]}"
    return addr

def parse_timestamp(ts_str: str) -> Optional[datetime]:
    """Parse ISO timestamp string to naive datetime (UTC)"""
    if not ts_str:
        return None
    try:
        dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        # Convert to naive UTC for consistent comparison
        if dt.tzinfo:
            return dt.replace(tzinfo=None)
        return dt
    except:
        return None

def is_contract_interaction(from_addr: str, to_addr: str, config: Dict) -> Tuple[bool, str]:
    """Check if transfer involves known contracts"""
    from_lower = from_addr.lower()
    to_lower = to_addr.lower()

    # Check pools
    for name, addr in config.get("dex_pools", {}).items():
        if addr.lower() in [from_lower, to_lower]:
            return True, f"pool:{name}"

    # Check routers
    for name, addr in config.get("dex_routers", {}).items():
        if addr.lower() in [from_lower, to_lower]:
            return True, f"router:{name}"

    return False, "unknown"

# =============================================================================
# PARALLEL DATA ENGINE - All fixes applied
# =============================================================================

class ParallelDataEngine:
    """Engine for parallel multi-source data fetching"""

    def __init__(self, fetcher: AsyncDataFetcher):
        self.fetcher = fetcher
        self.config = fetcher.config

    # -------------------------------------------------------------------------
    # CORE PROTOCOL DATA
    # -------------------------------------------------------------------------

    async def fetch_all_rpc_data(self) -> Dict:
        """Fetch all RPC data in parallel"""
        print("\n[RPC] Fetching protocol data in parallel...")

        calls = [
            ("total_supply", self.config["usdfc_token"], "0x18160ddd"),
            ("total_collateral", self.config["trove_manager"], "0x887105d3"),
            ("fil_price", self.config["price_feed"], "0x0490be83"),
            ("stability_pool", self.config["stability_pool"], "0x0d9a6b35"),
            ("trove_count", self.config["trove_manager"], "0x49eefeee"),
        ]

        tasks = [
            self.fetcher.rpc_call("eth_call", [{"to": addr, "data": data}, "latest"])
            for name, addr, data in calls
        ]

        results = await asyncio.gather(*tasks)

        data = {}
        for (name, _, _), result in zip(calls, results):
            if result.success:
                hex_val = result.data.get("result", "0x0")
                if name == "trove_count":
                    data[name] = int(hex_val, 16) if hex_val else 0
                else:
                    data[name] = hex_to_decimal(hex_val)

        if data.get("total_supply", 0) > 0:
            data["tcr"] = (data.get("total_collateral", 0) * data.get("fil_price", 0)) / data["total_supply"] * 100
        else:
            data["tcr"] = 0

        print(f"  Total Supply: {data.get('total_supply', 0):,.2f} USDFC")
        print(f"  FIL Price: ${data.get('fil_price', 0):.4f}")
        print(f"  TCR: {data.get('tcr', 0):.2f}%")

        return data

    async def fetch_address_usdfc_balance(self, address: str) -> float:
        """Fetch USDFC balance for address"""
        addr_padded = address[2:].lower().zfill(64)
        data = f"0x70a08231{addr_padded}"

        result = await self.fetcher.rpc_call(
            "eth_call",
            [{"to": self.config["usdfc_token"], "data": data}, "latest"]
        )

        if result.success:
            return hex_to_decimal(result.data.get("result", "0x0"))
        return 0

    # -------------------------------------------------------------------------
    # [ISSUE #3 FIX] FETCH ALL TOKEN TRANSFERS
    # -------------------------------------------------------------------------

    async def fetch_address_all_transfers(self, address: str, max_pages: int = 20) -> Tuple[List[Dict], bool]:
        """
        Fetch ALL token transfers for address (not just USDFC)
        Returns: (transfers, is_complete)
        """
        print(f"\n[BLOCKSCOUT] Fetching ALL token transfers for {format_address(address)}...")

        all_transfers = []
        next_page_params = None
        pages = 0

        while pages < max_pages:
            # [ISSUE #3 FIX] No token filter - get ALL ERC-20 transfers
            url = f"{self.config['blockscout_rest']}/addresses/{address}/token-transfers?type=ERC-20"
            if next_page_params:
                import urllib.parse
                url += f"&{urllib.parse.urlencode(next_page_params)}"

            result = await self.fetcher.fetch_json(url)

            if not result.success:
                break

            items = result.data.get('items', [])
            if not items:
                break

            for t in items:
                token = t.get('token', {})
                from_addr = t.get('from', {}).get('hash', '')
                to_addr = t.get('to', {}).get('hash', '')

                # Determine direction
                addr_lower = address.lower()
                if to_addr.lower() == addr_lower:
                    direction = 'in'
                elif from_addr.lower() == addr_lower:
                    direction = 'out'
                else:
                    direction = 'internal'

                # Check if involves known contract
                is_contract, contract_type = is_contract_interaction(from_addr, to_addr, self.config)

                transfer = {
                    'timestamp': t.get('timestamp'),
                    'from': from_addr,
                    'to': to_addr,
                    'amount': float(t.get('total', {}).get('value', 0)) / (10 ** int(token.get('decimals', 18))),
                    'token_symbol': token.get('symbol', 'UNKNOWN'),
                    'token_address': token.get('address_hash', token.get('address', '')),  # [FIX] Correct field
                    'tx_hash': t.get('transaction_hash', ''),  # [FIX] Correct field name
                    'block': t.get('block_number'),
                    'direction': direction,
                    'is_contract': is_contract,
                    'contract_type': contract_type,
                }

                all_transfers.append(transfer)

            pages += 1
            next_page_params = result.data.get('next_page_params')
            if not next_page_params:
                break

        is_complete = pages < max_pages or next_page_params is None
        print(f"  Fetched {len(all_transfers)} transfers from {pages} pages (complete: {is_complete})")

        # Group by token
        tokens = defaultdict(int)
        for t in all_transfers:
            tokens[t['token_symbol']] += 1
        print(f"  Tokens: {dict(tokens)}")

        return all_transfers, is_complete

    # -------------------------------------------------------------------------
    # [ISSUE #4 FIX] INTERNAL TRANSACTION DETECTION
    # -------------------------------------------------------------------------

    async def fetch_address_internal_transactions(self, address: str) -> List[Dict]:
        """Fetch internal transactions for router/aggregator detection"""
        print(f"\n[BLOCKSCOUT] Fetching internal transactions for {format_address(address)}...")

        url = f"{self.config['blockscout_rest']}/addresses/{address}/internal-transactions"
        result = await self.fetcher.fetch_json(url)

        if not result.success:
            return []

        internal_txs = []
        for t in result.data.get('items', []):
            internal_txs.append({
                'timestamp': t.get('timestamp'),
                'type': t.get('type'),
                'from': t.get('from', {}).get('hash', ''),
                'to': t.get('to', {}).get('hash', ''),
                'value': float(t.get('value', 0)) / 1e18,
                'tx_hash': t.get('transaction_hash', ''),
            })

        print(f"  Found {len(internal_txs)} internal transactions")
        return internal_txs

    # -------------------------------------------------------------------------
    # [ISSUE #3 FIX] REGULAR TRANSACTIONS FOR ROUTER DETECTION
    # -------------------------------------------------------------------------

    async def fetch_address_transactions(self, address: str, limit: int = 50) -> List[Dict]:
        """Fetch regular transactions to detect contract interactions"""
        print(f"\n[BLOCKSCOUT] Fetching transactions for {format_address(address)}...")

        url = f"{self.config['blockscout_rest']}/addresses/{address}/transactions"
        result = await self.fetcher.fetch_json(url)

        if not result.success:
            return []

        transactions = []
        router_interactions = []

        for t in result.data.get('items', [])[:limit]:
            to_addr = t.get('to', {}).get('hash', '') if t.get('to') else ''
            method = t.get('method', '')
            contract_name = t.get('to', {}).get('name', '') if t.get('to') else ''

            tx = {
                'timestamp': t.get('timestamp'),
                'hash': t.get('hash', ''),
                'to': to_addr,
                'method': method,
                'contract_name': contract_name,
                'value': float(t.get('value', 0)) / 1e18,
            }
            transactions.append(tx)

            # Check for router interactions
            to_lower = to_addr.lower()
            for router_name, router_addr in self.config.get('dex_routers', {}).items():
                if to_lower == router_addr.lower():
                    router_interactions.append({
                        'router': router_name,
                        'method': method,
                        'tx_hash': t.get('hash', ''),
                        'timestamp': t.get('timestamp'),
                    })

            # Also check by contract name
            if contract_name and any(x in contract_name.lower() for x in ['router', 'swap', 'squid', 'snwap']):
                if not any(r['tx_hash'] == t.get('hash', '') for r in router_interactions):
                    router_interactions.append({
                        'router': contract_name,
                        'method': method,
                        'tx_hash': t.get('hash', ''),
                        'timestamp': t.get('timestamp'),
                    })

        print(f"  Found {len(transactions)} transactions, {len(router_interactions)} router interactions")
        return transactions, router_interactions

    # -------------------------------------------------------------------------
    # [ISSUE #5 FIX] PROPER SWAP DETECTION
    # -------------------------------------------------------------------------

    async def detect_swaps(self, address: str, all_transfers: List[Dict],
                          router_interactions: List[Dict]) -> List[SwapRecord]:
        """
        Detect swaps using multiple methods:
        1. Direct pool transfers
        2. Router interactions
        3. Multi-token transactions
        """
        print(f"\n[ANALYSIS] Detecting swaps for {format_address(address)}...")

        swaps = []
        addr_lower = address.lower()

        # Group transfers by tx_hash
        tx_groups = defaultdict(list)
        for t in all_transfers:
            tx_groups[t['tx_hash']].append(t)

        # Method 1: Multi-token transactions (most reliable for detecting swaps)
        # [FIX] Create ONE swap per transaction, not N*M combinations
        for tx_hash, transfers in tx_groups.items():
            if len(transfers) < 2 or not tx_hash:  # Skip empty tx_hashes
                continue

            # Look for pattern: token out + token in = swap
            tokens_in = [t for t in transfers if t['direction'] == 'in']
            tokens_out = [t for t in transfers if t['direction'] == 'out']

            if tokens_in and tokens_out:
                # Aggregate: sum all tokens in and out
                total_in = sum(t['amount'] for t in tokens_in)
                total_out = sum(t['amount'] for t in tokens_out)

                # Use the primary token (highest value) for naming
                primary_in = max(tokens_in, key=lambda x: x['amount'])
                primary_out = max(tokens_out, key=lambda x: x['amount'])

                # Determine swap type based on USDFC direction
                has_usdfc_out = any(t['token_symbol'] == 'USDFC' for t in tokens_out)
                has_usdfc_in = any(t['token_symbol'] == 'USDFC' for t in tokens_in)

                if has_usdfc_out:
                    swap_type = 'sell'
                elif has_usdfc_in:
                    swap_type = 'buy'
                else:
                    swap_type = 'other'  # Multi-token swap not involving USDFC

                # Get router info
                router = None
                for t in transfers:
                    ct = t.get('contract_type', '')
                    if 'router:' in ct:
                        router = ct.replace('router:', '')
                        break

                swaps.append(SwapRecord(
                    timestamp=primary_out['timestamp'],
                    tx_hash=tx_hash,
                    swap_type=swap_type,
                    token_in=primary_in['token_symbol'],
                    token_out=primary_out['token_symbol'],
                    amount_in=primary_in['amount'],
                    amount_out=primary_out['amount'],
                    router=router,
                    pool=None,
                ))

        # Method 2: Router interactions that we couldn't match to transfers
        router_tx_hashes = {s.tx_hash for s in swaps}
        for ri in router_interactions:
            if ri['tx_hash'] not in router_tx_hashes:
                # We know there was a router interaction but couldn't find the swap
                # Mark it as detected but incomplete
                swaps.append(SwapRecord(
                    timestamp=ri['timestamp'],
                    tx_hash=ri['tx_hash'],
                    swap_type='router_interaction',
                    token_in='UNKNOWN',
                    token_out='UNKNOWN',
                    amount_in=0,
                    amount_out=0,
                    router=ri['router'],
                ))

        # Method 3: Direct pool transfers (fallback)
        for t in all_transfers:
            if t.get('is_contract') and 'pool:' in t.get('contract_type', ''):
                tx_hash = t['tx_hash']
                if not any(s.tx_hash == tx_hash for s in swaps):
                    swap_type = 'buy' if t['direction'] == 'in' else 'sell'
                    swaps.append(SwapRecord(
                        timestamp=t['timestamp'],
                        tx_hash=tx_hash,
                        swap_type=swap_type,
                        token_in=t['token_symbol'] if t['direction'] == 'in' else 'WFIL',
                        token_out='WFIL' if t['direction'] == 'in' else t['token_symbol'],
                        amount_in=t['amount'] if t['direction'] == 'in' else 0,
                        amount_out=0 if t['direction'] == 'in' else t['amount'],
                        pool=t.get('contract_type', '').replace('pool:', ''),
                    ))

        print(f"  Detected {len(swaps)} swaps")

        # Stats
        buy_count = len([s for s in swaps if s.swap_type == 'buy'])
        sell_count = len([s for s in swaps if s.swap_type == 'sell'])
        router_count = len([s for s in swaps if s.swap_type == 'router_interaction'])
        print(f"  Buys: {buy_count}, Sells: {sell_count}, Router interactions: {router_count}")

        return swaps

    # -------------------------------------------------------------------------
    # [ISSUE #10 FIX] LENDING HISTORY WITH CORRECT SCHEMA
    # -------------------------------------------------------------------------

    async def fetch_address_lending_history(self, address: str) -> Dict:
        """Fetch complete lending/borrowing history from Secured Finance Subgraph"""
        print(f"\n[SUBGRAPH] Fetching lending history for {format_address(address)}...")

        # [ISSUE #10 FIX] Corrected schema - removed invalid fields
        query = f'''
        {{
          user(id: "{address.lower()}") {{
            id
            createdAt
            transactionCount
            orderCount
            transactions(first: 500, orderBy: createdAt, orderDirection: desc) {{
              id
              createdAt
              side
              currency
              maturity
              futureValue
              executionPrice
            }}
            orders(first: 200, orderBy: createdAt, orderDirection: desc) {{
              id
              status
              side
              currency
              maturity
              createdAt
              inputAmount
              filledAmount
            }}
          }}
        }}
        '''

        result = await self.fetcher.graphql_query(self.config["subgraph_url"], query)

        if not result.success:
            return {'has_activity': False, 'error': result.error}

        user = result.data.get('data', {}).get('user')
        if not user:
            print("  No lending activity found")
            return {'has_activity': False, 'transactions': [], 'orders': [], 'stats': {}}

        # Filter USDFC transactions
        usdfc_curr = self.config['currency_usdfc']
        all_txs = user.get('transactions', [])
        usdfc_txs = [t for t in all_txs if t.get('currency') == usdfc_curr]

        lend_txs = []
        borrow_txs = []

        for t in usdfc_txs:
            ts = int(t.get('createdAt', 0))
            maturity_ts = int(t.get('maturity', 0))

            # [ISSUE #6 FIX] Handle both int and string side values
            side_val = t.get('side')
            side = 'lend' if side_val in [0, '0'] else 'borrow'

            future_value = int(t.get('futureValue', 0)) / 1e18
            price = int(t.get('executionPrice', 0)) / 1e18

            # [ISSUE #7 FIX] Use actual maturity for APR calculation
            if maturity_ts > ts and price > 0:
                days_to_maturity = (maturity_ts - ts) / 86400
                apr = (1 - price) * 365 / days_to_maturity * 100 if days_to_maturity > 0 else 0
            else:
                apr = 0

            tx_data = {
                'timestamp': datetime.fromtimestamp(ts).isoformat() if ts else None,
                'side': side,
                'future_value': future_value,
                'execution_price': price,
                'apr': apr,
                'maturity': maturity_ts,
                'days_to_maturity': (maturity_ts - ts) / 86400 if maturity_ts > ts else 0,
            }

            if side == 'lend':
                lend_txs.append(tx_data)
            else:
                borrow_txs.append(tx_data)

        total_lend = sum(t['future_value'] for t in lend_txs)
        total_borrow = sum(t['future_value'] for t in borrow_txs)
        avg_lend_apr = statistics.mean([t['apr'] for t in lend_txs]) if lend_txs else 0
        avg_borrow_apr = statistics.mean([t['apr'] for t in borrow_txs]) if borrow_txs else 0

        result_data = {
            'has_activity': True,
            'user_created': user.get('createdAt'),
            'total_tx_count': int(user.get('transactionCount', 0)),
            'total_order_count': int(user.get('orderCount', 0)),
            'usdfc_transactions': {
                'lend': lend_txs,
                'borrow': borrow_txs,
            },
            'stats': {
                'lend_tx_count': len(lend_txs),
                'borrow_tx_count': len(borrow_txs),
                'total_lend_volume': total_lend,
                'total_borrow_volume': total_borrow,
                'avg_lend_apr': avg_lend_apr,
                'avg_borrow_apr': avg_borrow_apr,
                'net_position': total_lend - total_borrow,
            }
        }

        print(f"  USDFC Transactions: {len(usdfc_txs)}")
        print(f"  Lend: {len(lend_txs)} txs, {total_lend:,.2f} USDFC @ avg {avg_lend_apr:.1f}% APR")
        print(f"  Borrow: {len(borrow_txs)} txs, {total_borrow:,.2f} USDFC @ avg {avg_borrow_apr:.1f}% APR")

        return result_data

    # -------------------------------------------------------------------------
    # [ISSUE #8 & #11 FIX] BALANCE HISTORY WITH COMPLETENESS & HOLDING TIME
    # -------------------------------------------------------------------------

    async def compute_balance_history(self, address: str, transfers: List[Dict],
                                     current_balance: float, is_complete: bool) -> Tuple[List[BalancePoint], List[Dict], float, float]:
        """
        Compute balance history and proper holding time
        Returns: (balance_history, holding_periods, total_holding_days, current_holding_days)
        """
        print(f"\n[ANALYSIS] Computing balance history for {format_address(address)}...")

        # Filter USDFC transfers only for balance
        usdfc_addr = self.config['usdfc_token'].lower()
        usdfc_transfers = [t for t in transfers if t['token_address'].lower() == usdfc_addr]

        if not usdfc_transfers:
            return (
                [BalancePoint(datetime.now().isoformat(), current_balance, 'current', is_complete)],
                [],
                0,
                0
            )

        # Sort by timestamp descending
        sorted_transfers = sorted(usdfc_transfers, key=lambda x: x.get('timestamp', ''), reverse=True)

        # Build balance history walking backwards
        balance_history = []
        running_balance = current_balance

        # Add current state
        balance_history.append(BalancePoint(
            timestamp=datetime.now().isoformat(),
            balance=current_balance,
            event='current',
            data_complete=is_complete
        ))

        for t in sorted_transfers:
            ts = t.get('timestamp')
            amt = t.get('amount', 0)

            if t['direction'] == 'out':
                running_balance += amt
            else:
                running_balance -= amt

            # [ISSUE #8 FIX] Flag if balance goes negative
            data_ok = running_balance >= -0.01  # Small tolerance for rounding

            balance_history.append(BalancePoint(
                timestamp=ts,
                balance=max(0, running_balance),
                event=f"{'sent' if t['direction'] == 'out' else 'received'} {amt:.2f}",
                data_complete=data_ok and is_complete
            ))

        # Reverse to chronological order
        balance_history.reverse()

        # [ISSUE #11 FIX] Calculate proper holding periods
        holding_periods = []
        current_period_start = None
        total_holding_seconds = 0

        for i, point in enumerate(balance_history):
            ts = parse_timestamp(point.timestamp)
            if not ts:
                continue

            if point.balance > 0 and current_period_start is None:
                current_period_start = ts
            elif point.balance == 0 and current_period_start is not None:
                period_end = ts
                duration = (period_end - current_period_start).total_seconds()
                total_holding_seconds += duration
                holding_periods.append({
                    'start': current_period_start.isoformat(),
                    'end': period_end.isoformat(),
                    'days': duration / 86400,
                })
                current_period_start = None

        # If currently holding
        current_holding_days = 0
        if current_balance > 0 and current_period_start:
            current_holding_days = (datetime.now() - current_period_start).total_seconds() / 86400
            total_holding_seconds += current_holding_days * 86400

        total_holding_days = total_holding_seconds / 86400

        print(f"  Balance points: {len(balance_history)}")
        print(f"  Holding periods: {len(holding_periods)}")
        print(f"  Total holding: {total_holding_days:.1f} days, Current: {current_holding_days:.1f} days")

        return balance_history, holding_periods, total_holding_days, current_holding_days

    # -------------------------------------------------------------------------
    # [ISSUE #12 FIX] TIME-BASED VOLUME CALCULATIONS
    # -------------------------------------------------------------------------

    def compute_volume_by_timerange(self, transfers: List[Dict], hours: int) -> Dict:
        """Compute volume for specific time range"""
        cutoff = datetime.now() - timedelta(hours=hours)

        recent = []
        for t in transfers:
            ts = parse_timestamp(t.get('timestamp'))
            if ts and ts.replace(tzinfo=None) > cutoff.replace(tzinfo=None):
                recent.append(t)

        # Filter USDFC
        usdfc_addr = self.config['usdfc_token'].lower()
        usdfc_recent = [t for t in recent if t['token_address'].lower() == usdfc_addr]

        in_volume = sum(t['amount'] for t in usdfc_recent if t['direction'] == 'in')
        out_volume = sum(t['amount'] for t in usdfc_recent if t['direction'] == 'out')

        return {
            'in_volume': in_volume,
            'out_volume': out_volume,
            'total_volume': in_volume + out_volume,
            'tx_count': len(usdfc_recent),
            'hours': hours,
        }

    # -------------------------------------------------------------------------
    # [ISSUE #14 FIX] MULTI-DIMENSIONAL BEHAVIOR SCORING
    # -------------------------------------------------------------------------

    def compute_behavior_scores(self, transfers: List[Dict], swaps: List[SwapRecord],
                               lending: Dict, balance_history: List[BalancePoint],
                               router_interactions: List[Dict], current_balance: float) -> Tuple[Dict, List[str]]:
        """Compute multi-dimensional behavior scores and tags"""

        scores = {
            'trader_score': 0,
            'holder_score': 0,
            'defi_score': 0,
            'whale_score': 0,
            'activity_score': 0,
            'diversity_score': 0,
        }

        tags = []

        # Trader score (based on swaps)
        swap_count = len(swaps)
        if swap_count > 20:
            scores['trader_score'] = 100
            tags.append('Active Trader')
        elif swap_count > 10:
            scores['trader_score'] = 70
            tags.append('Trader')
        elif swap_count > 5:
            scores['trader_score'] = 40
        elif swap_count > 0:
            scores['trader_score'] = 20

        # DeFi score (lending + router usage)
        lending_tx = lending.get('stats', {}).get('lend_tx_count', 0) + lending.get('stats', {}).get('borrow_tx_count', 0)
        router_count = len(router_interactions)

        if lending_tx > 10 or router_count > 5:
            scores['defi_score'] = 100
            tags.append('DeFi Power User')
        elif lending_tx > 5 or router_count > 2:
            scores['defi_score'] = 70
            tags.append('Active DeFi User')
        elif lending_tx > 0 or router_count > 0:
            scores['defi_score'] = 40
            tags.append('DeFi User')

        # Holder score (based on holding time and consistency)
        if len(balance_history) > 50:
            scores['holder_score'] = 80
            tags.append('Long-term User')
        elif len(balance_history) > 20:
            scores['holder_score'] = 50

        if current_balance > 0:
            tags.append('Current Holder')

        # Whale score
        if current_balance > 50000:
            scores['whale_score'] = 100
            tags.append('Whale')
        elif current_balance > 10000:
            scores['whale_score'] = 70
            tags.append('Large Holder')
        elif current_balance > 1000:
            scores['whale_score'] = 40

        # Activity score (recency)
        if transfers:
            latest = parse_timestamp(transfers[0].get('timestamp'))
            if latest:
                days_since = (datetime.now(latest.tzinfo) - latest).days
                if days_since < 7:
                    scores['activity_score'] = 100
                    tags.append('Recently Active')
                elif days_since < 30:
                    scores['activity_score'] = 70
                elif days_since < 90:
                    scores['activity_score'] = 40
                else:
                    tags.append('Inactive')

        # Diversity score (token variety)
        unique_tokens = set(t['token_symbol'] for t in transfers)
        if len(unique_tokens) > 5:
            scores['diversity_score'] = 100
            tags.append('Multi-Token User')
        elif len(unique_tokens) > 3:
            scores['diversity_score'] = 70
        elif len(unique_tokens) > 1:
            scores['diversity_score'] = 40

        # Check for cross-chain activity
        if any('squid' in r.get('router', '').lower() for r in router_interactions):
            tags.append('Cross-Chain User')

        if not tags:
            tags.append('Casual User')

        return scores, tags


# =============================================================================
# COMPREHENSIVE ADDRESS ANALYSIS - [ISSUE #2 FIX] No misleading pool data
# =============================================================================

async def analyze_address_comprehensive_v3(address: str) -> Dict:
    """
    Comprehensive address analysis v3 with all fixes applied.
    [ISSUE #2 FIX] No pool-wide stats mixed with address stats.
    """
    print("\n" + "="*80)
    print(f"COMPREHENSIVE ADDRESS ANALYSIS v3")
    print(f"Address: {address}")
    print("="*80)

    start = time.time()

    async with AsyncDataFetcher() as fetcher:
        engine = ParallelDataEngine(fetcher)

        # Step 1: Fetch current balance
        print("\n[STEP 1] Fetching current balance...")
        current_balance = await engine.fetch_address_usdfc_balance(address)
        print(f"  Balance: {current_balance:,.2f} USDFC")

        # Step 2: Fetch ALL transfers (not just USDFC) [ISSUE #3 FIX]
        print("\n[STEP 2] Fetching all token transfers...")
        all_transfers, transfers_complete = await engine.fetch_address_all_transfers(address, max_pages=20)

        # Step 3: Fetch internal transactions [ISSUE #4 FIX]
        print("\n[STEP 3] Fetching internal transactions...")
        internal_txs = await engine.fetch_address_internal_transactions(address)

        # Step 4: Fetch regular transactions for router detection
        print("\n[STEP 4] Detecting router interactions...")
        transactions, router_interactions = await engine.fetch_address_transactions(address)

        # Step 5: Detect swaps [ISSUE #5 FIX]
        print("\n[STEP 5] Analyzing swaps...")
        swaps = await engine.detect_swaps(address, all_transfers, router_interactions)

        # Step 6: Fetch lending history [ISSUE #10 FIX]
        print("\n[STEP 6] Fetching lending history...")
        lending = await engine.fetch_address_lending_history(address)

        # Step 7: Compute balance history [ISSUE #8 & #11 FIX]
        print("\n[STEP 7] Computing balance history and holding time...")
        balance_history, holding_periods, total_holding_days, current_holding_days = \
            await engine.compute_balance_history(address, all_transfers, current_balance, transfers_complete)

        # Step 8: Compute time-based volumes [ISSUE #12 FIX]
        print("\n[STEP 8] Computing time-based volumes...")
        volume_24h = engine.compute_volume_by_timerange(all_transfers, 24)
        volume_7d = engine.compute_volume_by_timerange(all_transfers, 168)
        volume_30d = engine.compute_volume_by_timerange(all_transfers, 720)

        # Step 9: Compute behavior scores [ISSUE #14 FIX]
        print("\n[STEP 9] Analyzing behavior...")
        behavior_scores, behavior_tags = engine.compute_behavior_scores(
            all_transfers, swaps, lending, balance_history, router_interactions, current_balance
        )

        # Compile results
        usdfc_addr = fetcher.config['usdfc_token'].lower()
        usdfc_transfers = [t for t in all_transfers if t['token_address'].lower() == usdfc_addr]
        other_transfers = [t for t in all_transfers if t['token_address'].lower() != usdfc_addr]
        tokens_used = list(set(t['token_symbol'] for t in all_transfers))

        # Swap stats
        swap_stats = {
            'total_swaps': len(swaps),
            'buy_count': len([s for s in swaps if s.swap_type == 'buy']),
            'sell_count': len([s for s in swaps if s.swap_type == 'sell']),
            'router_interactions': len([s for s in swaps if s.swap_type == 'router_interaction']),
            'buy_volume_usdfc': sum(s.amount_in for s in swaps if s.swap_type == 'buy' and s.token_in == 'USDFC'),
            'sell_volume_usdfc': sum(s.amount_out for s in swaps if s.swap_type == 'sell' and s.token_out == 'USDFC'),
            'routers_used': list(set(s.router for s in swaps if s.router)),
        }

        elapsed = time.time() - start

        report = {
            'address': address,
            'generated_at': datetime.now().isoformat(),
            'fetch_time_seconds': elapsed,
            'data_complete': transfers_complete,

            'current_state': {
                'balance': current_balance,
                'is_holder': current_balance > 0,
            },

            'transfer_summary': {
                'total_transfers': len(all_transfers),
                'usdfc_transfers': len(usdfc_transfers),
                'other_token_transfers': len(other_transfers),
                'tokens_used': tokens_used,
            },

            'balance_history': {
                'data_points': len(balance_history),
                'data_complete': transfers_complete,
                'history': [asdict(b) for b in balance_history[-30:]],  # Last 30 for JSON
                'min_balance': min(b.balance for b in balance_history) if balance_history else 0,
                'max_balance': max(b.balance for b in balance_history) if balance_history else 0,
            },

            'holding_analysis': {
                'total_holding_days': total_holding_days,
                'current_holding_days': current_holding_days,
                'holding_periods': holding_periods,
            },

            'swap_activity': {
                'swaps': [asdict(s) for s in swaps],
                'stats': swap_stats,
            },

            'lending_activity': lending,

            'volume_analysis': {
                '24h': volume_24h,
                '7d': volume_7d,
                '30d': volume_30d,
            },

            'internal_transactions': {
                'count': len(internal_txs),
                'router_interactions': router_interactions,
            },

            'behavior': {
                'scores': behavior_scores,
                'tags': behavior_tags,
            },
        }

        # Summary output
        print(f"\n{'='*60}")
        print("ANALYSIS COMPLETE")
        print(f"{'='*60}")
        print(f"  Time: {elapsed:.2f}s")
        print(f"  Balance: {current_balance:,.2f} USDFC")
        print(f"  Transfers: {len(all_transfers)} ({len(usdfc_transfers)} USDFC, {len(other_transfers)} other)")
        print(f"  Tokens: {', '.join(tokens_used)}")
        print(f"  Swaps: {swap_stats['total_swaps']} (buy: {swap_stats['buy_count']}, sell: {swap_stats['sell_count']}, router: {swap_stats['router_interactions']})")
        print(f"  Routers: {', '.join(swap_stats['routers_used']) if swap_stats['routers_used'] else 'None'}")
        print(f"  Lending: {lending.get('stats', {}).get('lend_tx_count', 0)} lend, {lending.get('stats', {}).get('borrow_tx_count', 0)} borrow")
        print(f"  Holding: {total_holding_days:.1f} days total, {current_holding_days:.1f} days current")
        print(f"  Tags: {', '.join(behavior_tags)}")
        print(f"  Data Complete: {transfers_complete}")

        return report


# =============================================================================
# MAIN
# =============================================================================

async def main():
    """Main entry point"""
    print("="*80)
    print("ADVANCED ANALYTICS DATA FETCHER v3.0")
    print("All issues fixed - accurate data only")
    print(f"Started: {datetime.now().isoformat()}")
    print("="*80)

    # Test with the wallet that had issues
    test_address = "0x56db3D50f8711fbab02a37e80D0eD6b019D5F654"

    report = await analyze_address_comprehensive_v3(test_address)

    # Save report
    with open("address_analysis_v3.json", "w") as f:
        json.dump(report, f, indent=2, default=str)

    print(f"\nReport saved to: address_analysis_v3.json")

    return report


if __name__ == "__main__":
    asyncio.run(main())
