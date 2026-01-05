use std::sync::OnceLock;

/// Global application configuration
#[derive(Clone, Debug)]
pub struct Config {
    // API Endpoints
    pub rpc_url: String,
    pub rpc_fallback_urls: Vec<String>,
    pub subgraph_url: String,
    pub blockscout_url: String,
    pub geckoterminal_url: String,
    
    // Contract Addresses
    pub usdfc_token: String,
    pub trove_manager: String,
    pub sorted_troves: String,
    pub price_feed: String,
    pub multi_trove_getter: String,
    pub stability_pool: String,
    pub active_pool: String,
    pub borrower_operations: String,
    
    // Currency Identifiers
    pub currency_usdfc: String,
    pub currency_fil: String,
    
    // DEX Pool Addresses (GeckoTerminal)
    pub pool_usdfc_wfil: String,
    pub pool_usdfc_axlusdc: String,
    pub pool_usdfc_usdc: String,
    
    // Server Config
    pub host: String,
    pub port: u16,
    
    // Refresh Intervals
    pub refresh_interval_fast: u64,
    pub refresh_interval_medium: u64,
    pub refresh_interval_slow: u64,

    // UI Thresholds
    pub tcr_danger_threshold: f64,
    pub tcr_warning_threshold: f64,
    pub whale_threshold_usd: f64,

    // Timing
    pub refresh_interval_ms: u64,
    pub history_retention_secs: u64,

    // RPC Settings
    pub rpc_timeout_secs: u64,
    pub rpc_retry_count: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.node.glif.io/rpc/v1".to_string(),
            rpc_fallback_urls: vec![
                "https://filecoin.chainup.net/rpc/v1".to_string(),
                "https://rpc.ankr.com/filecoin".to_string(),
            ],
            subgraph_url: "https://api.goldsky.com/api/public/project_cm8i6ca9k24d601wy45zzbsrq/subgraphs/sf-filecoin-mainnet/latest/gn".to_string(),
            blockscout_url: "https://filecoin.blockscout.com/api/v2".to_string(),
            geckoterminal_url: "https://api.geckoterminal.com/api/v2/networks/filecoin".to_string(),
            
            usdfc_token: "0x80B98d3aa09ffff255c3ba4A241111Ff1262F045".to_string(),
            trove_manager: "0x5aB87c2398454125Dd424425e39c8909bBE16022".to_string(),
            sorted_troves: "0x2C32e48e358d5b893C46906b69044D342d8DDd5F".to_string(),
            price_feed: "0x80e651c9739C1ed15A267c11b85361780164A368".to_string(),
            multi_trove_getter: "0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F".to_string(),
            stability_pool: "0x791Ad78bBc58324089D3E0A8689E7D045B9592b5".to_string(),
            active_pool: "0x8637Ac7FdBB4c763B72e26504aFb659df71c7803".to_string(),
            borrower_operations: "0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0".to_string(),
            
            currency_usdfc: "0x5553444643000000000000000000000000000000000000000000000000000000".to_string(),
            currency_fil: "0x46494c0000000000000000000000000000000000000000000000000000000000".to_string(),
            
            pool_usdfc_wfil: "0x4e07447bd38e60b94176764133788be1a0736b30".to_string(),
            pool_usdfc_axlusdc: "0x21ca72fe39095db9642ca9cc694fa056f906037f".to_string(),
            pool_usdfc_usdc: "0xc8f38dbaf661b897b6a2ee5721aac5a8766ffa13".to_string(),
            
            host: "127.0.0.1".to_string(),
            port: 3000,
            
            refresh_interval_fast: 30,
            refresh_interval_medium: 60,
            refresh_interval_slow: 300,

            // UI Thresholds - defaults
            tcr_danger_threshold: 150.0,
            tcr_warning_threshold: 200.0,
            whale_threshold_usd: 100000.0,

            // Timing - defaults
            refresh_interval_ms: 30000,
            history_retention_secs: 604800,

            // RPC Settings - defaults
            rpc_timeout_secs: 30,
            rpc_retry_count: 3,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        // Try to load .env file, but don't fail if it doesn't exist
        let _ = dotenvy::dotenv();
        
        Self {
            rpc_url: std::env::var("RPC_URL").expect("RPC_URL must be set"),
            rpc_fallback_urls: std::env::var("RPC_FALLBACK_URLS")
                .ok()
                .map(|s| s.split(',').map(|u| u.trim().to_string()).collect())
                .unwrap_or_else(|| vec![
                    "https://filecoin.chainup.net/rpc/v1".to_string(),
                    "https://rpc.ankr.com/filecoin".to_string(),
                ]),
            subgraph_url: std::env::var("SUBGRAPH_URL").expect("SUBGRAPH_URL must be set"),
            blockscout_url: std::env::var("BLOCKSCOUT_URL").expect("BLOCKSCOUT_URL must be set"),
            geckoterminal_url: std::env::var("GECKOTERMINAL_URL").expect("GECKOTERMINAL_URL must be set"),

            usdfc_token: std::env::var("USDFC_TOKEN").expect("USDFC_TOKEN must be set"),
            trove_manager: std::env::var("TROVE_MANAGER").expect("TROVE_MANAGER must be set"),
            sorted_troves: std::env::var("SORTED_TROVES").expect("SORTED_TROVES must be set"),
            price_feed: std::env::var("PRICE_FEED").expect("PRICE_FEED must be set"),
            multi_trove_getter: std::env::var("MULTI_TROVE_GETTER").expect("MULTI_TROVE_GETTER must be set"),
            stability_pool: std::env::var("STABILITY_POOL").expect("STABILITY_POOL must be set"),
            active_pool: std::env::var("ACTIVE_POOL").expect("ACTIVE_POOL must be set"),
            borrower_operations: std::env::var("BORROWER_OPERATIONS").expect("BORROWER_OPERATIONS must be set"),

            currency_usdfc: std::env::var("CURRENCY_USDFC").expect("CURRENCY_USDFC must be set"),
            currency_fil: std::env::var("CURRENCY_FIL").expect("CURRENCY_FIL must be set"),

            pool_usdfc_wfil: std::env::var("POOL_USDFC_WFIL").expect("POOL_USDFC_WFIL must be set"),
            pool_usdfc_axlusdc: std::env::var("POOL_USDFC_AXLUSDC").expect("POOL_USDFC_AXLUSDC must be set"),
            pool_usdfc_usdc: std::env::var("POOL_USDFC_USDC").expect("POOL_USDFC_USDC must be set"),

            host: std::env::var("HOST").expect("HOST must be set"),
            port: std::env::var("PORT")
                .expect("PORT must be set")
                .parse()
                .expect("PORT must be a valid integer"),

            refresh_interval_fast: std::env::var("REFRESH_INTERVAL_FAST")
                .expect("REFRESH_INTERVAL_FAST must be set")
                .parse()
                .expect("REFRESH_INTERVAL_FAST must be a valid integer"),
            refresh_interval_medium: std::env::var("REFRESH_INTERVAL_MEDIUM")
                .expect("REFRESH_INTERVAL_MEDIUM must be set")
                .parse()
                .expect("REFRESH_INTERVAL_MEDIUM must be a valid integer"),
            refresh_interval_slow: std::env::var("REFRESH_INTERVAL_SLOW")
                .expect("REFRESH_INTERVAL_SLOW must be set")
                .parse()
                .expect("REFRESH_INTERVAL_SLOW must be a valid integer"),

            // UI Thresholds - optional with defaults
            tcr_danger_threshold: std::env::var("TCR_DANGER_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(150.0),
            tcr_warning_threshold: std::env::var("TCR_WARNING_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200.0),
            whale_threshold_usd: std::env::var("WHALE_THRESHOLD_USD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100000.0),

            // Timing - optional with defaults
            refresh_interval_ms: std::env::var("REFRESH_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30000),
            history_retention_secs: std::env::var("HISTORY_RETENTION_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(604800),

            // RPC Settings - optional with defaults
            rpc_timeout_secs: std::env::var("RPC_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            rpc_retry_count: std::env::var("RPC_RETRY_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        }
    }
}

/// Global config instance
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get or initialize global configuration
/// On the server (SSR), loads from environment variables
/// On the client (WASM), uses hardcoded defaults
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            // Client-side: use defaults (env vars not available in browser)
            Config::default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Server-side: load from environment
            Config::from_env()
        }
    })
}
