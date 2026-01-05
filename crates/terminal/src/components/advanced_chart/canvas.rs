//! Chart Canvas - SVG rendering for chart data with all metrics

use leptos::*;
use usdfc_core::types::{
    ChartResolution, ChartMetric, ChartType, ChartDataResponse,
};
use std::collections::HashSet;

/// Chart canvas dimensions
const CHART_WIDTH: f64 = 800.0;
const CHART_HEIGHT: f64 = 400.0;
const VOLUME_HEIGHT: f64 = 80.0; // Bottom section for volume bars
const PADDING_TOP: f64 = 20.0;
const PADDING_RIGHT: f64 = 70.0;
const PADDING_BOTTOM: f64 = 30.0;
const PADDING_LEFT: f64 = 10.0;

/// Chart canvas component that renders SVG charts with all metrics
#[allow(unused_variables)]
#[component]
pub fn ChartCanvas(
    data: ReadSignal<ChartDataResponse>,
    resolution: ReadSignal<ChartResolution>,
    chart_type: ReadSignal<ChartType>,
    visible_metrics: ReadSignal<HashSet<ChartMetric>>,
    mouse_pos: RwSignal<Option<(f64, f64)>>,
    hovered_index: RwSignal<Option<usize>>,
    is_loading: ReadSignal<bool>,
) -> impl IntoView {
    // Calculate drawing areas
    let price_draw_height = CHART_HEIGHT - PADDING_TOP - PADDING_BOTTOM - VOLUME_HEIGHT;
    let draw_width = CHART_WIDTH - PADDING_LEFT - PADDING_RIGHT;
    let volume_top = CHART_HEIGHT - PADDING_BOTTOM - VOLUME_HEIGHT;

    // Compute price range from candles
    let price_range = create_memo(move |_| {
        let candles = &data.get().price_candles;
        if candles.is_empty() {
            return (0.98, 1.02); // Default range for stablecoin
        }
        let min = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let max = candles.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max - min) * 0.1;
        ((min - padding).max(0.0), max + padding)
    });

    // Compute volume range
    let volume_range = create_memo(move |_| {
        let candles = &data.get().price_candles;
        if candles.is_empty() {
            return (0.0, 1.0);
        }
        let max = candles.iter().map(|c| c.volume).fold(0.0_f64, f64::max);
        (0.0, max * 1.1)
    });

    // Generate Y-axis labels for price
    let y_labels = create_memo(move |_| {
        let (min, max) = price_range.get();
        let step = (max - min) / 5.0;
        (0..=5).map(|i| min + step * i as f64).collect::<Vec<_>>()
    });

    // Scale price to Y coordinate
    let price_to_y = move |price: f64| {
        let (min, max) = price_range.get();
        if (max - min).abs() < 0.0001 {
            return PADDING_TOP + price_draw_height / 2.0;
        }
        let ratio = (price - min) / (max - min);
        PADDING_TOP + price_draw_height * (1.0 - ratio)
    };

    // Scale volume to Y coordinate (in volume section)
    let volume_to_y = move |volume: f64| {
        let (_, max) = volume_range.get();
        if max < 0.0001 {
            return CHART_HEIGHT - PADDING_BOTTOM;
        }
        let ratio = volume / max;
        volume_top + VOLUME_HEIGHT * (1.0 - ratio)
    };

    // Scale index to X coordinate
    let index_to_x = move |index: usize, total: usize| {
        if total <= 1 {
            return PADDING_LEFT + draw_width / 2.0;
        }
        PADDING_LEFT + (index as f64 / (total - 1) as f64) * draw_width
    };

    // Generate area path for price series
    let area_path = create_memo(move |_| {
        let candles = &data.get().price_candles;
        if candles.is_empty() {
            return String::new();
        }

        let total = candles.len();
        let mut path = String::new();

        // Start from bottom-left of price area
        path.push_str(&format!("M{},{}", index_to_x(0, total), volume_top));

        // Line to first point
        path.push_str(&format!(" L{},{}", index_to_x(0, total), price_to_y(candles[0].close)));

        // Draw top line
        for (i, candle) in candles.iter().enumerate() {
            let x = index_to_x(i, total);
            let y = price_to_y(candle.close);
            path.push_str(&format!(" L{:.2},{:.2}", x, y));
        }

        // Close to bottom-right and back
        path.push_str(&format!(" L{},{}", index_to_x(total - 1, total), volume_top));
        path.push_str(" Z");

        path
    });

    // Generate line path for price series
    let line_path = create_memo(move |_| {
        let candles = &data.get().price_candles;
        if candles.is_empty() {
            return String::new();
        }

        let total = candles.len();
        let mut path = String::new();

        for (i, candle) in candles.iter().enumerate() {
            let x = index_to_x(i, total);
            let y = price_to_y(candle.close);
            if i == 0 {
                path.push_str(&format!("M{:.2},{:.2}", x, y));
            } else {
                path.push_str(&format!(" L{:.2},{:.2}", x, y));
            }
        }

        path
    });

    // Handle mouse move
    let on_mouse_move = move |ev: ev::MouseEvent| {
        let x = ev.client_x() as f64;
        let y = ev.client_y() as f64;
        mouse_pos.set(Some((x, y)));

        let candles = &data.get().price_candles;
        let total = candles.len();
        if total > 0 {
            let relative_x = (x - PADDING_LEFT) / draw_width;
            let idx = ((relative_x.max(0.0) * (total - 1) as f64).round() as usize).min(total - 1);
            hovered_index.set(Some(idx));
        }
    };

    let on_mouse_leave = move |_: ev::MouseEvent| {
        mouse_pos.set(None);
        hovered_index.set(None);
    };

    view! {
        <div class="chart-canvas-wrapper">
            <svg
                class="chart-canvas"
                viewBox=format!("0 0 {} {}", CHART_WIDTH, CHART_HEIGHT)
                preserveAspectRatio="xMidYMid meet"
                on:mousemove=on_mouse_move
                on:mouseleave=on_mouse_leave
            >
                // Gradients
                <defs>
                    <linearGradient id="priceGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="#00d4ff" stop-opacity="0.4"/>
                        <stop offset="100%" stop-color="#00d4ff" stop-opacity="0.05"/>
                    </linearGradient>
                    <linearGradient id="volumeGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="#8b5cf6" stop-opacity="0.8"/>
                        <stop offset="100%" stop-color="#8b5cf6" stop-opacity="0.3"/>
                    </linearGradient>
                </defs>

                // Background
                <rect
                    x="0" y="0"
                    width=CHART_WIDTH.to_string()
                    height=CHART_HEIGHT.to_string()
                    fill="var(--chart-bg, #0a0a14)"
                />

                // Separator line between price and volume sections
                <line
                    x1=PADDING_LEFT.to_string()
                    y1=volume_top.to_string()
                    x2=(CHART_WIDTH - PADDING_RIGHT).to_string()
                    y2=volume_top.to_string()
                    stroke="var(--grid-color, #1a1a2e)"
                    stroke-width="1"
                />

                // Grid lines for price section
                <g class="grid-lines">
                    <For
                        each={move || y_labels.get().into_iter().enumerate()}
                        key=|(i, _)| *i
                        children=move |(_, price)| {
                            let y = price_to_y(price);
                            view! {
                                <line
                                    x1=PADDING_LEFT.to_string()
                                    y1=y.to_string()
                                    x2=(CHART_WIDTH - PADDING_RIGHT).to_string()
                                    y2=y.to_string()
                                    stroke="var(--grid-color, #1a1a2e)"
                                    stroke-width="1"
                                />
                                <text
                                    x=(CHART_WIDTH - PADDING_RIGHT + 5.0).to_string()
                                    y=y.to_string()
                                    fill="var(--text-muted, #6b7280)"
                                    font-size="10"
                                    dominant-baseline="middle"
                                >
                                    {format!("${:.4}", price)}
                                </text>
                            }
                        }
                    />
                </g>

                // Volume bars section
                <Show when=move || visible_metrics.get().contains(&ChartMetric::Volume)>
                    <g class="volume-bars">
                        <For
                            each={move || {
                                let candles = data.get().price_candles;
                                candles.into_iter().enumerate().collect::<Vec<_>>()
                            }}
                            key=|(i, _)| *i
                            children=move |(i, candle)| {
                                let total = data.get().price_candles.len();
                                let x = index_to_x(i, total);
                                let bar_width = if total > 1 { draw_width / total as f64 * 0.7 } else { 10.0 };
                                let bar_height = CHART_HEIGHT - PADDING_BOTTOM - volume_to_y(candle.volume);
                                let bar_y = volume_to_y(candle.volume);
                                let is_bullish = candle.close >= candle.open;

                                view! {
                                    <rect
                                        x=(x - bar_width / 2.0).to_string()
                                        y=bar_y.to_string()
                                        width=bar_width.to_string()
                                        height=bar_height.max(1.0).to_string()
                                        fill=if is_bullish { "rgba(34, 197, 94, 0.6)" } else { "rgba(239, 68, 68, 0.6)" }
                                        class="volume-bar"
                                    />
                                }
                            }
                        />
                    </g>
                </Show>

                // Price series (Area or Line based on chart_type)
                <Show when=move || visible_metrics.get().contains(&ChartMetric::Price)>
                    <Show
                        when=move || chart_type.get() == ChartType::Area
                        fallback=move || {
                            view! {
                                <path
                                    d=move || line_path.get()
                                    fill="none"
                                    stroke="#00d4ff"
                                    stroke-width="2"
                                    class="price-line"
                                />
                            }
                        }
                    >
                        <path
                            d=move || area_path.get()
                            fill="url(#priceGradient)"
                            class="price-area"
                        />
                        <path
                            d=move || line_path.get()
                            fill="none"
                            stroke="#00d4ff"
                            stroke-width="2"
                            class="price-line"
                        />
                    </Show>
                </Show>

                // Candlestick rendering
                <Show when=move || chart_type.get() == ChartType::Candle && visible_metrics.get().contains(&ChartMetric::Price)>
                    <g class="candlesticks">
                        <For
                            each={move || {
                                let candles = data.get().price_candles;
                                candles.into_iter().enumerate().collect::<Vec<_>>()
                            }}
                            key=|(i, _)| *i
                            children=move |(i, candle)| {
                                let total = data.get().price_candles.len();
                                let x = index_to_x(i, total);
                                let candle_width = if total > 1 { draw_width / total as f64 * 0.6 } else { 10.0 };

                                let is_bullish = candle.close >= candle.open;
                                let body_top = price_to_y(if is_bullish { candle.close } else { candle.open });
                                let body_bottom = price_to_y(if is_bullish { candle.open } else { candle.close });
                                let body_height = (body_bottom - body_top).max(1.0);

                                let wick_top = price_to_y(candle.high);
                                let wick_bottom = price_to_y(candle.low);

                                let color = if is_bullish { "#22c55e" } else { "#ef4444" };

                                view! {
                                    <g class="candle">
                                        // Wick
                                        <line
                                            x1=x.to_string()
                                            y1=wick_top.to_string()
                                            x2=x.to_string()
                                            y2=wick_bottom.to_string()
                                            stroke=color
                                            stroke-width="1"
                                        />
                                        // Body
                                        <rect
                                            x=(x - candle_width / 2.0).to_string()
                                            y=body_top.to_string()
                                            width=candle_width.to_string()
                                            height=body_height.to_string()
                                            fill=color
                                        />
                                    </g>
                                }
                            }
                        />
                    </g>
                </Show>

                // Current value indicators on right side (metric stats) - handle Option values
                <g class="metric-indicators">
                    // TCR indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::TCR)>
                        {move || {
                            let tcr_display = data.get().current_tcr
                                .map(|v| format!("TCR: {:.1}%", v))
                                .unwrap_or_else(|| "TCR: --".to_string());
                            let y_pos = PADDING_TOP + 15.0;
                            view! {
                                <g class="metric-indicator tcr">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#22c55e"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#22c55e" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {tcr_display}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Liquidity indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::Liquidity)>
                        {move || {
                            let liq_display = data.get().current_liquidity
                                .map(|v| format!("Liq: ${:.0}K", v / 1000.0))
                                .unwrap_or_else(|| "Liq: --".to_string());
                            let y_pos = PADDING_TOP + 30.0;
                            view! {
                                <g class="metric-indicator liquidity">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#06b6d4"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#06b6d4" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {liq_display}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Supply indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::Supply)>
                        {move || {
                            let supply_display = data.get().current_supply
                                .map(|v| format!("Supply: {:.0}K", v / 1000.0))
                                .unwrap_or_else(|| "Supply: --".to_string());
                            let y_pos = PADDING_TOP + 45.0;
                            view! {
                                <g class="metric-indicator supply">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#f59e0b"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#f59e0b" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {supply_display}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Holders indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::Holders)>
                        {move || {
                            let holders_display = data.get().current_holders
                                .map(|v| format!("Holders: {}", v))
                                .unwrap_or_else(|| "Holders: --".to_string());
                            let y_pos = PADDING_TOP + 60.0;
                            view! {
                                <g class="metric-indicator holders">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#ec4899"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#ec4899" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {holders_display}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Lend APR indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::LendAPR)>
                        {move || {
                            let apr = data.get().current_lend_apr.unwrap_or(0.0);
                            let y_pos = PADDING_TOP + 75.0;
                            view! {
                                <g class="metric-indicator lend-apr">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#10b981"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#10b981" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {format!("Lend: {:.2}%", apr)}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Borrow APR indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::BorrowAPR)>
                        {move || {
                            let apr = data.get().current_borrow_apr.unwrap_or(0.0);
                            let y_pos = PADDING_TOP + 90.0;
                            view! {
                                <g class="metric-indicator borrow-apr">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#f97316"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#f97316" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {format!("Borrow: {:.2}%", apr)}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>

                    // Transfers indicator
                    <Show when=move || visible_metrics.get().contains(&ChartMetric::Transfers)>
                        {move || {
                            let data_val = data.get();
                            let transfers = data_val.transfers_data.first().map(|(_, v)| *v).unwrap_or(0);
                            let y_pos = PADDING_TOP + 105.0;
                            view! {
                                <g class="metric-indicator transfers">
                                    <circle cx=(CHART_WIDTH - 10.0).to_string() cy=y_pos.to_string() r="4" fill="#6366f1"/>
                                    <text x=(CHART_WIDTH - 15.0).to_string() y=y_pos.to_string()
                                        fill="#6366f1" font-size="9" text-anchor="end" dominant-baseline="middle">
                                        {format!("Txs: {}", transfers)}
                                    </text>
                                </g>
                            }
                        }}
                    </Show>
                </g>

                // Crosshair on hover
                <Show when=move || mouse_pos.get().is_some()>
                    {move || {
                        let (x, y) = mouse_pos.get().unwrap_or((0.0, 0.0));
                        // Clamp x within chart bounds
                        let x_clamped = x.max(PADDING_LEFT).min(CHART_WIDTH - PADDING_RIGHT);
                        view! {
                            <g class="crosshair">
                                <line
                                    x1=x_clamped.to_string()
                                    y1=PADDING_TOP.to_string()
                                    x2=x_clamped.to_string()
                                    y2=(CHART_HEIGHT - PADDING_BOTTOM).to_string()
                                    stroke="rgba(255,255,255,0.3)"
                                    stroke-width="1"
                                    stroke-dasharray="4,4"
                                />
                                <line
                                    x1=PADDING_LEFT.to_string()
                                    y1=y.to_string()
                                    x2=(CHART_WIDTH - PADDING_RIGHT).to_string()
                                    y2=y.to_string()
                                    stroke="rgba(255,255,255,0.3)"
                                    stroke-width="1"
                                    stroke-dasharray="4,4"
                                />
                            </g>
                        }
                    }}
                </Show>

                // Hover point indicator
                <Show when=move || hovered_index.get().is_some()>
                    {move || {
                        let idx = hovered_index.get().unwrap_or(0);
                        let candles = &data.get().price_candles;
                        if idx < candles.len() {
                            let candle = &candles[idx];
                            let x = index_to_x(idx, candles.len());
                            let y = price_to_y(candle.close);
                            view! {
                                <circle
                                    cx=x.to_string()
                                    cy=y.to_string()
                                    r="5"
                                    fill="#00d4ff"
                                    stroke="white"
                                    stroke-width="2"
                                    class="hover-point"
                                />
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}
                </Show>

                // Data source label
                <text
                    x=PADDING_LEFT.to_string()
                    y=(CHART_HEIGHT - 5.0).to_string()
                    fill="var(--text-muted, #4b5563)"
                    font-size="8"
                >
                    "Data: GeckoTerminal, Blockscout, Secured Finance"
                </text>
            </svg>

            // Loading overlay
            <Show when=move || is_loading.get()>
                <div class="chart-loading-overlay">
                    <div class="chart-spinner"></div>
                    <span class="loading-text">"Loading real-time data..."</span>
                </div>
            </Show>

            // Tooltip
            <Show when=move || hovered_index.get().is_some() && mouse_pos.get().is_some()>
                {move || {
                    let idx = hovered_index.get().unwrap_or(0);
                    let (mx, my) = mouse_pos.get().unwrap_or((0.0, 0.0));
                    let candles = &data.get().price_candles;

                    if idx < candles.len() {
                        let candle = &candles[idx];
                        let is_bullish = candle.close >= candle.open;
                        let change_pct = if candle.open > 0.0 {
                            (candle.close - candle.open) / candle.open * 100.0
                        } else {
                            0.0
                        };

                        view! {
                            <div
                                class="chart-tooltip"
                                style=format!("left: {}px; top: {}px;", mx + 15.0, my - 80.0)
                            >
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"O:"</span>
                                    <span class="tooltip-value">{format!("${:.4}", candle.open)}</span>
                                </div>
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"H:"</span>
                                    <span class="tooltip-value highlight-high">{format!("${:.4}", candle.high)}</span>
                                </div>
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"L:"</span>
                                    <span class="tooltip-value highlight-low">{format!("${:.4}", candle.low)}</span>
                                </div>
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"C:"</span>
                                    <span class="tooltip-value">{format!("${:.4}", candle.close)}</span>
                                </div>
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"Vol:"</span>
                                    <span class="tooltip-value">{format_volume(candle.volume)}</span>
                                </div>
                                <div class="tooltip-row">
                                    <span class="tooltip-label">"Chg:"</span>
                                    <span class=format!("tooltip-value {}", if is_bullish { "positive" } else { "negative" })>
                                        {format!("{:+.2}%", change_pct)}
                                    </span>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }
                }}
            </Show>
        </div>
    }
}

/// Format volume with K/M suffix
fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("${:.2}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format!("${:.2}K", volume / 1_000.0)
    } else {
        format!("${:.2}", volume)
    }
}
