use leptos::*;
use crate::server_fn::get_recent_transactions;
use std::collections::HashMap;

#[component]
pub fn NetworkGraph() -> impl IntoView {
    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(100)).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Network Graph"</h1>
                <p class="page-subtitle">"Interactive USDFC holder and transaction network"</p>
            </div>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Transaction Network"</h3>
                        <p class="card-subtitle">"Node size = volume, edge thickness = transfer amount"</p>
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
                        <div class="skeleton" style="width: 100%; height: 500px;"></div>
                    </div>
                }>
                    {move || {
                        recent_tx.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    if txs.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-title">"No network data"</div>
                                                <div class="empty-state-desc">"Recent transfers required to build network."</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        // Build network data
                                        let mut edges: HashMap<(String, String), f64> = HashMap::new();
                                        let mut node_volumes: HashMap<String, f64> = HashMap::new();

                                        for tx in &txs {
                                            let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
                                            if amount > 0.0 {
                                                let key = (tx.from.clone(), tx.to.clone());
                                                *edges.entry(key).or_insert(0.0) += amount;
                                                *node_volumes.entry(tx.from.clone()).or_insert(0.0) += amount;
                                                *node_volumes.entry(tx.to.clone()).or_insert(0.0) += amount;
                                            }
                                        }

                                        // Get top edges and nodes
                                        let mut ranked_edges: Vec<_> = edges.into_iter().collect();
                                        ranked_edges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                                        let top_edges: Vec<_> = ranked_edges.into_iter().take(15).collect();

                                        // Collect unique nodes from top edges
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
                                        <div class="empty-state-title">"Network Error"</div>
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
                            recent_tx.get().map(|res| {
                                match res {
                                    Ok(txs) => {
                                        let mut edges: HashMap<(String, String), (f64, u32)> = HashMap::new();
                                        for tx in &txs {
                                            let amount = tx.amount.to_string().parse::<f64>().unwrap_or(0.0);
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
                                                                <span style="color: var(--accent-cyan);">{format_amount(*amount)}</span>
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
                        <div style="display: flex; align-items: center; gap: 12px;">
                            <div style="width: 12px; height: 12px; background: var(--accent-green); border-radius: 50%;"></div>
                            <span style="color: var(--text-secondary); font-size: 13px;">"Green = High activity sender"</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 12px;">
                            <div style="width: 12px; height: 12px; background: var(--accent-yellow); border-radius: 50%;"></div>
                            <span style="color: var(--text-secondary); font-size: 13px;">"Yellow = High activity receiver"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
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

    // Calculate node positions in a circle
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
            // Draw edges first (behind nodes)
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
                            style="cursor: pointer; transition: stroke-opacity 0.2s, stroke-width 0.2s;"
                        >
                            <title>{format!("{} → {}: {}", shorten_hash(from), shorten_hash(to), format_amount(*amount))}</title>
                        </line>
                        // Arrow head
                        {
                            let dx = to_pos.0 - from_pos.0;
                            let dy = to_pos.1 - from_pos.1;
                            let dist = ((dx * dx + dy * dy) as f64).sqrt();
                            if dist > 0.0 {
                                let node_radius = 15.0; // offset from node center
                                let ux = dx as f64 / dist;
                                let uy = dy as f64 / dist;
                                let arrow_x = to_pos.0 as f64 - ux * node_radius;
                                let arrow_y = to_pos.1 as f64 - uy * node_radius;
                                view! {
                                    <circle
                                        cx=arrow_x
                                        cy=arrow_y
                                        r=3
                                        fill=color
                                        opacity=opacity
                                    />
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }
                        }
                    </g>
                }
            }).collect_view()}

            // Draw nodes
            {nodes.iter().enumerate().map(|(i, addr)| {
                let default_pos = (center, center);
                let pos = node_positions.get(addr).unwrap_or(&default_pos);
                let volume = node_volumes.get(addr).unwrap_or(&0.0);
                let node_size = ((*volume / max_volume) * 20.0).max(8.0).min(25.0);
                let color = get_node_color(i);

                view! {
                    <g class="network-node" style="cursor: pointer;">
                        // Glow effect
                        <circle
                            cx=pos.0
                            cy=pos.1
                            r=node_size + 5.0
                            fill=color
                            opacity="0.2"
                        />
                        // Main node
                        <circle
                            cx=pos.0
                            cy=pos.1
                            r=node_size
                            fill=color
                            stroke="var(--bg-primary)"
                            stroke-width="2"
                            style="transition: r 0.2s;"
                        >
                            <title>{format!("{}\nVolume: {}", addr, format_amount(*volume))}</title>
                        </circle>
                        // Label
                        <text
                            x=pos.0
                            y=pos.1 + node_size as i32 + 14
                            text-anchor="middle"
                            fill="var(--text-secondary)"
                            font-size="9"
                            font-family="monospace"
                        >
                            {shorten_hash(addr)}
                        </text>
                    </g>
                }
            }).collect_view()}

            // Center label
            <text x=center y=center - 5 text-anchor="middle" fill="var(--text-muted)" font-size="12">"USDFC"</text>
            <text x=center y=center + 10 text-anchor="middle" fill="var(--text-muted)" font-size="10">"Network"</text>
        </svg>
    }
}

fn get_node_color(index: usize) -> &'static str {
    const COLORS: [&str; 6] = [
        "#00d4ff", // cyan
        "#22c55e", // green
        "#f59e0b", // yellow
        "#a855f7", // purple
        "#ec4899", // pink
        "#6366f1", // indigo
    ];
    COLORS[index % COLORS.len()]
}

fn get_edge_color(index: usize) -> &'static str {
    const COLORS: [&str; 4] = [
        "#a855f7", // purple
        "#00d4ff", // cyan
        "#6366f1", // indigo
        "#14b8a6", // teal
    ];
    COLORS[index % COLORS.len()]
}

fn format_amount(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        format!("{:.2}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.2}K", amount / 1_000.0)
    } else {
        format!("{:.2}", amount)
    }
}

fn shorten_hash(value: &str) -> String {
    if value.len() > 12 {
        format!("{}...{}", &value[0..6], &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}
