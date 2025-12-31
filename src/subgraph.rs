use crate::config::config;
use crate::error::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Subgraph GraphQL client
#[derive(Clone)]
pub struct SubgraphClient {
    client: reqwest::Client,
    url: String,
}

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
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    message: String,
}

// Lending Market types
#[derive(Deserialize, Debug, Clone)]
pub struct LendingMarket {
    pub id: String,
    pub currency: String,
    pub maturity: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "lastLendUnitPrice")]
    pub last_lend_unit_price: Option<String>,
    #[serde(rename = "lastBorrowUnitPrice")]
    pub last_borrow_unit_price: Option<String>,
    pub volume: Option<String>,
}

#[derive(Deserialize, Debug)]
struct LendingMarketsData {
    #[serde(rename = "lendingMarkets")]
    lending_markets: Vec<LendingMarket>,
}

// Order types
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Order {
    pub id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub side: i32, // 0 = LEND, 1 = BORROW
    pub currency: String,
    pub maturity: String,
    #[serde(rename = "inputAmount")]
    pub input_amount: String,
    #[serde(rename = "filledAmount")]
    pub filled_amount: String,
    #[serde(rename = "inputUnitPrice")]
    pub input_unit_price: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OrdersData {
    orders: Vec<Order>,
}

// Transaction types
#[derive(Deserialize, Debug, Clone)]
pub struct SubgraphTransaction {
    pub id: String,
    pub currency: String,
    pub maturity: String,
    pub side: i32,
    pub amount: String,
    #[serde(rename = "executionPrice")]
    pub execution_price: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Deserialize, Debug)]
struct TransactionsData {
    transactions: Vec<SubgraphTransaction>,
}

// Daily Volume types for historical data
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DailyVolume {
    pub id: String,
    pub currency: String,
    pub maturity: String,
    pub day: String,
    pub volume: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug)]
struct DailyVolumesData {
    #[serde(rename = "dailyVolumes")]
    daily_volumes: Vec<DailyVolume>,
}

// Transaction Candlestick for OHLC data
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TransactionCandleStick {
    pub id: String,
    pub interval: String,
    pub currency: String,
    pub maturity: String,
    pub timestamp: String,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String,
    pub average: String,
    pub volume: String,
    #[serde(rename = "volumeInFV")]
    pub volume_in_fv: String,
}

#[derive(Deserialize, Debug)]
struct CandleSticksData {
    #[serde(rename = "transactionCandleSticks")]
    candlesticks: Vec<TransactionCandleStick>,
}

impl SubgraphClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build subgraph HTTP client"),
            url: config().subgraph_url.clone(),
        }
    }

    /// Execute a GraphQL query
    async fn query<T: for<'de> Deserialize<'de>>(&self, query: String) -> ApiResult<T> {
        let request = GraphQLRequest {
            query,
            variables: None,
        };

        let response = self
            .client
            .post(&self.url)
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

    /// Get lending markets (yield curve data)
    pub async fn get_lending_markets(&self) -> ApiResult<Vec<LendingMarket>> {
        let query = r#"
            query {
                lendingMarkets(
                    first: 50
                    orderBy: maturity
                    orderDirection: asc
                ) {
                    id
                    currency
                    maturity
                    isActive
                    lastLendUnitPrice
                    lastBorrowUnitPrice
                    volume
                }
            }
        "#.to_string();

        let data: LendingMarketsData = self.query(query).await?;
        Ok(data.lending_markets)
    }

    /// Get USDFC order book
    pub async fn get_usdfc_orders(&self, limit: i32) -> ApiResult<Vec<Order>> {
        let query = format!(
            r#"
            query {{
                orders(
                    first: {}
                    where: {{
                        currency: "{}"
                        status: "Open"
                    }}
                    orderBy: inputUnitPrice
                    orderDirection: desc
                ) {{
                    id
                    orderId
                    side
                    currency
                    maturity
                    inputAmount
                    filledAmount
                    inputUnitPrice
                    status
                    createdAt
                }}
            }}
        "#,
            limit,
            config().currency_usdfc
        );

        let data: OrdersData = self.query(query).await?;
        Ok(data.orders)
    }

    /// Get recent transactions
    pub async fn get_recent_transactions(&self, limit: i32) -> ApiResult<Vec<SubgraphTransaction>> {
        let query = format!(
            r#"
            query {{
                transactions(
                    first: {}
                    orderBy: createdAt
                    orderDirection: desc
                ) {{
                    id
                    currency
                    maturity
                    side
                    amount
                    executionPrice
                    createdAt
                }}
            }}
        "#,
            limit
        );

        let data: TransactionsData = self.query(query).await?;
        Ok(data.transactions)
    }

    /// Get daily volume data for historical charts
    pub async fn get_daily_volumes(&self, days: i32) -> ApiResult<Vec<DailyVolume>> {
        let query = format!(
            r#"
            query {{
                dailyVolumes(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                ) {{
                    id
                    currency
                    maturity
                    day
                    volume
                    timestamp
                }}
            }}
        "#,
            days
        );

        let data: DailyVolumesData = self.query(query).await?;
        Ok(data.daily_volumes)
    }

    /// Get OHLC candlestick data for price charts
    pub async fn get_candlesticks(&self, currency: &str, limit: i32) -> ApiResult<Vec<TransactionCandleStick>> {
        let query = format!(
            r#"
            query {{
                transactionCandleSticks(
                    first: {}
                    where: {{ currency: "{}" }}
                    orderBy: timestamp
                    orderDirection: desc
                ) {{
                    id
                    interval
                    currency
                    maturity
                    timestamp
                    open
                    close
                    high
                    low
                    average
                    volume
                    volumeInFV
                }}
            }}
        "#,
            limit, currency
        );

        let data: CandleSticksData = self.query(query).await?;
        Ok(data.candlesticks)
    }

    /// Get order book grouped by side (lend=0, borrow=1)
    pub async fn get_order_book(&self, currency: &str, maturity: Option<&str>, limit: i32) -> ApiResult<OrderBook> {
        let maturity_clause = maturity
            .map(|m| format!(r#", maturity: "{}""#, m))
            .unwrap_or_default();

        let query = format!(
            r#"
            query {{
                orders(
                    first: {}
                    where: {{
                        currency: "{}"
                        status: "Open"
                        {}
                    }}
                    orderBy: inputUnitPrice
                    orderDirection: desc
                ) {{
                    id
                    orderId
                    side
                    currency
                    maturity
                    inputAmount
                    filledAmount
                    inputUnitPrice
                    status
                    createdAt
                    user
                }}
            }}
        "#,
            limit, currency, maturity_clause
        );

        let data: OrdersData = self.query(query).await?;

        // Separate into lend (side=0) and borrow (side=1) orders
        let mut lend_orders = Vec::new();
        let mut borrow_orders = Vec::new();

        for order in data.orders {
            if order.side == 0 {
                lend_orders.push(order);
            } else {
                borrow_orders.push(order);
            }
        }

        // Lend orders (bids) sorted by price descending (best bid first)
        lend_orders.sort_by(|a, b| {
            let price_a = a.input_unit_price.parse::<i64>().unwrap_or(0);
            let price_b = b.input_unit_price.parse::<i64>().unwrap_or(0);
            price_b.cmp(&price_a)
        });

        // Borrow orders (asks) sorted by price ascending (best ask first)
        borrow_orders.sort_by(|a, b| {
            let price_a = a.input_unit_price.parse::<i64>().unwrap_or(0);
            let price_b = b.input_unit_price.parse::<i64>().unwrap_or(0);
            price_a.cmp(&price_b)
        });

        Ok(OrderBook {
            currency: currency.to_string(),
            maturity: maturity.map(|s| s.to_string()),
            lend_orders,
            borrow_orders,
        })
    }
}

/// Order book with lend and borrow sides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub currency: String,
    pub maturity: Option<String>,
    pub lend_orders: Vec<Order>,
    pub borrow_orders: Vec<Order>,
}

impl Default for SubgraphClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert unit price (basis points) to APR
/// This function is SSR-only because it requires current system time
/// Returns 0.0 for invalid/edge cases instead of erroring
#[cfg(feature = "ssr")]
pub fn unit_price_to_apr(unit_price: &str, maturity_timestamp: i64) -> ApiResult<f64> {
    let price = unit_price.parse::<f64>().unwrap_or(0.0);

    // Handle edge cases gracefully
    if price <= 0.0 || price > 10000.0 {
        return Ok(0.0); // Return 0% APR for invalid prices
    }

    let bond_price = price / 10000.0;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let days_to_maturity = ((maturity_timestamp - now) / 86400).max(1);

    let discount = (1.0 / bond_price) - 1.0;
    Ok((discount * 365.0 / days_to_maturity as f64 * 100.0).max(0.0))
}

/// Non-SSR stub that returns 0 - APR must be pre-calculated server-side
#[cfg(not(feature = "ssr"))]
pub fn unit_price_to_apr(_unit_price: &str, _maturity_timestamp: i64) -> ApiResult<f64> {
    Ok(0.0) // APR is pre-calculated server-side
}

/// Decode currency bytes32 to string
pub fn decode_currency(bytes32: &str) -> String {
    if bytes32 == config().currency_usdfc {
        return "USDFC".to_string();
    }
    if bytes32 == config().currency_fil {
        return "FIL".to_string();
    }
    
    // Try to decode hex to ASCII
    let hex = bytes32.trim_start_matches("0x");
    let mut result = String::new();
    for i in (0..hex.len()).step_by(2) {
        if let Ok(byte) = u8::from_str_radix(&hex[i..i+2], 16) {
            if byte > 0 && byte < 128 {
                result.push(byte as char);
            }
        }
    }
    result.trim().to_string()
}
