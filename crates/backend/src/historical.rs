//! Historical Metric Snapshot Storage
//!
//! In-memory time-series store for metrics that don't have historical APIs.
//! Collects snapshots every 60 seconds and stores up to 1 week of data.
//! Data is persisted to SQLite to survive server restarts.

use std::collections::VecDeque;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use rusqlite::{Connection, params};
#[cfg(feature = "ssr")]
use std::sync::Mutex;

/// Default maximum number of snapshots to store (1 week at 1-minute intervals)
/// Configurable via HISTORY_RETENTION_SECS environment variable
const DEFAULT_MAX_SNAPSHOTS: usize = 10080;

/// Get configured max snapshots from history_retention_secs
#[cfg(feature = "ssr")]
fn max_snapshots() -> usize {
    (crate::config::config().history_retention_secs / 60) as usize
}

/// For non-SSR builds, use the default
#[cfg(not(feature = "ssr"))]
fn max_snapshots() -> usize {
    DEFAULT_MAX_SNAPSHOTS
}

/// Get the SQLite database path from environment or use default
#[cfg(feature = "ssr")]
fn db_path() -> String {
    std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "data/metrics_history.db".to_string())
}

/// A single point-in-time snapshot of all metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp: i64,
    pub tcr: f64,
    pub supply: f64,
    pub liquidity: f64,
    pub holders: u64,
    pub lend_apr: f64,
    pub borrow_apr: f64,
}

/// Global in-memory history store
pub static METRIC_HISTORY: Lazy<RwLock<VecDeque<MetricSnapshot>>> =
    Lazy::new(|| RwLock::new(VecDeque::with_capacity(DEFAULT_MAX_SNAPSHOTS)));

/// SQLite connection (SSR only)
#[cfg(feature = "ssr")]
pub static DB_CONN: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the SQLite database and load existing data into memory
#[cfg(feature = "ssr")]
pub fn init_db() -> Result<(), rusqlite::Error> {
    let path = db_path();

    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(&path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS metric_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL UNIQUE,
            tcr REAL NOT NULL,
            supply REAL NOT NULL,
            liquidity REAL NOT NULL,
            holders INTEGER NOT NULL,
            lend_apr REAL NOT NULL,
            borrow_apr REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON metric_snapshots(timestamp)",
        [],
    )?;

    // Load existing data into memory cache
    load_from_db(&conn)?;

    *DB_CONN.lock().unwrap() = Some(conn);
    Ok(())
}

/// Load snapshots from the database into the in-memory cache
#[cfg(feature = "ssr")]
fn load_from_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, tcr, supply, liquidity, holders, lend_apr, borrow_apr
         FROM metric_snapshots
         ORDER BY timestamp DESC
         LIMIT ?"
    )?;

    let snapshots = stmt.query_map([max_snapshots() as i64], |row| {
        Ok(MetricSnapshot {
            timestamp: row.get(0)?,
            tcr: row.get(1)?,
            supply: row.get(2)?,
            liquidity: row.get(3)?,
            holders: row.get(4)?,
            lend_apr: row.get(5)?,
            borrow_apr: row.get(6)?,
        })
    })?;

    if let Ok(mut history) = METRIC_HISTORY.write() {
        history.clear();
        for snap in snapshots.flatten() {
            history.push_front(snap); // Reverse order since we queried DESC
        }
        tracing::info!("Loaded {} snapshots from database", history.len());
    }

    Ok(())
}

/// Save a snapshot to the SQLite database
#[cfg(feature = "ssr")]
fn save_to_db(snapshot: &MetricSnapshot) -> Result<(), rusqlite::Error> {
    let db_lock = DB_CONN.lock().map_err(|e| {
        tracing::error!("Mutex poison error in save_to_db: {}", e);
        rusqlite::Error::InvalidQuery
    })?;

    if let Some(ref conn) = *db_lock {
        conn.execute(
            "INSERT OR REPLACE INTO metric_snapshots
             (timestamp, tcr, supply, liquidity, holders, lend_apr, borrow_apr)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.timestamp,
                snapshot.tcr,
                snapshot.supply,
                snapshot.liquidity,
                snapshot.holders,
                snapshot.lend_apr,
                snapshot.borrow_apr,
            ],
        )?;

        // Cleanup old entries beyond max_snapshots
        conn.execute(
            "DELETE FROM metric_snapshots WHERE id NOT IN
             (SELECT id FROM metric_snapshots ORDER BY timestamp DESC LIMIT ?)",
            [max_snapshots() as i64],
        )?;
    }
    Ok(())
}

impl MetricSnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new(
        tcr: f64,
        supply: f64,
        liquidity: f64,
        holders: u64,
        lend_apr: f64,
        borrow_apr: f64,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            timestamp,
            tcr,
            supply,
            liquidity,
            holders,
            lend_apr,
            borrow_apr,
        }
    }

    /// Record a snapshot to history (persists to SQLite on SSR)
    pub fn record(snapshot: MetricSnapshot) {
        // Persist to SQLite database (SSR only)
        #[cfg(feature = "ssr")]
        {
            if let Err(e) = save_to_db(&snapshot) {
                tracing::error!("Failed to save snapshot to DB: {}", e);
            }
        }

        // Update in-memory cache
        if let Ok(mut history) = METRIC_HISTORY.write() {
            // Keep max snapshots (remove oldest if full)
            if history.len() >= max_snapshots() {
                history.pop_front();
            }
            let ts = snapshot.timestamp;
            history.push_back(snapshot);
            tracing::debug!(
                "Recorded metric snapshot #{} at {}",
                history.len(),
                ts
            );
        }
    }

    /// Get number of snapshots currently stored
    pub fn count() -> usize {
        METRIC_HISTORY.read().map(|h| h.len()).unwrap_or(0)
    }

    /// Get historical data filtered by lookback and downsampled by resolution
    ///
    /// Returns up to `lookback_mins / resolution_mins + 1` data points.
    /// The extra point may occur when the current timestamp falls in a new bucket.
    /// This is expected behavior and provides the most up-to-date data.
    ///
    /// - `lookback_mins`: How far back to look (0 = all data)
    /// - `resolution_mins`: Time bucket size for downsampling
    pub fn get_history(lookback_mins: u32, resolution_mins: u32) -> Vec<MetricSnapshot> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let cutoff = if lookback_mins == 0 {
            0 // ALL data
        } else {
            now - (lookback_mins as i64 * 60)
        };

        let resolution_secs = (resolution_mins as i64 * 60).max(60); // Min 1 minute

        if let Ok(history) = METRIC_HISTORY.read() {
            let mut result = Vec::new();
            let mut last_bucket = 0i64;

            for snap in history.iter() {
                if snap.timestamp >= cutoff {
                    // Downsample: only take one point per resolution bucket
                    let bucket = snap.timestamp / resolution_secs;
                    if bucket != last_bucket {
                        result.push(snap.clone());
                        last_bucket = bucket;
                    }
                }
            }
            result
        } else {
            Vec::new()
        }
    }

    /// Extract TCR time series from snapshots
    pub fn tcr_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, f64)> {
        snapshots.iter().map(|s| (s.timestamp, s.tcr)).collect()
    }

    /// Extract supply time series from snapshots
    pub fn supply_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, f64)> {
        snapshots.iter().map(|s| (s.timestamp, s.supply)).collect()
    }

    /// Extract liquidity time series from snapshots
    pub fn liquidity_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, f64)> {
        snapshots.iter().map(|s| (s.timestamp, s.liquidity)).collect()
    }

    /// Extract holders time series from snapshots
    pub fn holders_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, u64)> {
        snapshots.iter().map(|s| (s.timestamp, s.holders)).collect()
    }

    /// Extract lend APR time series from snapshots
    pub fn lend_apr_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, f64)> {
        snapshots.iter().map(|s| (s.timestamp, s.lend_apr)).collect()
    }

    /// Extract borrow APR time series from snapshots
    pub fn borrow_apr_series(snapshots: &[MetricSnapshot]) -> Vec<(i64, f64)> {
        snapshots.iter().map(|s| (s.timestamp, s.borrow_apr)).collect()
    }
}

/// Collect current metrics and create a snapshot
#[cfg(feature = "ssr")]
pub async fn collect_current_snapshot() -> Option<MetricSnapshot> {
    use crate::rpc::RpcClient;
    use crate::gecko::GeckoClient;
    use crate::blockscout::BlockscoutClient;
    use crate::subgraph::SubgraphClient;
    use usdfc_core::config::config;
    use rust_decimal::prelude::ToPrimitive;

    let rpc = RpcClient::new();
    let gecko = GeckoClient::new();
    let blockscout = BlockscoutClient::new();
    let subgraph = SubgraphClient::new();

    // Parallel fetch (within rate budget)
    let (tcr_result, supply_result, pool_result, holders_result, markets_result) = tokio::join!(
        rpc.get_tcr(),
        rpc.get_total_supply(),
        gecko.get_pool_info(&config().pool_usdfc_wfil),
        blockscout.get_holder_count(),
        subgraph.get_lending_markets()
    );

    // Extract values with fallbacks
    let tcr = tcr_result.ok().and_then(|v| v.to_f64()).unwrap_or(0.0);
    let supply = supply_result.ok().and_then(|v| v.to_f64()).unwrap_or(0.0);
    let liquidity = pool_result
        .ok()
        .and_then(|p| p.reserve_in_usd)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let holders = holders_result.ok().unwrap_or(0);

    // Get best APRs from active markets
    let (lend_apr, borrow_apr) = markets_result
        .ok()
        .map(|markets| {
            let mut best_lend = 0.0f64;
            let mut best_borrow = 0.0f64;
            for market in markets {
                if market.is_active {
                    let maturity_ts = market.maturity.parse::<i64>().unwrap_or(0);
                    if let Some(ref lend_price) = market.last_lend_unit_price {
                        if let Ok(apr) = crate::subgraph::unit_price_to_apr(lend_price, maturity_ts) {
                            best_lend = best_lend.max(apr);
                        }
                    }
                    if let Some(ref borrow_price) = market.last_borrow_unit_price {
                        if let Ok(apr) = crate::subgraph::unit_price_to_apr(borrow_price, maturity_ts) {
                            best_borrow = best_borrow.max(apr);
                        }
                    }
                }
            }
            (best_lend, best_borrow)
        })
        .unwrap_or((0.0, 0.0));

    Some(MetricSnapshot::new(tcr, supply, liquidity, holders, lend_apr, borrow_apr))
}

/// Start the background snapshot collector task
#[cfg(feature = "ssr")]
pub fn start_snapshot_collector() {
    tokio::spawn(async move {
        use std::time::Duration;

        // Collect first snapshot immediately
        if let Some(snapshot) = collect_current_snapshot().await {
            MetricSnapshot::record(snapshot);
        }

        // Then collect every 60 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await; // Skip first tick (already collected)

        loop {
            interval.tick().await;

            match collect_current_snapshot().await {
                Some(snapshot) => {
                    MetricSnapshot::record(snapshot);
                }
                None => {
                    tracing::warn!("Failed to collect metric snapshot");
                }
            }
        }
    });
}

/// Check database health by executing a simple query
#[cfg(feature = "ssr")]
pub fn check_db_health() -> Result<(), String> {
    if let Some(ref conn) = *DB_CONN.lock().map_err(|e| e.to_string())? {
        // Simple query to verify database is accessible
        conn.execute_batch("SELECT 1")
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Database connection not initialized".to_string())
    }
}
