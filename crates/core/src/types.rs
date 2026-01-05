use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Protocol-wide metrics snapshot
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProtocolMetrics {
    pub total_supply: Decimal,
    pub circulating_supply: Decimal,
    pub total_collateral: Decimal,
    pub active_troves: u64,
    pub tcr: Decimal,
    pub stability_pool_balance: Decimal,
    pub treasury_balance: Decimal,
}

impl Default for ProtocolMetrics {
    fn default() -> Self {
        Self {
            total_supply: Decimal::ZERO,
            circulating_supply: Decimal::ZERO,
            total_collateral: Decimal::ZERO,
            active_troves: 0,
            tcr: Decimal::ZERO,
            stability_pool_balance: Decimal::ZERO,
            treasury_balance: Decimal::ZERO,
        }
    }
}

/// Transaction record
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub hash: String,
    pub tx_type: TransactionType,
    pub amount: Decimal,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub block: u64,
    pub status: TransactionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum TransactionType {
    Mint,
    Burn,
    Transfer,
    Deposit,
    Withdraw,
    Liquidation,
    Redemption,
}

impl TransactionType {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mint => "Mint",
            Self::Burn => "Burn",
            Self::Transfer => "Transfer",
            Self::Deposit => "Deposit",
            Self::Withdraw => "Withdraw",
            Self::Liquidation => "Liquidation",
            Self::Redemption => "Redemption",
        }
    }
    
    #[inline]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Mint => "type-badge mint",
            Self::Burn => "type-badge burn",
            Self::Transfer => "type-badge transfer",
            Self::Deposit => "type-badge deposit",
            Self::Withdraw => "type-badge withdraw",
            Self::Liquidation => "type-badge liquidation",
            Self::Redemption => "type-badge redemption",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
}

impl TransactionStatus {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Success => "Success",
            Self::Failed => "Failed",
        }
    }
    
    #[inline]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Pending => "status-badge pending",
            Self::Success => "status-badge success",
            Self::Failed => "status-badge failed",
        }
    }
}

/// Trove (collateralized debt position)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trove {
    pub address: String,
    pub collateral: Decimal,
    pub debt: Decimal,
    pub icr: Decimal,
    pub status: TroveStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum TroveStatus {
    Active,
    AtRisk,
    Critical,
    Closed,
}

impl TroveStatus {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::AtRisk => "At Risk",
            Self::Critical => "Critical",
            Self::Closed => "Closed",
        }
    }
    
    #[inline]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Active => "status-badge success",
            Self::AtRisk => "status-badge warning",
            Self::Critical => "status-badge danger",
            Self::Closed => "status-badge muted",
        }
    }
    
    /// Calculate status from ICR percentage
    #[inline]
    pub fn from_icr(icr: Decimal) -> Self {
        use rust_decimal::prelude::ToPrimitive;
        let icr_f64 = icr.to_f64().unwrap_or(0.0);
        if icr_f64 >= 150.0 {
            Self::Active
        } else if icr_f64 >= 125.0 {
            Self::AtRisk
        } else if icr_f64 >= 110.0 {
            Self::Critical
        } else {
            Self::Closed
        }
    }
}

/// Stability pool depositor
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StabilityDepositor {
    pub address: String,
    pub deposit: Decimal,
    pub share: Decimal,
    pub rewards: Decimal,
}

/// Smart contract info
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    pub name: String,
    pub address: String,
    pub contract_type: ContractType,
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum ContractType {
    Token,
    Core,
    Oracle,
    Governance,
    Utility,
}

impl ContractType {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Token => "ERC20",
            Self::Core => "Core",
            Self::Oracle => "Oracle",
            Self::Governance => "Governance",
            Self::Utility => "Utility",
        }
    }
}

/// Protocol entity (addresses with known roles)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub address: String,
    pub balance: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum EntityType {
    Protocol,
    Whale,
    Exchange,
    Contract,
}

impl EntityType {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Protocol => "Protocol",
            Self::Whale => "Whale",
            Self::Exchange => "Exchange",
            Self::Contract => "Contract",
        }
    }
}

/// API endpoint documentation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiEndpoint {
    pub method: HttpMethod,
    pub path: String,
    pub description: String,
    pub params: Vec<ApiParam>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
    
    #[inline]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Get => "method-badge get",
            Self::Post => "method-badge post",
            Self::Put => "method-badge put",
            Self::Delete => "method-badge delete",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiParam {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Alert rule configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub threshold: Decimal,
    pub enabled: bool,
    pub notifications: Vec<NotificationChannel>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum AlertCondition {
    TcrBelow,
    TcrAbove,
    LargeMint,
    LargeBurn,
    StabilityPoolDrain,
    PriceDeviation,
}

impl AlertCondition {
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::TcrBelow => "TCR < threshold",
            Self::TcrAbove => "TCR > threshold",
            Self::LargeMint => "Mint amount > threshold",
            Self::LargeBurn => "Burn amount > threshold",
            Self::StabilityPoolDrain => "SP balance decrease > threshold%",
            Self::PriceDeviation => "Price deviation > threshold%",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum NotificationChannel {
    Email,
    Webhook,
    Telegram,
    Discord,
}

impl NotificationChannel {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Webhook => "webhook",
            Self::Telegram => "telegram",
            Self::Discord => "discord",
        }
    }
}

/// Chart data point
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
}

impl ChartDataPoint {
    #[inline]
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self { label: label.into(), value }
    }
}

/// Network visualization node
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy)]
pub enum NodeType {
    Token,
    Contract,
    Oracle,
    User,
}

impl NodeType {
    #[inline]
    pub fn color(&self) -> &'static str {
        match self {
            Self::Token => "#00d4ff",
            Self::Contract => "#00ff88",
            Self::Oracle => "#ffd000",
            Self::User => "#ff6b6b",
        }
    }
}

/// Network visualization edge
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkEdge {
    pub from: String,
    pub to: String,
    pub value: f64,
}

/// Block info for footer
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockInfo {
    pub number: u64,
    pub timestamp: u64,
    pub gas_price: f64,
}

impl Default for BlockInfo {
    fn default() -> Self {
        Self {
            number: 0,
            timestamp: 0,
            gas_price: 0.0,
        }
    }
}

// ============================================================================
// TradingView-Style Chart Types
// ============================================================================

/// TradingView-compatible chart resolution
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ChartResolution {
    M1,   // 1 minute
    M5,   // 5 minutes
    M15,  // 15 minutes
    M30,  // 30 minutes
    #[default]
    H1,   // 1 hour
    H4,   // 4 hours
    H12,  // 12 hours
    D1,   // 1 day
    W1,   // 1 week
}

impl ChartResolution {
    /// Parse from URL query param (e.g., "1m", "5m", "1h", "1d")
    pub fn from_url_param(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "1m" => Some(Self::M1),
            "5m" => Some(Self::M5),
            "15m" => Some(Self::M15),
            "30m" => Some(Self::M30),
            "1h" => Some(Self::H1),
            "4h" => Some(Self::H4),
            "12h" => Some(Self::H12),
            "1d" => Some(Self::D1),
            "1w" => Some(Self::W1),
            _ => None,
        }
    }

    /// Convert to URL query param
    #[inline]
    pub fn to_url_param(&self) -> &'static str {
        self.label()
    }

    /// TradingView code (e.g., "1", "5", "15", "60", "240", "720", "D", "W")
    #[inline]
    pub fn tv_code(&self) -> &'static str {
        match self {
            Self::M1 => "1",
            Self::M5 => "5",
            Self::M15 => "15",
            Self::M30 => "30",
            Self::H1 => "60",
            Self::H4 => "240",
            Self::H12 => "720",
            Self::D1 => "D",
            Self::W1 => "W",
        }
    }

    /// Duration in minutes
    #[inline]
    pub fn minutes(&self) -> u32 {
        match self {
            Self::M1 => 1,
            Self::M5 => 5,
            Self::M15 => 15,
            Self::M30 => 30,
            Self::H1 => 60,
            Self::H4 => 240,
            Self::H12 => 720,
            Self::D1 => 1440,
            Self::W1 => 10080,
        }
    }

    /// Duration in seconds
    #[inline]
    pub fn seconds(&self) -> i64 {
        self.minutes() as i64 * 60
    }

    /// Display label
    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::M1 => "1m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H4 => "4h",
            Self::H12 => "12h",
            Self::D1 => "1d",
            Self::W1 => "1w",
        }
    }

    /// GeckoTerminal API parameters (timeframe, aggregate, limit)
    #[inline]
    pub fn gecko_params(&self) -> (&'static str, u32, u32) {
        match self {
            Self::M1 => ("minute", 1, 100),
            Self::M5 => ("minute", 5, 100),
            Self::M15 => ("minute", 15, 100),
            Self::M30 => ("minute", 30, 100),
            Self::H1 => ("hour", 1, 168),
            Self::H4 => ("hour", 4, 180),
            Self::H12 => ("hour", 12, 60),
            Self::D1 => ("day", 1, 100),
            Self::W1 => ("day", 7, 52),
        }
    }

    /// Get maximum safe lookback for this resolution based on API limits
    /// Returns the maximum number of minutes that can be safely fetched
    /// without exceeding GeckoTerminal API candle limits
    #[inline]
    pub fn max_safe_lookback_mins(&self) -> u32 {
        let (_, _, limit) = self.gecko_params();
        limit * self.minutes()  // candles × minutes_per_candle
    }

    /// Check if a lookback period is safe for this resolution
    #[inline]
    pub fn is_lookback_safe(&self, lookback_mins: u32) -> bool {
        lookback_mins <= self.max_safe_lookback_mins()
    }

    /// Get human-readable safe lookback description
    pub fn safe_lookback_description(&self) -> &'static str {
        match self {
            Self::M1 => "~1.7 hours",
            Self::M5 => "~8 hours",
            Self::M15 => "~1 day",
            Self::M30 => "~2 days",
            Self::H1 => "~1 week",
            Self::H4 => "~1 month",
            Self::H12 => "~1 month",
            Self::D1 => "~3 months",
            Self::W1 => "~1 year",
        }
    }

    /// All available resolutions
    pub fn all() -> &'static [ChartResolution] {
        &[
            Self::M1, Self::M5, Self::M15, Self::M30,
            Self::H1, Self::H4, Self::H12, Self::D1, Self::W1,
        ]
    }
}

/// Chart lookback period for real-time data
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ChartLookback {
    Hour1,   // 1 hour
    Hour4,   // 4 hours
    Hour12,  // 12 hours
    Day1,    // 1 day
    Day3,    // 3 days
    #[default]
    Week1,   // 1 week
    Week2,   // 2 weeks
    Month1,  // 1 month
    Month3,  // 3 months
    All,     // No limit
}

impl ChartLookback {
    /// Parse from URL query param (e.g., "1h", "1d", "1w", "1m")
    pub fn from_url_param(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "1h" => Some(Self::Hour1),
            "4h" => Some(Self::Hour4),
            "12h" => Some(Self::Hour12),
            "1d" => Some(Self::Day1),
            "3d" => Some(Self::Day3),
            "1w" => Some(Self::Week1),
            "2w" => Some(Self::Week2),
            "1m" => Some(Self::Month1),
            "3m" => Some(Self::Month3),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// Convert to URL query param
    #[inline]
    pub fn to_url_param(&self) -> &'static str {
        match self {
            Self::Hour1 => "1h",
            Self::Hour4 => "4h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
            Self::Week2 => "2w",
            Self::Month1 => "1m",
            Self::Month3 => "3m",
            Self::All => "all",
        }
    }

    /// Display label
    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Hour1 => "1h",
            Self::Hour4 => "4h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
            Self::Week2 => "2w",
            Self::Month1 => "1m",
            Self::Month3 => "3m",
            Self::All => "ALL",
        }
    }

    /// Duration in minutes (capped at 30 days for performance)
    #[inline]
    pub fn minutes(&self) -> u32 {
        match self {
            Self::Hour1 => 60,
            Self::Hour4 => 240,
            Self::Hour12 => 720,
            Self::Day1 => 1440,
            Self::Day3 => 4320,
            Self::Week1 => 10080,
            Self::Week2 => 20160,
            Self::Month1 => 43200,
            Self::Month3 => 129600,
            Self::All => 43200,  // Cap at 30 days (same as Month1) for performance
        }
    }

    /// Calculate cutoff timestamp (returns None for All)
    /// Note: Only available in SSR mode where std::time is available
    #[cfg(feature = "ssr")]
    pub fn cutoff_timestamp(&self) -> Option<i64> {
        let mins = self.minutes();
        if mins == 0 {
            return None;
        }
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Some(now - (mins as i64 * 60))
    }

    /// Calculate cutoff given a current timestamp (works on all platforms)
    pub fn cutoff_from(&self, now_timestamp: i64) -> Option<i64> {
        let mins = self.minutes();
        if mins == 0 {
            return None;
        }
        Some(now_timestamp - (mins as i64 * 60))
    }

    /// All available lookback periods
    pub fn all() -> &'static [ChartLookback] {
        &[
            Self::Hour1, Self::Hour4, Self::Hour12, Self::Day1,
            Self::Day3, Self::Week1, Self::Week2, Self::Month1,
            Self::Month3, Self::All,
        ]
    }
}

/// TradingView-compatible OHLCV candle
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TVCandle {
    pub time: i64,      // Unix timestamp (seconds)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl TVCandle {
    #[inline]
    pub fn new(time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self { time, open, high, low, close, volume }
    }

    /// Create from a single price point
    #[inline]
    pub fn from_price(time: i64, price: f64, volume: f64) -> Self {
        Self { time, open: price, high: price, low: price, close: price, volume }
    }

    /// Check if candle is bullish (close >= open)
    #[inline]
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    /// Get candle body size
    #[inline]
    pub fn body(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Get candle range (high - low)
    #[inline]
    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

/// Balance OHLC candle for wallet holdings chart
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BalanceCandle {
    pub time: i64,          // Unix timestamp (seconds)
    pub open: f64,          // Balance at candle open
    pub high: f64,          // Max balance in period
    pub low: f64,           // Min balance in period
    pub close: f64,         // Balance at candle close
    pub volume: f64,        // Total transfer volume
    pub tx_count: u32,      // Number of transactions
    pub net_change: f64,    // Net balance change
}

impl BalanceCandle {
    #[inline]
    pub fn new(time: i64, balance: f64) -> Self {
        Self {
            time,
            open: balance,
            high: balance,
            low: balance,
            close: balance,
            volume: 0.0,
            tx_count: 0,
            net_change: 0.0,
        }
    }

    /// Update candle with a new transaction
    pub fn update(&mut self, new_balance: f64, tx_amount: f64) {
        self.high = self.high.max(new_balance);
        self.low = self.low.min(new_balance);
        self.close = new_balance;
        self.volume += tx_amount.abs();
        self.tx_count += 1;
        self.net_change = self.close - self.open;
    }
}

/// Lending/Borrowing volume candle
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolumeCandle {
    pub time: i64,
    pub lend_volume: f64,
    pub borrow_volume: f64,
    pub lend_count: u32,
    pub borrow_count: u32,
    pub net_flow: f64,
    pub total_volume: f64,
}

impl VolumeCandle {
    #[inline]
    pub fn new(time: i64) -> Self {
        Self {
            time,
            lend_volume: 0.0,
            borrow_volume: 0.0,
            lend_count: 0,
            borrow_count: 0,
            net_flow: 0.0,
            total_volume: 0.0,
        }
    }

    /// Add a lend transaction
    pub fn add_lend(&mut self, amount: f64) {
        self.lend_volume += amount;
        self.lend_count += 1;
        self.net_flow += amount;
        self.total_volume += amount;
    }

    /// Add a borrow transaction
    pub fn add_borrow(&mut self, amount: f64) {
        self.borrow_volume += amount;
        self.borrow_count += 1;
        self.net_flow -= amount;
        self.total_volume += amount;
    }
}

/// Operation type for chart markers
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    // Trove Operations
    OpenTrove,
    AdjustTrove,
    CloseTrove,
    ClaimCollateral,

    // Stability Pool
    ProvideSP,
    WithdrawSP,

    // Lending (Secured Finance)
    Lend,
    Borrow,

    // DEX/Bridge
    Swap,
    Bridge,

    // Token
    Approve,
    Transfer,
    Mint,
    Redeem,

    // Risk
    Liquidate,

    Unknown,
}

impl OperationType {
    /// Short label for chart display
    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::OpenTrove => "Open",
            Self::AdjustTrove => "Adjust",
            Self::CloseTrove => "Close",
            Self::ClaimCollateral => "Claim",
            Self::ProvideSP => "SP+",
            Self::WithdrawSP => "SP-",
            Self::Lend => "Lend",
            Self::Borrow => "Borrow",
            Self::Swap => "Swap",
            Self::Bridge => "Bridge",
            Self::Approve => "Approve",
            Self::Transfer => "Transfer",
            Self::Mint => "Mint",
            Self::Redeem => "Redeem",
            Self::Liquidate => "Liq!",
            Self::Unknown => "?",
        }
    }

    /// Full description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::OpenTrove => "Open Trove",
            Self::AdjustTrove => "Adjust Trove",
            Self::CloseTrove => "Close Trove",
            Self::ClaimCollateral => "Claim Collateral",
            Self::ProvideSP => "Provide to Stability Pool",
            Self::WithdrawSP => "Withdraw from Stability Pool",
            Self::Lend => "Lend Order",
            Self::Borrow => "Borrow Order",
            Self::Swap => "Token Swap",
            Self::Bridge => "Bridge Transfer",
            Self::Approve => "Token Approval",
            Self::Transfer => "Token Transfer",
            Self::Mint => "Mint USDFC",
            Self::Redeem => "Redeem USDFC",
            Self::Liquidate => "Liquidation",
            Self::Unknown => "Unknown Operation",
        }
    }

    /// Hex color for marker
    #[inline]
    pub fn color(&self) -> &'static str {
        match self {
            Self::OpenTrove | Self::ProvideSP | Self::Lend | Self::Mint => "#22c55e",     // Green
            Self::CloseTrove | Self::WithdrawSP | Self::Redeem => "#ef4444",              // Red
            Self::AdjustTrove => "#3b82f6",                                                // Blue
            Self::ClaimCollateral => "#f59e0b",                                            // Amber
            Self::Swap => "#8b5cf6",                                                       // Purple
            Self::Bridge => "#06b6d4",                                                     // Cyan
            Self::Borrow => "#f97316",                                                     // Orange
            Self::Liquidate => "#dc2626",                                                  // Dark Red
            Self::Approve | Self::Transfer | Self::Unknown => "#6b7280",                   // Gray
        }
    }

    /// CSS class for styling
    #[inline]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::OpenTrove | Self::ProvideSP | Self::Lend | Self::Mint => "op-green",
            Self::CloseTrove | Self::WithdrawSP | Self::Redeem => "op-red",
            Self::AdjustTrove => "op-blue",
            Self::ClaimCollateral => "op-amber",
            Self::Swap => "op-purple",
            Self::Bridge => "op-cyan",
            Self::Borrow => "op-orange",
            Self::Liquidate => "op-danger",
            Self::Approve | Self::Transfer | Self::Unknown => "op-gray",
        }
    }

    /// Detect operation type from method name
    pub fn from_method(method: &str) -> Self {
        let m = method.to_lowercase();
        if m.contains("opentrove") {
            Self::OpenTrove
        } else if m.contains("adjusttrove") {
            Self::AdjustTrove
        } else if m.contains("closetrove") {
            Self::CloseTrove
        } else if m.contains("claimcollateral") {
            Self::ClaimCollateral
        } else if m.contains("providetosp") || m.contains("providetostabilitypool") {
            Self::ProvideSP
        } else if m.contains("withdrawfromsp") || m.contains("withdrawfromstabilitypool") {
            Self::WithdrawSP
        } else if m.contains("swap") || m.contains("snwap") {
            Self::Swap
        } else if m.contains("bridge") {
            Self::Bridge
        } else if m.contains("approve") {
            Self::Approve
        } else if m.contains("transfer") {
            Self::Transfer
        } else if m.contains("mint") {
            Self::Mint
        } else if m.contains("redeem") {
            Self::Redeem
        } else if m.contains("liquidate") {
            Self::Liquidate
        } else if m.contains("lend") || m.contains("deposit") {
            Self::Lend
        } else if m.contains("borrow") {
            Self::Borrow
        } else {
            Self::Unknown
        }
    }
}

/// Operation marker for chart overlay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OperationMarker {
    pub time: i64,
    pub operation: OperationType,
    pub amount: f64,
    pub tx_hash: String,
    pub label: String,
    pub color: String,
}

impl OperationMarker {
    pub fn new(time: i64, operation: OperationType, amount: f64, tx_hash: String) -> Self {
        Self {
            time,
            label: operation.label().to_string(),
            color: operation.color().to_string(),
            operation,
            amount,
            tx_hash,
        }
    }
}

/// Primary chart metric selection
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ChartMetric {
    #[default]
    Price,
    Volume,
    Liquidity,
    TCR,
    Supply,
    Holders,
    LendAPR,
    BorrowAPR,
    Transfers,
}

impl ChartMetric {
    /// Parse from URL query param (e.g., "price", "volume", "tcr")
    pub fn from_url_param(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "price" => Some(Self::Price),
            "volume" => Some(Self::Volume),
            "liquidity" => Some(Self::Liquidity),
            "tcr" => Some(Self::TCR),
            "supply" => Some(Self::Supply),
            "holders" => Some(Self::Holders),
            "lendapr" | "lend_apr" => Some(Self::LendAPR),
            "borrowapr" | "borrow_apr" => Some(Self::BorrowAPR),
            "transfers" => Some(Self::Transfers),
            _ => None,
        }
    }

    /// Convert to URL query param
    #[inline]
    pub fn to_url_param(&self) -> &'static str {
        match self {
            Self::Price => "price",
            Self::Volume => "volume",
            Self::Liquidity => "liquidity",
            Self::TCR => "tcr",
            Self::Supply => "supply",
            Self::Holders => "holders",
            Self::LendAPR => "lendapr",
            Self::BorrowAPR => "borrowapr",
            Self::Transfers => "transfers",
        }
    }

    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Price => "Price",
            Self::Volume => "Volume",
            Self::Liquidity => "Liquidity",
            Self::TCR => "TCR",
            Self::Supply => "Supply",
            Self::Holders => "Holders",
            Self::LendAPR => "Lend APR",
            Self::BorrowAPR => "Borrow APR",
            Self::Transfers => "Transfers",
        }
    }

    #[inline]
    pub fn unit(&self) -> &'static str {
        match self {
            Self::Price => "USD",
            Self::Volume | Self::Liquidity => "USD",
            Self::TCR | Self::LendAPR | Self::BorrowAPR => "%",
            Self::Supply => "USDFC",
            Self::Holders | Self::Transfers => "",
        }
    }

    #[inline]
    pub fn color(&self) -> &'static str {
        match self {
            Self::Price => "#00d4ff",
            Self::Volume => "#8b5cf6",
            Self::Liquidity => "#06b6d4",
            Self::TCR => "#22c55e",
            Self::Supply => "#f59e0b",
            Self::Holders => "#ec4899",
            Self::LendAPR => "#10b981",
            Self::BorrowAPR => "#f97316",
            Self::Transfers => "#6366f1",
        }
    }

    pub fn all() -> &'static [ChartMetric] {
        &[
            Self::Price, Self::Volume, Self::Liquidity, Self::TCR,
            Self::Supply, Self::Holders, Self::LendAPR, Self::BorrowAPR,
            Self::Transfers,
        ]
    }
}

/// Chart rendering type
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    Area,
    Line,
    Candle,
    Bar,
}

impl ChartType {
    /// Parse from URL query param (e.g., "area", "line", "candle")
    pub fn from_url_param(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "area" => Some(Self::Area),
            "line" => Some(Self::Line),
            "candle" | "candlestick" => Some(Self::Candle),
            "bar" => Some(Self::Bar),
            _ => None,
        }
    }

    /// Convert to URL query param
    #[inline]
    pub fn to_url_param(&self) -> &'static str {
        match self {
            Self::Area => "area",
            Self::Line => "line",
            Self::Candle => "candle",
            Self::Bar => "bar",
        }
    }

    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Area => "Area",
            Self::Line => "Line",
            Self::Candle => "Candle",
            Self::Bar => "Bar",
        }
    }
}

/// Aggregated chart data response with all metrics
/// SAFETY: Critical metrics use Option<f64> - None means unavailable (not fallback values)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChartDataResponse {
    pub resolution: ChartResolution,
    pub lookback: ChartLookback,
    pub generated_at: i64,
    pub fetch_time_ms: u32,
    // Price OHLCV candles from GeckoTerminal
    pub price_candles: Vec<TVCandle>,
    // Volume data - extracted from candles for separate rendering
    pub volume_data: Vec<(i64, f64)>,
    // Liquidity data from GeckoTerminal pool info
    pub liquidity_data: Vec<(i64, f64)>,
    // TCR data from RPC
    pub tcr_data: Vec<(i64, f64)>,
    // Supply data from RPC
    pub supply_data: Vec<(i64, f64)>,
    // Holder count from Blockscout
    pub holders_data: Vec<(i64, u64)>,
    // Lending APR from Secured Finance Subgraph
    pub lend_apr_data: Vec<(i64, f64)>,
    // Borrowing APR from Secured Finance Subgraph
    pub borrow_apr_data: Vec<(i64, f64)>,
    // Transfer count from Blockscout
    pub transfers_data: Vec<(i64, u64)>,
    // Current values for display - Option means unavailable, NOT fake fallbacks
    /// Current price - None if API failed (NEVER use 1.0 fallback - masks depegging)
    pub current_price: Option<f64>,
    pub current_volume_24h: Option<f64>,
    pub current_liquidity: Option<f64>,
    /// TCR - None if RPC failed (NEVER use 0.0 fallback - masks insolvency)
    pub current_tcr: Option<f64>,
    pub current_supply: Option<f64>,
    pub current_holders: Option<u64>,
    pub current_lend_apr: Option<f64>,
    pub current_borrow_apr: Option<f64>,
    // Metadata for progressive enhancement
    pub snapshot_count: usize,
    pub oldest_snapshot_time: Option<i64>,
}

impl Default for ChartDataResponse {
    fn default() -> Self {
        Self {
            resolution: ChartResolution::default(),
            lookback: ChartLookback::default(),
            generated_at: 0,
            fetch_time_ms: 0,
            price_candles: Vec::new(),
            volume_data: Vec::new(),
            liquidity_data: Vec::new(),
            tcr_data: Vec::new(),
            supply_data: Vec::new(),
            holders_data: Vec::new(),
            lend_apr_data: Vec::new(),
            borrow_apr_data: Vec::new(),
            transfers_data: Vec::new(),
            // SAFETY: All None - no fake fallback values that mask real issues
            current_price: None,
            current_volume_24h: None,
            current_liquidity: None,
            current_tcr: None,
            current_supply: None,
            current_holders: None,
            current_lend_apr: None,
            current_borrow_apr: None,
            snapshot_count: 0,
            oldest_snapshot_time: None,
        }
    }
}

/// Wallet-specific chart data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletChartData {
    pub address: String,
    pub resolution: ChartResolution,
    pub lookback: ChartLookback,
    pub generated_at: i64,
    pub fetch_time_ms: u32,
    pub balance_chart: BalanceChartData,
    pub lending_chart: LendingChartData,
    pub operations: OperationsData,
}

impl Default for WalletChartData {
    fn default() -> Self {
        Self {
            address: String::new(),
            resolution: ChartResolution::default(),
            lookback: ChartLookback::default(),
            generated_at: 0,
            fetch_time_ms: 0,
            balance_chart: BalanceChartData::default(),
            lending_chart: LendingChartData::default(),
            operations: OperationsData::default(),
        }
    }
}

/// Balance chart data section
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BalanceChartData {
    pub candles: Vec<BalanceCandle>,
    pub data_points: usize,
    pub latest_balance: f64,
    pub max_balance: f64,
    pub min_balance: f64,
}

/// Lending chart data section
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct LendingChartData {
    pub candles: Vec<VolumeCandle>,
    pub data_points: usize,
    pub total_lend: f64,
    pub total_borrow: f64,
    pub avg_lend_apr: f64,
    pub avg_borrow_apr: f64,
}

/// Operations data section
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct OperationsData {
    pub markers: Vec<OperationMarker>,
    pub count: usize,
    pub breakdown: std::collections::HashMap<String, usize>,
}
