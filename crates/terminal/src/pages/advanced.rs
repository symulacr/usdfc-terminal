//! Advanced Analytics Page - LayerZero Terminal Style
//!
//! Multi-source data visualization combining all USDFC data sources

use leptos::*;
use usdfc_core::config::config;
use usdfc_backend::server_fn::{
    get_protocol_metrics, get_usdfc_price_data,
    get_recent_transactions, get_lending_markets, get_holder_count,
    check_api_health, get_advanced_chart_data,
};
use usdfc_core::types::{ChartResolution, ChartLookback, ChartMetric, ChartType, ChartDataResponse};
use usdfc_core::format::{format_volume, format_usd_compact, decimal_to_f64, format_count};
use std::collections::HashSet;

// ============================================================================
// URL State Management
// ============================================================================

/// Chart state that can be serialized to/from URL query params
#[derive(Clone, Debug, Default)]
pub struct ChartUrlState {
    pub metrics: Vec<ChartMetric>,
    pub resolution: Option<ChartResolution>,
    pub lookback: Option<ChartLookback>,
    pub chart_type: Option<ChartType>,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl ChartUrlState {
    /// Parse chart state from URL query string
    /// Format: /advanced?metrics=price,volume&res=1h&lookback=1w&type=area&start=1704067200&end=1704153600
    #[cfg(feature = "hydrate")]
    pub fn from_url() -> Self {
        use web_sys::window;

        let mut state = Self::default();

        let Some(window) = window() else { return state };
        let Ok(location) = window.location().search() else { return state };

        // Skip the leading '?'
        let query = if location.starts_with('?') {
            &location[1..]
        } else {
            &location
        };

        if query.is_empty() {
            return state;
        }

        // Parse query params manually (avoiding external dependencies)
        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");

            match key {
                "metrics" => {
                    state.metrics = value
                        .split(',')
                        .filter_map(|m| ChartMetric::from_url_param(m))
                        .collect();
                }
                "res" => {
                    state.resolution = ChartResolution::from_url_param(value);
                }
                "lookback" => {
                    state.lookback = ChartLookback::from_url_param(value);
                }
                "type" => {
                    state.chart_type = ChartType::from_url_param(value);
                }
                "start" => {
                    state.start = value.parse().ok();
                }
                "end" => {
                    state.end = value.parse().ok();
                }
                _ => {}
            }
        }

        state
    }

    /// Build URL query string from current state
    pub fn to_query_string(
        metrics: &HashSet<ChartMetric>,
        resolution: ChartResolution,
        lookback: ChartLookback,
        chart_type: ChartType,
        start: Option<i64>,
        end: Option<i64>,
    ) -> String {
        let mut params = Vec::new();

        // Metrics (comma-separated)
        if !metrics.is_empty() {
            let mut metrics_list: Vec<&str> = metrics.iter().map(|m| m.to_url_param()).collect();
            metrics_list.sort(); // Consistent ordering
            params.push(format!("metrics={}", metrics_list.join(",")));
        }

        // Resolution
        params.push(format!("res={}", resolution.to_url_param()));

        // Chart type
        params.push(format!("type={}", chart_type.to_url_param()));

        // Custom date range or lookback
        if let (Some(s), Some(e)) = (start, end) {
            params.push(format!("start={}", s));
            params.push(format!("end={}", e));
        } else {
            params.push(format!("lookback={}", lookback.to_url_param()));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Update the browser URL without reloading the page
#[cfg(feature = "hydrate")]
fn update_browser_url(query: &str) {
    use web_sys::window;
    use wasm_bindgen::JsValue;

    if let Some(window) = window() {
        if let Ok(history) = window.history() {
            let url = format!("/advanced{}", query);
            let _ = history.replace_state_with_url(
                &JsValue::NULL,
                "",
                Some(&url),
            );
        }
    }
}

/// Copy text to clipboard and return success
#[cfg(feature = "hydrate")]
fn copy_to_clipboard(text: &str) -> bool {
    use web_sys::window;

    if let Some(window) = window() {
        let clipboard = window.navigator().clipboard();
        let text = text.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(
                clipboard.write_text(&text)
            ).await;
        });
        return true;
    }
    false
}

/// Get the full shareable URL
#[cfg(feature = "hydrate")]
fn get_share_url(query: &str) -> String {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(location) = window.location().origin() {
            return format!("{}/advanced{}", location, query);
        }
    }
    format!("/advanced{}", query)
}

#[component]
pub fn AdvancedAnalytics() -> impl IntoView {
    // Chart controls
    let resolution = create_rw_signal(ChartResolution::H1);
    let lookback = create_rw_signal(ChartLookback::Week1);
    let chart_type = create_rw_signal(ChartType::Area);
    let wallet_address = create_rw_signal(None::<String>);

    // Custom date range state
    let (custom_start, set_custom_start) = create_signal(None::<i64>);  // Unix timestamp
    let (custom_end, set_custom_end) = create_signal(None::<i64>);
    let (show_date_picker, set_show_date_picker) = create_signal(false);

    // Visible metrics
    let visible_metrics = create_rw_signal(HashSet::from([
        ChartMetric::Price,
        ChartMetric::Volume,
    ]));

    // Loading state
    let is_loading = create_rw_signal(true);
    let chart_data = create_rw_signal(ChartDataResponse::default());

    // Toast notification state for share button
    let (show_toast, set_show_toast) = create_signal(false);
    let (toast_message, set_toast_message) = create_signal(String::new());

    // Initialize state from URL on page load (client-side only)
    #[cfg(feature = "hydrate")]
    {
        // Run once on mount - parse URL and set signals
        create_effect(move |prev_run| {
            // Only run once on initial mount
            if prev_run.is_some() {
                return;
            }

            let url_state = ChartUrlState::from_url();

            // Apply URL state to signals if present
            if !url_state.metrics.is_empty() {
                visible_metrics.set(url_state.metrics.into_iter().collect());
            }
            if let Some(res) = url_state.resolution {
                resolution.set(res);
            }
            if let Some(lb) = url_state.lookback {
                lookback.set(lb);
            }
            if let Some(ct) = url_state.chart_type {
                chart_type.set(ct);
            }
            if let (Some(start), Some(end)) = (url_state.start, url_state.end) {
                set_custom_start.set(Some(start));
                set_custom_end.set(Some(end));
            }
        });
    }

    // Update URL when state changes (debounced)
    #[cfg(feature = "hydrate")]
    {
        // Track previous state to detect actual changes
        let prev_url = create_rw_signal(String::new());

        create_effect(move |_| {
            // Read all state
            let metrics = visible_metrics.get();
            let res = resolution.get();
            let lb = lookback.get();
            let ct = chart_type.get();
            let start = custom_start.get();
            let end = custom_end.get();

            // Build query string
            let query = ChartUrlState::to_query_string(&metrics, res, lb, ct, start, end);

            // Only update if changed
            if query != prev_url.get() {
                prev_url.set(query.clone());
                update_browser_url(&query);
            }
        });
    }

    // Check if custom date range is active
    let is_custom_range_active = move || custom_start.get().is_some() && custom_end.get().is_some();

    // Fetch chart data (client-only to avoid hydration mismatch)
    // When custom dates are set, they take priority over lookback on the server side.
    // We pass the optional start/end timestamps directly to the server function.
    let chart_resource = create_local_resource(
        move || (resolution.get(), lookback.get(), custom_start.get(), custom_end.get()),
        move |(res, lb, start, end)| async move {
            get_advanced_chart_data(res, lb, start, end).await
        }
    );

    // Update chart_data when resource changes
    create_effect(move |_| {
        match chart_resource.get() {
            Some(Ok(data)) => {
                chart_data.set(data);
                is_loading.set(false);
            }
            Some(Err(_)) => {
                is_loading.set(false);
            }
            None => {
                is_loading.set(true);
            }
        }
    });

    // Initialize ECharts when chart data loads (client-side only)
    #[cfg(feature = "hydrate")]
    {
        // Track if chart has been initialized (unused but kept for potential future use)
        let _echarts_initialized = create_rw_signal(false);

        create_effect(move |_| {
            let data = chart_data.get();
            let loading = is_loading.get();
            let ct = chart_type.get();
            let metrics = visible_metrics.get();

            // Only initialize when we have data and not loading
            if !loading && !data.price_candles.is_empty() {
                // Build chart configuration based on chart type and visible metrics
                let candles = &data.price_candles;

                // Determine series type based on chart_type
                let (series_type, show_area) = match ct {
                    ChartType::Candle => ("candlestick", false),
                    ChartType::Line => ("line", false),
                    ChartType::Area | ChartType::Bar => ("line", true),
                };

                // Build price data array
                let price_data_json: String = if series_type == "candlestick" {
                    // Candlestick: [timestamp, open, close, low, high]
                    candles.iter()
                        .map(|c| format!("[{},{},{},{},{}]", c.time * 1000, c.open, c.close, c.low, c.high))
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    // Line/Area: [timestamp, close]
                    candles.iter()
                        .map(|c| format!("[{},{}]", c.time * 1000, c.close))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let price_data_json = format!("[{}]", price_data_json);

                // Build volume data array
                let volume_data_json: String = candles.iter()
                    .map(|c| {
                        let color = if c.close >= c.open {
                            "rgba(139, 92, 246, 0.7)"
                        } else {
                            "rgba(139, 92, 246, 0.4)"
                        };
                        format!(r#"{{"value":[{},{}],"itemStyle":{{"color":"{}"}}}}"#, c.time * 1000, c.volume, color)
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let volume_data_json = format!("[{}]", volume_data_json);

                // Build data arrays for historical metrics (from MetricSnapshot)
                let liquidity_data_json: String = data.liquidity_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let liquidity_data_json = format!("[{}]", liquidity_data_json);

                let tcr_data_json: String = data.tcr_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let tcr_data_json = format!("[{}]", tcr_data_json);

                let supply_data_json: String = data.supply_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let supply_data_json = format!("[{}]", supply_data_json);

                let holders_data_json: String = data.holders_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let holders_data_json = format!("[{}]", holders_data_json);

                let lend_apr_data_json: String = data.lend_apr_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let lend_apr_data_json = format!("[{}]", lend_apr_data_json);

                let borrow_apr_data_json: String = data.borrow_apr_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let borrow_apr_data_json = format!("[{}]", borrow_apr_data_json);

                let transfers_data_json: String = data.transfers_data.iter()
                    .map(|(ts, v)| format!("[{},{}]", ts * 1000, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let transfers_data_json = format!("[{}]", transfers_data_json);

                // Build series configuration
                let show_price = metrics.contains(&ChartMetric::Price);
                let show_volume = metrics.contains(&ChartMetric::Volume);
                let show_liquidity = metrics.contains(&ChartMetric::Liquidity);
                let show_tcr = metrics.contains(&ChartMetric::TCR);
                let show_supply = metrics.contains(&ChartMetric::Supply);
                let show_holders = metrics.contains(&ChartMetric::Holders);
                let show_lend_apr = metrics.contains(&ChartMetric::LendAPR);
                let show_borrow_apr = metrics.contains(&ChartMetric::BorrowAPR);
                let show_transfers = metrics.contains(&ChartMetric::Transfers);

                let area_style = if show_area {
                    r#"areaStyle: {
                        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                            { offset: 0, color: 'rgba(0, 212, 255, 0.4)' },
                            { offset: 1, color: 'rgba(0, 212, 255, 0.02)' }
                        ])
                    },"#
                } else {
                    ""
                };

                let candlestick_style = if series_type == "candlestick" {
                    r#"itemStyle: {
                        color: '#22c55e',
                        color0: '#ef4444',
                        borderColor: '#22c55e',
                        borderColor0: '#ef4444'
                    },"#
                } else {
                    r#"smooth: true,
                    showSymbol: false,
                    lineStyle: { width: 2, color: '#00d4ff' },
                    itemStyle: { color: '#00d4ff' },"#
                };

                // Build the full ECharts configuration
                let js_code = format!(r#"
                    (function() {{
                        var el = document.getElementById('advanced-chart');
                        if (!el || !window.echarts) return;

                        // Dispose existing chart instance if any
                        var existingChart = echarts.getInstanceByDom(el);
                        if (existingChart) {{
                            existingChart.dispose();
                        }}

                        var chart = echarts.init(el, 'dark');

                        // Store price data for % change calculations
                        var priceData = {price_data};
                        var volumeData = {volume_data};

                        // Historical metric data from snapshots
                        var liquidityData = {liquidity_data};
                        var tcrData = {tcr_data};
                        var supplyData = {supply_data};
                        var holdersData = {holders_data};
                        var lendAprData = {lend_apr_data};
                        var borrowAprData = {borrow_apr_data};
                        var transfersData = {transfers_data};

                        // Track visible range start value (updated on dataZoom)
                        var visibleStartIndex = 0;
                        var visibleEndIndex = priceData.length - 1;

                        // Crosshair lock state
                        var isLocked = false;
                        var lockedParams = null;
                        var lockedDataIndex = null;

                        // Helper function to get value from data point
                        function getValue(dataPoint, seriesType) {{
                            if (!dataPoint) return null;
                            if (seriesType === 'candlestick') {{
                                return dataPoint[2]; // close price for candlestick
                            }}
                            if (Array.isArray(dataPoint)) {{
                                return dataPoint[1];
                            }}
                            if (dataPoint.value) {{
                                return dataPoint.value[1];
                            }}
                            return dataPoint;
                        }}

                        // Helper function to format % change with color
                        function formatChange(change, label) {{
                            if (isNaN(change) || !isFinite(change)) return '';
                            var color = change >= 0 ? '#22c55e' : '#ef4444';
                            var sign = change >= 0 ? '+' : '';
                            return '<span style="color: ' + color + '; font-size: 11px;">' + label + ': ' + sign + change.toFixed(2) + '%</span>';
                        }}

                        var series = [];

                        // Price series
                        if ({show_price}) {{
                            series.push({{
                                name: 'Price',
                                type: '{series_type}',
                                data: priceData,
                                {candlestick_style}
                                {area_style}
                                yAxisIndex: 0
                            }});
                        }}

                        // Volume series
                        if ({show_volume}) {{
                            series.push({{
                                name: 'Volume',
                                type: 'bar',
                                yAxisIndex: 1,
                                data: volumeData,
                                barMaxWidth: 20,
                                itemStyle: {{ color: 'rgba(139, 92, 246, 0.6)' }},
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Liquidity series (from historical snapshots)
                        if ({show_liquidity}) {{
                            series.push({{
                                name: 'Liquidity',
                                type: 'line',
                                smooth: true,
                                showSymbol: liquidityData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#06b6d4' }},
                                itemStyle: {{ color: '#06b6d4' }},
                                data: liquidityData,
                                yAxisIndex: 2,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // TCR series (from historical snapshots)
                        if ({show_tcr}) {{
                            series.push({{
                                name: 'TCR',
                                type: 'line',
                                smooth: true,
                                showSymbol: tcrData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#22c55e' }},
                                itemStyle: {{ color: '#22c55e' }},
                                data: tcrData,
                                yAxisIndex: 3,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Supply series (from historical snapshots)
                        if ({show_supply}) {{
                            series.push({{
                                name: 'Supply',
                                type: 'line',
                                smooth: true,
                                showSymbol: supplyData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#f59e0b' }},
                                itemStyle: {{ color: '#f59e0b' }},
                                data: supplyData,
                                yAxisIndex: 4,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Holders series (from historical snapshots)
                        if ({show_holders}) {{
                            series.push({{
                                name: 'Holders',
                                type: 'line',
                                smooth: true,
                                showSymbol: holdersData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#ec4899' }},
                                itemStyle: {{ color: '#ec4899' }},
                                data: holdersData,
                                yAxisIndex: 5,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Lend APR series (from historical snapshots)
                        if ({show_lend_apr}) {{
                            series.push({{
                                name: 'Lend APR',
                                type: 'line',
                                smooth: true,
                                showSymbol: lendAprData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#10b981' }},
                                itemStyle: {{ color: '#10b981' }},
                                data: lendAprData,
                                yAxisIndex: 6,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Borrow APR series (from historical snapshots)
                        if ({show_borrow_apr}) {{
                            series.push({{
                                name: 'Borrow APR',
                                type: 'line',
                                smooth: true,
                                showSymbol: borrowAprData.length <= 1,
                                symbolSize: 8,
                                lineStyle: {{ width: 2, color: '#f97316' }},
                                itemStyle: {{ color: '#f97316' }},
                                data: borrowAprData,
                                yAxisIndex: 6,  // Share with Lend APR
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        // Transfers series (from Blockscout)
                        if ({show_transfers}) {{
                            series.push({{
                                name: 'Transfers',
                                type: 'bar',
                                barMaxWidth: 15,
                                itemStyle: {{ color: 'rgba(99, 102, 241, 0.7)' }},
                                data: transfersData,
                                yAxisIndex: 7,
                                animation: true,
                                animationDuration: 300
                            }});
                        }}

                        chart.setOption({{
                            backgroundColor: 'transparent',
                            animation: true,
                            grid: {{
                                left: '3%',
                                right: '8%',
                                top: 60,
                                bottom: 80,
                                containLabel: true
                            }},
                            tooltip: {{
                                trigger: 'axis',
                                axisPointer: {{
                                    type: 'cross',
                                    crossStyle: {{ color: '#00d4ff' }},
                                    lineStyle: {{ type: 'dashed', color: '#00d4ff' }}
                                }},
                                backgroundColor: 'rgba(20, 20, 30, 0.95)',
                                borderColor: '#00d4ff',
                                borderWidth: 1,
                                textStyle: {{ color: '#fff' }},
                                formatter: function(params) {{
                                    // If locked, show locked data instead
                                    if (isLocked && lockedParams) {{
                                        params = lockedParams;
                                    }}

                                    if (!params || params.length === 0) return '';

                                    var timestamp = params[0].data[0] || params[0].data.value[0];
                                    var date = new Date(timestamp);
                                    var dateStr = date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {{hour: '2-digit', minute:'2-digit'}});

                                    // Lock indicator
                                    var lockIndicator = isLocked ? '<div style="color: #f59e0b; font-size: 10px; margin-bottom: 4px;">LOCKED (click to unlock)</div>' : '';

                                    var result = lockIndicator + '<div style="font-weight: bold; margin-bottom: 4px;">' + dateStr + '</div>';

                                    params.forEach(function(item) {{
                                        var value;
                                        var currentValue;
                                        var dataIndex = item.dataIndex;

                                        if (item.seriesType === 'candlestick') {{
                                            currentValue = item.data[2]; // close
                                            value = 'O: $' + item.data[1].toFixed(4) + ' C: $' + item.data[2].toFixed(4);

                                            // Calculate % changes for price
                                            var prevValue = dataIndex > 0 ? getValue(priceData[dataIndex - 1], 'candlestick') : currentValue;
                                            var startValue = getValue(priceData[visibleStartIndex], 'candlestick');

                                            var changeFromPrev = prevValue ? ((currentValue - prevValue) / prevValue) * 100 : 0;
                                            var changeFromStart = startValue ? ((currentValue - startValue) / startValue) * 100 : 0;

                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                            result += '<div style="display: flex; gap: 12px; margin-left: 4px;">'
                                                + formatChange(changeFromPrev, 'Prev')
                                                + formatChange(changeFromStart, 'Start')
                                                + '</div>';

                                        }} else if (item.seriesName === 'Price') {{
                                            currentValue = item.data[1] || item.data.value[1];
                                            value = '$' + currentValue.toFixed(4);

                                            // Calculate % changes for price
                                            var prevValue = dataIndex > 0 ? getValue(priceData[dataIndex - 1], 'line') : currentValue;
                                            var startValue = getValue(priceData[visibleStartIndex], 'line');

                                            var changeFromPrev = prevValue ? ((currentValue - prevValue) / prevValue) * 100 : 0;
                                            var changeFromStart = startValue ? ((currentValue - startValue) / startValue) * 100 : 0;

                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                            result += '<div style="display: flex; gap: 12px; margin-left: 4px;">'
                                                + formatChange(changeFromPrev, 'Prev')
                                                + formatChange(changeFromStart, 'Start')
                                                + '</div>';

                                        }} else if (item.seriesName === 'Volume') {{
                                            var vol = item.data[1] || item.data.value[1];
                                            value = '$' + vol.toLocaleString(undefined, {{maximumFractionDigits: 0}});
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'Liquidity') {{
                                            var liq = item.data[1] || item.data.value[1];
                                            if (liq >= 1000000) value = '$' + (liq/1000000).toFixed(2) + 'M';
                                            else if (liq >= 1000) value = '$' + (liq/1000).toFixed(1) + 'K';
                                            else value = '$' + liq.toFixed(0);
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'TCR') {{
                                            var tcr = item.data[1] || item.data.value[1];
                                            value = tcr.toFixed(1) + '%';
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'Supply') {{
                                            var supply = item.data[1] || item.data.value[1];
                                            if (supply >= 1000000) value = (supply/1000000).toFixed(2) + 'M';
                                            else if (supply >= 1000) value = (supply/1000).toFixed(1) + 'K';
                                            else value = supply.toFixed(0);
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'Holders') {{
                                            var holders = item.data[1] || item.data.value[1];
                                            value = holders.toFixed(0);
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'Lend APR' || item.seriesName === 'Borrow APR') {{
                                            var apr = item.data[1] || item.data.value[1];
                                            value = apr.toFixed(2) + '%';
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else if (item.seriesName === 'Transfers') {{
                                            var transfers = item.data[1] || item.data.value[1];
                                            value = transfers.toFixed(0);
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }} else {{
                                            // Generic fallback
                                            var val = item.data[1] || item.data.value[1];
                                            value = val.toLocaleString();
                                            result += '<div style="display: flex; justify-content: space-between; gap: 16px;">'
                                                + '<span style="color: ' + item.color + ';">' + item.seriesName + '</span>'
                                                + '<span style="font-weight: bold;">' + value + '</span></div>';
                                        }}
                                    }});
                                    return result;
                                }}
                            }},
                            toolbox: {{
                                show: true,
                                right: 10,
                                top: 10,
                                feature: {{
                                    saveAsImage: {{
                                        show: true,
                                        title: 'Export PNG',
                                        pixelRatio: 2,
                                        backgroundColor: '#0a0a0a'
                                    }}
                                }},
                                iconStyle: {{ borderColor: '#00d4ff' }}
                            }},
                            xAxis: {{
                                type: 'time',
                                boundaryGap: false,
                                axisLine: {{ lineStyle: {{ color: '#333' }} }},
                                axisLabel: {{
                                    color: '#888',
                                    formatter: function(value) {{
                                        var date = new Date(value);
                                        var now = new Date();
                                        var diffHours = (now - date) / (1000 * 60 * 60);

                                        // Dynamic formatting based on time range
                                        if (diffHours < 24) {{
                                            // Last 24 hours: show time
                                            return date.toLocaleTimeString('en-US', {{ hour: '2-digit', minute: '2-digit', hour12: false }});
                                        }} else if (diffHours < 168) {{
                                            // Last week: show day and time
                                            return date.toLocaleDateString('en-US', {{ month: 'short', day: 'numeric' }})
                                                + ' ' + date.toLocaleTimeString('en-US', {{ hour: '2-digit', hour12: false }});
                                        }} else {{
                                            // Longer: show date only
                                            return date.toLocaleDateString('en-US', {{ month: 'short', day: 'numeric' }});
                                        }}
                                    }},
                                    rotate: 0,
                                    hideOverlap: true
                                }},
                                splitLine: {{
                                    show: true,
                                    lineStyle: {{ color: 'rgba(255,255,255,0.05)' }}
                                }}
                            }},
                            yAxis: [
                                // 0: Price
                                {{
                                    type: 'value',
                                    name: 'Price',
                                    position: 'left',
                                    scale: true,
                                    axisLine: {{ lineStyle: {{ color: '#00d4ff' }} }},
                                    axisLabel: {{
                                        color: '#00d4ff',
                                        formatter: function(v) {{ return '$' + v.toFixed(4); }}
                                    }},
                                    splitLine: {{ lineStyle: {{ color: 'rgba(255,255,255,0.05)' }} }}
                                }},
                                // 1: Volume
                                {{
                                    type: 'value',
                                    name: 'Volume',
                                    position: 'right',
                                    scale: true,
                                    axisLine: {{ lineStyle: {{ color: '#8b5cf6' }}, show: false }},
                                    axisLabel: {{ show: false }},
                                    splitLine: {{ show: false }}
                                }},
                                // 2: Liquidity (USD)
                                {{
                                    type: 'value',
                                    name: 'Liquidity',
                                    position: 'right',
                                    offset: 0,
                                    scale: true,
                                    show: {show_liquidity},
                                    axisLine: {{ lineStyle: {{ color: '#06b6d4' }} }},
                                    axisLabel: {{
                                        color: '#06b6d4',
                                        formatter: function(v) {{
                                            if (v >= 1000000) return '$' + (v/1000000).toFixed(1) + 'M';
                                            if (v >= 1000) return '$' + (v/1000).toFixed(0) + 'K';
                                            return '$' + v.toFixed(0);
                                        }}
                                    }},
                                    splitLine: {{ show: false }}
                                }},
                                // 3: TCR (%)
                                {{
                                    type: 'value',
                                    name: 'TCR',
                                    position: 'right',
                                    offset: 60,
                                    scale: true,
                                    show: {show_tcr},
                                    axisLine: {{ lineStyle: {{ color: '#22c55e' }} }},
                                    axisLabel: {{
                                        color: '#22c55e',
                                        formatter: function(v) {{ return v.toFixed(0) + '%'; }}
                                    }},
                                    splitLine: {{ show: false }}
                                }},
                                // 4: Supply
                                {{
                                    type: 'value',
                                    name: 'Supply',
                                    position: 'right',
                                    offset: 120,
                                    scale: true,
                                    show: {show_supply},
                                    axisLine: {{ lineStyle: {{ color: '#f59e0b' }} }},
                                    axisLabel: {{
                                        color: '#f59e0b',
                                        formatter: function(v) {{
                                            if (v >= 1000000) return (v/1000000).toFixed(1) + 'M';
                                            if (v >= 1000) return (v/1000).toFixed(0) + 'K';
                                            return v.toFixed(0);
                                        }}
                                    }},
                                    splitLine: {{ show: false }}
                                }},
                                // 5: Holders
                                {{
                                    type: 'value',
                                    name: 'Holders',
                                    position: 'right',
                                    offset: 180,
                                    scale: true,
                                    show: {show_holders},
                                    axisLine: {{ lineStyle: {{ color: '#ec4899' }} }},
                                    axisLabel: {{
                                        color: '#ec4899',
                                        formatter: function(v) {{ return v.toFixed(0); }}
                                    }},
                                    splitLine: {{ show: false }}
                                }},
                                // 6: APR (%) - shared by Lend & Borrow
                                {{
                                    type: 'value',
                                    name: 'APR',
                                    position: 'right',
                                    offset: 240,
                                    scale: true,
                                    show: {show_lend_apr} || {show_borrow_apr},
                                    axisLine: {{ lineStyle: {{ color: '#10b981' }} }},
                                    axisLabel: {{
                                        color: '#10b981',
                                        formatter: function(v) {{ return v.toFixed(1) + '%'; }}
                                    }},
                                    splitLine: {{ show: false }}
                                }},
                                // 7: Transfers (count)
                                {{
                                    type: 'value',
                                    name: 'Transfers',
                                    position: 'right',
                                    offset: 300,
                                    scale: true,
                                    show: {show_transfers},
                                    axisLine: {{ lineStyle: {{ color: '#6366f1' }} }},
                                    axisLabel: {{
                                        color: '#6366f1',
                                        formatter: function(v) {{ return v.toFixed(0); }}
                                    }},
                                    splitLine: {{ show: false }}
                                }}
                            ],
                            dataZoom: [
                                {{
                                    type: 'inside',
                                    xAxisIndex: 0,
                                    start: 0,
                                    end: 100
                                }},
                                {{
                                    type: 'slider',
                                    xAxisIndex: 0,
                                    start: 0,
                                    end: 100,
                                    bottom: 10,
                                    height: 30,
                                    borderColor: '#333',
                                    backgroundColor: 'rgba(0,0,0,0.3)',
                                    fillerColor: 'rgba(0, 212, 255, 0.2)',
                                    handleStyle: {{ color: '#00d4ff' }},
                                    textStyle: {{ color: '#888' }}
                                }}
                            ],
                            series: series
                        }});

                        // Update visible range indices on dataZoom
                        chart.on('dataZoom', function(params) {{
                            var option = chart.getOption();
                            var dataZoom = option.dataZoom[0];
                            var start = dataZoom.start || 0;
                            var end = dataZoom.end || 100;

                            // Calculate visible indices based on percentage
                            var dataLen = priceData.length;
                            visibleStartIndex = Math.floor((start / 100) * dataLen);
                            visibleEndIndex = Math.min(Math.ceil((end / 100) * dataLen), dataLen - 1);

                            // Ensure valid bounds
                            if (visibleStartIndex < 0) visibleStartIndex = 0;
                            if (visibleStartIndex >= dataLen) visibleStartIndex = dataLen - 1;
                        }});

                        // Click-to-lock crosshair feature
                        chart.on('click', function(params) {{
                            if (isLocked) {{
                                // Unlock
                                isLocked = false;
                                lockedParams = null;
                                lockedDataIndex = null;

                                // Reset axisPointer style to dashed
                                chart.setOption({{
                                    tooltip: {{
                                        axisPointer: {{
                                            lineStyle: {{ type: 'dashed', color: '#00d4ff' }}
                                        }},
                                        borderColor: '#00d4ff'
                                    }}
                                }});
                            }} else {{
                                // Lock at current position
                                isLocked = true;
                                lockedDataIndex = params.dataIndex;

                                // Build locked params from available series data
                                lockedParams = [];
                                var option = chart.getOption();
                                option.series.forEach(function(s, idx) {{
                                    if (s.data && s.data[params.dataIndex]) {{
                                        lockedParams.push({{
                                            seriesName: s.name,
                                            seriesType: s.type,
                                            data: s.data[params.dataIndex],
                                            dataIndex: params.dataIndex,
                                            color: s.type === 'candlestick' ? '#22c55e' : (s.name === 'Price' ? '#00d4ff' : 'rgba(139, 92, 246, 0.7)')
                                        }});
                                    }}
                                }});

                                // Change axisPointer style to solid when locked
                                chart.setOption({{
                                    tooltip: {{
                                        axisPointer: {{
                                            lineStyle: {{ type: 'solid', color: '#f59e0b', width: 2 }}
                                        }},
                                        borderColor: '#f59e0b'
                                    }}
                                }});

                                // Keep tooltip visible at clicked position
                                chart.dispatchAction({{
                                    type: 'showTip',
                                    seriesIndex: 0,
                                    dataIndex: params.dataIndex
                                }});
                            }}
                        }});

                        // Handle window resize
                        var resizeHandler = function() {{ chart.resize(); }};
                        window.removeEventListener('resize', window.__usdfc_resize_handler);
                        window.addEventListener('resize', resizeHandler);
                        window.__usdfc_resize_handler = resizeHandler;

                        // Store chart instance for later access
                        window.__usdfc_echarts = chart;
                    }})()
                "#,
                    show_price = show_price,
                    show_volume = show_volume,
                    show_liquidity = show_liquidity,
                    show_tcr = show_tcr,
                    show_supply = show_supply,
                    show_holders = show_holders,
                    show_lend_apr = show_lend_apr,
                    show_borrow_apr = show_borrow_apr,
                    show_transfers = show_transfers,
                    series_type = series_type,
                    price_data = price_data_json,
                    volume_data = volume_data_json,
                    liquidity_data = liquidity_data_json,
                    tcr_data = tcr_data_json,
                    supply_data = supply_data_json,
                    holders_data = holders_data_json,
                    lend_apr_data = lend_apr_data_json,
                    borrow_apr_data = borrow_apr_data_json,
                    transfers_data = transfers_data_json,
                    candlestick_style = candlestick_style,
                    area_style = area_style,
                );

                // Use setTimeout(0) to defer chart init until after DOM updates
                // This fixes the race condition where the effect fires before the
                // Show component renders the chart container
                let js_code_clone = js_code.clone();
                gloo_timers::callback::Timeout::new(0, move || {
                    let _ = js_sys::eval(&js_code_clone);
                }).forget();
            }
        });
    }

    // Auto-refresh every 30 seconds (client-side only)
    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Interval;

        let interval = Interval::new(config().refresh_interval_ms as u32, move || {
            if !is_loading.get() {
                chart_resource.refetch();
            }
        });

        on_cleanup(move || drop(interval));
    }

    // Data sources for stats (client-only to avoid hydration mismatch)
    let protocol = create_local_resource(|| (), |_| async move { get_protocol_metrics().await });
    let price = create_local_resource(|| (), |_| async move { get_usdfc_price_data().await });
    let transactions = create_local_resource(|| (), |_| async move { get_recent_transactions(Some(50)).await });
    let lending = create_local_resource(|| (), |_| async move { get_lending_markets().await });
    let holders = create_local_resource(|| (), |_| async move { get_holder_count().await });
    let health = create_local_resource(|| (), |_| async move { check_api_health().await });

    // Toggle metric
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
        <div class="lz-page">
            // Page Header
            <div class="lz-header">
                <div class="lz-title-section">
                    <h1 class="lz-title">"Advanced Analytics"</h1>
                    <p class="lz-subtitle">"Multi-source USDFC protocol data"</p>
                </div>
                <div class="lz-controls">
                    // Wallet search
                    <div class="address-search">
                        <input
                            type="text"
                            placeholder="0x... wallet address"
                            class="address-input"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if value.starts_with("0x") && value.len() >= 40 {
                                    wallet_address.set(Some(value));
                                } else if value.is_empty() {
                                    wallet_address.set(None);
                                }
                            }
                        />
                    </div>
                </div>
            </div>

            // Main Chart Section - Unified
            <div class="lz-chart-section">
                <div class="lz-chart-header">
                    <div class="lz-chart-info">
                        <span class="lz-chart-label">"USDFC Price"</span>
                        <span class="lz-chart-value">
                            {move || chart_data.get().current_price.map(|v| format!("${:.4}", v)).unwrap_or_else(|| "Error".to_string())}
                        </span>
                    </div>

                    // Current metrics row - handle Option values safely
                    <div class="lz-metrics-row">
                        <span class="metric-item volume">
                            <span class="metric-label">"Vol"</span>
                            <span class="metric-value">{move || chart_data.get().current_volume_24h.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</span>
                        </span>
                        <span class="metric-item liquidity">
                            <span class="metric-label">"Liq"</span>
                            <span class="metric-value">{move || chart_data.get().current_liquidity.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</span>
                        </span>
                        <span class="metric-item tcr">
                            <span class="metric-label">"TCR"</span>
                            <span class="metric-value">{move || chart_data.get().current_tcr.map(|v| format!("{:.1}%", v)).unwrap_or_else(|| "--".to_string())}</span>
                        </span>
                        <span class="metric-item holders">
                            <span class="metric-label">"Holders"</span>
                            <span class="metric-value">{move || chart_data.get().current_holders.map(|v| format_count(v as usize)).unwrap_or_else(|| "--".to_string())}</span>
                        </span>
                        <span class="metric-item apr">
                            <span class="metric-label">"APR"</span>
                            <span class="metric-value">{move || chart_data.get().current_lend_apr.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "--".to_string())}</span>
                        </span>
                    </div>

                    <div class="lz-chart-types">
                        <button
                            class=move || if chart_type.get() == ChartType::Area { "lz-type-btn active" } else { "lz-type-btn" }
                            on:click=move |_| chart_type.set(ChartType::Area)
                        >
                            <svg viewBox="0 0 20 20" fill="currentColor"><path d="M2 16L6 10L10 13L14 7L18 11V16H2Z" opacity="0.3"/><path d="M2 16L6 10L10 13L14 7L18 11" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
                        </button>
                        <button
                            class=move || if chart_type.get() == ChartType::Line { "lz-type-btn active" } else { "lz-type-btn" }
                            on:click=move |_| chart_type.set(ChartType::Line)
                        >
                            <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="2,14 6,8 10,11 14,5 18,9"/><circle cx="6" cy="8" r="1.5" fill="currentColor"/><circle cx="10" cy="11" r="1.5" fill="currentColor"/><circle cx="14" cy="5" r="1.5" fill="currentColor"/></svg>
                        </button>
                        <button
                            class=move || if chart_type.get() == ChartType::Candle { "lz-type-btn active" } else { "lz-type-btn" }
                            on:click=move |_| chart_type.set(ChartType::Candle)
                        >
                            <svg viewBox="0 0 20 20" fill="currentColor"><rect x="3" y="6" width="3" height="8"/><line x1="4.5" y1="3" x2="4.5" y2="14" stroke="currentColor" stroke-width="1"/><rect x="9" y="8" width="3" height="6"/><line x1="10.5" y1="5" x2="10.5" y2="16" stroke="currentColor" stroke-width="1"/><rect x="15" y="4" width="3" height="10"/><line x1="16.5" y1="2" x2="16.5" y2="16" stroke="currentColor" stroke-width="1"/></svg>
                        </button>

                        // Share button
                        <button
                            class="lz-share-btn"
                            title="Copy shareable link"
                            on:click=move |_| {
                                #[cfg(feature = "hydrate")]
                                {
                                    let query = ChartUrlState::to_query_string(
                                        &visible_metrics.get(),
                                        resolution.get(),
                                        lookback.get(),
                                        chart_type.get(),
                                        custom_start.get(),
                                        custom_end.get(),
                                    );
                                    let url = get_share_url(&query);
                                    if copy_to_clipboard(&url) {
                                        set_toast_message.set("Link copied!".to_string());
                                        set_show_toast.set(true);
                                        // Auto-hide toast after 2 seconds
                                        let set_show = set_show_toast;
                                        wasm_bindgen_futures::spawn_local(async move {
                                            gloo_timers::future::TimeoutFuture::new(2000).await;
                                            set_show.set(false);
                                        });
                                    }
                                }
                            }
                        >
                            <svg viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
                                <path d="M15 8a3 3 0 10-2.977-2.63l-4.94 2.47a3 3 0 100 4.319l4.94 2.47a3 3 0 10.895-1.789l-4.94-2.47a3.027 3.027 0 000-.74l4.94-2.47C13.456 7.68 14.19 8 15 8z"/>
                            </svg>
                            <span>"Share"</span>
                        </button>
                    </div>
                </div>

                <div class="lz-chart-body">
                    <Show when=move || is_loading.get()>
                        <div class="lz-chart-loading">
                            <div class="lz-spinner"></div>
                        </div>
                    </Show>

                    <Show when=move || !is_loading.get()>
                        // ECharts container
                        <div id="advanced-chart" style="width: 100%; height: 400px;"></div>
                    </Show>
                </div>

                // Chart Footer with controls
                <div class="lz-chart-footer">
                    // WARNING BANNER: Show when lookback exceeds safe limit
                    <Show when=move || {
                        !resolution.get().is_lookback_safe(lookback.get().minutes())
                    }>
                        <div class="api-limit-warning">
                            <span class="warning-icon">"⚠"</span>
                            <span class="warning-text">
                                "Data limited to last "
                                {resolution.get().safe_lookback_description()}
                                " due to API constraints. Consider using "
                                {move || {
                                    // Suggest better resolution
                                    match resolution.get() {
                                        ChartResolution::M1 | ChartResolution::M5 => "15-minute or hourly",
                                        ChartResolution::M15 | ChartResolution::M30 => "hourly",
                                        _ => "daily"
                                    }
                                }}
                                " resolution for longer periods."
                            </span>
                        </div>
                    </Show>

                    // INFO BANNER: Show when historical snapshots are still building
                    <Show when=move || {
                        match chart_data.get() {
                            ChartDataResponse { snapshot_count, .. } if snapshot_count < 10 => true,
                            _ => false
                        }
                    }>
                        <div class="snapshot-info-banner">
                            <span class="info-icon">"ℹ️"</span>
                            <span class="info-text">
                                {move || {
                                    let count = chart_data.get().snapshot_count;
                                    if count == 0 {
                                        "Historical data collecting... Charts will show full trends in a few minutes (currently showing latest values only)".to_string()
                                    } else {
                                        format!("Building historical data: {} snapshots collected. Full charts available in ~{} minutes.", count, 10 - count)
                                    }
                                }}
                            </span>
                        </div>
                    </Show>

                    <div class="resolution-buttons">
                        <For
                            each={move || ChartResolution::all().iter().copied()}
                            key=|r| r.label()
                            children=move |r| {
                                let is_active = move || resolution.get() == r;
                                view! {
                                    <button
                                        class=move || if is_active() { "res-btn active" } else { "res-btn" }
                                        on:click=move |_| {
                                            // Only update if different from current selection
                                            if resolution.get() != r {
                                                resolution.set(r);
                                            }
                                        }
                                    >
                                        {r.label()}
                                    </button>
                                }
                            }
                        />
                    </div>

                    <div class="lookback-buttons">
                        // Show either preset buttons or "Custom" label based on active state
                        <Show
                            when=move || !is_custom_range_active()
                            fallback=move || view! {
                                <button
                                    class="lb-btn active custom-range-active"
                                    on:click=move |_| {
                                        // Clear custom range to go back to presets
                                        set_custom_start.set(None);
                                        set_custom_end.set(None);
                                    }
                                    title="Click to clear custom range"
                                >
                                    "Custom"
                                </button>
                            }
                        >
                            <For
                                each={move || ChartLookback::all().iter().copied()}
                                key=|lb| lb.label()
                                children=move |lb| {
                                    let is_active = move || lookback.get() == lb && !is_custom_range_active();
                                    view! {
                                        <button
                                            class=move || if is_active() { "lb-btn active" } else { "lb-btn" }
                                            on:click=move |_| {
                                                // Only update if different from current selection
                                                if lookback.get() != lb || is_custom_range_active() {
                                                    lookback.set(lb);
                                                    // Clear custom range when using presets
                                                    set_custom_start.set(None);
                                                    set_custom_end.set(None);
                                                }
                                            }
                                        >
                                            {lb.label()}
                                        </button>
                                    }
                                }
                            />
                        </Show>

                        // Date picker toggle button
                        <div class="date-picker-container">
                            <button
                                class=move || if show_date_picker.get() { "date-picker-btn active" } else { "date-picker-btn" }
                                on:click=move |_| set_show_date_picker.update(|v| *v = !*v)
                                title="Custom date range"
                            >
                                <svg viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
                                    <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zM4 8h12v8H4V8z" clip-rule="evenodd"/>
                                </svg>
                            </button>

                            // Date picker popup
                            <Show when=move || show_date_picker.get()>
                                <DateRangePicker
                                    custom_start=custom_start
                                    custom_end=custom_end
                                    set_custom_start=set_custom_start
                                    set_custom_end=set_custom_end
                                    set_show_date_picker=set_show_date_picker
                                    lookback=lookback
                                />
                            </Show>
                        </div>
                    </div>

                    <div class="fetch-indicator">
                        <span class="fetch-time">{move || format!("{}ms", chart_data.get().fetch_time_ms)}</span>
                    </div>
                </div>

                // Metric Legend
                <div class="lz-legend">
                    {[
                        (ChartMetric::Price, "#00d4ff", "Price"),
                        (ChartMetric::Volume, "#8b5cf6", "Volume"),
                        (ChartMetric::Liquidity, "#06b6d4", "Liquidity"),
                        (ChartMetric::TCR, "#22c55e", "TCR"),
                        (ChartMetric::Supply, "#f59e0b", "Supply"),
                        (ChartMetric::Holders, "#ec4899", "Holders"),
                        (ChartMetric::LendAPR, "#10b981", "Lend APR"),
                        (ChartMetric::BorrowAPR, "#f97316", "Borrow APR"),
                        (ChartMetric::Transfers, "#6366f1", "Transfers"),
                    ].into_iter().map(|(metric, color, label)| {
                        let is_active = move || visible_metrics.get().contains(&metric);
                        view! {
                            <button
                                class=move || if is_active() { "legend-chip active" } else { "legend-chip" }
                                on:click=move |_| toggle_metric(metric)
                            >
                                <span class="legend-dot" style=format!("background: {}", color)></span>
                                <span class="legend-text">{label}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Stats Grid
            <div class="lz-stats-grid">
                // Price Card - handles Option<f64> safely
                <Suspense fallback=move || view! { <LzStatLoading label="USDFC Price" /> }>
                    {move || {
                        price.get().map(|res| match res {
                            Ok(p) => {
                                let change = p.price_change_24h.unwrap_or(0.0);
                                let is_positive = change >= 0.0;
                                let price_display = p.price_usd.map(|v| format!("${:.4}", v)).unwrap_or_else(|| "Error".to_string());
                                let change_display = p.price_change_24h.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "--".to_string());
                                view! {
                                    <div class="lz-stat-card">
                                        <div class="lz-stat-header">
                                            <span class="lz-stat-icon">"$"</span>
                                            <span class="lz-stat-source">"GeckoTerminal"</span>
                                        </div>
                                        <div class="lz-stat-label">"USDFC Price"</div>
                                        <div class="lz-stat-row">
                                            <span class="lz-stat-value">{price_display}</span>
                                            <span class=if is_positive { "lz-stat-change up" } else { "lz-stat-change down" }>
                                                {if is_positive { "+" } else { "" }}{change_display}
                                            </span>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <LzStatError label="USDFC Price" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Volume Card - handles Option<f64> safely
                <Suspense fallback=move || view! { <LzStatLoading label="24h Volume" /> }>
                    {move || {
                        price.get().map(|res| match res {
                            Ok(p) => view! {
                                <div class="lz-stat-card">
                                    <div class="lz-stat-header">
                                        <span class="lz-stat-icon">"V"</span>
                                        <span class="lz-stat-source">"GeckoTerminal"</span>
                                    </div>
                                    <div class="lz-stat-label">"24h Volume"</div>
                                    <div class="lz-stat-row">
                                        <span class="lz-stat-value">{p.volume_24h.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</span>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(_) => view! { <LzStatError label="24h Volume" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Liquidity Card - handles Option<f64> safely
                <Suspense fallback=move || view! { <LzStatLoading label="Liquidity" /> }>
                    {move || {
                        price.get().map(|res| match res {
                            Ok(p) => view! {
                                <div class="lz-stat-card">
                                    <div class="lz-stat-header">
                                        <span class="lz-stat-icon">"L"</span>
                                        <span class="lz-stat-source">"GeckoTerminal"</span>
                                    </div>
                                    <div class="lz-stat-label">"DEX Liquidity"</div>
                                    <div class="lz-stat-row">
                                        <span class="lz-stat-value">{p.liquidity_usd.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</span>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(_) => view! { <LzStatError label="Liquidity" /> }.into_view()
                        })
                    }}
                </Suspense>

                // TCR Card
                <Suspense fallback=move || view! { <LzStatLoading label="System TCR" /> }>
                    {move || {
                        protocol.get().map(|res| match res {
                            Ok(m) => {
                                let tcr = decimal_to_f64(m.tcr);
                                let status = if tcr < 125.0 { "critical" } else if tcr < config().tcr_danger_threshold { "warning" } else { "healthy" };
                                view! {
                                    <div class="lz-stat-card">
                                        <div class="lz-stat-header">
                                            <span class="lz-stat-icon">"T"</span>
                                            <span class="lz-stat-source">"Filecoin RPC"</span>
                                        </div>
                                        <div class="lz-stat-label">"System TCR"</div>
                                        <div class="lz-stat-row">
                                            <span class=format!("lz-stat-value {}", status)>{format!("{:.1}%", tcr)}</span>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <LzStatError label="System TCR" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Supply Card
                <Suspense fallback=move || view! { <LzStatLoading label="Total Supply" /> }>
                    {move || {
                        protocol.get().map(|res| match res {
                            Ok(m) => view! {
                                <div class="lz-stat-card">
                                    <div class="lz-stat-header">
                                        <span class="lz-stat-icon">"S"</span>
                                        <span class="lz-stat-source">"Filecoin RPC"</span>
                                    </div>
                                    <div class="lz-stat-label">"Total Supply"</div>
                                    <div class="lz-stat-row">
                                        <span class="lz-stat-value">{format_volume(decimal_to_f64(m.total_supply))}</span>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(_) => view! { <LzStatError label="Total Supply" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Holders Card
                <Suspense fallback=move || view! { <LzStatLoading label="Holders" /> }>
                    {move || {
                        holders.get().map(|res| match res {
                            Ok(count) => view! {
                                <div class="lz-stat-card">
                                    <div class="lz-stat-header">
                                        <span class="lz-stat-icon">"H"</span>
                                        <span class="lz-stat-source">"Blockscout"</span>
                                    </div>
                                    <div class="lz-stat-label">"Token Holders"</div>
                                    <div class="lz-stat-row">
                                        <span class="lz-stat-value">{format_count(count as usize)}</span>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(_) => view! { <LzStatError label="Holders" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Markets Card
                <Suspense fallback=move || view! { <LzStatLoading label="Markets" /> }>
                    {move || {
                        lending.get().map(|res| match res {
                            Ok(markets) => view! {
                                <div class="lz-stat-card">
                                    <div class="lz-stat-header">
                                        <span class="lz-stat-icon">"M"</span>
                                        <span class="lz-stat-source">"Secured Finance"</span>
                                    </div>
                                    <div class="lz-stat-label">"Active Markets"</div>
                                    <div class="lz-stat-row">
                                        <span class="lz-stat-value">{format_count(markets.len())}</span>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(_) => view! { <LzStatError label="Markets" /> }.into_view()
                        })
                    }}
                </Suspense>

                // Transfers Card
                <Suspense fallback=move || view! { <LzStatLoading label="Transfers" /> }>
                    {move || {
                        transactions.get().map(|res| match res {
                            Ok(txs) => {
                                let vol: f64 = txs.iter().map(|tx| decimal_to_f64(tx.amount)).sum();
                                view! {
                                    <div class="lz-stat-card">
                                        <div class="lz-stat-header">
                                            <span class="lz-stat-icon">"X"</span>
                                            <span class="lz-stat-source">"Blockscout"</span>
                                        </div>
                                        <div class="lz-stat-label">"Recent Transfer Vol"</div>
                                        <div class="lz-stat-row">
                                            <span class="lz-stat-value">{format_volume(vol)}</span>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <LzStatError label="Transfers" /> }.into_view()
                        })
                    }}
                </Suspense>
            </div>

            // Data Sources Status
            <div class="lz-sources-section">
                <h2 class="lz-section-title">"Data Sources"</h2>
                <Suspense fallback=move || view! { <div class="lz-loading">"Checking connections..."</div> }>
                    {move || {
                        health.get().map(|res| match res {
                            Ok(h) => view! {
                                <div class="lz-sources-grid">
                                    <LzSource name="Filecoin RPC" endpoint="api.node.glif.io" connected=h.rpc_ok />
                                    <LzSource name="Blockscout API" endpoint="filecoin.blockscout.com" connected=h.blockscout_ok />
                                    <LzSource name="Secured Finance" endpoint="api.goldsky.com" connected=h.subgraph_ok />
                                    <LzSource name="GeckoTerminal" endpoint="api.geckoterminal.com" connected=h.gecko_ok />
                                    <LzSource name="History DB" endpoint="metrics_history.db" connected=h.database_ok />
                                </div>
                            }.into_view(),
                            Err(_) => view! { <div class="lz-error">"Failed to check sources"</div> }.into_view()
                        })
                    }}
                </Suspense>
            </div>

            // MEV Analysis - Coming Soon
            <div class="lz-mev-section" style="margin-top: 24px;">
                <div class="card" style="background: var(--bg-tertiary); border: 1px dashed var(--border-color);">
                    <div style="display: flex; align-items: center; gap: 16px; padding: 24px;">
                        <div style="font-size: 32px; opacity: 0.5;">"⚡"</div>
                        <div style="flex: 1;">
                            <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 8px;">
                                <h3 style="color: var(--text-primary); margin: 0;">"MEV Analysis"</h3>
                                <span style="background: var(--accent-purple); color: white; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;">"Coming Soon v2"</span>
                            </div>
                            <p style="color: var(--text-muted); margin: 0; font-size: 13px;">
                                "Analyze MEV opportunities, sandwich attacks, arbitrage detection, and transaction ordering on USDFC markets. Track frontrunning and backrunning patterns across DEX pools."
                            </p>
                        </div>
                    </div>
                    <div style="padding: 0 24px 24px; display: flex; gap: 24px; flex-wrap: wrap;">
                        <div style="display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-size: 12px;">
                            <span style="color: var(--accent-cyan);">"✓"</span>
                            "Sandwich Attack Detection"
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-size: 12px;">
                            <span style="color: var(--accent-cyan);">"✓"</span>
                            "Arbitrage Opportunity Tracking"
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-size: 12px;">
                            <span style="color: var(--accent-cyan);">"✓"</span>
                            "Gas Price Anomaly Detection"
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-size: 12px;">
                            <span style="color: var(--accent-cyan);">"✓"</span>
                            "Block Builder Analytics"
                        </div>
                    </div>
                </div>
            </div>

            // Toast notification
            <Show when=move || show_toast.get()>
                <div class="share-toast">
                    <svg viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                    </svg>
                    <span>{move || toast_message.get()}</span>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn LzStatLoading(label: &'static str) -> impl IntoView {
    view! {
        <div class="lz-stat-card loading">
            <div class="lz-stat-header">
                <span class="lz-stat-icon">"?"</span>
                <span class="lz-stat-source">"--"</span>
            </div>
            <div class="lz-stat-label">{label}</div>
            <div class="lz-stat-row">
                <span class="lz-stat-value skeleton"></span>
            </div>
        </div>
    }
}

#[component]
fn LzStatError(label: &'static str) -> impl IntoView {
    view! {
        <div class="lz-stat-card error">
            <div class="lz-stat-header">
                <span class="lz-stat-icon">"!"</span>
                <span class="lz-stat-source">"Error"</span>
            </div>
            <div class="lz-stat-label">{label}</div>
            <div class="lz-stat-row">
                <span class="lz-stat-value">"--"</span>
            </div>
        </div>
    }
}

#[component]
fn LzSource(name: &'static str, endpoint: &'static str, connected: bool) -> impl IntoView {
    view! {
        <div class="lz-source-card">
            <div class=if connected { "lz-source-dot online" } else { "lz-source-dot offline" }></div>
            <div class="lz-source-info">
                <div class="lz-source-name">{name}</div>
                <div class="lz-source-endpoint">{endpoint}</div>
            </div>
            <div class=if connected { "lz-source-status online" } else { "lz-source-status offline" }>
                {if connected { "Connected" } else { "Offline" }}
            </div>
        </div>
    }
}

/// DateRangePicker component for custom date range selection
#[allow(unused_variables)]
#[component]
fn DateRangePicker(
    custom_start: ReadSignal<Option<i64>>,
    custom_end: ReadSignal<Option<i64>>,
    set_custom_start: WriteSignal<Option<i64>>,
    set_custom_end: WriteSignal<Option<i64>>,
    set_show_date_picker: WriteSignal<bool>,
    lookback: RwSignal<ChartLookback>,
) -> impl IntoView {
    // Current view month/year for calendar navigation
    let (view_year, set_view_year) = create_signal(2024i32);
    let (view_month, set_view_month) = create_signal(12u32); // 1-12

    // Selection state: start date being selected, then end date
    let (selecting_start, set_selecting_start) = create_signal(true);
    let (temp_start, set_temp_start) = create_signal(None::<i64>);
    let (temp_end, set_temp_end) = create_signal(None::<i64>);

    // Initialize view to current month (client-side)
    #[cfg(feature = "hydrate")]
    {
        create_effect(move |_| {
            let now = js_sys::Date::new_0();
            set_view_year.set(now.get_full_year() as i32);
            set_view_month.set(now.get_month() as u32 + 1);
        });
    }

    // Get current timestamp for validation (no future dates)
    let get_now_timestamp = move || -> i64 {
        #[cfg(feature = "hydrate")]
        {
            (js_sys::Date::now() / 1000.0) as i64
        }
        #[cfg(not(feature = "hydrate"))]
        {
            0
        }
    };

    // Days in month helper
    let days_in_month = move |year: i32, month: u32| -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    };

    // First day of month (0 = Sunday, 1 = Monday, etc.)
    let first_day_of_month = move |year: i32, month: u32| -> u32 {
        #[cfg(feature = "hydrate")]
        {
            let date = js_sys::Date::new_with_year_month_day(year as u32, month as i32 - 1, 1);
            date.get_day()
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (year, month);
            0
        }
    };

    // Convert date to timestamp (start of day UTC)
    let date_to_timestamp = move |year: i32, month: u32, day: u32| -> i64 {
        #[cfg(feature = "hydrate")]
        {
            let date = js_sys::Date::new_with_year_month_day(year as u32, month as i32 - 1, day as i32);
            (date.get_time() / 1000.0) as i64
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (year, month, day);
            0
        }
    };

    // Format timestamp to display string
    let format_date = move |ts: i64| -> String {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsValue;
            let date = js_sys::Date::new(&JsValue::from_f64(ts as f64 * 1000.0));
            format!(
                "{:02}/{:02}/{}",
                date.get_month() + 1,
                date.get_date(),
                date.get_full_year()
            )
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = ts;
            String::new()
        }
    };

    // Navigate months
    let prev_month = move |_| {
        let m = view_month.get();
        let y = view_year.get();
        if m == 1 {
            set_view_month.set(12);
            set_view_year.set(y - 1);
        } else {
            set_view_month.set(m - 1);
        }
    };

    let next_month = move |_| {
        let m = view_month.get();
        let y = view_year.get();
        if m == 12 {
            set_view_month.set(1);
            set_view_year.set(y + 1);
        } else {
            set_view_month.set(m + 1);
        }
    };

    // Month names
    let month_name = move |m: u32| -> &'static str {
        match m {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        }
    };

    // Apply quick preset
    let apply_preset = move |preset: &str| {
        let now = get_now_timestamp();
        let day_secs = 86400i64;
        let (start, end) = match preset {
            "today" => (now - day_secs, now),
            "1w" => (now - 7 * day_secs, now),
            "1m" => (now - 30 * day_secs, now),
            "3m" => (now - 90 * day_secs, now),
            "all" => (now - 365 * day_secs, now), // Approx 1 year for "all"
            _ => (now - 7 * day_secs, now),
        };
        set_custom_start.set(Some(start));
        set_custom_end.set(Some(end));
        set_show_date_picker.set(false);
    };

    // Handle date click
    let on_date_click = move |day: u32| {
        let year = view_year.get();
        let month = view_month.get();
        let ts = date_to_timestamp(year, month, day);
        let now = get_now_timestamp();

        // Don't allow future dates
        if ts > now {
            return;
        }

        if selecting_start.get() {
            set_temp_start.set(Some(ts));
            set_temp_end.set(None);
            set_selecting_start.set(false);
        } else {
            let start = temp_start.get().unwrap_or(ts);
            // Ensure start < end
            if ts < start {
                set_temp_start.set(Some(ts));
                set_temp_end.set(Some(start));
            } else {
                set_temp_end.set(Some(ts));
            }
            set_selecting_start.set(true);
        }
    };

    // Apply selection
    let apply_selection = move |_| {
        if let (Some(start), Some(end)) = (temp_start.get(), temp_end.get()) {
            set_custom_start.set(Some(start));
            set_custom_end.set(Some(end));
            set_show_date_picker.set(false);
        }
    };

    // Cancel and close
    let cancel = move |_| {
        set_temp_start.set(None);
        set_temp_end.set(None);
        set_selecting_start.set(true);
        set_show_date_picker.set(false);
    };

    // Check if a date is in selection range
    let is_in_range = move |day: u32| -> bool {
        let ts = date_to_timestamp(view_year.get(), view_month.get(), day);
        match (temp_start.get(), temp_end.get()) {
            (Some(start), Some(end)) => ts >= start && ts <= end,
            (Some(start), None) => ts == start,
            _ => false,
        }
    };

    let is_start_date = move |day: u32| -> bool {
        let ts = date_to_timestamp(view_year.get(), view_month.get(), day);
        temp_start.get() == Some(ts)
    };

    let is_end_date = move |day: u32| -> bool {
        let ts = date_to_timestamp(view_year.get(), view_month.get(), day);
        temp_end.get() == Some(ts)
    };

    let is_future_date = move |day: u32| -> bool {
        let ts = date_to_timestamp(view_year.get(), view_month.get(), day);
        ts > get_now_timestamp()
    };

    view! {
        <div class="date-picker-popup">
            <style>
                ".date-picker-popup {
                    position: absolute;
                    top: 100%;
                    right: 0;
                    margin-top: 8px;
                    background: var(--bg-secondary, #1a1a2e);
                    border: 1px solid var(--border-color, #333);
                    border-radius: 8px;
                    padding: 16px;
                    z-index: 1000;
                    min-width: 300px;
                    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
                }
                .date-picker-popup::before {
                    content: '';
                    position: absolute;
                    top: -6px;
                    right: 12px;
                    width: 10px;
                    height: 10px;
                    background: var(--bg-secondary, #1a1a2e);
                    border-left: 1px solid var(--border-color, #333);
                    border-top: 1px solid var(--border-color, #333);
                    transform: rotate(45deg);
                }
                .dp-header {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    margin-bottom: 12px;
                }
                .dp-nav-btn {
                    background: transparent;
                    border: 1px solid var(--border-color, #333);
                    color: var(--text-primary, #fff);
                    width: 28px;
                    height: 28px;
                    border-radius: 4px;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: all 0.15s;
                }
                .dp-nav-btn:hover {
                    background: var(--bg-tertiary, #252538);
                    border-color: var(--accent-cyan, #00d4ff);
                }
                .dp-month-year {
                    font-weight: 600;
                    color: var(--text-primary, #fff);
                    font-size: 14px;
                }
                .dp-weekdays {
                    display: grid;
                    grid-template-columns: repeat(7, 1fr);
                    gap: 2px;
                    margin-bottom: 4px;
                }
                .dp-weekday {
                    text-align: center;
                    font-size: 11px;
                    color: var(--text-muted, #888);
                    padding: 4px 0;
                    font-weight: 500;
                }
                .dp-days {
                    display: grid;
                    grid-template-columns: repeat(7, 1fr);
                    gap: 2px;
                }
                .dp-day {
                    aspect-ratio: 1;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 12px;
                    border-radius: 4px;
                    cursor: pointer;
                    transition: all 0.15s;
                    color: var(--text-primary, #fff);
                    background: transparent;
                    border: none;
                }
                .dp-day:hover:not(.disabled):not(.empty) {
                    background: var(--bg-tertiary, #252538);
                }
                .dp-day.empty {
                    cursor: default;
                }
                .dp-day.disabled {
                    color: var(--text-muted, #555);
                    cursor: not-allowed;
                    opacity: 0.4;
                }
                .dp-day.in-range {
                    background: rgba(0, 212, 255, 0.15);
                }
                .dp-day.start-date,
                .dp-day.end-date {
                    background: var(--accent-cyan, #00d4ff);
                    color: #000;
                    font-weight: 600;
                }
                .dp-presets {
                    display: flex;
                    gap: 6px;
                    margin-bottom: 12px;
                    flex-wrap: wrap;
                }
                .dp-preset-btn {
                    background: var(--bg-tertiary, #252538);
                    border: 1px solid var(--border-color, #333);
                    color: var(--text-secondary, #aaa);
                    padding: 4px 10px;
                    border-radius: 4px;
                    font-size: 11px;
                    cursor: pointer;
                    transition: all 0.15s;
                }
                .dp-preset-btn:hover {
                    background: var(--accent-cyan, #00d4ff);
                    color: #000;
                    border-color: var(--accent-cyan, #00d4ff);
                }
                .dp-selection-info {
                    margin-top: 12px;
                    padding-top: 12px;
                    border-top: 1px solid var(--border-color, #333);
                    font-size: 12px;
                    color: var(--text-muted, #888);
                }
                .dp-selection-dates {
                    display: flex;
                    gap: 12px;
                    margin-bottom: 12px;
                }
                .dp-date-display {
                    flex: 1;
                    padding: 8px;
                    background: var(--bg-tertiary, #252538);
                    border-radius: 4px;
                    text-align: center;
                }
                .dp-date-label {
                    font-size: 10px;
                    color: var(--text-muted, #888);
                    margin-bottom: 2px;
                    text-transform: uppercase;
                }
                .dp-date-value {
                    font-size: 12px;
                    color: var(--text-primary, #fff);
                    font-weight: 500;
                }
                .dp-actions {
                    display: flex;
                    gap: 8px;
                    justify-content: flex-end;
                }
                .dp-action-btn {
                    padding: 6px 14px;
                    border-radius: 4px;
                    font-size: 12px;
                    cursor: pointer;
                    transition: all 0.15s;
                }
                .dp-action-btn.cancel {
                    background: transparent;
                    border: 1px solid var(--border-color, #333);
                    color: var(--text-secondary, #aaa);
                }
                .dp-action-btn.cancel:hover {
                    border-color: var(--text-secondary, #aaa);
                }
                .dp-action-btn.apply {
                    background: var(--accent-cyan, #00d4ff);
                    border: 1px solid var(--accent-cyan, #00d4ff);
                    color: #000;
                    font-weight: 500;
                }
                .dp-action-btn.apply:hover {
                    filter: brightness(1.1);
                }
                .dp-action-btn.apply:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                }
                .date-picker-container {
                    position: relative;
                }
                .date-picker-btn {
                    background: transparent;
                    border: 1px solid var(--border-color, #333);
                    color: var(--text-secondary, #aaa);
                    padding: 6px 8px;
                    border-radius: 4px;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: all 0.15s;
                }
                .date-picker-btn:hover,
                .date-picker-btn.active {
                    background: var(--bg-tertiary, #252538);
                    border-color: var(--accent-cyan, #00d4ff);
                    color: var(--accent-cyan, #00d4ff);
                }
                .custom-range-active {
                    background: rgba(0, 212, 255, 0.15) !important;
                    border-color: var(--accent-cyan, #00d4ff) !important;
                    color: var(--accent-cyan, #00d4ff) !important;
                }
                .dp-hint {
                    font-size: 11px;
                    color: var(--text-muted, #666);
                    margin-bottom: 8px;
                    text-align: center;
                }"
            </style>

            // Quick presets
            <div class="dp-presets">
                <button class="dp-preset-btn" on:click=move |_| apply_preset("today")>"Today"</button>
                <button class="dp-preset-btn" on:click=move |_| apply_preset("1w")>"1W"</button>
                <button class="dp-preset-btn" on:click=move |_| apply_preset("1m")>"1M"</button>
                <button class="dp-preset-btn" on:click=move |_| apply_preset("3m")>"3M"</button>
                <button class="dp-preset-btn" on:click=move |_| apply_preset("all")>"All"</button>
            </div>

            // Calendar header
            <div class="dp-header">
                <button class="dp-nav-btn" on:click=prev_month>
                    <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14">
                        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd"/>
                    </svg>
                </button>
                <span class="dp-month-year">
                    {move || format!("{} {}", month_name(view_month.get()), view_year.get())}
                </span>
                <button class="dp-nav-btn" on:click=next_month>
                    <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14">
                        <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                    </svg>
                </button>
            </div>

            // Hint for selection
            <div class="dp-hint">
                {move || if selecting_start.get() {
                    "Select start date"
                } else {
                    "Select end date"
                }}
            </div>

            // Weekday headers
            <div class="dp-weekdays">
                <span class="dp-weekday">"Su"</span>
                <span class="dp-weekday">"Mo"</span>
                <span class="dp-weekday">"Tu"</span>
                <span class="dp-weekday">"We"</span>
                <span class="dp-weekday">"Th"</span>
                <span class="dp-weekday">"Fr"</span>
                <span class="dp-weekday">"Sa"</span>
            </div>

            // Calendar days
            <div class="dp-days">
                {move || {
                    let year = view_year.get();
                    let month = view_month.get();
                    let total_days = days_in_month(year, month);
                    let first_day = first_day_of_month(year, month);

                    let mut cells = Vec::new();

                    // Empty cells for days before month starts
                    for _ in 0..first_day {
                        cells.push(view! { <button class="dp-day empty"></button> }.into_view());
                    }

                    // Days of the month
                    for day in 1..=total_days {
                        let day_val = day;
                        let is_disabled = is_future_date(day);
                        let in_range = is_in_range(day);
                        let is_start = is_start_date(day);
                        let is_end = is_end_date(day);

                        let class_str = format!(
                            "dp-day{}{}{}{}",
                            if is_disabled { " disabled" } else { "" },
                            if in_range { " in-range" } else { "" },
                            if is_start { " start-date" } else { "" },
                            if is_end { " end-date" } else { "" }
                        );

                        cells.push(view! {
                            <button
                                class=class_str
                                on:click=move |_| on_date_click(day_val)
                                disabled=is_disabled
                            >
                                {day}
                            </button>
                        }.into_view());
                    }

                    cells.collect_view()
                }}
            </div>

            // Selection display and actions
            <div class="dp-selection-info">
                <div class="dp-selection-dates">
                    <div class="dp-date-display">
                        <div class="dp-date-label">"Start"</div>
                        <div class="dp-date-value">
                            {move || temp_start.get().map(format_date).unwrap_or_else(|| "--/--/----".to_string())}
                        </div>
                    </div>
                    <div class="dp-date-display">
                        <div class="dp-date-label">"End"</div>
                        <div class="dp-date-value">
                            {move || temp_end.get().map(format_date).unwrap_or_else(|| "--/--/----".to_string())}
                        </div>
                    </div>
                </div>

                <div class="dp-actions">
                    <button class="dp-action-btn cancel" on:click=cancel>"Cancel"</button>
                    <button
                        class="dp-action-btn apply"
                        on:click=apply_selection
                        attr:disabled=move || temp_start.get().is_none() || temp_end.get().is_none()
                    >
                        "Apply"
                    </button>
                </div>
            </div>
        </div>
    }
}
