//! Main Advanced Chart Container Component

use leptos::*;
use crate::types::{ChartResolution, ChartLookback, ChartMetric, ChartType, ChartDataResponse};
use crate::server_fn::get_advanced_chart_data;
use super::{ChartHeader, ChartCanvas, ChartLegend};

/// Main Advanced Chart container component
#[component]
pub fn AdvancedChart() -> impl IntoView {
    // Chart state signals
    let resolution = create_rw_signal(ChartResolution::H1);
    let lookback = create_rw_signal(ChartLookback::Week1);
    let chart_type = create_rw_signal(ChartType::Area);
    let wallet_address = create_rw_signal(None::<String>);

    // Visible metrics (multi-select) - start with Price and Volume
    let visible_metrics = create_rw_signal(std::collections::HashSet::from([
        ChartMetric::Price,
        ChartMetric::Volume,
    ]));

    // Mouse interaction state
    let mouse_pos = create_rw_signal(None::<(f64, f64)>);
    let hovered_index = create_rw_signal(None::<usize>);

    // Loading state
    let is_loading = create_rw_signal(true);

    // Error state
    let error_message = create_rw_signal(None::<String>);

    // Chart data signal with default
    let chart_data = create_rw_signal(ChartDataResponse::default());

    // Fetch chart data resource - re-fetches when resolution or lookback changes
    // Using create_local_resource to avoid hydration mismatch
    // Advanced chart container does not expose a custom date range yet, so we
    // pass None for start/end to use the configured lookback window.
    let chart_resource = create_local_resource(
        move || (resolution.get(), lookback.get()),
        move |(res, lb)| async move {
            get_advanced_chart_data(res, lb, None, None).await
        }
    );

    // Effect to update chart_data and loading state when resource changes
    create_effect(move |_| {
        match chart_resource.get() {
            Some(Ok(data)) => {
                chart_data.set(data);
                is_loading.set(false);
                error_message.set(None);
            }
            Some(Err(e)) => {
                is_loading.set(false);
                error_message.set(Some(format!("Failed to load chart data: {}", e)));
            }
            None => {
                is_loading.set(true);
            }
        }
    });

    // Derived state: is wallet mode
    let is_wallet_mode = create_memo(move |_| wallet_address.get().is_some());

    // Toggle metric visibility
    let toggle_metric = move |metric: ChartMetric| {
        visible_metrics.update(|set| {
            if set.contains(&metric) {
                set.remove(&metric);
            } else {
                set.insert(metric);
            }
        });
    };

    view! {
        <div class="advanced-chart-container">
            <ChartHeader
                resolution=resolution
                lookback=lookback
                chart_type=chart_type
                wallet_address=wallet_address
                is_loading=is_loading.read_only()
            />

            // Error display
            <Show when=move || error_message.get().is_some()>
                <div class="chart-error">
                    <span class="error-icon">"!"</span>
                    <span class="error-text">{move || error_message.get().unwrap_or_default()}</span>
                    <button class="retry-btn" on:click=move |_| {
                        error_message.set(None);
                        is_loading.set(true);
                        chart_resource.refetch();
                    }>"Retry"</button>
                </div>
            </Show>

            // Current values display - handle Option<f64> safely
            <div class="chart-current-values">
                <div class="current-value price">
                    <span class="value-label">"Price"</span>
                    <span class="value-amount">{move || chart_data.get().current_price.map(|v| format!("${:.4}", v)).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value volume">
                    <span class="value-label">"24h Vol"</span>
                    <span class="value-amount">{move || chart_data.get().current_volume_24h.map(format_large_number).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value liquidity">
                    <span class="value-label">"Liquidity"</span>
                    <span class="value-amount">{move || chart_data.get().current_liquidity.map(format_large_number).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value tcr">
                    <span class="value-label">"TCR"</span>
                    <span class="value-amount">{move || chart_data.get().current_tcr.map(|v| format!("{:.1}%", v)).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value supply">
                    <span class="value-label">"Supply"</span>
                    <span class="value-amount">{move || chart_data.get().current_supply.map(format_large_number).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value holders">
                    <span class="value-label">"Holders"</span>
                    <span class="value-amount">{move || chart_data.get().current_holders.map(|v| v.to_string()).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value lend-apr">
                    <span class="value-label">"Lend APR"</span>
                    <span class="value-amount">{move || chart_data.get().current_lend_apr.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "--".to_string())}</span>
                </div>
                <div class="current-value borrow-apr">
                    <span class="value-label">"Borrow APR"</span>
                    <span class="value-amount">{move || chart_data.get().current_borrow_apr.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "--".to_string())}</span>
                </div>
            </div>

            <div class="chart-body">
                <ChartCanvas
                    data=chart_data.read_only()
                    resolution=resolution.read_only()
                    chart_type=chart_type.read_only()
                    visible_metrics=visible_metrics.read_only()
                    mouse_pos=mouse_pos
                    hovered_index=hovered_index
                    is_loading=is_loading.read_only()
                />

                <ChartLegend
                    visible_metrics=visible_metrics.read_only()
                    on_toggle=toggle_metric
                />
            </div>

            <Show when=move || is_wallet_mode.get()>
                <div class="wallet-analytics-panel">
                    <div class="wallet-analytics-header">
                        <span class="wallet-label">"Wallet Analytics"</span>
                        <span class="wallet-address">
                            {move || wallet_address.get().unwrap_or_default()}
                        </span>
                    </div>
                    <div class="coming-soon-placeholder">
                        <span class="coming-soon-badge">"Coming Soon v2"</span>
                        <span class="coming-soon-text">"Wallet-specific charts and analytics"</span>
                    </div>
                </div>
            </Show>

            <div class="chart-footer">
                <div class="resolution-buttons">
                    <For
                        each={move || ChartResolution::all().iter().copied()}
                        key=|r| r.label()
                        children=move |r| {
                            let is_active = move || resolution.get() == r;
                            view! {
                                <button
                                    class:active=is_active
                                    class="resolution-btn"
                                    on:click=move |_| resolution.set(r)
                                >
                                    {r.label()}
                                </button>
                            }
                        }
                    />
                </div>

                <div class="lookback-buttons">
                    <For
                        each={move || ChartLookback::all().iter().copied()}
                        key=|lb| lb.label()
                        children=move |lb| {
                            let is_active = move || lookback.get() == lb;
                            view! {
                                <button
                                    class:active=is_active
                                    class="lookback-btn"
                                    on:click=move |_| lookback.set(lb)
                                >
                                    {lb.label()}
                                </button>
                            }
                        }
                    />
                </div>

                <div class="chart-type-selector">
                    <button
                        class:active=move || chart_type.get() == ChartType::Area
                        on:click=move |_| chart_type.set(ChartType::Area)
                    >
                        "Area"
                    </button>
                    <button
                        class:active=move || chart_type.get() == ChartType::Line
                        on:click=move |_| chart_type.set(ChartType::Line)
                    >
                        "Line"
                    </button>
                    <button
                        class:active=move || chart_type.get() == ChartType::Candle
                        on:click=move |_| chart_type.set(ChartType::Candle)
                    >
                        "Candle"
                    </button>
                </div>

                // Fetch time indicator
                <div class="fetch-time">
                    <span class="fetch-label">"Fetched in "</span>
                    <span class="fetch-ms">{move || format!("{}ms", chart_data.get().fetch_time_ms)}</span>
                </div>
            </div>
        </div>
    }
}

/// Format large numbers with K/M/B suffix
fn format_large_number(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("${:.2}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("${:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.2}K", value / 1_000.0)
    } else {
        format!("${:.2}", value)
    }
}
