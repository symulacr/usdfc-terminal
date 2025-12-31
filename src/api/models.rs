//! REST API response models for USDFC Terminal
//!
//! These types wrap internal data structures in a consistent API format.

use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            timestamp: current_timestamp(),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            timestamp: current_timestamp(),
            error: Some(message.into()),
        }
    }
}

/// Price data response
#[derive(Serialize)]
pub struct PriceResponse {
    /// USDFC price in USD (None if unavailable)
    pub usdfc_usd: Option<f64>,
    /// FIL price in USD (None if unavailable)
    pub fil_usd: Option<f64>,
    /// 24h price change percentage (None if unavailable)
    pub change_24h: Option<f64>,
    /// 24h trading volume in USD (None if unavailable)
    pub volume_24h: Option<f64>,
    /// Pool liquidity in USD (None if unavailable)
    pub liquidity_usd: Option<f64>,
}

/// Protocol metrics response
#[derive(Serialize)]
pub struct MetricsResponse {
    /// Total Collateralization Ratio as percentage
    pub tcr: String,
    /// Total USDFC supply
    pub total_supply: String,
    /// Circulating USDFC supply (excludes stability pool)
    pub circulating_supply: String,
    /// Total FIL collateral locked
    pub total_collateral: String,
    /// Number of active troves
    pub active_troves: u64,
    /// Number of USDFC token holders
    pub holders: Option<u64>,
    /// 24h trading volume in USD (None if unavailable)
    pub volume_24h: Option<f64>,
    /// Pool liquidity in USD (None if unavailable)
    pub liquidity_usd: Option<f64>,
    /// Stability pool USDFC balance
    pub stability_pool_balance: String,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,
    /// Individual service statuses
    pub services: Vec<ServiceStatus>,
}

/// Individual service health status
#[derive(Serialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Status: "healthy", "degraded", or "unhealthy"
    pub status: String,
    /// Response latency in milliseconds (None if not measured)
    pub latency_ms: Option<u64>,
}

/// Trove data response
#[derive(Serialize)]
pub struct TroveResponse {
    /// Owner address
    pub address: String,
    /// Collateral amount in FIL
    pub collateral: String,
    /// Debt amount in USDFC
    pub debt: String,
    /// Individual Collateralization Ratio as percentage
    pub icr: String,
    /// Status: "active", "at_risk", "critical", or "closed"
    pub status: String,
}

/// Paginated troves response
#[derive(Serialize)]
pub struct TrovesListResponse {
    /// List of troves
    pub troves: Vec<TroveResponse>,
    /// Total count of troves
    pub total: u64,
    /// Current page offset
    pub offset: u32,
    /// Page size limit
    pub limit: u32,
}

/// Transaction data response
#[derive(Serialize)]
pub struct TransactionResponse {
    /// Transaction hash
    pub hash: String,
    /// Transaction type
    pub tx_type: String,
    /// Amount in USDFC
    pub amount: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// Block number
    pub block: u64,
    /// Status: "pending", "success", or "failed"
    pub status: String,
}

/// Paginated transactions response
#[derive(Serialize)]
pub struct TransactionsListResponse {
    /// List of transactions
    pub transactions: Vec<TransactionResponse>,
    /// Total count of transactions
    pub total: u64,
    /// Current page offset
    pub offset: u32,
    /// Page size limit
    pub limit: u32,
}

/// Address info response
#[derive(Serialize)]
pub struct AddressInfoResponse {
    /// Normalized address (EVM format)
    pub address: String,
    /// USDFC balance
    pub usdfc_balance: String,
    /// Total number of transfers
    pub transfer_count: u64,
    /// First seen timestamp
    pub first_seen: String,
    /// Address type: "eoa", "contract", "protocol"
    pub address_type: String,
    /// Filecoin f4 address (if applicable)
    pub f4_address: Option<String>,
}

/// Lending market data response
#[derive(Serialize)]
pub struct LendingMarketResponse {
    /// Market maturity timestamp
    pub maturity: String,
    /// Currency symbol
    pub currency: String,
    /// Current lending APR
    pub lend_apr: f64,
    /// Current borrowing APR
    pub borrow_apr: f64,
    /// Total volume
    pub volume: String,
    /// Whether market is active
    pub is_active: bool,
}

/// Lending markets list response
#[derive(Serialize)]
pub struct LendingMarketsResponse {
    /// List of lending markets
    pub markets: Vec<LendingMarketResponse>,
}

/// Historical data point
#[derive(Serialize)]
pub struct HistoricalDataPoint {
    /// Unix timestamp
    pub timestamp: i64,
    /// Metric value
    pub value: f64,
}

/// Historical data response
#[derive(Serialize)]
pub struct HistoricalResponse {
    /// Metric name
    pub metric: String,
    /// Data resolution (e.g., "1h", "1d")
    pub resolution: String,
    /// Start timestamp
    pub from: i64,
    /// End timestamp
    pub to: i64,
    /// Data points
    pub data: Vec<HistoricalDataPoint>,
}

/// Query parameters for historical data
#[derive(Deserialize)]
pub struct HistoryQuery {
    /// Metric to retrieve: price, volume, tcr, supply, holders, etc.
    pub metric: Option<String>,
    /// Start timestamp (Unix seconds)
    pub from: Option<i64>,
    /// End timestamp (Unix seconds)
    pub to: Option<i64>,
    /// Resolution: 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w
    pub resolution: Option<String>,
}

/// Query parameters for pagination
#[derive(Deserialize)]
pub struct PaginationQuery {
    /// Number of items per page (default: 20, max: 100)
    pub limit: Option<u32>,
    /// Page offset (default: 0)
    pub offset: Option<u32>,
}

/// Token holder response
#[derive(Serialize)]
pub struct TokenHolderResponse {
    /// Holder address
    pub address: String,
    /// USDFC balance
    pub balance: String,
    /// Percentage of total supply
    pub share: Option<f64>,
}

/// Top holders list response
#[derive(Serialize)]
pub struct TopHoldersResponse {
    /// List of top holders
    pub holders: Vec<TokenHolderResponse>,
    /// Total holder count
    pub total_holders: Option<u64>,
}

// Helper function to get current Unix timestamp
fn current_timestamp() -> i64 {
    #[cfg(feature = "ssr")]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
    #[cfg(not(feature = "ssr"))]
    {
        0
    }
}
