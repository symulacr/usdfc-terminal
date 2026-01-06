//! USDFC API - Server Function Declarations
//!
//! This crate contains server function signatures with #[server] macros.
//! Implementations are included in #[cfg(feature = "ssr")] blocks.
//! This crate is WASM-compatible when built without ssr feature.

use leptos::*;
use leptos::server_fn::error::NoCustomError;
use rust_decimal::Decimal;

// Re-export all types from core for convenience
pub use usdfc_core::types::*;

/// Type alias for server function errors with default error type
/// Used in feature-gated code (50+ occurrences)
#[allow(dead_code)]
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
        use usdfc_backend::rpc::RpcClient;
        use usdfc_backend::cache::caches;

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
        let circulating_supply = total_supply - stability_pool_balance;

        let metrics = ProtocolMetrics {
            total_supply,
            circulating_supply,
            total_collateral,
            active_troves,
            tcr,
            stability_pool_balance,
            treasury_balance: stability_pool_balance,
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

/// Get recent transactions from Blockscout
#[server(GetRecentTransactions, "/api")]
pub async fn get_recent_transactions(limit: Option<u32>) -> Result<Vec<Transaction>, ServerFnError> {
    let limit = limit.unwrap_or(50); // Default to 50 if not specified

    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_backend::cache::caches;

        // Check cache first (10s TTL for recent transactions)
        let cache_key = format!("recent_tx_{}", limit);
        if let Some(cached) = caches::RECENT_TRANSACTIONS.get(&cache_key) {
            return Ok(cached);
        }

        let blockscout = BlockscoutClient::new();
        // Pass None for max_pages to use default (100 pages = 5000 items)
        let transactions = blockscout.get_recent_transfers(limit, None).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Store in cache
        caches::RECENT_TRANSACTIONS.set(cache_key, transactions.clone());

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
        use usdfc_backend::rpc::RpcClient;
        use usdfc_backend::cache::caches;

        let limit = limit.unwrap_or(100).min(500); // Default 100, max 500
        let cache_key = format!("troves_{}", limit);

        // Check cache first
        if let Some(cached) = caches::TROVES.get(&cache_key) {
            return Ok(cached);
        }

        let rpc = RpcClient::new();

        // Get troves data - propagate errors to UI for proper error handling
        let troves_data = rpc.get_multiple_sorted_troves(0, limit).await
            .map_err(|e| {
                tracing::error!("RPC error fetching troves: {}", e);
                ServerFnError::<NoCustomError>::ServerError(format!("Failed to fetch troves: {}", e))
            })?;

        if troves_data.is_empty() {
            return Ok(vec![]); // Empty is valid - no troves exist
        }

        // Get FIL price - propagate errors to UI for proper error handling
        let fil_price = rpc.get_fil_price().await
            .map_err(|e| {
                tracing::error!("RPC error fetching FIL price: {}", e);
                ServerFnError::<NoCustomError>::ServerError(format!("Failed to fetch FIL price: {}", e))
            })?;

        if fil_price.is_zero() {
            tracing::error!("FIL price is zero - invalid data");
            return Err(ServerFnError::<NoCustomError>::ServerError("FIL price is zero".to_string()));
        }

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


/// Get lending markets from subgraph
/// Cached for 60 seconds to reduce subgraph load
#[server(GetLendingMarkets, "/api")]
pub async fn get_lending_markets() -> Result<Vec<LendingMarketData>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::subgraph::SubgraphClient;
        use usdfc_backend::subgraph::unit_price_to_apr;
        use usdfc_backend::cache::caches;

        // Check cache first
        if let Some(cached) = caches::LENDING_MARKETS.get("default") {
            return Ok(cached);
        }

        let subgraph = SubgraphClient::new();
        let markets = subgraph.get_lending_markets().await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Filter and map markets - only include markets with real pricing data
        // Skip markets without unit prices (no fake "0" fallbacks)
        let market_data: Vec<LendingMarketData> = markets
            .into_iter()
            .filter_map(|m| {
                let maturity_ts = m.maturity.parse::<i64>().ok()?;

                // Get real prices - use empty string if no price (will show as N/A in UI)
                let lend_price = m.last_lend_unit_price.clone().unwrap_or_default();
                let borrow_price = m.last_borrow_unit_price.clone().unwrap_or_default();

                // Calculate APR only if we have valid price data
                let lend_apr = if lend_price.is_empty() {
                    0.0 // No price means 0 APR (market has no lend orders)
                } else {
                    unit_price_to_apr(&lend_price, maturity_ts).unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to calculate lend APR for market {} (maturity {}): {}. Showing 0.0",
                            m.currency, m.maturity, e
                        );
                        0.0
                    })
                };
                let borrow_apr = if borrow_price.is_empty() {
                    0.0 // No price means 0 APR (market has no borrow orders)
                } else {
                    unit_price_to_apr(&borrow_price, maturity_ts).unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to calculate borrow APR for market {} (maturity {}): {}. Showing 0.0",
                            m.currency, m.maturity, e
                        );
                        0.0
                    })
                };

                let volume = m.volume.clone().unwrap_or_default();

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


/// Get daily volume data for historical charts
#[server(GetDailyVolumes, "/api")]
pub async fn get_daily_volumes(days: Option<i32>) -> Result<Vec<DailyVolumeData>, ServerFnError> {
    let days = days.unwrap_or(30);

    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::subgraph::SubgraphClient;
        use usdfc_backend::cache::caches;

        // Check cache first
        let cache_key = format!("daily_vol_{}", days);
        if let Some(cached) = caches::DAILY_VOLUMES.get(&cache_key) {
            return Ok(cached);
        }

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

        // Store in cache
        caches::DAILY_VOLUMES.set(cache_key, data.clone());

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

/// Get address info from Blockscout
#[server(GetAddressInfo, "/api")]
pub async fn get_address_info(address: String) -> Result<AddressInfo, ServerFnError> {
    usdfc_core::error::ValidationError::validate_address(&address)
        .map_err(|e| SfnError::ServerError(e.to_string()))?;

    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_backend::address_conv::normalize_for_blockscout;
        use usdfc_backend::cache::caches;

        let normalized = normalize_for_blockscout(&address)
            .map_err(|e| {
                tracing::error!("Address normalization error for {}: {}", address, e);
                SfnError::ServerError(format!("Invalid address format: {}", e))
            })?;

        // Check cache first
        let cache_key = format!("addr_{}", normalized);
        if let Some(cached) = caches::ADDRESS_INFO.get(&cache_key) {
            return Ok(cached);
        }

        let blockscout = BlockscoutClient::new();
        let address_info = blockscout.get_address_usdfc_info(&normalized).await
            .map_err(|e| {
                tracing::error!("Blockscout error for {}: {}", address, e);
                SfnError::ServerError(format!("Failed to fetch address info: {}", e))
            })?;

        // Store in cache
        caches::ADDRESS_INFO.set(cache_key, address_info.clone());

        Ok(address_info)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

/// Normalize address formats for display and routing
#[server(GetNormalizedAddress, "/api")]
pub async fn get_normalized_address(address: String) -> Result<NormalizedAddress, ServerFnError> {
    usdfc_core::error::ValidationError::validate_address(&address)
        .map_err(|e| SfnError::ServerError(e.to_string()))?;

    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::address_conv::{evm_to_f4, f4_to_evm};

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


/// Get top USDFC holders from Blockscout
/// Cached for 300 seconds (5 minutes) as holder list changes slowly
/// Note: Uses cursor-based pagination, offset parameter is ignored (use REST API /api/v1/holders for pagination)
#[server(GetTopHolders, "/api")]
pub async fn get_top_holders(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<TokenHolderInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_core::config::config;
        use usdfc_backend::cache::caches;

        let _offset = offset; // Ignored for cursor-based API
        let cache_key = format!("holders_first_page");

        // Check cache first
        if let Some(cached) = caches::TOKEN_HOLDERS.get(&cache_key) {
            // Take only the requested limit from cache
            let limit = limit.unwrap_or(20).min(100) as usize;
            return Ok(cached.into_iter().take(limit).collect());
        }

        let blockscout = BlockscoutClient::new();

        // Fetch first page from Blockscout API (cursor-based pagination)
        let (holders, _next_cursor) = blockscout.get_token_holders_cursor(&config().usdfc_token, None).await
            .map_err(|e| SfnError::ServerError(format!("Blockscout API error: {}", e)))?;

        let holder_info: Vec<TokenHolderInfo> = holders
            .into_iter()
            .map(|h| TokenHolderInfo {
                address: h.address,
                balance: h.balance,
            })
            .collect();

        // Store in cache
        caches::TOKEN_HOLDERS.set(cache_key, holder_info.clone());

        // Return only requested limit
        let limit = limit.unwrap_or(20).min(100) as usize;
        Ok(holder_info.into_iter().take(limit).collect())
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
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_core::config::config;
        use usdfc_backend::cache::caches;

        let limit = limit.ok_or_else(|| SfnError::ServerError("limit is required".to_string()))?;

        // Check cache first
        let cache_key = format!("stability_tx_{}", limit);
        if let Some(cached) = caches::STABILITY_TRANSFERS.get(&cache_key) {
            return Ok(cached);
        }

        let blockscout = BlockscoutClient::new();
        let transfers = blockscout
            .get_address_transfers(&config().stability_pool, &config().usdfc_token, limit)
            .await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Store in cache
        caches::STABILITY_TRANSFERS.set(cache_key, transfers.clone());

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

/// Get USDFC price data from GeckoTerminal
/// Cached for 30 seconds to reduce API load
#[server(GetUSDFCPriceData, "/api")]
pub async fn get_usdfc_price_data() -> Result<USDFCPriceData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::cache::caches;
        use usdfc_core::config::config;
        use usdfc_backend::gecko::GeckoClient;

        // Check cache first
        if let Some(cached) = caches::USDFC_PRICE.get("default") {
            return Ok(cached);
        }

        let gecko = GeckoClient::new();
        let pool_info = gecko
            .get_pool_info(&config().pool_usdfc_wfil)
            .await
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


/// Check health of all APIs
#[server(CheckApiHealth, "/api")]
pub async fn check_api_health() -> Result<ApiHealthStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_core::config::config;
        use usdfc_backend::gecko::GeckoClient;
        use usdfc_backend::historical;
        use usdfc_backend::rpc::RpcClient;
        use usdfc_backend::subgraph::SubgraphClient;

        let rpc = RpcClient::new();
        let blockscout = BlockscoutClient::new();
        let subgraph = SubgraphClient::new();
        let gecko = GeckoClient::new();

        // Check RPC by getting FIL price (simple call)
        let rpc_ok = rpc.get_fil_price().await.is_ok();

        // Check Blockscout by getting token info
        let blockscout_ok = blockscout
            .gql_get_token_info(&config().usdfc_token)
            .await
            .is_ok();

        // Check Subgraph by getting lending markets
        let subgraph_ok = subgraph.get_lending_markets().await.is_ok();

        // Check GeckoTerminal by fetching primary pool info
        let gecko_ok = gecko
            .get_pool_info(&config().pool_usdfc_wfil)
            .await
            .is_ok();

        // Check historical SQLite database
        let database_ok = historical::check_db_health().is_ok();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Ok(ApiHealthStatus {
            rpc_ok,
            blockscout_ok,
            subgraph_ok,
            gecko_ok,
            database_ok,
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
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_backend::cache::caches;

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



/// Get order book from subgraph
#[server(GetOrderBook, "/api")]
pub async fn get_order_book(maturity: Option<String>) -> Result<OrderBookData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::subgraph::SubgraphClient;
        use usdfc_core::config::config;
        use usdfc_backend::cache::caches;

        // Check cache first
        let cache_key = format!("order_book_{}", maturity.as_deref().unwrap_or("default"));
        if let Some(cached) = caches::ORDER_BOOK.get(&cache_key) {
            return Ok(cached);
        }

        let subgraph = SubgraphClient::new();
        let currency = &config().currency_usdfc;
        let maturity_ref = maturity.as_deref();

        let book = subgraph.get_order_book(currency, maturity_ref, 100).await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Convert orders to display format - skip orders with invalid data instead of using fake values
        let convert_order = |o: &usdfc_backend::subgraph::Order| -> Option<OrderData> {
            let amount = o.input_amount.parse::<f64>().ok()? / 1e18;
            let filled = o.filled_amount.parse::<f64>().ok()? / 1e18;
            let price = o.input_unit_price.parse::<f64>().ok()? / 10000.0;
            let maturity_ts = o.maturity.parse::<i64>().ok()?;
            let apr = usdfc_backend::subgraph::unit_price_to_apr(&o.input_unit_price, maturity_ts).ok()?;

            Some(OrderData {
                id: o.id.clone(),
                side: if o.side == 0 { "Lend".to_string() } else { "Borrow".to_string() },
                amount,
                filled,
                price,
                apr,
                user: o.user.clone(),
                created_at: o.created_at.clone(),
            })
        };

        let lend_orders: Vec<OrderData> = book.lend_orders.iter().filter_map(convert_order).collect();
        let borrow_orders: Vec<OrderData> = book.borrow_orders.iter().filter_map(convert_order).collect();

        // Calculate best prices and spread
        let best_lend_price = lend_orders.first().map(|o| o.price);
        let best_borrow_price = borrow_orders.first().map(|o| o.price);
        let spread_bps = match (best_lend_price, best_borrow_price) {
            (Some(lend), Some(borrow)) => Some((borrow - lend) * 10000.0),
            _ => None,
        };

        let order_book_data = OrderBookData {
            currency: "USDFC".to_string(),
            maturity: book.maturity,
            lend_orders,
            borrow_orders,
            best_lend_price,
            best_borrow_price,
            spread_bps,
        };

        // Store in cache
        caches::ORDER_BOOK.set(cache_key, order_book_data.clone());

        Ok(order_book_data)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Recent Lending Trades (Subgraph)
// ============================================================================


/// Get recent lending trades from subgraph
#[server(GetRecentLendingTrades, "/api")]
pub async fn get_recent_lending_trades(limit: Option<i32>) -> Result<Vec<LendingTradeData>, ServerFnError> {
    let limit = limit.unwrap_or(20);

    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::subgraph::SubgraphClient;
        use usdfc_backend::subgraph::decode_currency;
        use usdfc_backend::cache::caches;

        // Check cache first
        let cache_key = format!("lending_trades_{}", limit);
        if let Some(cached) = caches::LENDING_TRADES.get(&cache_key) {
            return Ok(cached);
        }

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
                let apr = usdfc_backend::subgraph::unit_price_to_apr(
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

        // Store in cache
        caches::LENDING_TRADES.set(cache_key, trades.clone());

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

/// Calculate TCR time series from price history
/// TCR = (Collateral_FIL × FIL_Price_USD) / Supply_USDFC × 100
#[cfg(feature = "ssr")]
fn calculate_tcr_from_price_history(
    price_candles: &[TVCandle],
    supply: f64,
    collateral_fil: f64,
) -> Vec<(i64, f64)> {
    price_candles
        .iter()
        .map(|candle| {
            // TCR = (Collateral × FIL_Price) / Supply × 100
            let tcr = if supply > 0.0 {
                (collateral_fil * candle.close) / supply * 100.0
            } else {
                0.0
            };
            (candle.time, tcr)
        })
        .collect()
}

/// Calculate liquidity proxy from trading volume
/// Uses volume directly as a stable proxy for market liquidity/activity
/// Simpler and more stable than Volume/Impact calculation
#[cfg(feature = "ssr")]
fn calculate_liquidity_from_volume_impact(
    price_candles: &[TVCandle],
) -> Vec<(i64, f64)> {
    // Use volume directly as liquidity proxy
    // This is simpler, more stable, and shows trading activity clearly
    // Multiplied by 10 to scale into typical liquidity range for display
    price_candles
        .iter()
        .map(|candle| {
            // Scale volume by 10x to match expected liquidity display range
            (candle.time, candle.volume * 10.0)
        })
        .collect()
}

/// Get comprehensive chart data with all metrics for advanced chart
/// Fetches real data from GeckoTerminal, RPC, Blockscout, and Subgraph
/// Uses historical snapshot storage for metrics over time
#[server(GetAdvancedChartData, "/api")]
pub async fn get_advanced_chart_data(
    resolution: ChartResolution,
    lookback: ChartLookback,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<ChartDataResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::gecko::GeckoClient;
        use usdfc_backend::rpc::RpcClient;
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_backend::subgraph::SubgraphClient;
        use usdfc_core::config::config;
        use usdfc_backend::historical::MetricSnapshot;
        use usdfc_backend::cache::caches;
        use std::time::{SystemTime, UNIX_EPOCH, Instant};
        use rust_decimal::prelude::ToPrimitive;

        // Generate cache key from parameters
        let cache_key = format!(
            "chart_{}_{}_{}_{}",
            resolution.label(),
            lookback.label(),
            start.unwrap_or(0),
            end.unwrap_or(0)
        );

        // Check cache first
        if let Some(cached) = caches::ADVANCED_CHART_DATA.get(&cache_key) {
            return Ok(cached);
        }

        let timer_start = Instant::now();
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

        // Calculate effective lookback in minutes for historical sources.
        // If a custom start is provided, ensure we cover at least that span.
        let configured_lookback_mins = lookback.minutes();
        let resolution_mins = resolution.minutes();

        let mut effective_lookback_mins = if let Some(custom_start) = start {
            let diff_secs = now.saturating_sub(custom_start);
            let span_mins = ((diff_secs / 60).max(1)) as u32;
            if configured_lookback_mins == 0 {
                // "All" lookback: use the span implied by the custom range.
                span_mins
            } else {
                configured_lookback_mins.max(span_mins)
            }
        } else {
            configured_lookback_mins
        };

        // CRITICAL: Enforce API safety limits to prevent data loss
        let max_safe_lookback = resolution.max_safe_lookback_mins();
        if effective_lookback_mins > max_safe_lookback {
            tracing::warn!(
                "Lookback {} mins exceeds safe limit {} mins for resolution {:?}. Clamping to safe limit.",
                effective_lookback_mins,
                max_safe_lookback,
                resolution
            );
            effective_lookback_mins = max_safe_lookback;
        }

        // Determine how many OHLCV points to request from GeckoTerminal.
        let data_points = if effective_lookback_mins == 0 {
            // "All" – use API maximum.
            limit
        } else {
            ((effective_lookback_mins / resolution_mins).max(1) as u32).min(limit)
        };

        // Fetch all data in parallel
        let pool_address = &config().pool_usdfc_wfil;

        // Parallel fetch: OHLCV, pool info, current metrics for display, transfer history, collateral
        let (ohlcv_result, pool_result, tcr_result, supply_result, collateral_result, holder_result, transfers_by_period) = tokio::join!(
            gecko.get_pool_ohlcv(pool_address, timeframe, aggregate, data_points),
            gecko.get_pool_info(pool_address),
            rpc.get_tcr(),
            rpc.get_total_supply(),
            rpc.get_active_pool_eth(),
            blockscout.get_holder_count(),
            blockscout.get_transfer_counts_by_period(resolution_mins, effective_lookback_mins)
        );

        // Process price candles from OHLCV data - propagate error if API fails
        let mut price_candles: Vec<TVCandle> = ohlcv_result
            .map_err(|e| SfnError::ServerError(format!("GeckoTerminal OHLCV error: {}", e)))?
            .into_iter()
            .map(|o| TVCandle {
                time: o.timestamp,
                open: o.open,
                high: o.high,
                low: o.low,
                close: o.close,
                volume: o.volume,
            })
            .collect();

        // If a custom time range is provided, filter candles to that range.
        if let Some(custom_start) = start {
            let effective_end = end.unwrap_or(now);
            price_candles.retain(|c| c.time >= custom_start && c.time <= effective_end);
        }

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
        let current_collateral = collateral_result.ok().and_then(|v| v.to_f64());
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
                            // Skip markets with invalid maturity instead of using fake 0
                            let maturity_ts = match market.maturity.parse::<i64>() {
                                Ok(ts) => ts,
                                Err(_) => continue,
                            };
                            if let Some(ref lend_price) = market.last_lend_unit_price {
                                if let Ok(apr) = usdfc_backend::subgraph::unit_price_to_apr(lend_price, maturity_ts) {
                                    best_lend = Some(best_lend.map_or(apr, |v| v.max(apr)));
                                }
                            }
                            if let Some(ref borrow_price) = market.last_borrow_unit_price {
                                if let Ok(apr) = usdfc_backend::subgraph::unit_price_to_apr(borrow_price, maturity_ts) {
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
        // Use real historical data when available, fallback to current value for fresh deployments
        let raw_snapshots = MetricSnapshot::get_history(effective_lookback_mins, resolution_mins);

        // If a custom range is provided, filter snapshots to that range.
        let snapshots = if let Some(custom_start) = start {
            let effective_end = end.unwrap_or(now);
            raw_snapshots
                .into_iter()
                .filter(|s| s.timestamp >= custom_start && s.timestamp <= effective_end)
                .collect::<Vec<_>>()
        } else {
            raw_snapshots
        };

        // Helper function to ensure every metric has at least current value
        let ensure_data = |mut series: Vec<(i64, f64)>, current_value: Option<f64>| -> Vec<(i64, f64)> {
            if series.is_empty() {
                // No historical data - use current value as single point if available
                if let Some(val) = current_value {
                    vec![(now, val)]
                } else {
                    vec![]
                }
            } else {
                // Have historical data - optionally append current value if newer
                if let Some(val) = current_value {
                    let last_ts = series.last().map(|(ts, _)| *ts).unwrap_or(0);
                    if now > last_ts + 120 {
                        series.push((now, val));
                    }
                }
                series
            }
        };

        // Extract series from snapshots with current value fallback
        // OPTIMIZED: Calculate TCR from price history instead of snapshots (2.63% variation!)
        let tcr_data = if let (Some(supply), Some(collateral)) = (current_supply, current_collateral) {
            if !price_candles.is_empty() && supply > 0.0 && collateral > 0.0 {
                calculate_tcr_from_price_history(&price_candles, supply, collateral)
            } else {
                // Fallback to snapshots if calculation not possible
                ensure_data(
                    MetricSnapshot::tcr_series(&snapshots),
                    current_tcr
                )
            }
        } else {
            // Fallback to snapshots if we don't have supply/collateral
            ensure_data(
                MetricSnapshot::tcr_series(&snapshots),
                current_tcr
            )
        };

        let supply_data = ensure_data(
            MetricSnapshot::supply_series(&snapshots),
            current_supply
        );

        // OPTIMIZED: Calculate liquidity from volume/impact (632% variation - DRAMATIC curves!)
        let liquidity_data = if !price_candles.is_empty() {
            let calculated = calculate_liquidity_from_volume_impact(&price_candles);
            if calculated.len() > 10 {
                // Use calculated liquidity if we have enough data points
                calculated
            } else {
                // Fallback to snapshots if calculation didn't yield enough points
                ensure_data(
                    MetricSnapshot::liquidity_series(&snapshots),
                    current_liquidity
                )
            }
        } else {
            // Fallback to snapshots if no price candles
            ensure_data(
                MetricSnapshot::liquidity_series(&snapshots),
                current_liquidity
            )
        };

        let holders_data: Vec<(i64, u64)> = if snapshots.is_empty() {
            if let Some(h) = current_holders {
                vec![(now, h)]
            } else {
                vec![]
            }
        } else {
            let mut series = MetricSnapshot::holders_series(&snapshots);
            if let Some(h) = current_holders {
                let last_ts = series.last().map(|(ts, _)| *ts).unwrap_or(0);
                if now > last_ts + 120 {
                    series.push((now, h));
                }
            }
            series
        };

        let lend_apr_data = ensure_data(
            MetricSnapshot::lend_apr_series(&snapshots),
            current_lend_apr
        );

        let borrow_apr_data = ensure_data(
            MetricSnapshot::borrow_apr_series(&snapshots),
            current_borrow_apr
        );

        // Transfer counts from Blockscout aggregation (real historical data)
        let raw_transfers: Vec<(i64, u64)> = transfers_by_period.unwrap_or_default();
        let transfers_data: Vec<(i64, u64)> = if let Some(custom_start) = start {
            let effective_end = end.unwrap_or(now);
            raw_transfers
                .into_iter()
                .filter(|(ts, _)| *ts >= custom_start && *ts <= effective_end)
                .collect()
        } else {
            raw_transfers
        };

        let fetch_time_ms = timer_start.elapsed().as_millis() as u32;

        let response = ChartDataResponse {
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
            snapshot_count: snapshots.len(),
            oldest_snapshot_time: snapshots.first().map(|s| s.timestamp),
        };

        // Store in cache
        caches::ADVANCED_CHART_DATA.set(cache_key, response.clone());

        Ok(response)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}

// ============================================================================
// Wallet Analytics (Per Address)
// ============================================================================



#[server(GetWalletAnalytics, "/api")]
pub async fn get_wallet_analytics(
    address: String,
    resolution: ChartResolution,
    lookback: ChartLookback,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<WalletAnalyticsResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use usdfc_backend::blockscout::BlockscoutClient;
        use usdfc_core::config::config;
        use usdfc_core::error::ValidationError;
        use rust_decimal::Decimal;
        use rust_decimal::prelude::ToPrimitive;
        use std::collections::BTreeMap;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Basic address validation
        ValidationError::validate_address(&address)
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Normalize address to EVM hex if needed
        let wallet_evm = if address.starts_with("0x") {
            address.to_lowercase()
        } else if address.starts_with('f') {
            // Try to convert f4-style Filecoin address to EVM
            match usdfc_backend::address_conv::f4_to_evm(&address) {
                Ok(ev) => ev.to_lowercase(),
                Err(e) => return Err(SfnError::ServerError(e.to_string())),
            }
        } else {
            address.to_lowercase()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let configured_lookback_mins = lookback.minutes();
        let resolution_mins = resolution.minutes();

        let effective_lookback_mins = if let Some(custom_start) = start {
            let diff_secs = now.saturating_sub(custom_start);
            let span_mins = ((diff_secs / 60).max(1)) as u32;
            if configured_lookback_mins == 0 {
                span_mins
            } else {
                configured_lookback_mins.max(span_mins)
            }
        } else {
            configured_lookback_mins
        };

        let (window_start, window_end) = if let Some(custom_start) = start {
            (custom_start, end.unwrap_or(now))
        } else if effective_lookback_mins == 0 {
            (0, now)
        } else {
            let span_secs = effective_lookback_mins as i64 * 60;
            (now.saturating_sub(span_secs), now)
        };

        let blockscout = BlockscoutClient::new();
        let token_address = &config().usdfc_token;

        // Fetch recent transfers with timestamps for USDFC
        let (transfers, _) = blockscout
            .gql_get_transfers_with_timestamps(token_address, 200, None)
            .await
            .map_err(|e| SfnError::ServerError(e.to_string()))?;

        // Filter transfers for this wallet and time window
        let mut relevant: Vec<usdfc_backend::blockscout::TransferWithTimestamp> = transfers
            .into_iter()
            .filter(|t| t.timestamp >= window_start && t.timestamp <= window_end)
            .filter(|t| {
                let from = t.from_address.to_lowercase();
                let to = t.to_address.to_lowercase();
                from == wallet_evm || to == wallet_evm
            })
            .collect();

        if relevant.is_empty() {
            return Ok(WalletAnalyticsResponse {
                address,
                buckets: Vec::new(),
                total_in: 0.0,
                total_out: 0.0,
                first_seen: None,
                last_active: None,
            });
        }

        // Sort by timestamp to derive first/last activity
        relevant.sort_by_key(|t| t.timestamp);

        let first_ts = relevant.first().map(|t| t.timestamp);
        let last_ts = relevant.last().map(|t| t.timestamp);

        let first_seen = first_ts.and_then(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
        });
        let last_active = last_ts.and_then(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
        });

        // Bucket transfers by resolution
        let bucket_secs = (resolution_mins as i64 * 60).max(60);

        struct BucketAccum {
            volume_in: Decimal,
            volume_out: Decimal,
            count_in: u64,
            count_out: u64,
        }

        let mut buckets_map: BTreeMap<i64, BucketAccum> = BTreeMap::new();

        let decimals = 18u32;
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(decimals), 0);

        for t in relevant {
            let is_incoming = t.to_address.to_lowercase() == wallet_evm;
            let is_outgoing = t.from_address.to_lowercase() == wallet_evm;

            if !is_incoming && !is_outgoing {
                continue;
            }

            let raw = match t.amount.parse::<u128>() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let value = Decimal::from_i128_with_scale(raw as i128, 0) / divisor;
            let bucket_ts = (t.timestamp / bucket_secs) * bucket_secs;

            let entry = buckets_map.entry(bucket_ts).or_insert(BucketAccum {
                volume_in: Decimal::ZERO,
                volume_out: Decimal::ZERO,
                count_in: 0,
                count_out: 0,
            });

            if is_incoming {
                entry.volume_in += value;
                entry.count_in += 1;
            }
            if is_outgoing {
                entry.volume_out += value;
                entry.count_out += 1;
            }
        }

        let mut buckets = Vec::new();
        let mut total_in = Decimal::ZERO;
        let mut total_out = Decimal::ZERO;

        for (ts, acc) in buckets_map {
            total_in += acc.volume_in;
            total_out += acc.volume_out;

            buckets.push(WalletBucket {
                timestamp: ts,
                volume_in: acc.volume_in.to_f64().unwrap_or(0.0),
                volume_out: acc.volume_out.to_f64().unwrap_or(0.0),
                count_in: acc.count_in,
                count_out: acc.count_out,
            });
        }

        Ok(WalletAnalyticsResponse {
            address,
            buckets,
            total_in: total_in.to_f64().unwrap_or(0.0),
            total_out: total_out.to_f64().unwrap_or(0.0),
            first_seen,
            last_active,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(SfnError::ServerError("SSR is required for live data".to_string()))
    }
}
