use crate::config::config;
use crate::error::{ApiError, ApiResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// Simple JSON-RPC client for Ethereum-compatible chains
#[derive(Clone)]
pub struct RpcClient {
    client: reqwest::Client,
    url: String,
    fallback_urls: Vec<String>,
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<Value>,
    id: u64,
}

#[derive(Deserialize, Debug)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: u64,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    code: i64,
    message: String,
}

impl RpcClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(config().rpc_timeout_secs))
                .build()
                .expect("failed to build RPC HTTP client"),
            url: config().rpc_url.clone(),
            fallback_urls: config().rpc_fallback_urls.clone(),
        }
    }

    /// Try a single RPC URL with retries
    async fn call_with_url(&self, url: &str, request: &JsonRpcRequest) -> ApiResult<Value> {
        let max_retries = config().rpc_retry_count;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            // Exponential backoff: 0ms, 100ms, 200ms, 400ms, 800ms...
            if attempt > 0 {
                let backoff_ms = 100 * (1 << (attempt - 1));
                tracing::warn!(
                    "RPC retry attempt {}/{} for {} on {} after {}ms backoff",
                    attempt,
                    max_retries,
                    request.method,
                    url,
                    backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }

            let response = match self
                .client
                .post(url)
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(ApiError::RpcError(format!("HTTP error: {}", e)));
                    continue; // Retry on network errors
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "failed to read body".to_string());

                // Retry on 5xx server errors, fail immediately on 4xx client errors
                if status.is_server_error() {
                    last_error = Some(ApiError::RpcError(format!("HTTP {}: {}", status, body)));
                    continue;
                } else {
                    return Err(ApiError::RpcError(format!("HTTP {}: {}", status, body)));
                }
            }

            let rpc_response: JsonRpcResponse = match response.json().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(ApiError::RpcError(format!("Parse error: {}", e)));
                    continue; // Retry on parse errors
                }
            };

            if let Some(error) = rpc_response.error {
                // RPC-level errors (contract reverts, etc.) should not retry
                return Err(ApiError::RpcError(format!(
                    "RPC error {}: {}",
                    error.code, error.message
                )));
            }

            // Success
            return rpc_response
                .result
                .ok_or_else(|| ApiError::RpcError("No result in response".to_string()));
        }

        // All retries exhausted for this URL
        Err(last_error.unwrap_or_else(|| ApiError::RpcError("All retries failed".to_string())))
    }

    /// Make a JSON-RPC call with retry logic, exponential backoff, and fallback URLs
    async fn call(&self, method: &str, params: Vec<Value>) -> ApiResult<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        // Try primary URL first
        match self.call_with_url(&self.url, &request).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!("Primary RPC URL {} failed: {}. Trying fallbacks...", self.url, e);
            }
        }

        // Try fallback URLs
        for fallback_url in &self.fallback_urls {
            tracing::info!("Trying fallback RPC URL: {}", fallback_url);
            match self.call_with_url(fallback_url, &request).await {
                Ok(result) => {
                    tracing::info!("Fallback RPC URL {} succeeded", fallback_url);
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Fallback RPC URL {} failed: {}", fallback_url, e);
                    continue;
                }
            }
        }

        // All URLs exhausted
        Err(ApiError::RpcError(format!(
            "All RPC endpoints failed (tried {} URLs)",
            1 + self.fallback_urls.len()
        )))
    }

    /// Call a contract method (eth_call)
    async fn eth_call(&self, to: &str, data: &str) -> ApiResult<String> {
        let params = vec![
            json!({
                "to": to,
                "data": data
            }),
            json!("latest"),
        ];

        let result = self.call("eth_call", params).await?;
        result
            .as_str()
            .ok_or_else(|| ApiError::RpcError("Invalid result format".to_string()))
            .map(|s| s.to_string())
    }

    /// Get total supply of USDFC token
    pub async fn get_total_supply(&self) -> ApiResult<Decimal> {
        // totalSupply() function signature: 0x18160ddd
        let data = "0x18160ddd";
        let result = self.eth_call(&config().usdfc_token, data).await?;
        
        // Parse hex result to decimal
        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;
        
        // Convert from wei (18 decimals) to USDFC - use from_i128 to avoid overflow
        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Get total collateral in system
    pub async fn get_total_collateral(&self) -> ApiResult<Decimal> {
        // getEntireSystemColl() function signature: 0x887105d3
        let data = "0x887105d3";
        let result = self.eth_call(&config().trove_manager, data).await?;
        
        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;
        
        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Get number of active troves
    pub async fn get_trove_owners_count(&self) -> ApiResult<u64> {
        // getTroveOwnersCount() function signature: 0x49eefeee
        let data = "0x49eefeee";
        let result = self.eth_call(&config().trove_manager, data).await?;
        
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))
    }

    /// Get FIL price from oracle
    pub async fn get_fil_price(&self) -> ApiResult<Decimal> {
        // lastGoodPrice() function signature: 0x0490be83
        let data = "0x0490be83";
        let result = self.eth_call(&config().price_feed, data).await?;
        
        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;
        
        // Price is returned with 18 decimals
        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Get stability pool balance
    pub async fn get_stability_pool_balance(&self) -> ApiResult<Decimal> {
        // getTotalDebtTokenDeposits() function signature: 0x0d9a6b35
        let data = "0x0d9a6b35";
        let result = self.eth_call(&config().stability_pool, data).await?;

        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;

        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Get active pool collateral (FIL) - used for historical TCR calculation
    pub async fn get_active_pool_eth(&self) -> ApiResult<Decimal> {
        // getETH() function signature: 0x4a59ff51
        let data = "0x4a59ff51";
        let result = self.eth_call(&config().active_pool, data).await?;

        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;

        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Get total system debt from TroveManager
    pub async fn get_total_debt(&self) -> ApiResult<Decimal> {
        // getEntireSystemDebt() function signature: 0x284ce5d8
        let data = "0x284ce5d8";
        let result = self.eth_call(&config().trove_manager, data).await?;

        let value = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse error: {}", e)))?;

        let wei = Decimal::from_i128_with_scale(value as i128, 0);
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        Ok(wei / divisor)
    }

    /// Calculate TCR (Total Collateral Ratio)
    pub async fn get_tcr(&self) -> ApiResult<Decimal> {
        // TCR = (total_collateral * fil_price) / total_debt * 100
        //
        // NOTE: getEntireSystemDebt() can revert on some contract versions.
        // Fallback to using total USDFC supply as an approximation of system debt,
        // since in Liquity-style protocols: total_debt ≈ total_usdfc_supply
        let total_debt = match self.get_total_debt().await {
            Ok(debt) => debt,
            Err(e) => {
                // CRITICAL: Log fallback behavior for monitoring
                tracing::warn!(
                    "get_total_debt() failed with error: {}. Falling back to total supply as debt approximation. \
                    This may indicate contract incompatibility or RPC issues.",
                    e
                );
                // Fallback: use USDFC total supply as debt approximation
                self.get_total_supply().await?
            }
        };
        let total_collateral = self.get_total_collateral().await?;
        let fil_price = self.get_fil_price().await?;

        if total_debt.is_zero() {
            // No debt means infinite collateralization - return a very high TCR
            return Ok(Decimal::new(999999, 0));
        }

        let collateral_value = total_collateral * fil_price;
        let tcr = (collateral_value / total_debt) * Decimal::new(100, 0);

        Ok(tcr)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> ApiResult<u64> {
        let result = self.call("eth_blockNumber", vec![]).await?;
        
        let hex = result
            .as_str()
            .ok_or_else(|| ApiError::RpcError("Invalid block number format".to_string()))?;
        
        u64::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|e| ApiError::RpcError(format!("Parse block number: {}", e)))
    }

    /// Get multiple sorted troves via MultiTroveGetter contract
    pub async fn get_multiple_sorted_troves(&self, start_idx: i32, count: u32) -> ApiResult<Vec<TroveData>> {
        // Function: getMultipleSortedTroves(int256,uint256)
        // Selector: 0xb90bce45
        let selector = "0xb90bce45";
        
        // Encode parameters: int256 + uint256 (each 32 bytes)
        let start_hex = if start_idx >= 0 {
            format!("{:064x}", start_idx as u64)
        } else {
            // 256-bit two's complement: for negative n, result is 2^256 + n
            // Since we can't represent 2^256, we use the fact that for display,
            // -1 is all 1s (64 f's), -2 is 64 f's minus 1, etc.
            let neg_val = start_idx as i64;  // sign-extend to i64
            // For the lower 64 bits in hex
            let lower = neg_val as u64;
            // Upper 192 bits are all 1s for negative numbers
            format!("ffffffffffffffffffffffffffffffffffffffffffffffff{:016x}", lower)
        };
        let count_hex = format!("{:064x}", count);
        
        let data = format!("{}{}{}", selector, start_hex, count_hex);
        let result = self.eth_call(&config().multi_trove_getter, &data).await?;
        
        // Parse response: dynamic array of CombinedTroveData structs
        parse_trove_response(&result)
    }
}

/// Trove data from MultiTroveGetter
#[derive(Debug, Clone)]
pub struct TroveData {
    pub owner: String,
    pub debt: Decimal,
    pub coll: Decimal,
    pub stake: Decimal,
    pub snapshot_fil: Decimal,
    pub snapshot_debt: Decimal,
}

/// Parse MultiTroveGetter response
fn parse_trove_response(hex_result: &str) -> ApiResult<Vec<TroveData>> {
    let hex = hex_result.trim_start_matches("0x");
    
    // First 64 chars = array length
    if hex.len() < 64 {
        return Ok(vec![]);
    }
    
    let array_len = u64::from_str_radix(&hex[0..64], 16)
        .map_err(|e| ApiError::RpcError(format!("Parse array length: {}", e)))?;
    
    if array_len == 0 {
        return Ok(vec![]);
    }
    
    // Each struct is 6 × 32 bytes = 192 bytes = 384 hex chars
    let mut troves = Vec::new();
    
    for i in 0..array_len {
        let offset = 64 + (i as usize * 384);
        
        if hex.len() < offset + 384 {
            break;  // Incomplete data
        }
        
        // Extract fields (each 64 hex chars = 32 bytes)
        let owner_hex = &hex[offset + 24..offset + 64];  // Skip first 24 chars (address is 20 bytes)
        let debt_hex = &hex[offset + 64..offset + 128];
        let coll_hex = &hex[offset + 128..offset + 192];
        let stake_hex = &hex[offset + 192..offset + 256];
        let snapshot_fil_hex = &hex[offset + 256..offset + 320];
        let snapshot_debt_hex = &hex[offset + 320..offset + 384];
        
        // Convert to decimals (divide by 1e18)
        let debt = u128::from_str_radix(debt_hex, 16)
            .map_err(|e| ApiError::RpcError(format!("Parse debt: {}", e)))?;
        let coll = u128::from_str_radix(coll_hex, 16)
            .map_err(|e| ApiError::RpcError(format!("Parse coll: {}", e)))?;
        let stake = u128::from_str_radix(stake_hex, 16)
            .map_err(|e| ApiError::RpcError(format!("Parse stake: {}", e)))?;
        let snapshot_fil = u128::from_str_radix(snapshot_fil_hex, 16)
            .map_err(|e| ApiError::RpcError(format!("Parse snapshot_fil: {}", e)))?;
        let snapshot_debt = u128::from_str_radix(snapshot_debt_hex, 16)
            .map_err(|e| ApiError::RpcError(format!("Parse snapshot_debt: {}", e)))?;
        
        let divisor = Decimal::from_i128_with_scale(10_i128.pow(18), 0);
        troves.push(TroveData {
            owner: format!("0x{}", owner_hex),
            debt: Decimal::from_i128_with_scale(debt as i128, 0) / divisor,
            coll: Decimal::from_i128_with_scale(coll as i128, 0) / divisor,
            stake: Decimal::from_i128_with_scale(stake as i128, 0) / divisor,
            snapshot_fil: Decimal::from_i128_with_scale(snapshot_fil as i128, 0) / divisor,
            snapshot_debt: Decimal::from_i128_with_scale(snapshot_debt as i128, 0) / divisor,
        });
    }
    
    Ok(troves)
}


impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}
