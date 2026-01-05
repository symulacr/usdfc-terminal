//! Analytics Page
//!
//! Combined view of Flow, Sankey, and Network visualizations.
//! Uses tabs for navigation between sections.

use leptos::*;
use crate::components::tabs::{TabNav, TabContent, Tab};
use crate::components::icons::*;
use usdfc_api::{get_recent_transactions, get_daily_volumes};
use usdfc_core::format::{format_amount, format_timestamp, shorten_hash, format_volume, decimal_to_f64};
use usdfc_core::types::Transaction;
use std::collections::HashMap;

/// Generate CSV content from transaction data
fn generate_csv(transactions: &[Transaction]) -> String {
    let mut csv = String::from("Hash,Type,Amount,From,To,Timestamp,Block,Status\n");
    for tx in transactions {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            tx.hash,
            tx.tx_type.as_str(),
            decimal_to_f64(tx.amount),
            tx.from,
            tx.to,
            tx.timestamp,
            tx.block,
            tx.status.as_str()
        ));
    }
    csv
}

/// Trigger CSV download in browser using data URL
#[cfg(feature = "hydrate")]
fn download_csv(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    // URL-encode the CSV content for data URL
    let encoded: String = content
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                format!("{}", b as char)
            } else {
                format!("%{:02X}", b)
            }
        })
        .collect();
    let data_url = format!("data:text/csv;charset=utf-8,{}", encoded);

    // Create temporary link and click it
    let link = document.create_element("a").expect("create element failed");
    let link: web_sys::HtmlAnchorElement = link.dyn_into().expect("cast failed");
    link.set_href(&data_url);
    link.set_download(filename);
    link.click();
}

#[cfg(not(feature = "hydrate"))]
fn download_csv(_filename: &str, _content: &str) {
    // No-op on server side
}

#[component]
pub fn Analytics() -> impl IntoView {
    let active_tab = create_rw_signal("flow".to_string());

    // Shared transaction data for all tabs
    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(100)).await }
    );

    let tabs = vec![
        Tab { id: "flow", label: "Flow" },
        Tab { id: "sankey", label: "Sankey" },
        Tab { id: "network", label: "Network" },
        Tab { id: "volume", label: "Volume" },
    ];

    view! {
        <div class="fade-in">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Analytics"</h1>
                    <p class="page-subtitle">"USDFC flow and network visualizations"</p>
                </div>
                <div style="display: flex; gap: 8px;">
                    <button
                        class="btn btn-primary"
                        on:click=move |_| {
                            #[cfg(target_arch = "wasm32")]
                            if let Some(Ok(txs)) = recent_tx.get() {
                                let csv = generate_csv(&txs);
                                download_csv("usdfc_analytics.csv", &csv);
                            }
                        }
                    >
                        <DownloadIcon />
                        "Export CSV"
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| recent_tx.refetch()
                    >
                        <RefreshIcon />
                        "Refresh"
                    </button>
                </div>
            </div>

            <TabNav tabs=tabs active=active_tab />

            <TabContent id="flow" active=active_tab>
                <FlowTab tx_resource=recent_tx />
            </TabContent>

            <TabContent id="sankey" active=active_tab>
                <SankeyTab tx_resource=recent_tx />
            </TabContent>

            <TabContent id="network" active=active_tab>
                <NetworkTab tx_resource=recent_tx />
            </TabContent>

            <TabContent id="volume" active=active_tab>
                <VolumeTab />
            </TabContent>
        </div>
    }
}

// ============================================================================
// Flow Tab
// ============================================================================

#[component]
fn FlowTab(
    tx_resource: Resource<(), Result<Vec<Transaction>, ServerFnError>>,
) -> impl IntoView {
    // Filter state
    let (tx_type_filter, set_tx_type_filter) = create_signal(String::new());
    let (min_amount, set_min_amount) = create_signal(String::new());
    let (max_amount, set_max_amount) = create_signal(String::new());
    let (address_filter, set_address_filter) = create_signal(String::new());
    let (time_range, set_time_range) = create_signal("all".to_string());
    let (show_filters, set_show_filters) = create_signal(false);

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
                if !tx_type.is_empty() && tx.tx_type.as_str() != tx_type {
                    return false;
                }
                let amount = decimal_to_f64(tx.amount);
                if let Some(m) = min {
                    if amount < m { return false; }
                }
                if let Some(m) = max {
                    if amount > m { return false; }
                }
                if !addr.is_empty() {
                    if !tx.from.to_lowercase().contains(&addr) && !tx.to.to_lowercase().contains(&addr) {
                        return false;
                    }
                }
                if time_threshold > 0 && (tx.timestamp as i64) < time_threshold {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    };

    view! {
        // Inline Filter Bar
        <div class="flow-filter-bar" style="display: flex; align-items: center; gap: 16px; margin-bottom: 16px; flex-wrap: wrap;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <span style="color: var(--text-muted); font-size: 12px;">"Time:"</span>
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
                style="font-size: 12px; padding: 6px 12px;"
                on:click=move |_| set_show_filters.update(|v| *v = !*v)
            >
                {move || if show_filters.get() { "- Filters" } else { "+ Filters" }}
            </button>
        </div>

        <div class=move || if show_filters.get() { "filter-panel" } else { "filter-panel hidden" } style="margin-bottom: 16px; padding: 16px; background: var(--bg-tertiary); border: 1px solid var(--border-color);">
            <div style="display: flex; gap: 16px; flex-wrap: wrap; align-items: flex-end;">
                <div style="flex: 1; min-width: 120px;">
                    <label style="display: block; font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">"Type"</label>
                    <select
                        class="input"
                        style="width: 100%;"
                        on:change=move |ev| set_tx_type_filter.set(event_target_value(&ev))
                    >
                        <option value="">"All Types"</option>
                        <option value="Transfer">"Transfer"</option>
                        <option value="Mint">"Mint"</option>
                        <option value="Burn">"Burn"</option>
                        <option value="Swap">"Swap"</option>
                    </select>
                </div>
                <div style="flex: 1; min-width: 100px;">
                    <label style="display: block; font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">"Min Amount"</label>
                    <input
                        type="number"
                        class="input"
                        style="width: 100%;"
                        placeholder="0"
                        on:input=move |ev| set_min_amount.set(event_target_value(&ev))
                    />
                </div>
                <div style="flex: 1; min-width: 100px;">
                    <label style="display: block; font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">"Max Amount"</label>
                    <input
                        type="number"
                        class="input"
                        style="width: 100%;"
                        placeholder="∞"
                        on:input=move |ev| set_max_amount.set(event_target_value(&ev))
                    />
                </div>
                <div style="flex: 2; min-width: 200px;">
                    <label style="display: block; font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">"Address"</label>
                    <input
                        type="text"
                        class="input"
                        style="width: 100%;"
                        placeholder="0x..."
                        on:input=move |ev| set_address_filter.set(event_target_value(&ev))
                    />
                </div>
                <button
                    class="btn btn-secondary"
                    style="font-size: 12px; padding: 8px 16px;"
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

        // Volume chart
        <div class="card">
            <h3 class="card-title" style="margin-bottom: 16px;">"Transfer Volume"</h3>
            <Suspense fallback=move || view! {
                <div style="text-align: center; padding: 60px;">
                    <div class="skeleton" style="width: 100%; height: 300px;"></div>
                </div>
            }>
                {move || {
                    tx_resource.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                let filtered = filter_transactions(&txs);
                                if filtered.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <div class="empty-state-title">"No matching transfers"</div>
                                            <div class="empty-state-desc">"Try adjusting filters."</div>
                                        </div>
                                    }.into_view()
                                } else {
                                    let mut hourly: HashMap<i64, f64> = HashMap::new();
                                    for tx in &filtered {
                                        let hour = (tx.timestamp as i64) / 3600 * 3600;
                                        let amount = decimal_to_f64(tx.amount);
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
                                    <div class="empty-state-title">"Error"</div>
                                    <div class="empty-state-desc">{err.to_string()}</div>
                                </div>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>

        // Type distribution
        <div class="grid-2" style="margin-top: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Transfer Type Distribution"</h3>
                <Suspense fallback=move || view! { <div class="skeleton" style="height: 250px;"></div> }>
                    {move || {
                        tx_resource.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    let filtered = filter_transactions(&txs);
                                    let mut counts: HashMap<String, (u32, f64)> = HashMap::new();
                                    for tx in &filtered {
                                        let key = tx.tx_type.as_str().to_string();
                                        let amount = decimal_to_f64(tx.amount);
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
                        tx_resource.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    let mut filtered = filter_transactions(&txs);
                                    filtered.sort_by(|a, b| {
                                        let a_val = decimal_to_f64(a.amount);
                                        let b_val = decimal_to_f64(b.amount);
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
    }
}

// ============================================================================
// Sankey Tab
// ============================================================================

#[component]
fn SankeyTab(
    tx_resource: Resource<(), Result<Vec<Transaction>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="card">
            <h3 class="card-title" style="margin-bottom: 16px;">"Token Flow Diagram"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Proportional view of transfer paths"</p>
            <Suspense fallback=move || view! {
                <div style="text-align: center; padding: 60px;">
                    <div class="skeleton" style="width: 100%; height: 400px;"></div>
                </div>
            }>
                {move || {
                    tx_resource.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                if txs.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <div class="empty-state-title">"No flow data"</div>
                                        </div>
                                    }.into_view()
                                } else {
                                    let mut edges: HashMap<(String, String), f64> = HashMap::new();
                                    for tx in &txs {
                                        let amount = decimal_to_f64(tx.amount);
                                        if amount > 0.0 {
                                            let key = (tx.from.clone(), tx.to.clone());
                                            *edges.entry(key).or_insert(0.0) += amount;
                                        }
                                    }

                                    let mut ranked: Vec<_> = edges.into_iter().collect();
                                    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                                    let top_flows: Vec<_> = ranked.into_iter().take(8).collect();
                                    let max_amount = top_flows.first().map(|(_, v)| *v).unwrap_or(1.0);

                                    let mut left_nodes: Vec<String> = Vec::new();
                                    let mut right_nodes: Vec<String> = Vec::new();
                                    for ((from, to), _) in &top_flows {
                                        if !left_nodes.contains(from) { left_nodes.push(from.clone()); }
                                        if !right_nodes.contains(to) { right_nodes.push(to.clone()); }
                                    }

                                    view! {
                                        <div style="position: relative;">
                                            <SankeySVG
                                                flows=top_flows.clone()
                                                left_nodes=left_nodes.clone()
                                                right_nodes=right_nodes.clone()
                                                max_amount=max_amount
                                            />
                                            <div style="margin-top: 24px; display: flex; flex-wrap: wrap; gap: 16px;">
                                                {top_flows.iter().enumerate().map(|(i, ((from, to), amount))| {
                                                    let color = get_flow_color(i);
                                                    view! {
                                                        <div style="display: flex; align-items: center; gap: 8px; font-size: 12px;">
                                                            <div style=format!("width: 12px; height: 12px; background: {}; border-radius: 2px;", color)></div>
                                                            <span style="color: var(--text-muted);">
                                                                {format!("{} → {}: ", shorten_hash(from), shorten_hash(to))}
                                                            </span>
                                                            <span style="color: var(--text-primary); font-family: monospace;">
                                                                {format_volume(*amount)}
                                                            </span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_view()
                                }
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

        // Stats
        <div class="grid-3" style="margin-top: 24px;">
            <Suspense fallback=move || view! { <div class="card"><div class="skeleton" style="height: 80px;"></div></div> }>
                {move || {
                    tx_resource.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                let total: f64 = txs.iter()
                                    .map(|tx| decimal_to_f64(tx.amount))
                                    .sum();
                                let unique_senders: std::collections::HashSet<_> = txs.iter().map(|tx| &tx.from).collect();
                                let unique_receivers: std::collections::HashSet<_> = txs.iter().map(|tx| &tx.to).collect();

                                view! {
                                    <div class="card">
                                        <div class="metric-label">"Total Volume"</div>
                                        <div class="metric-value cyan">{format_volume(total)}</div>
                                    </div>
                                    <div class="card">
                                        <div class="metric-label">"Unique Senders"</div>
                                        <div class="metric-value green">{unique_senders.len().to_string()}</div>
                                    </div>
                                    <div class="card">
                                        <div class="metric-label">"Unique Receivers"</div>
                                        <div class="metric-value purple">{unique_receivers.len().to_string()}</div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! {
                                <div class="card"><div class="metric-label">"Error"</div></div>
                                <div class="card"></div>
                                <div class="card"></div>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// ============================================================================
// Network Tab
// ============================================================================

#[component]
fn NetworkTab(
    tx_resource: Resource<(), Result<Vec<Transaction>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="card">
            <h3 class="card-title" style="margin-bottom: 16px;">"Transaction Network"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Node size = volume, edge thickness = transfer amount"</p>
            <Suspense fallback=move || view! {
                <div style="text-align: center; padding: 60px;">
                    <div class="skeleton" style="width: 100%; height: 500px;"></div>
                </div>
            }>
                {move || {
                    tx_resource.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                if txs.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <div class="empty-state-title">"No network data"</div>
                                        </div>
                                    }.into_view()
                                } else {
                                    let mut edges: HashMap<(String, String), f64> = HashMap::new();
                                    let mut node_volumes: HashMap<String, f64> = HashMap::new();

                                    for tx in &txs {
                                        let amount = decimal_to_f64(tx.amount);
                                        if amount > 0.0 {
                                            let key = (tx.from.clone(), tx.to.clone());
                                            *edges.entry(key).or_insert(0.0) += amount;
                                            *node_volumes.entry(tx.from.clone()).or_insert(0.0) += amount;
                                            *node_volumes.entry(tx.to.clone()).or_insert(0.0) += amount;
                                        }
                                    }

                                    let mut ranked_edges: Vec<_> = edges.into_iter().collect();
                                    ranked_edges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                                    let top_edges: Vec<_> = ranked_edges.into_iter().take(15).collect();

                                    let mut nodes: Vec<String> = Vec::new();
                                    for ((from, to), _) in &top_edges {
                                        if !nodes.contains(from) { nodes.push(from.clone()); }
                                        if !nodes.contains(to) { nodes.push(to.clone()); }
                                    }

                                    let max_volume = node_volumes.values().cloned().fold(0.0f64, f64::max);
                                    let max_edge = top_edges.first().map(|(_, v)| *v).unwrap_or(1.0);

                                    view! {
                                        <NetworkSVG
                                            nodes=nodes.clone()
                                            edges=top_edges.clone()
                                            node_volumes=node_volumes.clone()
                                            max_volume=max_volume
                                            max_edge=max_edge
                                        />
                                    }.into_view()
                                }
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

        // Network stats
        <div class="grid-2" style="margin-top: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Top Connections"</h3>
                <Suspense fallback=move || view! { <div class="skeleton" style="height: 200px;"></div> }>
                    {move || {
                        tx_resource.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    let mut edges: HashMap<(String, String), (f64, u32)> = HashMap::new();
                                    for tx in &txs {
                                        let amount = decimal_to_f64(tx.amount);
                                        let key = (tx.from.clone(), tx.to.clone());
                                        let entry = edges.entry(key).or_insert((0.0, 0));
                                        entry.0 += amount;
                                        entry.1 += 1;
                                    }
                                    let mut ranked: Vec<_> = edges.into_iter().collect();
                                    ranked.sort_by(|a, b| b.1.1.cmp(&a.1.1));
                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: 8px;">
                                            {ranked.iter().take(5).map(|((from, to), (amount, count))| {
                                                view! {
                                                    <div class="stat-row">
                                                        <span style="font-family: monospace; font-size: 11px; color: var(--text-secondary);">
                                                            {format!("{} → {}", shorten_hash(from), shorten_hash(to))}
                                                        </span>
                                                        <span style="font-family: monospace; font-size: 12px;">
                                                            <span style="color: var(--accent-cyan);">{format_volume(*amount)}</span>
                                                            <span style="color: var(--text-muted); margin-left: 8px;">{format!("({} txs)", count)}</span>
                                                        </span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! { <div>"Error"</div> }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Network Legend"</h3>
                <div style="display: flex; flex-direction: column; gap: 12px;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <div style="width: 20px; height: 20px; background: var(--accent-cyan); border-radius: 50%;"></div>
                        <span style="color: var(--text-secondary); font-size: 13px;">"Node = Address (size by volume)"</span>
                    </div>
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <div style="width: 30px; height: 4px; background: var(--accent-purple);"></div>
                        <span style="color: var(--text-secondary); font-size: 13px;">"Edge = Transfer (width by amount)"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Volume Tab (Historical)
// ============================================================================

#[component]
fn VolumeTab() -> impl IntoView {
    let daily_volumes = create_resource(
        || (),
        |_| async move { get_daily_volumes(Some(30)).await }
    );

    view! {
        <div class="card">
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
                                        </div>
                                    }.into_view()
                                } else {
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
                                    <div class="empty-state-title">"Error"</div>
                                    <div class="empty-state-desc">{err.to_string()}</div>
                                </div>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// ============================================================================
// SVG Components
// ============================================================================

#[component]
fn VolumeBarChart(bars: Vec<(i64, f64)>, max_volume: f64) -> impl IntoView {
    let svg_width = 800;
    let svg_height = 320;
    let bar_width = if bars.is_empty() { 20 } else { (svg_width - 100) / bars.len().max(1) - 4 };
    let chart_height = svg_height - 60;

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_width, svg_height)
            style="width: 100%; height: 320px; background: var(--bg-tertiary); border-radius: 4px;"
        >
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

            {bars.iter().rev().enumerate().map(|(i, (ts, vol))| {
                let x = 60 + i * (bar_width + 4);
                let height = if max_volume > 0.0 { (*vol / max_volume * chart_height as f64) as i32 } else { 0 };
                let y = 30 + chart_height - height;
                let color = get_bar_color(i);

                view! {
                    <g class="bar-group" style="cursor: pointer;">
                        <rect
                            x=x
                            y=y
                            width=bar_width
                            height=height.max(2)
                            fill=color
                            rx=2
                        >
                            <title>{format!("{}: {}", format_time(*ts), format_vol(*vol))}</title>
                        </rect>
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

            <text x=15 y=svg_height / 2 text-anchor="middle" fill="var(--text-muted)" font-size="10" transform=format!("rotate(-90 15 {})", svg_height / 2)>
                "Volume (USDFC)"
            </text>
        </svg>
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

            {bars.iter().enumerate().map(|(i, (day, vol))| {
                let x = 55 + i * (bar_width + 2);
                let height = if max_volume > 0.0 { (*vol / max_volume * chart_height as f64) as i32 } else { 0 };
                let y = 20 + chart_height - height;

                view! {
                    <g class="bar-group" style="cursor: pointer;">
                        <rect
                            x=x
                            y=y
                            width=bar_width
                            height=height.max(1)
                            fill="#6366f1"
                            rx=1
                            opacity="0.8"
                        >
                            <title>{format!("{}: {}", day, format_vol(*vol))}</title>
                        </rect>
                    </g>
                }
            }).collect_view()}

            {bars.iter().enumerate().filter(|(i, _)| i % 5 == 0).map(|(i, (day, _))| {
                let x = 55 + i * (bar_width + 2) + bar_width / 2;
                let short_day = day.split('-').next().unwrap_or(day).to_string();
                view! {
                    <text
                        x=x
                        y=svg_height - 5
                        text-anchor="middle"
                        fill="var(--text-muted)"
                        font-size="10"
                    >
                        {short_day}
                    </text>
                }
            }).collect_view()}
        </svg>
    }
}

#[component]
fn PieChartSVG(entries: Vec<(String, (u32, f64))>, total: f64) -> impl IntoView {
    let cx = 120;
    let cy = 120;
    let radius = 80;
    let inner_radius = 50;

    let mut current_angle: f64 = -90.0;

    view! {
        <div style="display: flex; align-items: center; gap: 24px;">
            <svg viewBox="0 0 240 240" style="max-width: 200px; width: 100%; height: auto;">
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
                            style="cursor: pointer;"
                        >
                            <title>{format!("{}: {} txs, {} ({:.1}%)", name_display, count, format_vol(*vol), percent * 100.0)}</title>
                        </path>
                    }
                }).collect_view()}
                <text x=cx y=cy - 5 text-anchor="middle" fill="var(--text-primary)" font-size="14" font-weight="bold">
                    {format_vol(total)}
                </text>
                <text x=cx y=cy + 12 text-anchor="middle" fill="var(--text-muted)" font-size="10">
                    "Total Volume"
                </text>
            </svg>
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

#[component]
fn SankeySVG(
    flows: Vec<((String, String), f64)>,
    left_nodes: Vec<String>,
    right_nodes: Vec<String>,
    max_amount: f64,
) -> impl IntoView {
    let svg_height = 400;
    let svg_width = 800;
    let node_width = 20;
    let left_x = 60;
    let right_x = svg_width - 60;

    let left_spacing = svg_height as f64 / (left_nodes.len() + 1) as f64;
    let right_spacing = svg_height as f64 / (right_nodes.len() + 1) as f64;

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_width, svg_height)
            style="width: 100%; height: 400px; background: var(--bg-tertiary); border-radius: 4px;"
        >
            <defs>
                {(0..8).map(|i| {
                    let color = get_flow_color(i);
                    view! {
                        <linearGradient id=format!("flow-gradient-{}", i) x1="0%" y1="0%" x2="100%" y2="0%">
                            <stop offset="0%" style=format!("stop-color:{};stop-opacity:0.8", color) />
                            <stop offset="100%" style=format!("stop-color:{};stop-opacity:0.4", color) />
                        </linearGradient>
                    }
                }).collect_view()}
            </defs>

            {flows.iter().enumerate().map(|(i, ((from, to), amount))| {
                let from_idx = left_nodes.iter().position(|n| n == from).unwrap_or(0);
                let to_idx = right_nodes.iter().position(|n| n == to).unwrap_or(0);

                let from_y = ((from_idx + 1) as f64 * left_spacing) as i32;
                let to_y = ((to_idx + 1) as f64 * right_spacing) as i32;
                let thickness = ((amount / max_amount) * 40.0).max(4.0) as i32;

                let mid_x = (left_x + right_x) / 2;
                let path = format!(
                    "M {} {} C {} {}, {} {}, {} {}",
                    left_x + node_width, from_y,
                    mid_x, from_y,
                    mid_x, to_y,
                    right_x, to_y
                );

                view! {
                    <g class="sankey-flow">
                        <path
                            d=path.clone()
                            fill="none"
                            stroke=format!("url(#flow-gradient-{})", i)
                            stroke-width=thickness
                            stroke-linecap="round"
                            style="cursor: pointer;"
                        >
                            <title>{format!("{} → {}: {}", shorten_hash(from), shorten_hash(to), format_vol(*amount))}</title>
                        </path>
                    </g>
                }
            }).collect_view()}

            {left_nodes.iter().enumerate().map(|(i, addr)| {
                let y = ((i + 1) as f64 * left_spacing) as i32;
                view! {
                    <g>
                        <rect x=left_x y=y - 15 width=node_width height=30 fill="var(--accent-cyan)" rx=4 />
                        <text x=left_x - 8 y=y + 4 text-anchor="end" fill="var(--text-secondary)" font-size="10" font-family="monospace">
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            {right_nodes.iter().enumerate().map(|(i, addr)| {
                let y = ((i + 1) as f64 * right_spacing) as i32;
                view! {
                    <g>
                        <rect x=right_x y=y - 15 width=node_width height=30 fill="var(--accent-purple)" rx=4 />
                        <text x=right_x + node_width + 8 y=y + 4 text-anchor="start" fill="var(--text-secondary)" font-size="10" font-family="monospace">
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            <text x=left_x + 10 y=20 fill="var(--text-muted)" font-size="11" text-anchor="middle">"SENDERS"</text>
            <text x=right_x + 10 y=20 fill="var(--text-muted)" font-size="11" text-anchor="middle">"RECEIVERS"</text>
        </svg>
    }
}

#[component]
fn NetworkSVG(
    nodes: Vec<String>,
    edges: Vec<((String, String), f64)>,
    node_volumes: HashMap<String, f64>,
    max_volume: f64,
    max_edge: f64,
) -> impl IntoView {
    let svg_size = 500;
    let center = svg_size / 2;
    let radius = 180;

    let node_positions: HashMap<String, (i32, i32)> = nodes.iter().enumerate()
        .map(|(i, addr)| {
            let angle = (i as f64 / nodes.len() as f64) * 2.0 * std::f64::consts::PI - std::f64::consts::PI / 2.0;
            let x = center + (radius as f64 * angle.cos()) as i32;
            let y = center + (radius as f64 * angle.sin()) as i32;
            (addr.clone(), (x, y))
        })
        .collect();

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_size, svg_size)
            style="width: 100%; height: 500px; background: var(--bg-tertiary); border-radius: 4px;"
        >
            {edges.iter().enumerate().map(|(i, ((from, to), amount))| {
                let default_pos = (center, center);
                let from_pos = node_positions.get(from).unwrap_or(&default_pos);
                let to_pos = node_positions.get(to).unwrap_or(&default_pos);
                let thickness = ((amount / max_edge) * 8.0).max(1.0).min(10.0);
                let opacity = 0.3 + (amount / max_edge) * 0.5;
                let color = get_edge_color(i);

                view! {
                    <g class="network-edge">
                        <line
                            x1=from_pos.0
                            y1=from_pos.1
                            x2=to_pos.0
                            y2=to_pos.1
                            stroke=color
                            stroke-width=thickness
                            stroke-opacity=opacity
                            stroke-linecap="round"
                            style="cursor: pointer;"
                        >
                            <title>{format!("{} → {}: {}", shorten_hash(from), shorten_hash(to), format_vol(*amount))}</title>
                        </line>
                        {
                            let dx = to_pos.0 - from_pos.0;
                            let dy = to_pos.1 - from_pos.1;
                            let dist = ((dx * dx + dy * dy) as f64).sqrt();
                            if dist > 0.0 {
                                let node_radius = 15.0;
                                let ux = dx as f64 / dist;
                                let uy = dy as f64 / dist;
                                let arrow_x = to_pos.0 as f64 - ux * node_radius;
                                let arrow_y = to_pos.1 as f64 - uy * node_radius;
                                view! {
                                    <circle cx=arrow_x cy=arrow_y r=3 fill=color opacity=opacity />
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }
                        }
                    </g>
                }
            }).collect_view()}

            {nodes.iter().enumerate().map(|(i, addr)| {
                let default_pos = (center, center);
                let pos = node_positions.get(addr).unwrap_or(&default_pos);
                let volume = node_volumes.get(addr).unwrap_or(&0.0);
                let node_size = ((*volume / max_volume) * 20.0).max(8.0).min(25.0);
                let color = get_node_color(i);

                view! {
                    <g class="network-node" style="cursor: pointer;">
                        <circle cx=pos.0 cy=pos.1 r=node_size + 5.0 fill=color opacity="0.2" />
                        <circle
                            cx=pos.0
                            cy=pos.1
                            r=node_size
                            fill=color
                            stroke="var(--bg-primary)"
                            stroke-width="2"
                        >
                            <title>{format!("{}\nVolume: {}", addr, format_vol(*volume))}</title>
                        </circle>
                        <text x=pos.0 y=pos.1 + node_size as i32 + 14 text-anchor="middle" fill="var(--text-secondary)" font-size="9" font-family="monospace">
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            <text x=center y=center - 5 text-anchor="middle" fill="var(--text-muted)" font-size="12">"USDFC"</text>
            <text x=center y=center + 10 text-anchor="middle" fill="var(--text-muted)" font-size="10">"Network"</text>
        </svg>
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_bar_color(index: usize) -> &'static str {
    if index < 6 { "#00d4ff" }
    else if index < 12 { "#6366f1" }
    else if index < 18 { "#a855f7" }
    else { "#ec4899" }
}

fn get_pie_color(index: usize) -> &'static str {
    const COLORS: [&str; 6] = ["#00d4ff", "#a855f7", "#22c55e", "#f59e0b", "#ec4899", "#6366f1"];
    COLORS[index % COLORS.len()]
}

fn get_flow_color(index: usize) -> &'static str {
    const COLORS: [&str; 8] = ["#00d4ff", "#a855f7", "#22c55e", "#f59e0b", "#ec4899", "#6366f1", "#14b8a6", "#f97316"];
    COLORS[index % COLORS.len()]
}

fn get_node_color(index: usize) -> &'static str {
    const COLORS: [&str; 6] = ["#00d4ff", "#22c55e", "#f59e0b", "#a855f7", "#ec4899", "#6366f1"];
    COLORS[index % COLORS.len()]
}

fn get_edge_color(index: usize) -> &'static str {
    const COLORS: [&str; 4] = ["#a855f7", "#00d4ff", "#6366f1", "#14b8a6"];
    COLORS[index % COLORS.len()]
}

fn format_vol(amount: f64) -> String {
    format_volume(amount)
}

fn format_time(ts: i64) -> String {
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
