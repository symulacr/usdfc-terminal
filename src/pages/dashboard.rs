//! Dashboard Page - Redesigned with Viewport-Locked Layout
//!
//! Features:
//! - No-scroll single screen design
//! - Functional time range that updates all data
//! - Split layout with chart and sidebar
//! - Inline stats in header

use leptos::*;
use crate::components::controls::{TimeRange, TimeRangeSelector, ChartTypeSelector, StatusLevel};
use crate::components::{MetricRowSkeleton, ActivityItemSkeleton, ChartSkeleton};
use crate::config::config;
use crate::server_fn::{
    get_protocol_metrics, get_usdfc_price_data, get_daily_volumes,
    get_recent_transactions, check_api_health,
};
use crate::format::{format_volume, format_usd_compact, decimal_to_f64};

#[component]
pub fn Dashboard() -> impl IntoView {
    // === STATE MANAGEMENT ===
    // Time range is the SOURCE OF TRUTH - changes here update all data
    let time_range = create_rw_signal(TimeRange::Day7);
    let chart_type = create_rw_signal("area".to_string());

    // === DATA RESOURCES - Reactive to time_range ===
    // Protocol metrics (always fetched)
    let protocol = create_resource(|| (), |_| async move {
        get_protocol_metrics().await
    });

    // Price data (always fetched)
    let price = create_resource(|| (), |_| async move {
        get_usdfc_price_data().await
    });

    // Volume data - REACTIVE to time_range
    let volumes = create_resource(
        move || time_range.get().days(),
        |days| async move {
            get_daily_volumes(days).await
        }
    );

    // Recent transactions - REACTIVE to time_range
    let transactions = create_resource(
        move || time_range.get(),
        |range| async move {
            let limit = match range {
                TimeRange::Hour1 | TimeRange::Hour6 => Some(20),
                TimeRange::Hour24 => Some(50),
                TimeRange::Day7 => Some(100),
                _ => Some(200),
            };
            get_recent_transactions(limit).await
        }
    );

    // API health check
    let health = create_resource(|| (), |_| async move {
        check_api_health().await
    });

    // === DERIVED SIGNALS ===
    // Handle Option<f64> prices - show "--" or "Error" when unavailable
    let price_display = create_memo(move |_| {
        price.get()
            .and_then(|r| r.ok())
            .and_then(|p| p.price_usd)  // Handle Option<f64>
            .map(|v| format!("${:.4}", v))
            .unwrap_or_else(|| "--".to_string())
    });

    let price_change = create_memo(move |_| {
        price.get()
            .and_then(|r| r.ok())
            .and_then(|p| p.price_change_24h)  // Handle Option<f64>
            .unwrap_or(0.0)  // 0.0 is safe for change display
    });

    // TCR display - returns Option to differentiate "not loaded" from "actual zero"
    let tcr_display = create_memo(move |_| {
        protocol.get()
            .and_then(|r| r.ok())
            .map(|m| decimal_to_f64(m.tcr))
    });

    let tcr_status = create_memo(move |_| {
        match tcr_display.get() {
            Some(tcr) if tcr < 125.0 => "negative",
            Some(tcr) if tcr < config().tcr_danger_threshold => "warning",
            Some(_) => "positive",
            None => "", // No status class when data not loaded
        }
    });

    let source_status = create_memo(move |_| {
        health.get()
            .and_then(|r| r.ok())
            .map(|h| vec![
                ("RPC", if h.rpc_ok { StatusLevel::Online } else { StatusLevel::Offline }),
                ("Blockscout", if h.blockscout_ok { StatusLevel::Online } else { StatusLevel::Offline }),
                ("Subgraph", if h.subgraph_ok { StatusLevel::Online } else { StatusLevel::Offline }),
                ("Gecko", StatusLevel::Online),
            ])
            .unwrap_or_else(|| vec![
                ("RPC", StatusLevel::Unknown),
                ("Blockscout", StatusLevel::Unknown),
                ("Subgraph", StatusLevel::Unknown),
                ("Gecko", StatusLevel::Unknown),
            ])
    });

    // Refresh all data
    let refresh_all = move || {
        protocol.refetch();
        price.refetch();
        volumes.refetch();
        transactions.refetch();
        health.refetch();
    };

    view! {
        <div class="page-viewport">
            // === HEADER BAR ===
            <div class="page-header-bar">
                <div class="page-header-left">
                    <h1 class="page-header-title">"Dashboard"</h1>

                    // Inline Stats
                    <div class="inline-stat">
                        <span class="inline-stat-label">"USDFC"</span>
                        <span class="inline-stat-value">{move || price_display.get()}</span>
                        {move || {
                            let change = price_change.get();
                            let class = if change >= 0.0 { "inline-stat-change up" } else { "inline-stat-change down" };
                            view! { <span class=class>{format!("{:+.2}%", change)}</span> }
                        }}
                    </div>

                    <div class="inline-stat">
                        <span class="inline-stat-label">"TCR"</span>
                        {move || {
                            let status = tcr_status.get();
                            let class = format!("inline-stat-value {}", status);
                            let tcr_text = tcr_display.get()
                                .map(|v| format!("{:.1}%", v))
                                .unwrap_or_else(|| "--".to_string());
                            view! { <span class=class>{tcr_text}</span> }
                        }}
                    </div>

                    // Status Dots
                    <div class="status-dots">
                        {move || {
                            source_status.get().into_iter().map(|(name, status)| {
                                view! { <div class=status.class() title=name></div> }
                            }).collect_view()
                        }}
                    </div>
                </div>

                <div class="page-header-right">
                    <TimeRangeSelector
                        selected=time_range
                        options=vec![TimeRange::Hour24, TimeRange::Day7, TimeRange::Day30, TimeRange::All]
                    />
                    <button class="refresh-btn" on:click=move |_| refresh_all()>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M23 4v6h-6"/>
                            <path d="M1 20v-6h6"/>
                            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
                        </svg>
                    </button>
                </div>
            </div>

            // === PRIMARY ZONE ===
            <div class="page-primary-zone">
                // Main Chart Area
                <div class="page-main-content">
                    <div class="split-panel" style="height: 100%;">
                        <div class="split-panel-header">
                            <span class="split-panel-title">"Trading Volume"</span>
                            <ChartTypeSelector selected=chart_type />
                        </div>
                        <div class="split-panel-body" style="display: flex; align-items: center; justify-content: center;">
                            <Suspense fallback=move || view! {
                                <ChartSkeleton />
                            }>
                                {move || {
                                    let ct = chart_type.get();
                                    volumes.get().map(|res| {
                                        match res {
                                            Ok(data) => {
                                                if data.is_empty() {
                                                    return view! {
                                                        <div style="text-align: center; color: #666;">"No volume data for selected range"</div>
                                                    }.into_view();
                                                }

                                                // Aggregate by day
                                                let mut daily: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
                                                for v in &data {
                                                    *daily.entry(v.day.clone()).or_insert(0.0) += v.volume;
                                                }
                                                let mut sorted: Vec<_> = daily.into_iter().collect();
                                                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                                                let max_vol = sorted.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);

                                                view! {
                                                    <DashboardChart data=sorted max_value=max_vol chart_type=ct />
                                                }.into_view()
                                            }
                                            Err(e) => view! {
                                                <div style="text-align: center; color: #ef4444;">{e.to_string()}</div>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>
                </div>

                // Sidebar Panel
                <div class="page-sidebar-panel">
                    // Key Metrics
                    <div class="split-panel">
                        <div class="split-panel-header">
                            <span class="split-panel-title">"Key Metrics"</span>
                        </div>
                        <div class="split-panel-body" style="padding: 12px;">
                            <Suspense fallback=move || view! {
                                <div style="display: flex; flex-direction: column; gap: 10px;">
                                    <MetricRowSkeleton />
                                    <MetricRowSkeleton />
                                    <MetricRowSkeleton />
                                    <MetricRowSkeleton />
                                </div>
                            }>
                                {move || {
                                    let proto = protocol.get();
                                    let pr = price.get();

                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: 10px;">
                                            // Supply
                                            <div class="metric-row">
                                                <span class="metric-row-label">"Total Supply"</span>
                                                <span class="metric-row-value">
                                                    {proto.as_ref().and_then(|r| r.as_ref().ok()).map(|m| format_volume(decimal_to_f64(m.total_supply))).unwrap_or_else(|| "--".to_string())}
                                                </span>
                                            </div>
                                            // Volume - handle Option<f64>
                                            <div class="metric-row">
                                                <span class="metric-row-label">"24h Volume"</span>
                                                <span class="metric-row-value">
                                                    {pr.as_ref().and_then(|r| r.as_ref().ok()).and_then(|p| p.volume_24h).map(format_usd_compact).unwrap_or_else(|| "--".to_string())}
                                                </span>
                                            </div>
                                            // Liquidity - handle Option<f64>
                                            <div class="metric-row">
                                                <span class="metric-row-label">"Liquidity"</span>
                                                <span class="metric-row-value">
                                                    {pr.as_ref().and_then(|r| r.as_ref().ok()).and_then(|p| p.liquidity_usd).map(format_usd_compact).unwrap_or_else(|| "--".to_string())}
                                                </span>
                                            </div>
                                            // Active Troves
                                            <div class="metric-row">
                                                <span class="metric-row-label">"Active Troves"</span>
                                                <span class="metric-row-value">
                                                    {proto.as_ref().and_then(|r| r.as_ref().ok()).map(|m| m.active_troves.to_string()).unwrap_or_else(|| "--".to_string())}
                                                </span>
                                            </div>
                                        </div>
                                    }.into_view()
                                }}
                            </Suspense>
                        </div>
                    </div>

                    // Recent Activity
                    <div class="split-panel" style="flex: 1; min-height: 0;">
                        <div class="split-panel-header">
                            <span class="split-panel-title">"Recent Activity"</span>
                            <span style="font-size: 11px; color: #666;">
                                {move || time_range.get().label()}
                            </span>
                        </div>
                        <div class="split-panel-body" style="padding: 8px 12px; overflow-y: auto;">
                            <Suspense fallback=move || view! {
                                <div style="display: flex; flex-direction: column; gap: 8px;">
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                </div>
                            }>
                                {move || {
                                    transactions.get().map(|res| {
                                        match res {
                                            Ok(txs) => {
                                                if txs.is_empty() {
                                                    return view! {
                                                        <div style="text-align: center; padding: 20px; color: #666;">"No activity in range"</div>
                                                    }.into_view();
                                                }

                                                // Filter by time range
                                                let now = get_current_timestamp();
                                                let threshold = time_range.get().seconds().unwrap_or(0);
                                                let filtered: Vec<_> = if threshold > 0 {
                                                    txs.iter().filter(|tx| (tx.timestamp as i64) >= (now - threshold)).take(8).collect()
                                                } else {
                                                    txs.iter().take(8).collect()
                                                };

                                                if filtered.is_empty() {
                                                    return view! {
                                                        <div style="text-align: center; padding: 20px; color: #666;">"No activity in range"</div>
                                                    }.into_view();
                                                }

                                                filtered.into_iter().map(|tx| {
                                                    let amount = decimal_to_f64(tx.amount);
                                                    let time_ago = format_time_ago(tx.timestamp);
                                                    view! {
                                                        <div class="activity-item">
                                                            <div class="activity-type">{tx.tx_type.as_str()}</div>
                                                            <div class="activity-amount">{format_volume(amount)}</div>
                                                            <div class="activity-time">{time_ago}</div>
                                                        </div>
                                                    }
                                                }).collect_view()
                                            }
                                            Err(_) => view! {
                                                <div style="text-align: center; padding: 20px; color: #ef4444;">"Error loading"</div>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    }
}

// === CHART COMPONENT ===
#[component]
fn DashboardChart(data: Vec<(String, f64)>, max_value: f64, chart_type: String) -> impl IntoView {
    let w = 800;
    let h = 280;
    let pad_l = 50;
    let pad_r = 20;
    let pad_t = 20;
    let pad_b = 30;
    let chart_w = w - pad_l - pad_r;
    let chart_h = h - pad_t - pad_b;

    let n = data.len();
    let step = if n > 1 { chart_w as f64 / (n - 1) as f64 } else { chart_w as f64 };

    let points: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, (_, v))| {
        let x = pad_l as f64 + i as f64 * step;
        let y = pad_t as f64 + chart_h as f64 * (1.0 - v / max_value.max(1.0));
        (x, y)
    }).collect();

    let line_path = points.iter().enumerate()
        .map(|(i, (x, y))| if i == 0 { format!("M{:.1},{:.1}", x, y) } else { format!("L{:.1},{:.1}", x, y) })
        .collect::<Vec<_>>().join(" ");

    let area_path = if !points.is_empty() {
        let first_x = points.first().map(|(x, _)| *x).unwrap_or(pad_l as f64);
        let last_x = points.last().map(|(x, _)| *x).unwrap_or((w - pad_r) as f64);
        let bottom = (pad_t + chart_h) as f64;
        format!("{} L{:.1},{:.1} L{:.1},{:.1} Z", line_path, last_x, bottom, first_x, bottom)
    } else { String::new() };

    let bar_w = if n > 0 { ((chart_w as f64 / n as f64) * 0.7).min(30.0) } else { 20.0 };

    view! {
        <svg viewBox=format!("0 0 {} {}", w, h) style="width: 100%; height: auto;">
            <defs>
                <linearGradient id="dashAreaGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="rgba(99, 102, 241, 0.3)"/>
                    <stop offset="100%" stop-color="rgba(99, 102, 241, 0)"/>
                </linearGradient>
            </defs>

            // Grid lines
            {(0..=4).map(|i| {
                let y = pad_t as f64 + (i as f64 / 4.0) * chart_h as f64;
                let val = max_value * (1.0 - i as f64 / 4.0);
                let text_x = (pad_l - 8) as f64;
                let text_y = y + 4.0;
                let line_x2 = (w - pad_r) as f64;
                view! {
                    <g>
                        <line x1=pad_l y1=y x2=line_x2 y2=y stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                        <text x=text_x y=text_y text-anchor="end" fill="rgba(255,255,255,0.3)" font-size="12">{format_volume(val)}</text>
                    </g>
                }
            }).collect_view()}

            // X axis labels
            {data.iter().enumerate().filter(|(i, _)| n <= 10 || i % (n / 6).max(1) == 0).map(|(i, (day, _))| {
                let x = pad_l as f64 + i as f64 * step;
                let label_y = (h - 8) as f64;
                let label = day.split('-').last().unwrap_or(day);
                view! {
                    <text x=x y=label_y text-anchor="middle" fill="rgba(255,255,255,0.3)" font-size="10">{label.to_string()}</text>
                }
            }).collect_view()}

            // Chart content
            {match chart_type.as_str() {
                "area" => view! {
                    <g>
                        <path d=area_path.clone() fill="url(#dashAreaGrad)"/>
                        <path d=line_path.clone() fill="none" stroke="#6366f1" stroke-width="2"/>
                    </g>
                }.into_view(),
                "line" => view! {
                    <g>
                        <path d=line_path.clone() fill="none" stroke="#6366f1" stroke-width="2"/>
                        {points.iter().map(|(x, y)| {
                            view! { <circle cx=*x cy=*y r="3" fill="#0a0a0a" stroke="#6366f1" stroke-width="2"/> }
                        }).collect_view()}
                    </g>
                }.into_view(),
                "bars" => view! {
                    <g>
                        {data.iter().enumerate().map(|(i, (_, v))| {
                            let x = pad_l as f64 + i as f64 * step - bar_w / 2.0;
                            let bar_h = (v / max_value.max(1.0)) * chart_h as f64;
                            let y = pad_t as f64 + chart_h as f64 - bar_h;
                            view! {
                                <rect x=x y=y width=bar_w height=bar_h.max(1.0) fill="#6366f1" rx="2"/>
                            }
                        }).collect_view()}
                    </g>
                }.into_view(),
                _ => view! { <g></g> }.into_view()
            }}
        </svg>
    }
}

// === HELPER FUNCTIONS ===
fn get_current_timestamp() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

fn format_time_ago(timestamp: u64) -> String {
    let now = get_current_timestamp() as u64;
    let diff = now.saturating_sub(timestamp);

    if diff < 60 { return format!("{}s ago", diff); }
    if diff < 3600 { return format!("{}m ago", diff / 60); }
    if diff < 86400 { return format!("{}h ago", diff / 3600); }
    format!("{}d ago", diff / 86400)
}
