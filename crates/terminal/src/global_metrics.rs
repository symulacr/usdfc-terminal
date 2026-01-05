//! Global Metrics Context
//!
//! Provides shared metrics resources across all components.
//! Reduces duplicate API calls by centralizing data fetching.

use leptos::*;
use usdfc_api::{
    get_protocol_metrics, get_usdfc_price_data, get_holder_count,
    USDFCPriceData,
};
use usdfc_core::types::ProtocolMetrics;

/// Global metrics context shared across all pages
#[derive(Clone, Copy)]
pub struct GlobalMetrics {
    /// Protocol metrics (supply, collateral, TCR, troves, stability pool)
    pub protocol: Resource<u32, Result<ProtocolMetrics, ServerFnError>>,
    /// Price data (price, volume, liquidity)
    pub price: Resource<u32, Result<USDFCPriceData, ServerFnError>>,
    /// Token holder count
    pub holders: Resource<u32, Result<u64, ServerFnError>>,
    /// Trigger for manual refresh
    pub refresh_trigger: RwSignal<u32>,
}

impl GlobalMetrics {
    /// Create new global metrics with shared resources
    pub fn new() -> Self {
        let refresh_trigger = create_rw_signal(0u32);

        // Protocol metrics - shared across Dashboard, Protocol page
        let protocol = create_resource(
            move || refresh_trigger.get(),
            |_| async move { get_protocol_metrics().await }
        );

        // Price data - shared across Header, Dashboard
        // Use create_local_resource to avoid SSR hydration issues
        let price = create_local_resource(
            move || refresh_trigger.get(),
            |_| async move { get_usdfc_price_data().await }
        );

        // Holder count - shared across Dashboard, Supply
        let holders = create_resource(
            move || refresh_trigger.get(),
            |_| async move { get_holder_count().await }
        );

        Self {
            protocol,
            price,
            holders,
            refresh_trigger,
        }
    }

    /// Refresh all metrics
    pub fn refresh_all(&self) {
        self.refresh_trigger.update(|n| *n += 1);
    }

    /// Refresh protocol metrics only
    pub fn refresh_protocol(&self) {
        self.protocol.refetch();
    }

    /// Refresh price data only
    pub fn refresh_price(&self) {
        self.price.refetch();
    }
}

/// Hook to access global metrics from any component
pub fn use_global_metrics() -> GlobalMetrics {
    use_context::<GlobalMetrics>().expect("GlobalMetrics must be provided")
}
