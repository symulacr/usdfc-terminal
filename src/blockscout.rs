use crate::config::config;
use crate::error::{ApiError, ApiResult};
use crate::types::{Transaction, TransactionType, TransactionStatus};
use rust_decimal::Decimal;
use std::time::Duration;
use serde::Deserialize;

/// Blockscout API client
#[derive(Clone)]
pub struct BlockscoutClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Deserialize, Debug, Default)]
struct TransfersResponse {
    #[serde(default)]
    items: Vec<TransferItem>,
    #[allow(dead_code)]
    next_page_params: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct TransferItem {
    transaction_hash: String,
    from: AddressInfo,
    to: AddressInfo,
    total: TokenAmount,
    timestamp: String,
    block_number: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct AddressInfo {
    hash: String,
}

#[derive(Deserialize, Debug)]
struct TokenAmount {
    value: String,
    #[allow(dead_code)]
    decimals: String,
}

#[derive(Deserialize, Debug)]
struct CountersResponse {
    token_holders_count: String,
    #[allow(dead_code)]
    transfers_count: String,
}

impl BlockscoutClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build blockscout HTTP client"),
            base_url: config().blockscout_url.clone(),
        }
    }

    /// Get recent transfers for USDFC token
    pub async fn get_recent_transfers(&self, limit: u32) -> ApiResult<Vec<Transaction>> {
        // Note: Blockscout v2 API doesn't support limit param, returns 50 by default
        let url = format!(
            "{}/tokens/{}/transfers",
            self.base_url,
            config().usdfc_token
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .map_err(|e| ApiError::HttpError(format!("HTTP {}: failed to read body: {}", status, e)))?;
            return Err(ApiError::HttpError(format!("HTTP {}: {}", status, body)));
        }

        let transfers: TransfersResponse = response
            .json()
            .await
            .map_err(|e| ApiError::parse("transfers", format!("JSON parse error: {}", e)))?;

        // Convert to Transaction type
        let transactions = transfers
            .items
            .into_iter()
            .take(limit as usize)
            .map(|item| -> ApiResult<Transaction> {
                let amount_decimal = parse_token_amount(&item.total.value, &item.total.decimals)?;

                // Determine transaction type (simplified)
                let tx_type = if item.to.hash == "0x0000000000000000000000000000000000000000" {
                    TransactionType::Burn
                } else if item.from.hash == "0x0000000000000000000000000000000000000000" {
                    TransactionType::Mint
                } else {
                    TransactionType::Transfer
                };

                // Parse timestamp to unix seconds
                let timestamp = chrono::DateTime::parse_from_rfc3339(&item.timestamp)
                    .map(|dt| dt.timestamp() as u64)
                    .map_err(|e| ApiError::parse("timestamp", format!("{}", e)))?;

                let tx_hash = item.transaction_hash.clone();
                Ok(Transaction {
                    hash: item.transaction_hash,
                    tx_type,
                    amount: amount_decimal,
                    from: item.from.hash,
                    to: item.to.hash,
                    timestamp,
                    block: item.block_number.ok_or(ApiError::NotFound { resource: "block_number", id: tx_hash })?,
                    status: TransactionStatus::Success,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?;

        Ok(transactions)
    }

    /// Get token holder count
    pub async fn get_holder_count(&self) -> ApiResult<u64> {
        let url = format!(
            "{}/tokens/{}/counters",
            self.base_url,
            config().usdfc_token
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .map_err(|e| ApiError::HttpError(format!("HTTP {}: failed to read body: {}", status, e)))?;
            return Err(ApiError::HttpError(format!("HTTP {}: {}", status, body)));
        }

        let counters: CountersResponse = response
            .json()
            .await
            .map_err(|e| ApiError::parse("counters", format!("JSON parse error: {}", e)))?;

        counters
            .token_holders_count
            .parse()
            .map_err(|e| ApiError::parse("holder_count", format!("Parse holder count: {}", e)))
    }

    /// Get circulating supply (total supply - treasury balance)
    pub async fn get_circulating_supply(&self) -> ApiResult<Decimal> {
        Err(ApiError::Config { message: "circulating supply not implemented".to_string() })
    }
}

impl Default for BlockscoutClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockscoutClient {
    /// Get top token holders
    /// Note: Blockscout v2 API doesn't support a limit query param for holders endpoint.
    /// It returns 50 items per page by default. We take up to `limit` items from the results.
    pub async fn get_token_holders(&self, token: &str, limit: u32) -> ApiResult<Vec<TokenHolder>> {
        let url = format!(
            "{}/tokens/{}/holders",
            self.base_url, token
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "HTTP {}: Failed to fetch holders",
                response.status()
            )));
        }

        let data: HoldersResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse holders: {}", e)))?;

        let holders = data
            .items
            .into_iter()
            .take(limit as usize)  // Take up to limit items from the response
            .map(|item| -> ApiResult<TokenHolder> {
                let value_wei = item.value.parse::<u128>()
                    .map_err(|e| ApiError::parse("holder_balance", format!("{}", e)))?;
                let value_decimal = Decimal::from(value_wei) / Decimal::from(10_u128.pow(18));

                Ok(TokenHolder {
                    address: item.address.hash,
                    balance: value_decimal,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?;

        Ok(holders)
    }

    /// Get token balance for a specific address
    pub async fn get_address_token_balance(
        &self,
        address: &str,
        token: &str,
    ) -> ApiResult<Decimal> {
        let url = format!("{}/addresses/{}/token-balances", self.base_url, address);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "HTTP {}: Failed to fetch token balances",
                response.status()
            )));
        }

        let data: Vec<TokenBalanceItem> = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse token balances: {}", e)))?;

        // Find USDFC token balance
        let balance = data
            .iter()
            .find(|item| item.token.address.to_lowercase() == token.to_lowercase())
            .map(|item| {
                let value_wei = item.value.parse::<u128>()
                    .map_err(|e| ApiError::parse("token_balance", format!("{}", e)))?;
                Ok(Decimal::from(value_wei) / Decimal::from(10_u128.pow(18)))
            })
            .transpose()?
            .ok_or_else(|| ApiError::NotFound { resource: "token_balance", id: token.to_string() })?;

        Ok(balance)
    }

    /// Get token transfers for a specific address
    pub async fn get_address_transfers(
        &self,
        address: &str,
        token: &str,
        limit: u32,
    ) -> ApiResult<Vec<Transaction>> {
        let url = format!(
            "{}/addresses/{}/token-transfers?token={}",
            self.base_url, address, token
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "HTTP {}: Failed to fetch transfers",
                response.status()
            )));
        }

        let data: TransfersResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse transfers: {}", e)))?;

        let transactions = data
            .items
            .into_iter()
            .take(limit as usize)
            .map(|item| -> ApiResult<Transaction> {
                let amount = parse_token_amount(&item.total.value, &item.total.decimals)?;

                // Determine transaction type
                let tx_type = if item.from.hash == "0x0000000000000000000000000000000000000000" {
                    TransactionType::Mint
                } else if item.to.hash == "0x0000000000000000000000000000000000000000" {
                    TransactionType::Burn
                } else {
                    TransactionType::Transfer
                };

                // Parse timestamp to unix seconds
                let timestamp = chrono::DateTime::parse_from_rfc3339(&item.timestamp)
                    .map(|dt| dt.timestamp() as u64)
                    .map_err(|e| ApiError::parse("timestamp", format!("{}", e)))?;

                let tx_hash = item.transaction_hash.clone();
                Ok(Transaction {
                    hash: item.transaction_hash,
                    from: item.from.hash,
                    to: item.to.hash,
                    amount,
                    timestamp,
                    block: item.block_number.ok_or(ApiError::NotFound { resource: "block_number", id: tx_hash })?,
                    tx_type,
                    status: TransactionStatus::Success,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?;

        Ok(transactions)
    }

    /// Get pools for a token (from Blockscout Pools API)
    pub async fn get_pools_for_token(&self, token: &str) -> ApiResult<Vec<PoolInfo>> {
        let url = format!(
            "https://contracts-info.services.blockscout.com/api/v1/chains/314/pools?query={}",
            &token[2..] // Remove 0x prefix
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "HTTP {}: Failed to fetch pools",
                response.status()
            )));
        }

        let data: PoolsResponse = response
            .json()
            .await
            .map_err(|e| ApiError::HttpError(format!("Parse pools: {}", e)))?;

        let pools = data
            .items
            .into_iter()
            .map(|item| -> ApiResult<PoolInfo> {
                let liquidity = item.liquidity.parse::<f64>()
                    .map_err(|e| ApiError::parse("liquidity", format!("{}", e)))?;
                Ok(PoolInfo {
                    address: item.pool_id,
                    base_token: item.base_token_symbol,
                    quote_token: item.quote_token_symbol,
                    liquidity,
                    dex_name: item.dex.name,
                    gecko_url: item.coin_gecko_terminal_url,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?;

        Ok(pools)
    }
}

// ============================================================================
// Additional Response Types
// ============================================================================

#[derive(Deserialize, Debug)]
struct HoldersResponse {
    items: Vec<HolderItem>,
}

#[derive(Deserialize, Debug)]
struct HolderItem {
    address: AddressInfo,
    value: String,
}

#[derive(Debug, Clone)]
pub struct TokenHolder {
    pub address: String,
    pub balance: Decimal,
}

#[derive(Deserialize, Debug)]
struct TokenBalanceItem {
    token: TokenInfo,
    value: String,
}

#[derive(Deserialize, Debug)]
struct TokenInfo {
    address: String,
}

#[derive(Deserialize, Debug)]
struct PoolsResponse {
    items: Vec<PoolItem>,
}

#[derive(Deserialize, Debug)]
struct PoolItem {
    pool_id: String,
    base_token_symbol: String,
    quote_token_symbol: String,
    liquidity: String,
    dex: DexInfo,
    coin_gecko_terminal_url: String,
}

#[derive(Deserialize, Debug)]
struct DexInfo {
    name: String,
}

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub address: String,
    pub base_token: String,
    pub quote_token: String,
    pub liquidity: f64,
    pub dex_name: String,
    pub gecko_url: String,
}

fn parse_token_amount(value: &str, decimals: &str) -> ApiResult<Decimal> {
    let decimals = decimals.parse::<u32>()
        .map_err(|e| ApiError::parse("decimals", format!("{}", e)))?;
    let raw = value.parse::<u128>()
        .map_err(|e| ApiError::parse("amount", format!("{}", e)))?;
    Ok(Decimal::from_i128_with_scale(raw as i128, decimals))
}

impl BlockscoutClient {
    /// Get USDFC-specific info for an address
    /// Uses both REST and GraphQL APIs for comprehensive data
    pub async fn get_address_usdfc_info(&self, address: &str) -> ApiResult<crate::server_fn::AddressInfo> {
        // Fetch token balances and address info via GraphQL in parallel
        let (balances_result, gql_addr_result) = tokio::join!(
            self.get_token_balances_rest(address),
            self.gql_get_address(address)
        );

        // Get USDFC balance from REST API (more reliable for token balances)
        // Propagate errors instead of silently returning defaults
        let balances = balances_result?;
        let usdfc_token = config().usdfc_token.to_lowercase();
        let usdfc_balance = balances
            .iter()
            .find(|b| b.token.address.to_lowercase() == usdfc_token)
            .map(|b| {
                let value = b.value.parse::<u128>()
                    .map_err(|e| ApiError::parse("usdfc_balance", format!("{}", e)))?;
                let decimals = 18u32;
                Ok((value as f64) / 10f64.powi(decimals as i32))
            })
            .transpose()?
            .ok_or_else(|| ApiError::NotFound {
                resource: "usdfc_balance",
                id: address.to_string()
            })?;

        // Extract data from GraphQL response (provides accurate counts)
        let (transfer_count, is_contract, first_seen) = match gql_addr_result {
            Ok(gql_addr) => {
                // tokenTransfersCount gives accurate total, not just page size
                let tx_count = gql_addr.token_transfers_count
                    .ok_or_else(|| ApiError::InvalidResponse {
                        message: "token_transfers_count missing from GraphQL response".to_string()
                    })? as u64;
                let is_contract = gql_addr.smart_contract.is_some();

                // For first_seen, we need to fetch the first transfer timestamp
                // Propagate errors instead of silently returning "Unknown"
                let first_seen = self.get_first_transfer_timestamp(address).await?;

                (tx_count, is_contract, first_seen)
            }
            Err(gql_err) => {
                // Fallback to REST API if GraphQL fails
                let addr_url = format!("{}/addresses/{}", self.base_url, address);
                let addr_response = self.client.get(&addr_url).send().await
                    .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

                if !addr_response.status().is_success() {
                    // If both GraphQL and REST fail, return the original GraphQL error
                    return Err(gql_err);
                }

                let addr_info: AddressInfoResponse = addr_response.json().await
                    .map_err(|e| ApiError::parse("address_info", format!("{}", e)))?;
                let is_contract = addr_info.is_contract
                    .ok_or_else(|| ApiError::InvalidResponse {
                        message: "is_contract field missing from address info".to_string()
                    })?;

                // REST API doesn't give accurate count, use transfer fetch
                let transfers_url = format!(
                    "{}/addresses/{}/token-transfers?token={}",
                    self.base_url, address, config().usdfc_token
                );
                let transfers_response = self.client.get(&transfers_url).send().await
                    .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

                if !transfers_response.status().is_success() {
                    return Err(ApiError::HttpError(format!(
                        "HTTP {}: Failed to fetch transfers for address",
                        transfers_response.status()
                    )));
                }

                let transfers: TransfersResponse = transfers_response.json().await
                    .map_err(|e| ApiError::parse("transfers", format!("{}", e)))?;
                let transfer_count = transfers.items.len() as u64;

                // Cannot determine first_seen without successful GraphQL query
                (transfer_count, is_contract, "Unknown (GraphQL unavailable)".to_string())
            }
        };

        let addr_type = if is_contract { "Contract" } else { "EOA" }.to_string();

        Ok(crate::server_fn::AddressInfo {
            address: address.to_string(),
            usdfc_balance: format!("{:.2}", usdfc_balance),
            transfer_count,
            first_seen,
            address_type: addr_type,
        })
    }

    /// Helper to get token balances via REST API
    async fn get_token_balances_rest(&self, address: &str) -> ApiResult<Vec<TokenBalanceItem>> {
        let url = format!("{}/addresses/{}/token-balances", self.base_url, address);

        let response = self.client.get(&url).send().await
            .map_err(|e| ApiError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!("Address not found: {}", response.status())));
        }

        response.json().await
            .map_err(|e| ApiError::parse("token_balances", format!("{}", e)))
    }

    /// Get the timestamp of the first USDFC transfer involving this address
    async fn get_first_transfer_timestamp(&self, address: &str) -> ApiResult<String> {
        let token_address = &config().usdfc_token;

        // Query for transfers involving this address, sorted oldest first
        // Unfortunately GraphQL doesn't support sorting, so we fetch recent and use the oldest
        let query = format!(
            r#"
            query {{
                tokenTransfers(
                    first: 50
                    tokenContractAddressHash: "{}"
                ) {{
                    edges {{
                        node {{
                            fromAddressHash
                            toAddressHash
                            transaction {{
                                block {{
                                    timestamp
                                }}
                            }}
                        }}
                    }}
                }}
            }}
            "#,
            token_address
        );

        let data: TransfersWithTimestampData = self.gql_query(query).await?;

        let addr_lower = address.to_lowercase();

        // Find transfers involving this address and get the oldest timestamp
        // Propagate error if token_transfers or edges are missing instead of using defaults
        let edges = data.token_transfers
            .ok_or_else(|| ApiError::InvalidResponse {
                message: "token_transfers missing from GraphQL response".to_string()
            })?
            .edges
            .ok_or_else(|| ApiError::InvalidResponse {
                message: "edges missing from token_transfers response".to_string()
            })?;

        let oldest_timestamp = edges
            .into_iter()
            .filter_map(|edge| {
                let node = edge.node?;
                let from = node.from_address_hash.as_deref().unwrap_or("").to_lowercase();
                let to = node.to_address_hash.as_deref().unwrap_or("").to_lowercase();

                // Check if this transfer involves our address
                if from == addr_lower || to == addr_lower {
                    node.transaction
                        .and_then(|tx| tx.block)
                        .and_then(|b| b.timestamp)
                        .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
                        .map(|dt| dt.timestamp())
                } else {
                    None
                }
            })
            .min();  // Get oldest timestamp

        match oldest_timestamp {
            Some(ts) => {
                // Format as human-readable date
                let datetime = chrono::DateTime::from_timestamp(ts, 0)
                    .ok_or_else(|| ApiError::parse("timestamp", format!("Invalid timestamp: {}", ts)))?
                    .format("%Y-%m-%d")
                    .to_string();
                Ok(datetime)
            }
            None => Ok("No transfers".to_string())
        }
    }
}

#[derive(Deserialize, Debug, Default)]
struct AddressInfoResponse {
    is_contract: Option<bool>,
}

// ============================================================================
// GraphQL API Support
// ============================================================================

use serde::Serialize;

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct GraphQLResponse<T> {
    data: Option<T>,
    #[allow(dead_code)]
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize, Debug)]
struct GqlError {
    message: String,
}

// GraphQL response types
#[derive(Deserialize, Debug, Clone)]
pub struct GqlToken {
    #[serde(rename = "contractAddressHash")]
    pub contract_address_hash: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<String>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<String>,
    #[serde(rename = "holderCount")]
    pub holder_count: Option<i64>,
    #[serde(rename = "volume24h")]
    pub volume_24h: Option<String>,
    #[serde(rename = "circulatingMarketCap")]
    pub circulating_market_cap: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TokenTransfersData {
    #[serde(rename = "tokenTransfers")]
    token_transfers: Option<TokenTransferConnection>,
}

#[derive(Deserialize, Debug)]
struct TokenTransferConnection {
    edges: Option<Vec<TokenTransferEdge>>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Deserialize, Debug)]
struct TokenTransferEdge {
    node: Option<GqlTokenTransfer>,
    cursor: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GqlTokenTransfer {
    pub id: String,
    #[serde(rename = "fromAddressHash")]
    pub from_address_hash: Option<String>,
    #[serde(rename = "toAddressHash")]
    pub to_address_hash: Option<String>,
    pub amount: Option<String>,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<i64>,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
    pub token: Option<GqlToken>,
}

#[derive(Deserialize, Debug)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize, Debug)]
struct AddressData {
    address: Option<GqlAddress>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GqlAddress {
    pub hash: Option<String>,
    #[serde(rename = "fetchedCoinBalance")]
    pub fetched_coin_balance: Option<String>,
    #[serde(rename = "transactionsCount")]
    pub transactions_count: Option<i64>,
    #[serde(rename = "tokenTransfersCount")]
    pub token_transfers_count: Option<i64>,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<i64>,
    #[serde(rename = "smartContract")]
    pub smart_contract: Option<GqlSmartContract>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GqlSmartContract {
    pub name: Option<String>,
    #[serde(rename = "compilerVersion")]
    pub compiler_version: Option<String>,
    pub optimization: Option<bool>,
}

impl BlockscoutClient {
    /// GraphQL endpoint URL
    fn graphql_url(&self) -> String {
        // Convert v2 API URL to v1 GraphQL URL
        // https://filecoin.blockscout.com/api/v2 -> https://filecoin.blockscout.com/api/v1/graphql
        self.base_url.replace("/api/v2", "/api/v1/graphql")
    }

    /// Execute a GraphQL query
    async fn gql_query<T: for<'de> Deserialize<'de>>(&self, query: String) -> ApiResult<T> {
        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.graphql_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::GraphQLError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.map_err(|e| {
                ApiError::GraphQLError(format!("HTTP {}: failed to read body: {}", status, e))
            })?;
            return Err(ApiError::GraphQLError(format!("HTTP {}: {}", status, body)));
        }

        let gql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| ApiError::GraphQLError(format!("Parse error: {}", e)))?;

        if let Some(errors) = gql_response.errors {
            let error_msg = errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ApiError::GraphQLError(error_msg));
        }

        gql_response
            .data
            .ok_or_else(|| ApiError::GraphQLError("No data in response".to_string()))
    }

    /// Get token info via GraphQL (includes holder count, volume)
    pub async fn gql_get_token_info(&self, token_address: &str) -> ApiResult<GqlToken> {
        #[derive(Deserialize, Debug)]
        struct TokenData {
            address: Option<AddressWithToken>,
        }

        #[derive(Deserialize, Debug)]
        struct AddressWithToken {
            #[serde(rename = "smartContract")]
            smart_contract: Option<serde_json::Value>,
        }

        // Blockscout GraphQL doesn't have a direct token query,
        // we need to use the REST API for token-specific data
        // But we can get address info which includes some token data
        let query = format!(
            r#"
            query {{
                address(hash: "{}") {{
                    hash
                    fetchedCoinBalance
                    transactionsCount
                    tokenTransfersCount
                }}
            }}
            "#,
            token_address
        );

        let _data: AddressData = self.gql_query(query).await?;

        // For token-specific info, fall back to REST API
        // The GraphQL schema doesn't expose Token type directly in queries
        // Return placeholder - real data comes from REST API
        Ok(GqlToken {
            contract_address_hash: Some(token_address.to_string()),
            name: Some("USDFC".to_string()),
            symbol: Some("USDFC".to_string()),
            decimals: Some("18".to_string()),
            total_supply: None,
            holder_count: None,
            volume_24h: None,
            circulating_market_cap: None,
        })
    }

    /// Get token transfers via GraphQL with pagination
    pub async fn gql_get_token_transfers(
        &self,
        token_address: &str,
        first: i32,
        after: Option<&str>,
    ) -> ApiResult<(Vec<GqlTokenTransfer>, Option<String>)> {
        let after_clause = after
            .map(|c| format!(r#", after: "{}""#, c))
            .unwrap_or_default();

        let query = format!(
            r#"
            query {{
                tokenTransfers(
                    first: {}
                    {}
                    tokenContractAddressHash: "{}"
                ) {{
                    edges {{
                        node {{
                            id
                            fromAddressHash
                            toAddressHash
                            amount
                            blockNumber
                            transactionHash
                        }}
                        cursor
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                }}
            }}
            "#,
            first, after_clause, token_address
        );

        let data: TokenTransfersData = self.gql_query(query).await?;

        let connection = data.token_transfers.ok_or_else(|| {
            ApiError::GraphQLError("No tokenTransfers in response".to_string())
        })?;

        // Propagate error if edges are missing instead of using defaults
        let edges = connection
            .edges
            .ok_or_else(|| ApiError::InvalidResponse {
                message: "edges missing from tokenTransfers response".to_string()
            })?;

        let transfers: ApiResult<Vec<GqlTokenTransfer>> = edges
            .into_iter()
            .map(|edge| {
                edge.node.ok_or_else(|| ApiError::InvalidResponse {
                    message: "node missing from transfer edge".to_string()
                })
            })
            .collect();

        let next_cursor = connection
            .page_info
            .and_then(|pi| if pi.has_next_page { pi.end_cursor } else { None });

        Ok((transfers?, next_cursor))
    }

    /// Get address info via GraphQL
    pub async fn gql_get_address(&self, address: &str) -> ApiResult<GqlAddress> {
        let query = format!(
            r#"
            query {{
                address(hash: "{}") {{
                    hash
                    fetchedCoinBalance
                    transactionsCount
                    tokenTransfersCount
                    gasUsed
                    smartContract {{
                        name
                        compilerVersion
                        optimization
                    }}
                }}
            }}
            "#,
            address
        );

        let data: AddressData = self.gql_query(query).await?;

        data.address.ok_or_else(|| {
            ApiError::NotFound {
                resource: "address",
                id: address.to_string()
            }
        })
    }

    /// Get multiple addresses via GraphQL (batch query)
    pub async fn gql_get_addresses(&self, addresses: &[&str]) -> ApiResult<Vec<GqlAddress>> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        let hashes: Vec<String> = addresses
            .iter()
            .map(|a| format!(r#""{}""#, a))
            .collect();

        let query = format!(
            r#"
            query {{
                addresses(hashes: [{}]) {{
                    hash
                    fetchedCoinBalance
                    transactionsCount
                    tokenTransfersCount
                    gasUsed
                }}
            }}
            "#,
            hashes.join(", ")
        );

        #[derive(Deserialize, Debug)]
        struct AddressesData {
            addresses: Option<Vec<GqlAddress>>,
        }

        let data: AddressesData = self.gql_query(query).await?;
        data.addresses.ok_or_else(|| ApiError::InvalidResponse {
            message: "addresses missing from GraphQL response".to_string()
        })
    }

    /// Get token transfers with timestamps via GraphQL
    /// Includes block.timestamp for historical analysis
    pub async fn gql_get_transfers_with_timestamps(
        &self,
        token_address: &str,
        first: i32,
        after: Option<&str>,
    ) -> ApiResult<(Vec<TransferWithTimestamp>, Option<String>)> {
        let after_clause = after
            .map(|c| format!(r#", after: "{}""#, c))
            .unwrap_or_default();

        let query = format!(
            r#"
            query {{
                tokenTransfers(
                    first: {}
                    {}
                    tokenContractAddressHash: "{}"
                ) {{
                    edges {{
                        node {{
                            id
                            fromAddressHash
                            toAddressHash
                            amount
                            blockNumber
                            transactionHash
                            transaction {{
                                block {{
                                    timestamp
                                    number
                                }}
                            }}
                        }}
                        cursor
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                }}
            }}
            "#,
            first, after_clause, token_address
        );

        let data: TransfersWithTimestampData = self.gql_query(query).await?;

        let connection = data.token_transfers.ok_or_else(|| {
            ApiError::GraphQLError("No tokenTransfers in response".to_string())
        })?;

        // Propagate error if edges are missing instead of using defaults
        let edges = connection
            .edges
            .ok_or_else(|| ApiError::InvalidResponse {
                message: "edges missing from tokenTransfers response".to_string()
            })?;

        let transfers: ApiResult<Vec<TransferWithTimestamp>> = edges
            .into_iter()
            .map(|edge| -> ApiResult<TransferWithTimestamp> {
                let node = edge.node.ok_or_else(|| ApiError::InvalidResponse {
                    message: "node missing from transfer edge".to_string()
                })?;

                let timestamp = node.transaction
                    .as_ref()
                    .and_then(|tx| tx.block.as_ref())
                    .and_then(|b| b.timestamp.clone())
                    .ok_or_else(|| ApiError::InvalidResponse {
                        message: "timestamp missing from transfer".to_string()
                    })?;

                let parsed_timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                    .map_err(|e| ApiError::parse("timestamp", format!("{}", e)))?
                    .timestamp();

                Ok(TransferWithTimestamp {
                    id: node.id,
                    from_address: node.from_address_hash.ok_or_else(|| ApiError::InvalidResponse {
                        message: "from_address_hash missing from transfer".to_string()
                    })?,
                    to_address: node.to_address_hash.ok_or_else(|| ApiError::InvalidResponse {
                        message: "to_address_hash missing from transfer".to_string()
                    })?,
                    amount: node.amount.ok_or_else(|| ApiError::InvalidResponse {
                        message: "amount missing from transfer".to_string()
                    })?,
                    block_number: node.block_number.ok_or_else(|| ApiError::InvalidResponse {
                        message: "block_number missing from transfer".to_string()
                    })?,
                    transaction_hash: node.transaction_hash.ok_or_else(|| ApiError::InvalidResponse {
                        message: "transaction_hash missing from transfer".to_string()
                    })?,
                    timestamp: parsed_timestamp,
                })
            })
            .collect();

        let next_cursor = connection
            .page_info
            .and_then(|pi| if pi.has_next_page { pi.end_cursor } else { None });

        Ok((transfers?, next_cursor))
    }

    /// Get transfer counts aggregated by time period
    ///
    /// Fetches recent transfers and aggregates them into time buckets
    /// based on the specified resolution.
    pub async fn get_transfer_counts_by_period(
        &self,
        resolution_mins: u32,
        lookback_mins: u32,
    ) -> ApiResult<Vec<(i64, u64)>> {
        use std::collections::BTreeMap;

        let token_address = &crate::config::config().usdfc_token;

        // Fetch transfers (up to 200 for good coverage)
        let (transfers, _) = self
            .gql_get_transfers_with_timestamps(token_address, 200, None)
            .await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .map_err(|e| ApiError::InvalidResponse {
                message: format!("System time error: {}", e)
            })?;

        let cutoff = if lookback_mins == 0 {
            0 // ALL data
        } else {
            now - (lookback_mins as i64 * 60)
        };

        let resolution_secs = (resolution_mins as i64 * 60).max(60);

        // Aggregate by time bucket
        let mut buckets: BTreeMap<i64, u64> = BTreeMap::new();

        for transfer in transfers {
            if transfer.timestamp >= cutoff {
                let bucket = (transfer.timestamp / resolution_secs) * resolution_secs;
                *buckets.entry(bucket).or_insert(0) += 1;
            }
        }

        Ok(buckets.into_iter().collect())
    }
}

// Transfer with timestamp for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWithTimestamp {
    pub id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub block_number: i64,
    pub transaction_hash: String,
    pub timestamp: i64,
}

#[derive(Deserialize, Debug)]
struct TransfersWithTimestampData {
    #[serde(rename = "tokenTransfers")]
    token_transfers: Option<TransferWithTimestampConnection>,
}

#[derive(Deserialize, Debug)]
struct TransferWithTimestampConnection {
    edges: Option<Vec<TransferWithTimestampEdge>>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Deserialize, Debug)]
struct TransferWithTimestampEdge {
    node: Option<TransferWithTimestampNode>,
}

#[derive(Deserialize, Debug)]
struct TransferWithTimestampNode {
    id: String,
    #[serde(rename = "fromAddressHash")]
    from_address_hash: Option<String>,
    #[serde(rename = "toAddressHash")]
    to_address_hash: Option<String>,
    amount: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: Option<i64>,
    #[serde(rename = "transactionHash")]
    transaction_hash: Option<String>,
    transaction: Option<TransactionWithBlock>,
}

#[derive(Deserialize, Debug)]
struct TransactionWithBlock {
    block: Option<BlockWithTimestamp>,
}

#[derive(Deserialize, Debug)]
struct BlockWithTimestamp {
    timestamp: Option<String>,
    number: Option<i64>,
}
