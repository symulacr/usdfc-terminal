//! GeckoTerminal API Client
//!
//! Provides access to GeckoTerminal's DEX aggregator API for:
//! - Token price and market data
//! - Historical OHLCV data
//! - DEX pool information
//! - Recent trades

use crate::config::config;
use crate::error::{ApiError, ApiResult};
use governor::{Quota, RateLimiter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Maximum retry attempts for rate-limited requests
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Global rate limiter: 30 requests per 60 seconds (GeckoTerminal API limit)
static RATE_LIMITER: Lazy<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> = Lazy::new(|| {
    Arc::new(RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(30).unwrap())
    ))
});

/// GeckoTerminal API client
pub struct GeckoClient {
    client: Client,
    base_url: String,
}

impl GeckoClient {
    /// Create a new GeckoTerminal client
    pub fn new() -> Self {
        // Use configured GeckoTerminal base URL, trimming any trailing slash for consistency.
        // Expected format (see .env.example):
        // GECKOTERMINAL_URL=https://api.geckoterminal.com/api/v2/networks/filecoin
        let base = config().geckoterminal_url.trim_end_matches('/').to_string();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build gecko HTTP client"),
            base_url: base,
        }
    }

    /// Make a rate-limited request with automatic retry on 429 responses.
    ///
    /// This method:
    /// 1. Waits for a rate limit permit before making the request
    /// 2. Retries with exponential backoff if a 429 (Too Many Requests) is received
    /// 3. Returns an error after MAX_RETRY_ATTEMPTS failed attempts
    async fn rate_limited_request(&self, url: &str) -> ApiResult<reqwest::Response> {
        let mut attempts = 0;

        loop {
            // Wait for rate limit permit
            RATE_LIMITER.until_ready().await;

            let response = self
                .client
                .get(url)
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| ApiError::HttpError(format!("GeckoTerminal request failed: {}", e)))?;

            // Handle rate limit response (429 Too Many Requests)
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                attempts += 1;
                if attempts >= MAX_RETRY_ATTEMPTS {
                    return Err(ApiError::RateLimit {
                        retry_after: Duration::from_secs(60),
                    });
                }
                // Exponential backoff: 2^attempts seconds (2, 4, 8, ...)
                let backoff_secs = 2_u64.pow(attempts);
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                continue;
            }

            return Ok(response);
        }
    }

    /// Get token information (price, supply, market cap)
    pub async fn get_token_info(&self, token_address: &str) -> ApiResult<TokenInfo> {
        let url = format!("{}/tokens/{}", self.base_url, token_address);

        let response = self.rate_limited_request(&url).await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "GeckoTerminal API error: {}",
                response.status()
            )));
        }

        let data: TokenResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse token info: {}", e)))?;

        Ok(data.data.attributes)
    }

    /// Get pool OHLCV data (price, volume over time)
    ///
    /// # Arguments
    /// * `pool_address` - DEX pool address
    /// * `timeframe` - "minute", "hour", or "day"
    /// * `aggregate` - Aggregation interval (e.g., 1, 5, 15 for minute; 1, 4, 12 for hour)
    /// * `limit` - Number of data points (max 100)
    pub async fn get_pool_ohlcv(
        &self,
        pool_address: &str,
        timeframe: &str,
        aggregate: u32,
        limit: u32,
    ) -> ApiResult<Vec<OHLCV>> {
        let url = format!(
            "{}/pools/{}/ohlcv/{}?aggregate={}&limit={}",
            self.base_url, pool_address, timeframe, aggregate, limit
        );

        let response = self.rate_limited_request(&url).await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "GeckoTerminal API error: {}",
                response.status()
            )));
        }

        let data: OHLCVResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse OHLCV data: {}", e)))?;

        // Convert array format to struct format
        let ohlcv_list = data
            .data
            .attributes
            .ohlcv_list
            .into_iter()
            .map(|arr| OHLCV {
                timestamp: arr[0] as i64,
                open: arr[1],
                high: arr[2],
                low: arr[3],
                close: arr[4],
                volume: arr[5],
            })
            .collect();

        Ok(ohlcv_list)
    }

    /// Get pool information (liquidity, volume, transactions)
    pub async fn get_pool_info(&self, pool_address: &str) -> ApiResult<PoolInfo> {
        let url = format!("{}/pools/{}", self.base_url, pool_address);

        let response = self.rate_limited_request(&url).await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "GeckoTerminal API error: {}",
                response.status()
            )));
        }

        let data: PoolResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse pool info: {}", e)))?;

        Ok(data.data.attributes)
    }

    /// Get recent trades from a pool
    pub async fn get_pool_trades(
        &self,
        pool_address: &str,
        limit: u32,
    ) -> ApiResult<Vec<Trade>> {
        let url = format!("{}/pools/{}/trades", self.base_url, pool_address);

        let response = self.rate_limited_request(&url).await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "GeckoTerminal API error: {}",
                response.status()
            )));
        }

        let data: TradesResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse trades: {}", e)))?;

        Ok(data
            .data
            .into_iter()
            .take(limit as usize)
            .map(|t| t.attributes)
            .collect())
    }

    /// Get all pools for a token
    pub async fn get_token_pools(&self, token_address: &str) -> ApiResult<Vec<PoolInfo>> {
        let url = format!("{}/tokens/{}/pools", self.base_url, token_address);

        let response = self.rate_limited_request(&url).await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "GeckoTerminal API error: {}",
                response.status()
            )));
        }

        let data: PoolsResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse pools: {}", e)))?;

        Ok(data.data.into_iter().map(|p| p.attributes).collect())
    }
}

impl Default for GeckoClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct TokenResponse {
    data: TokenData,
}

#[derive(Debug, Deserialize)]
struct TokenData {
    attributes: TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub coingecko_coin_id: Option<String>,
    pub total_supply: String,
    #[serde(default)]
    pub price_usd: Option<String>,
    #[serde(default)]
    pub fdv_usd: Option<String>,
    #[serde(default)]
    pub market_cap_usd: Option<String>,
    #[serde(default)]
    pub volume_usd: Option<VolumeData>,
    #[serde(default)]
    pub price_change_percentage: Option<PriceChangeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeData {
    #[serde(default)]
    pub m5: Option<String>,
    #[serde(default)]
    pub h1: Option<String>,
    #[serde(default)]
    pub h6: Option<String>,
    #[serde(default)]
    pub h24: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceChangeData {
    #[serde(default)]
    pub m5: Option<String>,
    #[serde(default)]
    pub h1: Option<String>,
    #[serde(default)]
    pub h6: Option<String>,
    #[serde(default)]
    pub h24: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OHLCVResponse {
    data: OHLCVData,
}

#[derive(Debug, Deserialize)]
struct OHLCVData {
    attributes: OHLCVAttributes,
}

#[derive(Debug, Deserialize)]
struct OHLCVAttributes {
    ohlcv_list: Vec<Vec<f64>>, // [timestamp, open, high, low, close, volume]
}

/// OHLCV data point (Open, High, Low, Close, Volume)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCV {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Deserialize)]
struct PoolResponse {
    data: PoolData,
}

#[derive(Debug, Deserialize)]
struct PoolsResponse {
    data: Vec<PoolData>,
}

#[derive(Debug, Deserialize)]
struct PoolData {
    attributes: PoolInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: String,
    pub name: String,
    #[serde(default)]
    pub base_token_price_usd: Option<String>,
    #[serde(default)]
    pub quote_token_price_usd: Option<String>,
    #[serde(default)]
    pub reserve_in_usd: Option<String>,
    #[serde(default)]
    pub volume_usd: Option<VolumeData>,
    #[serde(default)]
    pub price_change_percentage: Option<PriceChangeData>,
    #[serde(default)]
    pub transactions: Option<TransactionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionData {
    #[serde(default)]
    pub m5: Option<TransactionCount>,
    #[serde(default)]
    pub h1: Option<TransactionCount>,
    #[serde(default)]
    pub h6: Option<TransactionCount>,
    #[serde(default)]
    pub h24: Option<TransactionCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCount {
    pub buys: u32,
    pub sells: u32,
    pub buyers: u32,
    pub sellers: u32,
}

#[derive(Debug, Deserialize)]
struct TradesResponse {
    data: Vec<TradeData>,
}

#[derive(Debug, Deserialize)]
struct TradeData {
    attributes: Trade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub block_number: u64,
    pub tx_hash: String,
    pub tx_from_address: String,
    #[serde(default)]
    pub from_token_amount: Option<String>,
    #[serde(default)]
    pub to_token_amount: Option<String>,
    #[serde(default)]
    pub price_from_in_usd: Option<String>,
    #[serde(default)]
    pub price_to_in_usd: Option<String>,
    pub block_timestamp: String,
    pub kind: String, // "buy" or "sell"
    #[serde(default)]
    pub volume_in_usd: Option<String>,
}

 
