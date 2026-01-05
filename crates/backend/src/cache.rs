//! Server-side caching utilities for USDFC Analytics Terminal
//!
//! Provides TTL-based caching for expensive API calls to improve performance.


use std::collections::HashMap;

use std::sync::RwLock;

use std::time::{Duration, Instant};

/// Cached data entry with TTL

struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

/// Simple TTL-based cache

pub struct Cache<T> {
    entries: RwLock<HashMap<String, CacheEntry<T>>>,
    ttl: Duration,
}


impl<T: Clone> Cache<T> {
    /// Create a new cache with the specified TTL
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get a cached value if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<T> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(key)?;

        if Instant::now() < entry.expires_at {
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Store a value in the cache
    pub fn set(&self, key: String, data: T) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key, CacheEntry {
                data,
                expires_at: Instant::now() + self.ttl,
            });
        }
    }

    /// Remove expired entries (call periodically to prevent memory leaks)
    pub fn cleanup(&self) {
        if let Ok(mut entries) = self.entries.write() {
            let now = Instant::now();
            entries.retain(|_, entry| now < entry.expires_at);
        }
    }
}

/// Global cache instances for different data types

pub mod caches {
    use super::*;
    use once_cell::sync::Lazy;
    use usdfc_core::types::{ProtocolMetrics, Trove, Transaction, ChartDataResponse};
    use crate::server_fn::{
        AddressInfo, USDFCPriceData, LendingMarketData, TokenHolderInfo, DailyVolumeData, OrderBookData, LendingTradeData,
    };

    /// Cache for protocol metrics (15 second TTL - updates frequently)
    pub static PROTOCOL_METRICS: Lazy<Cache<ProtocolMetrics>> = Lazy::new(|| Cache::new(15));

    /// Cache for troves list (30 second TTL - aligned with price updates for ICR accuracy)
    pub static TROVES: Lazy<Cache<Vec<Trove>>> = Lazy::new(|| Cache::new(30));

    /// Cache for USDFC price data (30 second TTL)
    pub static USDFC_PRICE: Lazy<Cache<USDFCPriceData>> = Lazy::new(|| Cache::new(30));

    /// Cache for lending pools/markets (60 second TTL)
    pub static LENDING_MARKETS: Lazy<Cache<Vec<LendingMarketData>>> = Lazy::new(|| Cache::new(60));

    /// Cache for token holders (300 second TTL - holder list changes slowly)
    pub static TOKEN_HOLDERS: Lazy<Cache<Vec<TokenHolderInfo>>> = Lazy::new(|| Cache::new(300));

    /// Cache for holder count (300 second TTL - count changes slowly)
    pub static HOLDER_COUNT: Lazy<Cache<u64>> = Lazy::new(|| Cache::new(300));

    /// Cache for advanced chart data (30 second TTL - balances freshness with API load)
    pub static ADVANCED_CHART_DATA: Lazy<Cache<ChartDataResponse>> = Lazy::new(|| Cache::new(30));

    // NEW CACHES FOR CORE CHANGE #3
    /// Cache for recent transactions (10 second TTL - new tx appear frequently)
    pub static RECENT_TRANSACTIONS: Lazy<Cache<Vec<Transaction>>> = Lazy::new(|| Cache::new(10));

    /// Cache for address info (30 second TTL - balance changes moderately)
    pub static ADDRESS_INFO: Lazy<Cache<AddressInfo>> = Lazy::new(|| Cache::new(30));

    /// Cache for daily volumes (300 second TTL - historical data, changes slowly)
    pub static DAILY_VOLUMES: Lazy<Cache<Vec<DailyVolumeData>>> = Lazy::new(|| Cache::new(300));

    /// Cache for stability pool transfers (30 second TTL)
    pub static STABILITY_TRANSFERS: Lazy<Cache<Vec<Transaction>>> = Lazy::new(|| Cache::new(30));

    /// Cache for order book data (5 second TTL - real-time trading data)
    pub static ORDER_BOOK: Lazy<Cache<OrderBookData>> = Lazy::new(|| Cache::new(5));

    /// Cache for recent lending trades (30 second TTL)
    pub static LENDING_TRADES: Lazy<Cache<Vec<LendingTradeData>>> = Lazy::new(|| Cache::new(30));

    /// Start background task to periodically clean expired cache entries
    /// Prevents memory leaks from accumulating expired entries
    pub fn start_cache_cleanup() {
        tokio::spawn(async {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;

                // Clean all cache instances
                PROTOCOL_METRICS.cleanup();
                TROVES.cleanup();
                USDFC_PRICE.cleanup();
                LENDING_MARKETS.cleanup();
                TOKEN_HOLDERS.cleanup();
                HOLDER_COUNT.cleanup();
                ADVANCED_CHART_DATA.cleanup();
                // Clean new caches
                RECENT_TRANSACTIONS.cleanup();
                ADDRESS_INFO.cleanup();
                DAILY_VOLUMES.cleanup();
                STABILITY_TRANSFERS.cleanup();
                ORDER_BOOK.cleanup();
                LENDING_TRADES.cleanup();

                tracing::debug!("Cleaned expired cache entries");
            }
        });
    }
}

/// Helper macro for cached server function calls
///
/// Usage:
/// ```ignore
/// cached_call!(caches::PROTOCOL_METRICS, "default", {
///     // expensive async operation
/// })
/// ```

#[macro_export]
macro_rules! cached_call {
    ($cache:expr, $key:expr, $fetch:expr) => {{
        // Try to get from cache first
        if let Some(cached) = $cache.get($key) {
            Ok(cached)
        } else {
            // Fetch fresh data
            let result = $fetch;
            match &result {
                Ok(data) => {
                    $cache.set($key.to_string(), data.clone());
                }
                Err(_) => {}
            }
            result
        }
    }};
}


pub use cached_call;
