use leptos::*;
use usdfc_backend::server_fn::get_recent_transactions;
use usdfc_core::format::format_count;
use std::collections::HashMap;

#[component]
pub fn SankeyCharts() -> impl IntoView {
    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(100)).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Sankey Charts"</h1>
                <p class="page-subtitle">"Interactive USDFC flow visualization"</p>
            </div>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Token Flow Diagram"</h3>
                        <p class="card-subtitle">"Proportional view of transfer paths (hover for details)"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| recent_tx.refetch()
                    >
                        "Refresh"
                    </button>
                </div>
                <Suspense fallback=move || view! {
                    <div style="text-align: center; padding: 60px;">
                        <div class="skeleton" style="width: 100%; height: 400px;"></div>
                    </div>
                }>
                    {move || {
                        recent_tx.get().map(|res: Result<_, leptos::ServerFnError>| {
                            match res {
                                Ok(txs) => {
                                    if txs.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-title">"No flow data"</div>
                                                <div class="empty-state-desc">"Recent transfers are required to build flow paths."</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        // Aggregate flows
                                        let mut edges: HashMap<(String, String), f64> = HashMap::new();
                                        let mut sources: HashMap<String, f64> = HashMap::new();
                                        let mut targets: HashMap<String, f64> = HashMap::new();

                                        for tx in &txs {
                                            let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            if amount > 0.0 {
                                                let key = (tx.from.clone(), tx.to.clone());
                                                *edges.entry(key).or_insert(0.0) += amount;
                                                *sources.entry(tx.from.clone()).or_insert(0.0) += amount;
                                                *targets.entry(tx.to.clone()).or_insert(0.0) += amount;
                                            }
                                        }

                                        // Sort and take top flows
                                        let mut ranked: Vec<_> = edges.into_iter().collect();
                                        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                                        let top_flows: Vec<_> = ranked.into_iter().take(8).collect();
                                        let max_amount = top_flows.first().map(|(_, v)| *v).unwrap_or(1.0);

                                        // Collect unique nodes
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
                                                // Legend
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
                                                                    {format_amount(*amount)}
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
                                        <div class="empty-state-title">"Flow Error"</div>
                                        <div class="empty-state-desc">{err.to_string()}</div>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Stats cards
            <div class="grid-3" style="margin-top: 24px;">
                <Suspense fallback=move || view! { <div class="card"><div class="skeleton" style="height: 80px;"></div></div> }>
                    {move || {
                        recent_tx.get().map(|res: Result<_, leptos::ServerFnError>| {
                            match res {
                                Ok(txs) => {
                                    let total: f64 = txs.iter()
                                        .map(|tx| tx.amount.to_string().parse::<f64>().unwrap_or(0.0))
                                        .sum();
                                    let unique_senders: std::collections::HashSet<_> = txs.iter().map(|tx| &tx.from).collect();
                                    let unique_receivers: std::collections::HashSet<_> = txs.iter().map(|tx| &tx.to).collect();

                                    view! {
                                        <div class="card">
                                            <div class="metric-label">"Total Volume"</div>
                                            <div class="metric-value cyan">{format_amount(total)}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Unique Senders"</div>
                                            <div class="metric-value green">{format_count(unique_senders.len())}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Unique Receivers"</div>
                                            <div class="metric-value purple">{format_count(unique_receivers.len())}</div>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="card"><div class="metric-label">"Error loading stats"</div></div>
                                    <div class="card"></div>
                                    <div class="card"></div>
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
fn SankeySVG(
    flows: Vec<((String, String), f64)>,
    left_nodes: Vec<String>,
    right_nodes: Vec<String>,
    max_amount: f64,
) -> impl IntoView {
    let svg_height = 400;
    let svg_width = 800;
    let node_width = 20;
    let left_x = 50;
    let right_x = svg_width - 70;

    // Calculate node positions
    let left_spacing = svg_height as f64 / (left_nodes.len() + 1) as f64;
    let right_spacing = svg_height as f64 / (right_nodes.len() + 1) as f64;

    view! {
        <svg
            viewBox=format!("0 0 {} {}", svg_width, svg_height)
            style="width: 100%; height: 400px; background: var(--bg-tertiary); border-radius: 4px;"
        >
            // Gradient definitions
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

            // Draw flow paths
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
                            style="transition: stroke-width 0.3s, opacity 0.3s; cursor: pointer;"
                            class="flow-path"
                        >
                            <title>{format!("{} → {}: {}", shorten_hash(from), shorten_hash(to), format_amount(*amount))}</title>
                        </path>
                    </g>
                }
            }).collect_view()}

            // Draw left nodes
            {left_nodes.iter().enumerate().map(|(i, addr)| {
                let y = ((i + 1) as f64 * left_spacing) as i32;
                view! {
                    <g>
                        <rect
                            x=left_x
                            y=y - 15
                            width=node_width
                            height=30
                            fill="var(--accent-cyan)"
                            rx=4
                        />
                        <text
                            x=left_x - 8
                            y=y + 4
                            text-anchor="end"
                            fill="var(--text-secondary)"
                            font-size="10"
                            font-family="monospace"
                        >
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            // Draw right nodes
            {right_nodes.iter().enumerate().map(|(i, addr)| {
                let y = ((i + 1) as f64 * right_spacing) as i32;
                view! {
                    <g>
                        <rect
                            x=right_x
                            y=y - 15
                            width=node_width
                            height=30
                            fill="var(--accent-purple)"
                            rx=4
                        />
                        <text
                            x=right_x + node_width + 8
                            y=y + 4
                            text-anchor="start"
                            fill="var(--text-secondary)"
                            font-size="10"
                            font-family="monospace"
                        >
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            // Labels
            <text x=left_x + 10 y=20 fill="var(--text-muted)" font-size="11" text-anchor="middle">"SENDERS"</text>
            <text x=right_x + 10 y=20 fill="var(--text-muted)" font-size="11" text-anchor="middle">"RECEIVERS"</text>
        </svg>
    }
}

fn get_flow_color(index: usize) -> &'static str {
    const COLORS: [&str; 8] = [
        "#00d4ff", // cyan
        "#a855f7", // purple
        "#22c55e", // green
        "#f59e0b", // yellow
        "#ec4899", // pink
        "#6366f1", // indigo
        "#14b8a6", // teal
        "#f97316", // orange
    ];
    COLORS[index % COLORS.len()]
}

fn format_amount(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        format!("{:.2}M USDFC", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.2}K USDFC", amount / 1_000.0)
    } else {
        format!("{:.2} USDFC", amount)
    }
}

fn shorten_hash(value: &str) -> String {
    if value.len() > 12 {
        format!("{}...{}", &value[0..6], &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}
