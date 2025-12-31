#!/usr/bin/env python3
"""
USDFC TradingView-Style Chart Data Generator
Real-time lookback with configurable timeframes

Features:
- TradingView-compatible OHLCV format
- Real-time lookback periods (1h, 4h, 1d, 7d, 30d)
- Multiple chart types: Balance, Lending, Price, Operations
- Candle aggregation for all data types
- Historical range queries
"""

import asyncio
import aiohttp
import json
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict, field
from typing import Dict, List, Optional, Tuple, Any
from collections import defaultdict
from enum import Enum
import time

# =============================================================================
# CONFIGURATION
# =============================================================================

CONFIG = {
    "blockscout_rest": "https://filecoin.blockscout.com/api/v2",
    "sf_subgraph": "https://api.goldsky.com/api/public/project_cm8i6ca9k24d601wy45zzbsrq/subgraphs/sf-filecoin-mainnet/latest/gn",
    "geckoterminal": "https://api.geckoterminal.com/api/v2",
    "usdfc_token": "0x80B98d3aa09ffff255c3ba4A241111Ff1262F045",
    "usdfc_wfil_pool": "0x4e07447bd38e60b94176764133788be1a0736b30",
    "borrower_operations": "0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0",
    "stability_pool": "0x791Ad78bBc58324089D3E0A8689E7D045B9592b5",
    "currency_usdfc": "0x5553444643000000000000000000000000000000000000000000000000000000",
}

# =============================================================================
# ENUMS & DATA STRUCTURES
# =============================================================================

class Resolution(Enum):
    """TradingView-style resolutions"""
    M1 = ("1", 1)           # 1 minute
    M5 = ("5", 5)           # 5 minutes
    M15 = ("15", 15)        # 15 minutes
    M30 = ("30", 30)        # 30 minutes
    H1 = ("60", 60)         # 1 hour
    H4 = ("240", 240)       # 4 hours
    D1 = ("D", 1440)        # 1 day
    W1 = ("W", 10080)       # 1 week

    def __init__(self, tv_code: str, minutes: int):
        self.tv_code = tv_code
        self.minutes = minutes

class Lookback(Enum):
    """Lookback periods for real-time charts"""
    HOUR_1 = ("1h", 60)
    HOUR_4 = ("4h", 240)
    HOUR_12 = ("12h", 720)
    DAY_1 = ("1d", 1440)
    DAY_3 = ("3d", 4320)
    WEEK_1 = ("1w", 10080)
    WEEK_2 = ("2w", 20160)
    MONTH_1 = ("1m", 43200)
    MONTH_3 = ("3m", 129600)
    ALL = ("all", 0)

    def __init__(self, label: str, minutes: int):
        self.label = label
        self.total_minutes = minutes

class OperationType(Enum):
    OPEN_TROVE = "open_trove"
    ADJUST_TROVE = "adjust_trove"
    CLOSE_TROVE = "close_trove"
    CLAIM_COLLATERAL = "claim_collateral"
    PROVIDE_SP = "provide_sp"
    WITHDRAW_SP = "withdraw_sp"
    LEND = "lend"
    BORROW = "borrow"
    SWAP = "swap"
    BRIDGE = "bridge"
    APPROVE = "approve"
    TRANSFER = "transfer"
    MINT = "mint"
    REDEEM = "redeem"
    LIQUIDATE = "liquidate"
    UNKNOWN = "unknown"

# TradingView-compatible candle format
@dataclass
class TVCandle:
    """TradingView OHLCV candle format"""
    time: int           # Unix timestamp (seconds)
    open: float
    high: float
    low: float
    close: float
    volume: float

@dataclass
class BalanceCandle:
    """Balance/Holding candle with OHLC"""
    time: int
    open: float         # Balance at candle open
    high: float         # Max balance in period
    low: float          # Min balance in period
    close: float        # Balance at candle close
    volume: float       # Total transfer volume
    tx_count: int       # Number of transactions
    net_change: float   # Net balance change

@dataclass
class VolumeCandle:
    """Lending/Borrowing volume candle"""
    time: int
    lend_volume: float
    borrow_volume: float
    lend_count: int
    borrow_count: int
    net_flow: float
    total_volume: float

@dataclass
class OperationMarker:
    """Operation marker for chart overlay"""
    time: int
    operation: str
    amount: float
    tx_hash: str
    label: str          # Short label for chart
    color: str          # Marker color

# =============================================================================
# OPERATION DETECTION
# =============================================================================

METHOD_MAPPING = {
    "opentrove": (OperationType.OPEN_TROVE, "Open", "#22c55e"),
    "adjusttrove": (OperationType.ADJUST_TROVE, "Adjust", "#3b82f6"),
    "closetrove": (OperationType.CLOSE_TROVE, "Close", "#ef4444"),
    "claimcollateral": (OperationType.CLAIM_COLLATERAL, "Claim", "#f59e0b"),
    "providetosp": (OperationType.PROVIDE_SP, "SP+", "#22c55e"),
    "withdrawfromsp": (OperationType.WITHDRAW_SP, "SP-", "#ef4444"),
    "snwap": (OperationType.SWAP, "Swap", "#8b5cf6"),
    "swap": (OperationType.SWAP, "Swap", "#8b5cf6"),
    "bridgecall": (OperationType.BRIDGE, "Bridge", "#06b6d4"),
    "callbridgecall": (OperationType.BRIDGE, "Bridge", "#06b6d4"),
    "fundandrunmulticall": (OperationType.BRIDGE, "Bridge", "#06b6d4"),
    "approve": (OperationType.APPROVE, "Approve", "#6b7280"),
    "transfer": (OperationType.TRANSFER, "Transfer", "#6b7280"),
    "mint": (OperationType.MINT, "Mint", "#22c55e"),
    "redeem": (OperationType.REDEEM, "Redeem", "#ef4444"),
    "liquidate": (OperationType.LIQUIDATE, "Liquidate", "#dc2626"),
}

def classify_operation(method: str) -> Tuple[OperationType, str, str]:
    """Classify operation and return (type, label, color)"""
    method_lower = method.lower().replace("_", "")
    for key, (op_type, label, color) in METHOD_MAPPING.items():
        if key in method_lower:
            return op_type, label, color
    return OperationType.UNKNOWN, "?", "#6b7280"

# =============================================================================
# ASYNC DATA FETCHER
# =============================================================================

class TradingViewDataFetcher:
    """Fetches and formats data for TradingView-style charts"""

    def __init__(self):
        self.session: Optional[aiohttp.ClientSession] = None
        self.cache: Dict[str, Tuple[float, Any]] = {}
        self.cache_ttl = 30  # 30 second cache for real-time

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, *args):
        if self.session:
            await self.session.close()

    async def fetch_json(self, url: str, method: str = "GET",
                         json_data: dict = None, use_cache: bool = True) -> dict:
        """Fetch JSON with optional caching"""
        cache_key = f"{method}:{url}:{json.dumps(json_data) if json_data else ''}"

        if use_cache and cache_key in self.cache:
            cached_time, cached_data = self.cache[cache_key]
            if time.time() - cached_time < self.cache_ttl:
                return cached_data

        try:
            if method == "POST":
                async with self.session.post(url, json=json_data, timeout=30) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        self.cache[cache_key] = (time.time(), data)
                        return data
            else:
                async with self.session.get(url, timeout=30) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        self.cache[cache_key] = (time.time(), data)
                        return data
        except Exception as e:
            print(f"  Error: {e}")
        return {}

    # -------------------------------------------------------------------------
    # TIME UTILITIES
    # -------------------------------------------------------------------------

    def round_to_resolution(self, dt: datetime, resolution: Resolution) -> datetime:
        """Round datetime to resolution bucket"""
        minutes = resolution.minutes

        if minutes >= 10080:  # Weekly
            # Round to start of week (Monday)
            days_since_monday = dt.weekday()
            return (dt - timedelta(days=days_since_monday)).replace(
                hour=0, minute=0, second=0, microsecond=0
            )
        elif minutes >= 1440:  # Daily
            return dt.replace(hour=0, minute=0, second=0, microsecond=0)
        elif minutes >= 60:  # Hourly
            hours = minutes // 60
            return dt.replace(
                minute=0, second=0, microsecond=0,
                hour=(dt.hour // hours) * hours
            )
        else:  # Sub-hourly
            return dt.replace(
                second=0, microsecond=0,
                minute=(dt.minute // minutes) * minutes
            )

    def get_lookback_cutoff(self, lookback: Lookback) -> Optional[datetime]:
        """Get cutoff datetime for lookback period"""
        if lookback == Lookback.ALL:
            return None
        return datetime.now() - timedelta(minutes=lookback.total_minutes)

    def generate_time_buckets(self, start: datetime, end: datetime,
                              resolution: Resolution) -> List[datetime]:
        """Generate all time buckets between start and end"""
        buckets = []
        current = self.round_to_resolution(start, resolution)
        end_rounded = self.round_to_resolution(end, resolution)

        while current <= end_rounded:
            buckets.append(current)
            current += timedelta(minutes=resolution.minutes)

        return buckets

    # -------------------------------------------------------------------------
    # BALANCE CHART (Holding OHLC)
    # -------------------------------------------------------------------------

    async def fetch_balance_chart(self, address: str, resolution: Resolution = Resolution.H1,
                                   lookback: Lookback = Lookback.WEEK_1) -> Dict:
        """Fetch balance history as OHLC candles"""
        print(f"\n[BALANCE] Fetching {resolution.tv_code} candles, {lookback.label} lookback...")

        cutoff = self.get_lookback_cutoff(lookback)

        # Fetch all USDFC transfers
        transfers = []
        url = f"{CONFIG['blockscout_rest']}/addresses/{address}/token-transfers?type=ERC-20"
        pages = 0
        max_pages = 20

        while url and pages < max_pages:
            data = await self.fetch_json(url, use_cache=False)
            items = data.get('items', [])

            for t in items:
                token = t.get('token', {})
                if token.get('address_hash', '').lower() != CONFIG['usdfc_token'].lower():
                    continue

                ts_str = t.get('timestamp')
                if not ts_str:
                    continue

                ts = datetime.fromisoformat(ts_str.replace('Z', '+00:00')).replace(tzinfo=None)

                # Apply lookback filter
                if cutoff and ts < cutoff:
                    url = None  # Stop fetching older data
                    break

                from_addr = t.get('from', {}).get('hash', '').lower()
                to_addr = t.get('to', {}).get('hash', '').lower()
                addr_lower = address.lower()

                direction = 1 if to_addr == addr_lower else -1
                amount = float(t.get('total', {}).get('value', 0)) / 1e18

                transfers.append({
                    'timestamp': ts,
                    'amount': amount * direction,
                    'abs_amount': amount,
                })

            pages += 1
            next_params = data.get('next_page_params')
            if next_params and url:
                url = f"{CONFIG['blockscout_rest']}/addresses/{address}/token-transfers?type=ERC-20&block_number={next_params.get('block_number')}&index={next_params.get('index')}"
            else:
                url = None

        print(f"  Found {len(transfers)} USDFC transfers")

        if not transfers:
            return {"candles": [], "resolution": resolution.tv_code, "lookback": lookback.label}

        # Sort chronologically
        transfers.sort(key=lambda x: x['timestamp'])

        # Build OHLC candles
        candles = self._build_balance_ohlc(transfers, resolution, cutoff)

        return {
            "candles": [asdict(c) for c in candles],
            "resolution": resolution.tv_code,
            "lookback": lookback.label,
            "data_points": len(candles),
            "latest_balance": candles[-1].close if candles else 0,
        }

    def _build_balance_ohlc(self, transfers: List[dict], resolution: Resolution,
                            cutoff: Optional[datetime]) -> List[BalanceCandle]:
        """Build OHLC candles from transfer data"""
        if not transfers:
            return []

        # Group transfers by bucket
        buckets = defaultdict(list)
        for t in transfers:
            bucket = self.round_to_resolution(t['timestamp'], resolution)
            buckets[bucket].append(t)

        # Calculate running balance and build candles
        candles = []
        running_balance = 0.0

        # Get all bucket times
        all_times = sorted(buckets.keys())
        if not all_times:
            return []

        # Generate continuous time series
        start_time = all_times[0]
        end_time = max(all_times[-1], datetime.now())
        all_buckets = self.generate_time_buckets(start_time, end_time, resolution)

        for bucket_time in all_buckets:
            bucket_transfers = buckets.get(bucket_time, [])

            open_balance = running_balance
            high_balance = running_balance
            low_balance = running_balance
            volume = 0.0
            tx_count = len(bucket_transfers)
            net_change = 0.0

            for t in bucket_transfers:
                running_balance += t['amount']
                net_change += t['amount']
                volume += t['abs_amount']
                high_balance = max(high_balance, running_balance)
                low_balance = min(low_balance, running_balance)

            close_balance = running_balance

            # Ensure non-negative
            open_balance = max(0, open_balance)
            high_balance = max(0, high_balance)
            low_balance = max(0, low_balance)
            close_balance = max(0, close_balance)

            candles.append(BalanceCandle(
                time=int(bucket_time.timestamp()),
                open=open_balance,
                high=high_balance,
                low=low_balance,
                close=close_balance,
                volume=volume,
                tx_count=tx_count,
                net_change=net_change,
            ))

        return candles

    # -------------------------------------------------------------------------
    # LENDING VOLUME CHART
    # -------------------------------------------------------------------------

    async def fetch_lending_chart(self, address: str, resolution: Resolution = Resolution.H1,
                                   lookback: Lookback = Lookback.MONTH_1) -> Dict:
        """Fetch lending/borrowing volume as candles"""
        print(f"\n[LENDING] Fetching {resolution.tv_code} candles, {lookback.label} lookback...")

        cutoff = self.get_lookback_cutoff(lookback)

        query = """
        {
            user(id: "%s") {
                transactions(first: 1000, orderBy: createdAt, orderDirection: desc) {
                    createdAt
                    side
                    currency
                    futureValue
                }
            }
        }
        """ % address.lower()

        data = await self.fetch_json(
            CONFIG['sf_subgraph'],
            method="POST",
            json_data={"query": query},
            use_cache=False
        )

        user = data.get('data', {}).get('user')
        if not user:
            return {"candles": [], "resolution": resolution.tv_code, "lookback": lookback.label}

        # Process transactions
        events = []
        for t in user.get('transactions', []):
            if t.get('currency') != CONFIG['currency_usdfc']:
                continue

            ts = datetime.fromtimestamp(int(t.get('createdAt', 0)))

            if cutoff and ts < cutoff:
                continue

            side = 'lend' if t.get('side') in [0, '0'] else 'borrow'
            amount = int(t.get('futureValue', 0)) / 1e18

            events.append({
                'timestamp': ts,
                'side': side,
                'amount': amount,
            })

        print(f"  Found {len(events)} lending transactions")

        if not events:
            return {"candles": [], "resolution": resolution.tv_code, "lookback": lookback.label}

        # Build volume candles
        candles = self._build_lending_candles(events, resolution)

        return {
            "candles": [asdict(c) for c in candles],
            "resolution": resolution.tv_code,
            "lookback": lookback.label,
            "data_points": len(candles),
            "total_lend": sum(c.lend_volume for c in candles),
            "total_borrow": sum(c.borrow_volume for c in candles),
        }

    def _build_lending_candles(self, events: List[dict],
                                resolution: Resolution) -> List[VolumeCandle]:
        """Build lending volume candles"""
        if not events:
            return []

        # Group by bucket
        buckets = defaultdict(lambda: {
            'lend_volume': 0, 'borrow_volume': 0,
            'lend_count': 0, 'borrow_count': 0
        })

        for e in events:
            bucket = self.round_to_resolution(e['timestamp'], resolution)
            if e['side'] == 'lend':
                buckets[bucket]['lend_volume'] += e['amount']
                buckets[bucket]['lend_count'] += 1
            else:
                buckets[bucket]['borrow_volume'] += e['amount']
                buckets[bucket]['borrow_count'] += 1

        candles = []
        for bucket_time in sorted(buckets.keys()):
            data = buckets[bucket_time]
            candles.append(VolumeCandle(
                time=int(bucket_time.timestamp()),
                lend_volume=data['lend_volume'],
                borrow_volume=data['borrow_volume'],
                lend_count=data['lend_count'],
                borrow_count=data['borrow_count'],
                net_flow=data['lend_volume'] - data['borrow_volume'],
                total_volume=data['lend_volume'] + data['borrow_volume'],
            ))

        return candles

    # -------------------------------------------------------------------------
    # PRICE OHLCV CHART (DEX Pool)
    # -------------------------------------------------------------------------

    async def fetch_price_chart(self, resolution: Resolution = Resolution.H1,
                                 lookback: Lookback = Lookback.WEEK_1,
                                 pool: str = None) -> Dict:
        """Fetch OHLCV price candles from GeckoTerminal"""
        pool = pool or CONFIG['usdfc_wfil_pool']

        print(f"\n[PRICE] Fetching {resolution.tv_code} candles, {lookback.label} lookback...")

        # Map resolution to GeckoTerminal API
        if resolution.minutes <= 15:
            tf_api = "minute"
            aggregate = resolution.minutes
        elif resolution.minutes <= 720:
            tf_api = "hour"
            aggregate = resolution.minutes // 60
        else:
            tf_api = "day"
            aggregate = resolution.minutes // 1440

        # Calculate limit based on lookback
        if lookback == Lookback.ALL:
            limit = 1000
        else:
            limit = min(1000, (lookback.total_minutes // resolution.minutes) + 10)

        url = f"{CONFIG['geckoterminal']}/networks/filecoin/pools/{pool}/ohlcv/{tf_api}?aggregate={aggregate}&limit={limit}"
        data = await self.fetch_json(url, use_cache=False)

        ohlcv_list = data.get('data', {}).get('attributes', {}).get('ohlcv_list', [])

        cutoff = self.get_lookback_cutoff(lookback)
        cutoff_ts = cutoff.timestamp() if cutoff else 0

        candles = []
        for c in ohlcv_list:
            ts = c[0]
            if cutoff and ts < cutoff_ts:
                continue

            candles.append(TVCandle(
                time=int(ts),
                open=float(c[1]),
                high=float(c[2]),
                low=float(c[3]),
                close=float(c[4]),
                volume=float(c[5]),
            ))

        # Sort chronologically
        candles.sort(key=lambda x: x.time)

        print(f"  Got {len(candles)} candles")

        return {
            "candles": [asdict(c) for c in candles],
            "resolution": resolution.tv_code,
            "lookback": lookback.label,
            "data_points": len(candles),
            "latest_price": candles[-1].close if candles else 0,
        }

    # -------------------------------------------------------------------------
    # OPERATION MARKERS (Chart Overlay)
    # -------------------------------------------------------------------------

    async def fetch_operation_markers(self, address: str,
                                       lookback: Lookback = Lookback.MONTH_1) -> Dict:
        """Fetch operation markers for chart overlay"""
        print(f"\n[OPERATIONS] Fetching markers, {lookback.label} lookback...")

        cutoff = self.get_lookback_cutoff(lookback)

        url = f"{CONFIG['blockscout_rest']}/addresses/{address}/transactions"
        data = await self.fetch_json(url, use_cache=False)

        items = data.get('items', [])
        markers = []
        op_counts = defaultdict(int)

        for tx in items:
            ts_str = tx.get('timestamp')
            if not ts_str:
                continue

            ts = datetime.fromisoformat(ts_str.replace('Z', '+00:00')).replace(tzinfo=None)

            if cutoff and ts < cutoff:
                continue

            method = tx.get('method') or 'unknown'
            op_type, label, color = classify_operation(method)
            amount = float(tx.get('value', 0)) / 1e18

            op_counts[op_type.value] += 1

            markers.append(OperationMarker(
                time=int(ts.timestamp()),
                operation=op_type.value,
                amount=amount,
                tx_hash=tx.get('hash', ''),
                label=label,
                color=color,
            ))

        print(f"  Found {len(markers)} operations")

        return {
            "markers": [asdict(m) for m in markers],
            "lookback": lookback.label,
            "count": len(markers),
            "breakdown": dict(op_counts),
        }

# =============================================================================
# COMPREHENSIVE CHART DATA API
# =============================================================================

async def get_realtime_chart_data(
    address: str,
    resolution: Resolution = Resolution.H1,
    lookback: Lookback = Lookback.WEEK_1,
    include_price: bool = True,
    include_lending: bool = True,
    include_operations: bool = True,
) -> Dict:
    """
    Get all chart data for TradingView-style display

    Args:
        address: Wallet address
        resolution: Candle resolution (M1, M5, M15, M30, H1, H4, D1, W1)
        lookback: Historical lookback period (1h, 4h, 1d, 7d, 30d, all)
        include_price: Include DEX price chart
        include_lending: Include lending volume chart
        include_operations: Include operation markers

    Returns:
        Dict with all chart data
    """
    print("=" * 70)
    print(f"TRADINGVIEW CHART DATA")
    print(f"Address: {address[:20]}...")
    print(f"Resolution: {resolution.tv_code} | Lookback: {lookback.label}")
    print("=" * 70)

    start_time = time.time()

    async with TradingViewDataFetcher() as fetcher:
        # Always fetch balance chart
        tasks = [fetcher.fetch_balance_chart(address, resolution, lookback)]

        if include_price:
            tasks.append(fetcher.fetch_price_chart(resolution, lookback))
        if include_lending:
            tasks.append(fetcher.fetch_lending_chart(address, resolution, lookback))
        if include_operations:
            tasks.append(fetcher.fetch_operation_markers(address, lookback))

        results = await asyncio.gather(*tasks)

    # Build response
    response = {
        "address": address,
        "resolution": resolution.tv_code,
        "lookback": lookback.label,
        "generated_at": datetime.now().isoformat(),
        "fetch_time_ms": int((time.time() - start_time) * 1000),
        "balance_chart": results[0],
    }

    idx = 1
    if include_price:
        response["price_chart"] = results[idx]
        idx += 1
    if include_lending:
        response["lending_chart"] = results[idx]
        idx += 1
    if include_operations:
        response["operations"] = results[idx]

    print(f"\n{'=' * 70}")
    print(f"COMPLETE - {response['fetch_time_ms']}ms")
    print(f"{'=' * 70}")

    return response

# =============================================================================
# STREAMING / REAL-TIME SUPPORT
# =============================================================================

class ChartDataStream:
    """Real-time chart data streaming"""

    def __init__(self, address: str, resolution: Resolution = Resolution.M1):
        self.address = address
        self.resolution = resolution
        self.running = False
        self.callbacks = []
        self.last_data = None

    def subscribe(self, callback):
        """Subscribe to data updates"""
        self.callbacks.append(callback)

    async def start(self, interval_seconds: int = 30):
        """Start streaming data"""
        self.running = True
        print(f"Starting real-time stream for {self.address[:10]}...")
        print(f"Resolution: {self.resolution.tv_code}, Update interval: {interval_seconds}s")

        while self.running:
            try:
                data = await get_realtime_chart_data(
                    self.address,
                    resolution=self.resolution,
                    lookback=Lookback.HOUR_1,
                    include_price=True,
                    include_lending=False,
                    include_operations=False,
                )

                self.last_data = data

                for callback in self.callbacks:
                    await callback(data)

            except Exception as e:
                print(f"Stream error: {e}")

            await asyncio.sleep(interval_seconds)

    def stop(self):
        """Stop streaming"""
        self.running = False

# =============================================================================
# MAIN - DEMO
# =============================================================================

async def main():
    address = "0x56db3D50f8711fbab02a37e80D0eD6b019D5F654"

    # Test different lookback periods (use longer periods for demo)
    lookbacks = [Lookback.MONTH_1, Lookback.MONTH_3, Lookback.ALL]
    resolutions = [Resolution.H1, Resolution.H4, Resolution.D1]

    for lb in lookbacks:
        for res in resolutions:
            print(f"\n\n{'#' * 70}")
            print(f"# {res.tv_code} Resolution | {lb.label} Lookback")
            print(f"{'#' * 70}")

            data = await get_realtime_chart_data(
                address,
                resolution=res,
                lookback=lb,
            )

            # Save to file
            filename = f"tv_chart_{res.tv_code}_{lb.label}.json"
            with open(filename, 'w') as f:
                json.dump(data, f, indent=2)
            print(f"Saved: {filename}")

            # Print summary
            print(f"\nSummary:")
            print(f"  Balance candles: {data['balance_chart'].get('data_points', len(data['balance_chart'].get('candles', [])))}")
            if 'price_chart' in data:
                print(f"  Price candles: {data['price_chart'].get('data_points', len(data['price_chart'].get('candles', [])))}")
            if 'lending_chart' in data:
                print(f"  Lending candles: {data['lending_chart'].get('data_points', len(data['lending_chart'].get('candles', [])))}")
            if 'operations' in data:
                print(f"  Operations: {data['operations'].get('count', len(data['operations'].get('markers', [])))}")

            # Only do first combination for demo
            break
        break

    # Show TradingView-compatible format
    print("\n\n" + "=" * 70)
    print("TRADINGVIEW-COMPATIBLE OUTPUT FORMAT")
    print("=" * 70)

    sample = await get_realtime_chart_data(
        address,
        resolution=Resolution.H1,
        lookback=Lookback.WEEK_1,
    )

    print("\n1. BALANCE CHART (OHLC Format):")
    for c in sample['balance_chart']['candles'][-3:]:
        dt = datetime.fromtimestamp(c['time'])
        print(f"   {dt} | O:{c['open']:.2f} H:{c['high']:.2f} L:{c['low']:.2f} C:{c['close']:.2f} V:{c['volume']:.2f}")

    print("\n2. PRICE CHART (Standard OHLCV):")
    for c in sample['price_chart']['candles'][-3:]:
        dt = datetime.fromtimestamp(c['time'])
        print(f"   {dt} | O:{c['open']:.4f} H:{c['high']:.4f} L:{c['low']:.4f} C:{c['close']:.4f} V:{c['volume']:.0f}")

    print("\n3. OPERATION MARKERS:")
    for m in sample['operations']['markers'][:5]:
        dt = datetime.fromtimestamp(m['time'])
        print(f"   {dt} | {m['label']:8} | {m['operation']:15} | {m['color']}")

if __name__ == "__main__":
    asyncio.run(main())
