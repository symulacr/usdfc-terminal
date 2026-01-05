//! Data source abstraction for API integration
//! 
//! This module provides traits for different data sources.
//! No mock data - all implementations should use real APIs.

use usdfc_core::error::ApiResult;
use usdfc_core::types::*;
use async_trait::async_trait;
use rust_decimal::Decimal;

/// Trait for data source implementations
#[async_trait(?Send)]
pub trait DataSource: Send + Sync {
    // Protocol metrics
    async fn get_protocol_metrics(&self) -> ApiResult<ProtocolMetrics>;
    
    // Supply data
    async fn get_total_supply(&self) -> ApiResult<Decimal>;
    async fn get_circulating_supply(&self) -> ApiResult<Decimal>;
    async fn get_treasury_supply(&self) -> ApiResult<Decimal>;
    async fn get_supply_history(&self) -> ApiResult<Vec<ChartDataPoint>>;
    
    // Collateral data
    async fn get_total_collateral(&self) -> ApiResult<Decimal>;
    async fn get_tcr(&self) -> ApiResult<Decimal>;
    async fn get_troves(&self, limit: Option<u32>, offset: Option<u32>) -> ApiResult<Vec<Trove>>;
    async fn get_icr_distribution(&self) -> ApiResult<Vec<ChartDataPoint>>;
    
    // Stability pool
    async fn get_stability_pool_balance(&self) -> ApiResult<Decimal>;
    async fn get_stability_depositors(&self, limit: Option<u32>) -> ApiResult<Vec<StabilityDepositor>>;
    
    // Transactions
    async fn get_recent_transactions(&self, limit: Option<u32>) -> ApiResult<Vec<Transaction>>;
    async fn get_transaction(&self, hash: &str) -> ApiResult<Transaction>;
    async fn search_transactions(&self, query: &TransactionQuery) -> ApiResult<Vec<Transaction>>;
    
    // Address lookup
    async fn get_address_balance(&self, address: &str) -> ApiResult<Decimal>;
    async fn get_address_transactions(&self, address: &str, limit: Option<u32>) -> ApiResult<Vec<Transaction>>;
    
    // Contracts
    async fn get_contracts(&self) -> ApiResult<Vec<Contract>>;
    async fn get_contract_abi(&self, address: &str) -> ApiResult<String>;
    
    // Entities
    async fn get_entities(&self) -> ApiResult<Vec<Entity>>;
    
    // Network graph
    async fn get_network_nodes(&self) -> ApiResult<Vec<NetworkNode>>;
    async fn get_network_edges(&self) -> ApiResult<Vec<NetworkEdge>>;
    
    // API documentation
    async fn get_api_endpoints(&self) -> ApiResult<Vec<ApiEndpoint>>;
    
    // Alerts
    async fn get_alert_rules(&self) -> ApiResult<Vec<AlertRule>>;
    async fn update_alert_rule(&self, rule: &AlertRule) -> ApiResult<()>;
    
    // Block info
    async fn get_current_block(&self) -> ApiResult<BlockInfo>;
}

/// Query parameters for transaction search
#[derive(Clone, Debug, Default)]
pub struct TransactionQuery {
    pub tx_type: Option<TransactionType>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
