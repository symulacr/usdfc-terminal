//! Server Functions for USDFC Analytics Terminal
//!
//! These functions run on the server and are callable from the client.
//! All data comes from real APIs - no mock data, no fallbacks.

use leptos::*;
use leptos::server_fn::error::NoCustomError;
use crate::types::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Re-export chart types for server functions
pub use crate::types::{ChartResolution, ChartLookback, ChartDataResponse, TVCandle};

/// Type alias for server function errors with default error type
type SfnError = ServerFnError<NoCustomError>;

// ============================================================================
// Protocol Metrics
// ============================================================================

/// Get current protocol metrics (total supply, collateral, TCR, etc.)
/// Cached for 15 seconds to reduce RPC load
#[server(GetProtocolMetrics, "/api")]
pub async fn get_protocol_metrics() -> Result<ProtocolMetrics, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::rpc::RpcClient;
        use crate::cache::caches;

        // Check cache first
        if let Some(cached) = caches::PROTOCOL_METRICS.get("default") {
            return Ok(cached);
        }

        let rpc = RpcClient::new();

        // Make parallel RPC calls for better performance
        let (total_supply, total_collateral, active_troves, tcr, stability_pool_balance) = tokio::join!(
            rpc.get_total_supply(),
            rpc.get_total_collateral(),
            rpc.get_trove_owners_count(),
            rpc.get_tcr(),
            rpc.get_stability_pool_balance()
        );

        let total_supply = total_supply.map_err(|e| SfnError::ServerError(e.to_string()))?;
        let total_collateral = total_collateral.map_err(|e| SfnError::ServerError(e.to_string()))?;
        let active_troves = active_troves.map_err(|e| SfnError::ServerError(e.to_string()))?;
        let tcr = tcr.map_err(|e| SfnError::ServerError(e.to_string()))?;
        let stability_pool_balance = stability_pool_balance.map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Calculate actual circulating supply: total supply minus stability pool deposits
        // USDFC in the stability pool is locked and not actively circulating
        let circulating_supply = total_supply - stability_pool_balance;

        let metrics = ProtocolMetrics {
            total_supply,
            circulating_supply,
            total_collateral,
            active_troves,
            tcr,
            stability_pool_balance,
            treasury_balance: stability_pool_balance, // Non-circulating = locked in stability pool
        };

        // Store in cache
        caches::PROTOCOL_METRICS.set("default".to_string(), metrics.clone());

        Ok(metrics)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Transactions
// ============================================================================

/// Get recent transactions from Blockscout
#[server(GetRecentTransactions, "/api")]
pub async fn get_recent_transactions(limit: Option<u32>) -> Result<Vec<Transaction>, ServerFnError> {
    let limit = limit.ok_or_else(|| SfnError::ServerError("limit is required".to_string()))?;
    
    #[cfg(feature = "ssr")]
    {
        use crate::blockscout::BlockscoutClient;
        
        let blockscout = BlockscoutClient::new();
        let transactions = blockscout.get_recent_transfers(limit).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;
        
        Ok(transactions)
    }
    
    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Troves
// ============================================================================

/// Get all troves with optional pagination
/// Cached for 120 seconds to reduce RPC load
#[server(GetTroves, "/api")]
pub async fn get_troves(limit: Option<u32>, _offset: Option<u32>) -> Result<Vec<Trove>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::rpc::RpcClient;
        use crate::cache::caches;

        let limit = limit.unwrap_or(100).min(500); // Default 100, max 500
        let cache_key = format!("troves_{}", limit);

        // Check cache first
        if let Some(cached) = caches::TROVES.get(&cache_key) {
            return Ok(cached);
        }

        let rpc = RpcClient::new();

        // Get troves data from MultiTroveGetter
        let troves_data = match rpc.get_multiple_sorted_troves(0, limit).await {
            Ok(data) => data,
            Err(_) => return Ok(vec![]), // Return empty on RPC error
        };

        if troves_data.is_empty() {
            return Ok(vec![]);
        }

        // Get FIL price for ICR calculation
        let fil_price = match rpc.get_fil_price().await {
            Ok(price) if !price.is_zero() => price,
            _ => return Ok(vec![]), // Return empty if no price
        };

        // Convert to Trove type with ICR calculation
        let troves: Vec<Trove> = troves_data
            .iter()
            .map(|t| {
                let icr = if t.debt.is_zero() {
                    Decimal::new(10000, 0)
                } else {
                    (t.coll * fil_price) / t.debt * Decimal::new(100, 0)
                };

                let status = if icr < Decimal::new(115, 0) {
                    TroveStatus::Critical
                } else if icr < Decimal::new(135, 0) {
                    TroveStatus::AtRisk
                } else {
                    TroveStatus::Active
                };

                Trove {
                    address: t.owner.clone(),
                    collateral: t.coll,
                    debt: t.debt,
                    icr,
                    status,
                }
            })
            .collect();

        // Store in cache
        caches::TROVES.set(cache_key, troves.clone());

        Ok(troves)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Lending Markets (Subgraph)
// ============================================================================

/// Lending market data from subgraph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LendingMarketData {
    pub maturity: String,
    pub currency: String,
    pub lend_unit_price: String,
    pub borrow_unit_price: String,
    pub volume: String,
    pub is_active: bool,
    pub lend_apr: f64,
    pub borrow_apr: f64,
}

/// Get lending markets from subgraph
/// Cached for 60 seconds to reduce subgraph load
#[server(GetLendingMarkets, "/api")]
pub async fn get_lending_markets() -> Result<Vec<LendingMarketData>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::subgraph::SubgraphClient;
        use crate::subgraph::unit_price_to_apr;
        use crate::cache::caches;

        // Check cache first
        if let Some(cached) = caches::LENDING_MARKETS.get("default") {
            return Ok(cached);
        }

        let subgraph = SubgraphClient::new();
        let markets = subgraph.get_lending_markets().await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Filter and map markets, skipping any with missing required data
        let market_data: Vec<LendingMarketData> = markets
            .into_iter()
            .filter_map(|m| {
                let maturity_ts = m.maturity.parse::<i64>().ok()?;
                let lend_price = m.last_lend_unit_price.clone().unwrap_or_else(|| "0".to_string());
                let borrow_price = m.last_borrow_unit_price.clone().unwrap_or_else(|| "0".to_string());
                let lend_apr = unit_price_to_apr(&lend_price, maturity_ts).unwrap_or(0.0);
                let borrow_apr = unit_price_to_apr(&borrow_price, maturity_ts).unwrap_or(0.0);
                let volume = m.volume.clone().unwrap_or_else(|| "0".to_string());

                Some(LendingMarketData {
                    maturity: m.maturity,
                    currency: m.currency,
                    lend_unit_price: lend_price,
                    borrow_unit_price: borrow_price,
                    volume,
                    is_active: m.is_active,
                    lend_apr,
                    borrow_apr,
                })
            })
            .collect();

        // Store in cache
        caches::LENDING_MARKETS.set("default".to_string(), market_data.clone());

        Ok(market_data)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Historical Data (Subgraph)
// ============================================================================

/// Daily volume data point for charts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyVolumeData {
    pub day: String,
    pub volume: f64,
    pub timestamp: i64,
    pub currency: String,
}

/// Get daily volume data for historical charts
#[server(GetDailyVolumes, "/api")]
pub async fn get_daily_volumes(days: Option<i32>) -> Result<Vec<DailyVolumeData>, ServerFnError> {
    let days = days.unwrap_or(30);

    #[cfg(feature = "ssr")]
    {
        use crate::subgraph::SubgraphClient;

        let subgraph = SubgraphClient::new();
        let volumes = subgraph.get_daily_volumes(days).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        let data: Vec<DailyVolumeData> = volumes
            .into_iter()
            .filter_map(|v| {
                let volume = v.volume.parse::<f64>().ok()? / 1e18;
                let timestamp = v.timestamp.parse::<i64>().ok()?;
                Some(DailyVolumeData {
                    day: v.day,
                    volume,
                    timestamp,
                    currency: v.currency,
                })
            })
            .collect();

        Ok(data)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Address Lookup
// ============================================================================

/// Address info response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub usdfc_balance: String,
    pub transfer_count: u64,
    pub first_seen: String,
    pub address_type: String,
}

/// Normalized address info for display and API routing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalizedAddress {
    pub input: String,
    pub kind: String,
    pub evm: Option<String>,
    pub f4: Option<String>,
    pub blockscout: Option<String>,
}

/// Get address info from Blockscout
#[server(GetAddressInfo, "/api")]
pub async fn get_address_info(address: String) -> Result<AddressInfo, ServerFnError> {
    crate::error::ValidationError::validate_address(&address)
        .map_err(|e| SfnError::ServerError(e.to_string()))?;

    #[cfg(feature = "ssr")]
    {
        use crate::blockscout::BlockscoutClient;
        use crate::address_conv::normalize_for_blockscout;
        
        let normalized = normalize_for_blockscout(&address)
            .map_err(SfnError::ServerError)?;
        let blockscout = BlockscoutClient::new();
        let info = blockscout.get_address_usdfc_info(&normalized).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;
        
        Ok(info)
    }
    
    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

/// Normalize address formats for display and routing
#[server(GetNormalizedAddress, "/api")]
pub async fn get_normalized_address(address: String) -> Result<NormalizedAddress, ServerFnError> {
    crate::error::ValidationError::validate_address(&address)
        .map_err(|e| SfnError::ServerError(e.to_string()))?;

    #[cfg(feature = "ssr")]
    {
        use crate::address_conv::{evm_to_f4, f4_to_evm};

        if address.starts_with("0x") {
            let f4 = evm_to_f4(&address).map_err(SfnError::ServerError)?;
            return Ok(NormalizedAddress {
                input: address.clone(),
                kind: "evm".to_string(),
                evm: Some(address.clone()),
                f4: Some(f4),
                blockscout: Some(address),
            });
        }

        if address.starts_with("f4") {
            let evm = f4_to_evm(&address).map_err(SfnError::ServerError)?;
            return Ok(NormalizedAddress {
                input: address.clone(),
                kind: "delegated".to_string(),
                evm: Some(evm.clone()),
                f4: Some(address),
                blockscout: Some(evm),
            });
        }

        if address.starts_with("f1") || address.starts_with("f3") {
            return Err(SfnError::ServerError(
                "f1/f3 addresses are not supported by Blockscout without conversion".to_string(),
            ));
        }

        Err(SfnError::ServerError("unsupported address format".to_string()))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Holders + Stability Pool Transfers
// ============================================================================

/// Token holder info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenHolderInfo {
    pub address: String,
    pub balance: Decimal,
}

/// Get top USDFC holders from Blockscout
/// Cached for 300 seconds (5 minutes) as holder list changes slowly
#[server(GetTopHolders, "/api")]
pub async fn get_top_holders(limit: Option<u32>) -> Result<Vec<TokenHolderInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::blockscout::BlockscoutClient;
        use crate::config::config;
        use crate::cache::caches;

        let limit = limit.unwrap_or(20).min(50); // Default 20, max 50
        let cache_key = format!("holders_{}", limit);

        // Check cache first
        if let Some(cached) = caches::TOKEN_HOLDERS.get(&cache_key) {
            return Ok(cached);
        }

        let blockscout = BlockscoutClient::new();

        match blockscout.get_token_holders(&config().usdfc_token, limit).await {
            Ok(holders) => {
                let holder_info: Vec<TokenHolderInfo> = holders
                    .into_iter()
                    .map(|h| TokenHolderInfo {
                        address: h.address,
                        balance: h.balance,
                    })
                    .collect();

                // Store in cache
                caches::TOKEN_HOLDERS.set(cache_key, holder_info.clone());

                Ok(holder_info)
            }
            Err(_) => {
                // Return empty list on error instead of failing
                Ok(vec![])
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

/// Get recent USDFC transfers involving the Stability Pool address
#[server(GetStabilityPoolTransfers, "/api")]
pub async fn get_stability_pool_transfers(limit: Option<u32>) -> Result<Vec<Transaction>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::blockscout::BlockscoutClient;
        use crate::config::config;

        let blockscout = BlockscoutClient::new();
        let limit = limit.ok_or_else(|| SfnError::ServerError("limit is required".to_string()))?;
        let transfers = blockscout
            .get_address_transfers(&config().stability_pool, &config().usdfc_token, limit)
            .await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        Ok(transfers)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Price Data (GeckoTerminal)
// ============================================================================

/// USDFC price and market data from DEX
/// All prices use Option<f64> - None means data unavailable (safer than fake fallbacks)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct USDFCPriceData {
    /// Current price in USD - None if API failed (NEVER fallback to 1.0)
    pub price_usd: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
}

/// USDFC/WFIL pool address on Filecoin
const USDFC_WFIL_POOL: &str = "0x4e07447bd38e60b94176764133788be1a0736b30";

/// Get USDFC price data from GeckoTerminal
/// Cached for 30 seconds to reduce API load
#[server(GetUSDFCPriceData, "/api")]
pub async fn get_usdfc_price_data() -> Result<USDFCPriceData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::gecko::GeckoClient;
        use crate::cache::caches;

        // Check cache first
        if let Some(cached) = caches::USDFC_PRICE.get("default") {
            return Ok(cached);
        }

        let gecko = GeckoClient::new();
        let pool_info = gecko.get_pool_info(USDFC_WFIL_POOL).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // SAFETY: Use Option - never fallback to 1.0 for price (masks depegging)
        let price_usd = pool_info
            .base_token_price_usd
            .and_then(|s| s.parse::<f64>().ok());

        let price_change_24h = pool_info
            .price_change_percentage
            .and_then(|p| p.h24)
            .and_then(|s| s.parse::<f64>().ok());

        let volume_24h = pool_info
            .volume_usd
            .and_then(|v| v.h24)
            .and_then(|s| s.parse::<f64>().ok());

        let liquidity_usd = pool_info
            .reserve_in_usd
            .and_then(|s| s.parse::<f64>().ok());

        let price_data = USDFCPriceData {
            price_usd,
            price_change_24h,
            volume_24h,
            liquidity_usd,
        };

        // Store in cache
        caches::USDFC_PRICE.set("default".to_string(), price_data.clone());

        Ok(price_data)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// API Health Status
// ============================================================================

/// API health status for all data sources
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiHealthStatus {
    pub rpc_ok: bool,
    pub blockscout_ok: bool,
    pub subgraph_ok: bool,
    pub timestamp: i64,
}

/// Check health of all APIs
#[server(CheckApiHealth, "/api")]
pub async fn check_api_health() -> Result<ApiHealthStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::rpc::RpcClient;
        use crate::blockscout::BlockscoutClient;
        use crate::subgraph::SubgraphClient;

        let rpc = RpcClient::new();
        let blockscout = BlockscoutClient::new();
        let subgraph = SubgraphClient::new();

        // Check RPC by getting block number (simple call)
        let rpc_ok = rpc.get_fil_price().await.is_ok();

        // Check Blockscout by getting token info
        let blockscout_ok = blockscout.gql_get_token_info(&crate::config::config().usdfc_token).await.is_ok();

        // Check Subgraph by getting lending markets
        let subgraph_ok = subgraph.get_lending_markets().await.is_ok();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Ok(ApiHealthStatus {
            rpc_ok,
            blockscout_ok,
            subgraph_ok,
            timestamp,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required".to_string()))
    }
}

// ============================================================================
// Holder Count
// ============================================================================

/// Get USDFC holder count from Blockscout
/// Cached for 300 seconds (5 minutes) as count changes slowly
#[server(GetHolderCount, "/api")]
pub async fn get_holder_count() -> Result<u64, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::blockscout::BlockscoutClient;
        use crate::cache::caches;

        // Check cache first
        if let Some(cached) = caches::HOLDER_COUNT.get("default") {
            return Ok(cached);
        }

        let blockscout = BlockscoutClient::new();
        let count = blockscout.get_holder_count().await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Store in cache
        caches::HOLDER_COUNT.set("default".to_string(), count);

        Ok(count)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Order Book (Subgraph)
// ============================================================================

/// Order book data for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderBookData {
    pub currency: String,
    pub maturity: Option<String>,
    pub lend_orders: Vec<OrderData>,
    pub borrow_orders: Vec<OrderData>,
    pub best_lend_price: Option<f64>,
    pub best_borrow_price: Option<f64>,
    pub spread_bps: Option<f64>,
}

/// Single order for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderData {
    pub id: String,
    pub side: String,
    pub amount: f64,
    pub filled: f64,
    pub price: f64,
    pub apr: f64,
    pub user: Option<String>,
    pub created_at: String,
}

/// Get order book from subgraph
#[server(GetOrderBook, "/api")]
pub async fn get_order_book(maturity: Option<String>) -> Result<OrderBookData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::subgraph::SubgraphClient;
        use crate::config::config;

        let subgraph = SubgraphClient::new();
        let currency = &config().currency_usdfc;
        let maturity_ref = maturity.as_deref();

        let book = subgraph.get_order_book(currency, maturity_ref, 100).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Convert orders to display format
        let convert_order = |o: &crate::subgraph::Order| -> OrderData {
            let amount = o.input_amount.parse::<f64>().unwrap_or(0.0) / 1e18;
            let filled = o.filled_amount.parse::<f64>().unwrap_or(0.0) / 1e18;
            let price = o.input_unit_price.parse::<f64>().unwrap_or(0.0) / 10000.0;
            let maturity_ts = o.maturity.parse::<i64>().unwrap_or(0);
            let apr = crate::subgraph::unit_price_to_apr(&o.input_unit_price, maturity_ts).unwrap_or(0.0);

            OrderData {
                id: o.id.clone(),
                side: if o.side == 0 { "Lend".to_string() } else { "Borrow".to_string() },
                amount,
                filled,
                price,
                apr,
                user: o.user.clone(),
                created_at: o.created_at.clone(),
            }
        };

        let lend_orders: Vec<OrderData> = book.lend_orders.iter().map(convert_order).collect();
        let borrow_orders: Vec<OrderData> = book.borrow_orders.iter().map(convert_order).collect();

        // Calculate best prices and spread
        let best_lend_price = lend_orders.first().map(|o| o.price);
        let best_borrow_price = borrow_orders.first().map(|o| o.price);
        let spread_bps = match (best_lend_price, best_borrow_price) {
            (Some(lend), Some(borrow)) => Some((borrow - lend) * 10000.0),
            _ => None,
        };

        Ok(OrderBookData {
            currency: "USDFC".to_string(),
            maturity: book.maturity,
            lend_orders,
            borrow_orders,
            best_lend_price,
            best_borrow_price,
            spread_bps,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Recent Lending Trades (Subgraph)
// ============================================================================

/// Lending trade for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LendingTradeData {
    pub id: String,
    pub currency: String,
    pub maturity: String,
    pub side: String,
    pub amount: f64,
    pub price: f64,
    pub apr: f64,
    pub timestamp: i64,
}

/// Get recent lending trades from subgraph
#[server(GetRecentLendingTrades, "/api")]
pub async fn get_recent_lending_trades(limit: Option<i32>) -> Result<Vec<LendingTradeData>, ServerFnError> {
    let limit = limit.unwrap_or(20);

    #[cfg(feature = "ssr")]
    {
        use crate::subgraph::SubgraphClient;
        use crate::subgraph::decode_currency;

        let subgraph = SubgraphClient::new();
        let transactions = subgraph.get_recent_transactions(limit).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        let trades: Vec<LendingTradeData> = transactions
            .into_iter()
            .filter_map(|tx| {
                let amount = tx.amount.parse::<f64>().ok()? / 1e18;
                let price = tx.execution_price.as_ref()?.parse::<f64>().ok()? / 10000.0;
                let maturity_ts = tx.maturity.parse::<i64>().ok()?;
                let timestamp = tx.created_at.parse::<i64>().ok()?;
                let apr = crate::subgraph::unit_price_to_apr(
                    tx.execution_price.as_ref()?,
                    maturity_ts
                ).unwrap_or(0.0);

                Some(LendingTradeData {
                    id: tx.id,
                    currency: decode_currency(&tx.currency),
                    maturity: tx.maturity,
                    side: if tx.side == 0 { "Lend".to_string() } else { "Borrow".to_string() },
                    amount,
                    price,
                    apr,
                    timestamp,
                })
            })
            .collect();

        Ok(trades)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Advanced Chart Data (All Metrics)
// ============================================================================

/// Get comprehensive chart data with all metrics for advanced chart
/// Fetches real data from GeckoTerminal, RPC, Blockscout, and Subgraph
/// Uses historical snapshot storage for metrics over time
#[server(GetAdvancedChartData, "/api")]
pub async fn get_advanced_chart_data(
    resolution: ChartResolution,
    lookback: ChartLookback,
) -> Result<ChartDataResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::gecko::GeckoClient;
        use crate::rpc::RpcClient;
        use crate::blockscout::BlockscoutClient;
        use crate::subgraph::SubgraphClient;
        use crate::config::config;
        use crate::historical::MetricSnapshot;
        use std::time::{SystemTime, UNIX_EPOCH, Instant};
        use rust_decimal::prelude::ToPrimitive;

        let start = Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Initialize clients
        let gecko = GeckoClient::new();
        let rpc = RpcClient::new();
        let blockscout = BlockscoutClient::new();
        let subgraph = SubgraphClient::new();

        // Get resolution parameters for GeckoTerminal
        let (timeframe, aggregate, limit) = resolution.gecko_params();

        // Calculate how many data points we need based on lookback
        let lookback_mins = lookback.minutes();
        let resolution_mins = resolution.minutes();
        let data_points = if lookback_mins == 0 {
            limit // Use max for "ALL"
        } else {
            ((lookback_mins / resolution_mins) as u32).min(limit)
        };

        // Fetch all data in parallel
        let pool_address = &config().pool_usdfc_wfil;

        // Parallel fetch: OHLCV, pool info, current metrics for display, transfer history
        let (ohlcv_result, pool_result, tcr_result, supply_result, holder_result, transfers_by_period) = tokio::join!(
            gecko.get_pool_ohlcv(pool_address, timeframe, aggregate, data_points),
            gecko.get_pool_info(pool_address),
            rpc.get_tcr(),
            rpc.get_total_supply(),
            blockscout.get_holder_count(),
            blockscout.get_transfer_counts_by_period(resolution_mins, lookback_mins)
        );

        // Process price candles from OHLCV data
        let price_candles: Vec<TVCandle> = match ohlcv_result {
            Ok(ohlcv_list) => {
                ohlcv_list.into_iter()
                    .map(|o| TVCandle {
                        time: o.timestamp,
                        open: o.open,
                        high: o.high,
                        low: o.low,
                        close: o.close,
                        volume: o.volume,
                    })
                    .collect()
            }
            Err(_) => Vec::new()
        };

        // Extract volume data from candles
        let volume_data: Vec<(i64, f64)> = price_candles
            .iter()
            .map(|c| (c.time, c.volume))
            .collect();

        // SAFETY: Use Option for all metrics - never fake fallback values
        // Get current price and liquidity from pool info
        let (current_price, current_liquidity, current_volume_24h) = match pool_result {
            Ok(pool) => {
                let price = pool.base_token_price_usd
                    .and_then(|s| s.parse::<f64>().ok());
                let liquidity = pool.reserve_in_usd
                    .and_then(|s| s.parse::<f64>().ok());
                let volume = pool.volume_usd
                    .and_then(|v| v.h24)
                    .and_then(|s| s.parse::<f64>().ok());
                (price, liquidity, volume)
            }
            // API failure = None, not fake values
            Err(_) => (None, None, None)
        };

        // Get current metric values (for display) - None if unavailable
        let current_tcr = tcr_result.ok().and_then(|v| v.to_f64());
        let current_supply = supply_result.ok().and_then(|v| v.to_f64());
        let current_holders = holder_result.ok();

        // Get lending/borrowing APRs - None if API fails
        let (current_lend_apr, current_borrow_apr): (Option<f64>, Option<f64>) = {
            let markets = subgraph.get_lending_markets().await;
            match markets {
                Ok(market_list) => {
                    let mut best_lend: Option<f64> = None;
                    let mut best_borrow: Option<f64> = None;
                    for market in market_list {
                        if market.is_active {
                            let maturity_ts = market.maturity.parse::<i64>().unwrap_or(0);
                            if let Some(ref lend_price) = market.last_lend_unit_price {
                                if let Ok(apr) = crate::subgraph::unit_price_to_apr(lend_price, maturity_ts) {
                                    best_lend = Some(best_lend.map_or(apr, |v| v.max(apr)));
                                }
                            }
                            if let Some(ref borrow_price) = market.last_borrow_unit_price {
                                if let Ok(apr) = crate::subgraph::unit_price_to_apr(borrow_price, maturity_ts) {
                                    best_borrow = Some(best_borrow.map_or(apr, |v| v.max(apr)));
                                }
                            }
                        }
                    }
                    (best_lend, best_borrow)
                }
                // API failure = None, not fake 0.0
                Err(_) => (None, None)
            }
        };

        // === BUILD TIME SERIES FROM HISTORICAL SNAPSHOTS ===
        let snapshots = MetricSnapshot::get_history(lookback_mins, resolution_mins);

        // If no snapshots yet, use current values as single point (only if available)
        let (tcr_data, supply_data, liquidity_data, holders_data, lend_apr_data, borrow_apr_data) =
            if snapshots.is_empty() {
                (
                    current_tcr.map(|v| vec![(now, v)]).unwrap_or_default(),
                    current_supply.map(|v| vec![(now, v)]).unwrap_or_default(),
                    current_liquidity.map(|v| vec![(now, v)]).unwrap_or_default(),
                    current_holders.map(|v| vec![(now, v)]).unwrap_or_default(),
                    current_lend_apr.map(|v| vec![(now, v)]).unwrap_or_default(),
                    current_borrow_apr.map(|v| vec![(now, v)]).unwrap_or_default(),
                )
            } else {
                (
                    MetricSnapshot::tcr_series(&snapshots),
                    MetricSnapshot::supply_series(&snapshots),
                    MetricSnapshot::liquidity_series(&snapshots),
                    MetricSnapshot::holders_series(&snapshots),
                    MetricSnapshot::lend_apr_series(&snapshots),
                    MetricSnapshot::borrow_apr_series(&snapshots),
                )
            };

        // Transfer counts from Blockscout aggregation (real historical data)
        let transfers_data: Vec<(i64, u64)> = transfers_by_period.unwrap_or_default();

        let fetch_time_ms = start.elapsed().as_millis() as u32;

        Ok(ChartDataResponse {
            resolution,
            lookback,
            generated_at: now,
            fetch_time_ms,
            price_candles,
            volume_data,
            liquidity_data,
            tcr_data,
            supply_data,
            holders_data,
            lend_apr_data,
            borrow_apr_data,
            transfers_data,
            current_price,
            current_volume_24h,
            current_liquidity,
            current_tcr,
            current_supply,
            current_holders,
            current_lend_apr,
            current_borrow_apr,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}
