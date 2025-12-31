//! REST API Handlers for USDFC Analytics Terminal
//!
//! These handlers provide JSON REST API endpoints that wrap the server functions.
//! They are designed for external integrations and third-party applications.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::api::models::{
    ApiResponse, PaginationQuery, HistoryQuery as HistoryQueryParams,
    PriceResponse, MetricsResponse, HealthResponse, ServiceStatus,
    TroveResponse, TrovesListResponse, TransactionResponse, TransactionsListResponse,
    AddressInfoResponse, LendingMarketResponse, LendingMarketsResponse,
    HistoricalResponse, HistoricalDataPoint, TopHoldersResponse, TokenHolderResponse,
};
use crate::server_fn::{
    get_protocol_metrics, get_recent_transactions, get_troves, get_lending_markets,
    get_address_info, get_usdfc_price_data, check_api_health, get_top_holders,
    get_holder_count, get_daily_volumes, get_normalized_address,
};
use crate::rpc::RpcClient;
use rust_decimal::prelude::ToPrimitive;

// ============================================================================
// Health Endpoint
// ============================================================================

/// GET /api/v1/health
/// Returns API health status for all data sources
pub async fn get_health() -> impl IntoResponse {
    match check_api_health().await {
        Ok(status) => {
            let services = vec![
                ServiceStatus {
                    name: "rpc".to_string(),
                    status: if status.rpc_ok { "healthy" } else { "unhealthy" }.to_string(),
                    latency_ms: None,
                },
                ServiceStatus {
                    name: "blockscout".to_string(),
                    status: if status.blockscout_ok { "healthy" } else { "unhealthy" }.to_string(),
                    latency_ms: None,
                },
                ServiceStatus {
                    name: "subgraph".to_string(),
                    status: if status.subgraph_ok { "healthy" } else { "unhealthy" }.to_string(),
                    latency_ms: None,
                },
            ];

            let all_healthy = status.rpc_ok && status.blockscout_ok && status.subgraph_ok;
            let overall_status = if all_healthy { "healthy" } else { "degraded" };

            let response = HealthResponse {
                status: overall_status.to_string(),
                services,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Price Endpoint
// ============================================================================

/// GET /api/v1/price
/// Returns current USDFC price data from GeckoTerminal
pub async fn get_price() -> impl IntoResponse {
    let rpc = RpcClient::new();
    let fil_price = rpc.get_fil_price().await.ok().and_then(|p| p.to_f64());

    match get_usdfc_price_data().await {
        Ok(price_data) => {
            let response = PriceResponse {
                usdfc_usd: price_data.price_usd,
                fil_usd: fil_price,
                change_24h: price_data.price_change_24h,
                volume_24h: price_data.volume_24h,
                liquidity_usd: price_data.liquidity_usd,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Metrics Endpoint
// ============================================================================

/// GET /api/v1/metrics
/// Returns protocol-wide metrics (total supply, collateral, TCR, etc.)
pub async fn get_metrics() -> impl IntoResponse {
    let (metrics_result, price_result, holders_result) = tokio::join!(
        get_protocol_metrics(),
        get_usdfc_price_data(),
        get_holder_count()
    );

    match metrics_result {
        Ok(metrics) => {
            let price_data = price_result.ok();
            let holders = holders_result.ok();

            let response = MetricsResponse {
                tcr: format!("{:.2}%", metrics.tcr),
                total_supply: metrics.total_supply.to_string(),
                circulating_supply: metrics.circulating_supply.to_string(),
                total_collateral: metrics.total_collateral.to_string(),
                active_troves: metrics.active_troves,
                holders,
                volume_24h: price_data.as_ref().and_then(|p| p.volume_24h),
                liquidity_usd: price_data.as_ref().and_then(|p| p.liquidity_usd),
                stability_pool_balance: metrics.stability_pool_balance.to_string(),
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// History Endpoint
// ============================================================================

/// GET /api/v1/history
/// Returns historical volume data
pub async fn get_history(Query(params): Query<HistoryQueryParams>) -> impl IntoResponse {
    let resolution = params.resolution.as_deref().unwrap_or("1d");
    let metric = params.metric.as_deref().unwrap_or("volume");

    // Calculate time range
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let from = params.from.unwrap_or(now - 30 * 24 * 60 * 60); // Default 30 days ago
    let to = params.to.unwrap_or(now);
    let days = ((to - from) / (24 * 60 * 60)) as i32;

    match get_daily_volumes(Some(days.max(1))).await {
        Ok(volumes) => {
            let data: Vec<HistoricalDataPoint> = volumes
                .into_iter()
                .filter(|v| v.timestamp >= from && v.timestamp <= to)
                .map(|v| HistoricalDataPoint {
                    timestamp: v.timestamp,
                    value: v.volume,
                })
                .collect();

            let response = HistoricalResponse {
                metric: metric.to_string(),
                resolution: resolution.to_string(),
                from,
                to,
                data,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Troves Endpoints
// ============================================================================

/// GET /api/v1/troves
/// Returns list of all active troves with pagination
pub async fn get_troves_list(Query(params): Query<PaginationQuery>) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    match get_troves(Some(limit + offset), Some(0)).await {
        Ok(troves) => {
            let total = troves.len() as u64;
            let paginated: Vec<TroveResponse> = troves
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .map(|t| TroveResponse {
                    address: t.address,
                    collateral: t.collateral.to_string(),
                    debt: t.debt.to_string(),
                    icr: format!("{:.2}%", t.icr),
                    status: t.status.as_str().to_lowercase().replace(' ', "_"),
                })
                .collect();

            let response = TrovesListResponse {
                troves: paginated,
                total,
                offset,
                limit,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// GET /api/v1/troves/:addr
/// Returns trove info for a specific address
pub async fn get_trove_by_address(Path(addr): Path<String>) -> impl IntoResponse {
    // Validate address format
    if let Err(e) = crate::error::ValidationError::validate_address(&addr) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(e.to_string())),
        );
    }

    // Get all troves and filter by address
    match get_troves(Some(500), Some(0)).await {
        Ok(troves) => {
            let addr_lower = addr.to_lowercase();
            if let Some(trove) = troves.into_iter().find(|t| t.address.to_lowercase() == addr_lower) {
                let response = TroveResponse {
                    address: trove.address,
                    collateral: trove.collateral.to_string(),
                    debt: trove.debt.to_string(),
                    icr: format!("{:.2}%", trove.icr),
                    status: trove.status.as_str().to_lowercase().replace(' ', "_"),
                };
                (StatusCode::OK, Json(ApiResponse::success(response)))
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Trove not found for address")),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Transactions Endpoint
// ============================================================================

/// GET /api/v1/transactions
/// Returns recent USDFC transactions
pub async fn get_transactions(Query(params): Query<PaginationQuery>) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    match get_recent_transactions(Some(limit + offset)).await {
        Ok(transactions) => {
            let total = transactions.len() as u64;
            let paginated: Vec<TransactionResponse> = transactions
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .map(|tx| TransactionResponse {
                    hash: tx.hash,
                    tx_type: tx.tx_type.as_str().to_lowercase(),
                    amount: tx.amount.to_string(),
                    from: tx.from,
                    to: tx.to,
                    timestamp: tx.timestamp,
                    block: tx.block,
                    status: tx.status.as_str().to_lowercase(),
                })
                .collect();

            let response = TransactionsListResponse {
                transactions: paginated,
                total,
                offset,
                limit,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Address Endpoint
// ============================================================================

/// GET /api/v1/address/:addr
/// Returns address info including USDFC balance and activity
pub async fn get_address(Path(addr): Path<String>) -> impl IntoResponse {
    // Validate address format
    if let Err(e) = crate::error::ValidationError::validate_address(&addr) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(e.to_string())),
        );
    }

    // Get address info and normalized address in parallel
    let (info_result, normalized_result) = tokio::join!(
        get_address_info(addr.clone()),
        get_normalized_address(addr.clone())
    );

    match info_result {
        Ok(info) => {
            let f4_address = normalized_result.ok().and_then(|n| n.f4);

            let response = AddressInfoResponse {
                address: info.address,
                usdfc_balance: info.usdfc_balance,
                transfer_count: info.transfer_count,
                first_seen: info.first_seen,
                address_type: info.address_type,
                f4_address,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Lending Endpoint
// ============================================================================

/// GET /api/v1/lending
/// Returns lending market data from Secured Finance subgraph
pub async fn get_lending() -> impl IntoResponse {
    match get_lending_markets().await {
        Ok(markets) => {
            let market_responses: Vec<LendingMarketResponse> = markets
                .into_iter()
                .map(|m| LendingMarketResponse {
                    maturity: m.maturity,
                    currency: m.currency,
                    lend_apr: m.lend_apr,
                    borrow_apr: m.borrow_apr,
                    volume: m.volume,
                    is_active: m.is_active,
                })
                .collect();

            let response = LendingMarketsResponse {
                markets: market_responses,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

// ============================================================================
// Holders Endpoint
// ============================================================================

/// GET /api/v1/holders
/// Returns top USDFC holders
pub async fn get_holders(Query(params): Query<PaginationQuery>) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(50);

    let (holders_result, count_result) = tokio::join!(
        get_top_holders(Some(limit)),
        get_holder_count()
    );

    match holders_result {
        Ok(holders) => {
            let holder_responses: Vec<TokenHolderResponse> = holders
                .into_iter()
                .map(|h| TokenHolderResponse {
                    address: h.address,
                    balance: h.balance.to_string(),
                    share: None, // Could calculate if we had total supply
                })
                .collect();

            let response = TopHoldersResponse {
                holders: holder_responses,
                total_holders: count_result.ok(),
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}
