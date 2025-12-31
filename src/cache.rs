//! Server-side caching utilities for USDFC Analytics Terminal
//!
//! Provides TTL-based caching for expensive API calls to improve performance.

#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use std::sync::RwLock;
#[cfg(feature = "ssr")]
use std::time::{Duration, Instant};

/// Cached data entry with TTL
#[cfg(feature = "ssr")]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

/// Simple TTL-based cache
#[cfg(feature = "ssr")]
pub struct Cache<T> {
    entries: RwLock<HashMap<String, CacheEntry<T>>>,
    ttl: Duration,
}

#[cfg(feature = "ssr")]
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
#[cfg(feature = "ssr")]
pub mod caches {
    use super::*;
    use once_cell::sync::Lazy;
    use crate::types::{ProtocolMetrics, Trove, Transaction};
    use crate::server_fn::{
        USDFCPriceData, LendingMarketData, TokenHolderInfo,
    };
    use rust_decimal::Decimal;

    /// Cache for protocol metrics (15 second TTL - updates frequently)
    pub static PROTOCOL_METRICS: Lazy<Cache<ProtocolMetrics>> = Lazy::new(|| Cache::new(15));

    /// Cache for troves list (120 second TTL - trove data changes slowly)
    pub static TROVES: Lazy<Cache<Vec<Trove>>> = Lazy::new(|| Cache::new(120));

    /// Cache for FIL price (30 second TTL - price sensitive but not ultra-frequent)
    pub static FIL_PRICE: Lazy<Cache<Decimal>> = Lazy::new(|| Cache::new(30));

    /// Cache for recent transactions (15 second TTL)
    pub static RECENT_TRANSACTIONS: Lazy<Cache<Vec<Transaction>>> = Lazy::new(|| Cache::new(15));

    /// Cache for USDFC price data (30 second TTL)
    pub static USDFC_PRICE: Lazy<Cache<USDFCPriceData>> = Lazy::new(|| Cache::new(30));

    /// Cache for total supply (60 second TTL - changes infrequently)
    pub static TOTAL_SUPPLY: Lazy<Cache<Decimal>> = Lazy::new(|| Cache::new(60));

    /// Cache for total collateral (60 second TTL - changes infrequently)
    pub static TOTAL_COLLATERAL: Lazy<Cache<Decimal>> = Lazy::new(|| Cache::new(60));

    /// Cache for TCR (30 second TTL)
    pub static TCR: Lazy<Cache<Decimal>> = Lazy::new(|| Cache::new(30));

    /// Cache for stability pool balance (60 second TTL)
    pub static STABILITY_POOL_BALANCE: Lazy<Cache<Decimal>> = Lazy::new(|| Cache::new(60));

    /// Cache for lending pools/markets (60 second TTL)
    pub static LENDING_MARKETS: Lazy<Cache<Vec<LendingMarketData>>> = Lazy::new(|| Cache::new(60));

    /// Cache for token holders (300 second TTL - holder list changes slowly)
    pub static TOKEN_HOLDERS: Lazy<Cache<Vec<TokenHolderInfo>>> = Lazy::new(|| Cache::new(300));

    /// Cache for holder count (300 second TTL - count changes slowly)
    pub static HOLDER_COUNT: Lazy<Cache<u64>> = Lazy::new(|| Cache::new(300));
}

/// Helper macro for cached server function calls
///
/// Usage:
/// ```ignore
/// cached_call!(caches::PROTOCOL_METRICS, "default", {
///     // expensive async operation
/// })
/// ```
#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
pub use cached_call;
