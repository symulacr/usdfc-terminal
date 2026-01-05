use leptos::*;
use usdfc_backend::server_fn::{get_recent_transactions, get_daily_volumes};
use usdfc_core::format::{format_amount, format_timestamp, shorten_hash};
use usdfc_core::types::Transaction;
use std::collections::HashMap;

#[component]
pub fn FlowDiagrams() -> impl IntoView {
    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(100)).await }
    );

    // Historical daily volume data from subgraph
    let daily_volumes = create_resource(
        || (),
        |_| async move { get_daily_volumes(Some(30)).await }
    );

    // Filter state
    let (tx_type_filter, set_tx_type_filter) = create_signal(String::new());
    let (min_amount, set_min_amount) = create_signal(String::new());
    let (max_amount, set_max_amount) = create_signal(String::new());
    let (address_filter, set_address_filter) = create_signal(String::new());
    let (time_range, set_time_range) = create_signal("all".to_string());
    let (show_filters, set_show_filters) = create_signal(false);

    // Filter function
    let filter_transactions = move |txs: &[Transaction]| -> Vec<Transaction> {
        let tx_type = tx_type_filter.get();
        let min = min_amount.get().parse::<f64>().ok();
        let max = max_amount.get().parse::<f64>().ok();
        let addr = address_filter.get().to_lowercase();
        let range = time_range.get();

        let now = get_current_timestamp();
        let time_threshold = match range.as_str() {
            "1h" => now - 3600,
            "6h" => now - 21600,
            "24h" => now - 86400,
            "7d" => now - 604800,
            _ => 0,
        };

        txs.iter()
            .filter(|tx| {
                // Type filter
                if !tx_type.is_empty() && tx.tx_type.as_str() != tx_type {
                    return false;
                }
                // Amount filters
                let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
                if let Some(m) = min {
                    if amount < m { return false; }
                }
                if let Some(m) = max {
                    if amount > m { return false; }
                }
                // Address filter (from or to)
                if !addr.is_empty() {
                    if !tx.from.to_lowercase().contains(&addr) && !tx.to.to_lowercase().contains(&addr) {
                        return false;
                    }
                }
                // Time range filter
                if time_threshold > 0 && (tx.timestamp as i64) < time_threshold {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    };

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Flow Diagrams"</h1>
                <p class="page-subtitle">"Interactive USDFC token flow analytics"</p>
            </div>

            // Filter Bar
            <div class="card" style="margin-bottom: 16px;">
                <div class="filter-bar">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| set_show_filters.update(|v| *v = !*v)
                    >
                        {move || if show_filters.get() { "Hide Filters" } else { "Show Filters" }}
                    </button>

                    // Time range quick filters
                    <div class="filter-group">
                        <span class="filter-label">"Time:"</span>
                        {["all", "1h", "6h", "24h", "7d"].iter().map(|r| {
                            let range = *r;
                            let label = match range {
                                "all" => "All",
                                "1h" => "1H",
                                "6h" => "6H",
                                "24h" => "24H",
                                "7d" => "7D",
                                _ => range,
                            };
                            view! {
                                <button
                                    class=move || if time_range.get() == range { "filter-chip active" } else { "filter-chip" }
                                    on:click=move |_| set_time_range.set(range.to_string())
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>

                    <button
                        class="btn btn-secondary"
                        on:click=move |_| recent_tx.refetch()
                    >
                        "Refresh"
                    </button>
                </div>

                // Expanded filters
                <div class=move || if show_filters.get() { "filter-expanded" } else { "filter-expanded hidden" }>
                    <div class="filter-row">
                        <div class="filter-field">
                            <label class="filter-label">"Type"</label>
                            <select
                                class="filter-input"
                                on:change=move |ev| set_tx_type_filter.set(event_target_value(&ev))
                            >
                                <option value="">"All Types"</option>
                                <option value="Transfer">"Transfer"</option>
                                <option value="Mint">"Mint"</option>
                                <option value="Burn">"Burn"</option>
                                <option value="Swap">"Swap"</option>
                            </select>
                        </div>
                        <div class="filter-field">
                            <label class="filter-label">"Min Amount"</label>
                            <input
                                type="number"
                                class="filter-input"
                                placeholder="0"
                                on:input=move |ev| set_min_amount.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="filter-field">
                            <label class="filter-label">"Max Amount"</label>
                            <input
                                type="number"
                                class="filter-input"
                                placeholder="∞"
                                on:input=move |ev| set_max_amount.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="filter-field" style="flex: 2;">
                            <label class="filter-label">"Address (from or to)"</label>
                            <input
                                type="text"
                                class="filter-input"
                                placeholder="0x..."
                                on:input=move |ev| set_address_filter.set(event_target_value(&ev))
                            />
                        </div>
                        <button
                            class="btn btn-secondary"
                            style="align-self: flex-end;"
                            on:click=move |_| {
                                set_tx_type_filter.set(String::new());
                                set_min_amount.set(String::new());
                                set_max_amount.set(String::new());
                                set_address_filter.set(String::new());
                                set_time_range.set("all".to_string());
                            }
                        >
                            "Clear"
                        </button>
                    </div>
                </div>
            </div>

            // Volume over time chart
            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Transfer Volume"</h3>
                        <p class="card-subtitle">"Recent transaction volumes (hover for details)"</p>
                    </div>
                </div>
                <Suspense fallback=move || view! {
                    <div style="text-align: center; padding: 60px;">
                        <div class="skeleton" style="width: 100%; height: 300px;"></div>
                    </div>
                }>
                    {move || {
                        recent_tx.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    let filtered = filter_transactions(&txs);
                                    if filtered.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-title">"No matching transfers"</div>
                                                <div class="empty-state-desc">"Try adjusting filters or refreshing."</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        // Group by hour
                                        let mut hourly: HashMap<i64, f64> = HashMap::new();
                                        for tx in &filtered {
                                            let hour = (tx.timestamp as i64) / 3600 * 3600;
                                            let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            *hourly.entry(hour).or_insert(0.0) += amount;
                                        }
                                        let mut sorted: Vec<_> = hourly.into_iter().collect();
                                        sorted.sort_by_key(|(ts, _)| *ts);
                                        let bars: Vec<_> = sorted.into_iter().rev().take(24).collect();
                                        let max_vol = bars.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
                                        let count = filtered.len();

                                        view! {
                                            <div class="filter-stats">
                                                <span class="filter-count">{format!("{} transactions", count)}</span>
                                            </div>
                                            <VolumeBarChart bars=bars.clone() max_volume=max_vol />
                                        }.into_view()
                                    }
                                }
                                Err(err) => view! {
                                    <div class="empty-state">
                                        <div class="empty-state-title">"Flow Error"</div>
                                        <div class="empty-state-desc">{err.to_string()}</div>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Flow type breakdown
            <div class="grid-2" style="margin-top: 24px;">
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Transfer Type Distribution"</h3>
                    <Suspense fallback=move || view! { <div class="skeleton" style="height: 250px;"></div> }>
                        {move || {
                            recent_tx.get().map(|res| {
                                match res {
                                    Ok(txs) => {
                                        let filtered = filter_transactions(&txs);
                                        let mut counts: HashMap<String, (u32, f64)> = HashMap::new();
                                        for tx in &filtered {
                                            let key = tx.tx_type.as_str().to_string();
                                            let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            let entry = counts.entry(key).or_insert((0, 0.0));
                                            entry.0 += 1;
                                            entry.1 += amount;
                                        }
                                        let mut entries: Vec<_> = counts.into_iter().collect();
                                        entries.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap_or(std::cmp::Ordering::Equal));
                                        let total = entries.iter().map(|(_, (_, v))| *v).sum::<f64>();

                                        view! {
                                            <PieChartSVG entries=entries.clone() total=total />
                                        }.into_view()
                                    }
                                    Err(err) => view! {
                                        <div class="empty-state">
                                            <div class="empty-state-title">"Error"</div>
                                            <div class="empty-state-desc">{err.to_string()}</div>
                                        </div>
                                    }.into_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Top Transfer Flows"</h3>
                    <Suspense fallback=move || view! { <div class="skeleton" style="height: 250px;"></div> }>
                        {move || {
                            recent_tx.get().map(|res| {
                                match res {
                                    Ok(txs) => {
                                        let mut filtered = filter_transactions(&txs);
                                        filtered.sort_by(|a, b| {
                                            let a_val = a.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            let b_val = b.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
                                        });
                                        view! {
                                            <div style="display: flex; flex-direction: column; gap: 8px;">
                                                {filtered.iter().take(6).map(|tx| {
                                                    let amount = format_amount(tx.amount);
                                                    let time = format_timestamp(tx.timestamp);
                                                    view! {
                                                        <div class="stat-row" style="padding: 8px 0; border-bottom: 1px solid var(--bg-tertiary);">
                                                            <div>
                                                                <div style="font-family: monospace; font-size: 11px; color: var(--text-secondary);">
                                                                    {format!("{} → {}", shorten_hash(&tx.from), shorten_hash(&tx.to))}
                                                                </div>
                                                                <div style="font-size: 10px; color: var(--text-muted); margin-top: 2px;">{time}</div>
                                                            </div>
                                                            <div style="font-family: monospace; color: var(--accent-cyan);">{amount}</div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_view()
                                    }
                                    Err(err) => view! {
                                        <div class="empty-state">
                                            <div class="empty-state-desc">{err.to_string()}</div>
                                        </div>
                                    }.into_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            // Historical Volume Chart (from Subgraph)
            <div class="card" style="margin-top: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Historical Daily Volume"</h3>
                        <p class="card-subtitle">"30-day volume from Secured Finance subgraph"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| daily_volumes.refetch()
                    >
                        "Refresh"
                    </button>
                </div>
                <Suspense fallback=move || view! {
                    <div style="text-align: center; padding: 60px;">
                        <div class="skeleton" style="width: 100%; height: 200px;"></div>
                    </div>
                }>
                    {move || {
                        daily_volumes.get().map(|res| {
                            match res {
                                Ok(data) => {
                                    if data.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-title">"No historical data"</div>
                                                <div class="empty-state-desc">"Subgraph data not available yet"</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        // Aggregate volumes by day
                                        let mut daily_totals: HashMap<String, f64> = HashMap::new();
                                        for v in &data {
                                            *daily_totals.entry(v.day.clone()).or_insert(0.0) += v.volume;
                                        }
                                        let mut sorted: Vec<_> = daily_totals.into_iter().collect();
                                        sorted.sort_by(|a, b| a.0.cmp(&b.0));
                                        let bars: Vec<(String, f64)> = sorted.into_iter().collect();
                                        let max_vol = bars.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);

                                        view! {
                                            <HistoricalVolumeChart bars=bars max_volume=max_vol />
                                        }.into_view()
                                    }
                                }
                                Err(err) => view! {
                                    <div class="empty-state">
                                        <div class="empty-state-title">"Data Error"</div>
                                        <div class="empty-state-desc">{err.to_string()}</div>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn HistoricalVolumeChart(bars: Vec<(String, f64)>, max_volume: f64) -> impl IntoView {
    let svg_width = 800;
    let svg_height = 200;
    let bar_width = if bars.is_empty() { 20 } else { (svg_width - 80) / bars.len().max(1) - 2 };
    let chart_height = svg_height - 40;

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_width, svg_height)
            style="width: 100%; height: 200px; background: var(--bg-tertiary); border-radius: 4px;"
        >
            // Grid lines
            {(0..4).map(|i| {
                let y = 20 + (i as f64 / 3.0 * chart_height as f64) as i32;
                let value = max_volume * (1.0 - i as f64 / 3.0);
                view! {
                    <g>
                        <line x1=50 y1=y x2=svg_width - 20 y2=y stroke="var(--bg-secondary)" stroke-width="1" stroke-dasharray="4" />
                        <text x=45 y=y + 4 text-anchor="end" fill="var(--text-muted)" font-size="9">
                            {format_vol(value)}
                        </text>
                    </g>
                }
            }).collect_view()}

            // Bars
            {bars.iter().enumerate().map(|(i, (day, vol))| {
                let x = 55 + i * (bar_width + 2);
                let height = if max_volume > 0.0 { (*vol / max_volume * chart_height as f64) as i32 } else { 0 };
                let y = 20 + chart_height - height;
                let color = "#6366f1";

                view! {
                    <g class="bar-group" style="cursor: pointer;">
                        <rect
                            x=x
                            y=y
                            width=bar_width
                            height=height.max(1)
                            fill=color
                            rx=1
                            opacity="0.8"
                        >
                            <title>{format!("{}: {}", day, format_vol(*vol))}</title>
                        </rect>
                    </g>
                }
            }).collect_view()}

            // X-axis labels (show every 5th day)
            {bars.iter().enumerate().filter(|(i, _)| i % 5 == 0).map(|(i, (day, _))| {
                let x = 55 + i * (bar_width + 2) + bar_width / 2;
                // Format day string (dd-mm-yyyy) to short format
                let short_day = day.split('-').next().unwrap_or(day).to_string();
                view! {
                    <text
                        x=x
                        y=svg_height - 5
                        text-anchor="middle"
                        fill="var(--text-muted)"
                        font-size="8"
                    >
                        {short_day}
                    </text>
                }
            }).collect_view()}
        </svg>
    }
}

#[component]
fn VolumeBarChart(bars: Vec<(i64, f64)>, max_volume: f64) -> impl IntoView {
    let svg_width = 800;
    let svg_height = 300;
    let bar_width = if bars.is_empty() { 20 } else { (svg_width - 100) / bars.len().max(1) - 4 };
    let chart_height = svg_height - 60;

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_width, svg_height)
            style="width: 100%; height: 300px; background: var(--bg-tertiary); border-radius: 4px;"
        >
            // Grid lines
            {(0..5).map(|i| {
                let y = 30 + (i as f64 / 4.0 * chart_height as f64) as i32;
                let value = max_volume * (1.0 - i as f64 / 4.0);
                view! {
                    <g>
                        <line x1=50 y1=y x2=svg_width - 20 y2=y stroke="var(--bg-secondary)" stroke-width="1" stroke-dasharray="4" />
                        <text x=45 y=y + 4 text-anchor="end" fill="var(--text-muted)" font-size="10">
                            {format_vol(value)}
                        </text>
                    </g>
                }
            }).collect_view()}

            // Bars
            {bars.iter().rev().enumerate().map(|(i, (ts, vol))| {
                let x = 60 + i * (bar_width + 4);
                let height = if max_volume > 0.0 { (*vol / max_volume * chart_height as f64) as i32 } else { 0 };
                let y = 30 + chart_height - height;
                let color = get_bar_color(i);

                view! {
                    <g class="bar-group" style="cursor: pointer;">
                        // Bar
                        <rect
                            x=x
                            y=y
                            width=bar_width
                            height=height.max(2)
                            fill=color
                            rx=2
                            style="transition: opacity 0.2s;"
                        >
                            <title>{format!("{}: {}", format_time(*ts), format_vol(*vol))}</title>
                        </rect>
                        // Time label (every 4th bar)
                        {if i % 4 == 0 {
                            view! {
                                <text
                                    x=x + bar_width / 2
                                    y=svg_height - 10
                                    text-anchor="middle"
                                    fill="var(--text-muted)"
                                    font-size="9"
                                >
                                    {format_hour(*ts)}
                                </text>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </g>
                }
            }).collect_view()}

            // Y-axis label
            <text x=15 y=svg_height / 2 text-anchor="middle" fill="var(--text-muted)" font-size="10" transform=format!("rotate(-90 15 {})", svg_height / 2)>
                "Volume (USDFC)"
            </text>
        </svg>
    }
}

#[component]
fn PieChartSVG(entries: Vec<(String, (u32, f64))>, total: f64) -> impl IntoView {
    let cx = 120;
    let cy = 120;
    let radius = 80;
    let inner_radius = 50;

    let mut current_angle: f64 = -90.0; // Start from top

    view! {
        <div style="display: flex; align-items: center; gap: 24px;">
            <svg viewBox="0 0 240 240" style="width: 200px; height: 200px;">
                {entries.iter().enumerate().map(|(i, (name, (count, vol)))| {
                    let percent = if total > 0.0 { *vol / total } else { 0.0 };
                    let angle = percent * 360.0;
                    let start_angle = current_angle;
                    let end_angle = current_angle + angle;
                    current_angle = end_angle;

                    let large_arc = if angle > 180.0 { 1 } else { 0 };

                    let start_rad = start_angle.to_radians();
                    let end_rad = end_angle.to_radians();

                    let x1 = cx as f64 + radius as f64 * start_rad.cos();
                    let y1 = cy as f64 + radius as f64 * start_rad.sin();
                    let x2 = cx as f64 + radius as f64 * end_rad.cos();
                    let y2 = cy as f64 + radius as f64 * end_rad.sin();
                    let x3 = cx as f64 + inner_radius as f64 * end_rad.cos();
                    let y3 = cy as f64 + inner_radius as f64 * end_rad.sin();
                    let x4 = cx as f64 + inner_radius as f64 * start_rad.cos();
                    let y4 = cy as f64 + inner_radius as f64 * start_rad.sin();

                    let path = format!(
                        "M {} {} A {} {} 0 {} 1 {} {} L {} {} A {} {} 0 {} 0 {} {} Z",
                        x1, y1, radius, radius, large_arc, x2, y2,
                        x3, y3, inner_radius, inner_radius, large_arc, x4, y4
                    );

                    let color = get_pie_color(i);
                    let name_display = name.clone();

                    view! {
                        <path
                            d=path
                            fill=color
                            stroke="var(--bg-primary)"
                            stroke-width="2"
                            style="cursor: pointer; transition: opacity 0.2s;"
                        >
                            <title>{format!("{}: {} txs, {} ({:.1}%)", name_display, count, format_vol(*vol), percent * 100.0)}</title>
                        </path>
                    }
                }).collect_view()}
                // Center text
                <text x=cx y=cy - 5 text-anchor="middle" fill="var(--text-primary)" font-size="14" font-weight="bold">
                    {format_vol(total)}
                </text>
                <text x=cx y=cy + 12 text-anchor="middle" fill="var(--text-muted)" font-size="10">
                    "Total Volume"
                </text>
            </svg>
            // Legend
            <div style="display: flex; flex-direction: column; gap: 8px;">
                {entries.iter().enumerate().map(|(i, (name, (count, vol)))| {
                    let percent = if total > 0.0 { *vol / total * 100.0 } else { 0.0 };
                    let color = get_pie_color(i);
                    let name_display = name.clone();
                    view! {
                        <div style="display: flex; align-items: center; gap: 8px; font-size: 12px;">
                            <div style=format!("width: 12px; height: 12px; background: {}; border-radius: 2px;", color)></div>
                            <span style="color: var(--text-secondary);">{name_display}</span>
                            <span style="color: var(--text-muted);">{format!("({} txs, {:.1}%)", count, percent)}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn get_bar_color(index: usize) -> &'static str {
    // Gradient from cyan to purple
    if index < 6 { "#00d4ff" }
    else if index < 12 { "#6366f1" }
    else if index < 18 { "#a855f7" }
    else { "#ec4899" }
}

fn get_pie_color(index: usize) -> &'static str {
    const COLORS: [&str; 6] = [
        "#00d4ff", // cyan
        "#a855f7", // purple
        "#22c55e", // green
        "#f59e0b", // yellow
        "#ec4899", // pink
        "#6366f1", // indigo
    ];
    COLORS[index % COLORS.len()]
}

fn format_vol(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        format!("{:.1}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.1}K", amount / 1_000.0)
    } else {
        format!("{:.0}", amount)
    }
}

fn format_time(ts: i64) -> String {
    // Format timestamp as absolute time to avoid SSR/client hydration mismatch
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%b %d, %H:%M").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_hour(ts: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_default()
}

fn get_current_timestamp() -> i64 {
    // Platform-specific current time implementation
    // Note: Used for filtering which runs after hydration (user interaction)
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
