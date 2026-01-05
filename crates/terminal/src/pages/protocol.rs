//! Protocol Page
//!
//! Unique insights for Supply, Collateral, and Stability.
//! No data duplication with Dashboard or Header.

use leptos::*;
use crate::components::tabs::{TabNav, TabContent, Tab};
use crate::components::icons::*;
use crate::components::gauge::GaugeChart;
use usdfc_core::config::config;
use crate::global_metrics::use_global_metrics;
use usdfc_api::{get_troves, get_top_holders, get_recent_transactions, get_stability_pool_transfers};
use usdfc_core::format::{format_usd, format_fil, format_usdfc, format_amount, format_timestamp, shorten_hash, format_volume, decimal_to_f64};
use usdfc_core::types::TransactionType;

/// Normalize negative zero to positive zero for display purposes
#[inline]
fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

#[component]
pub fn Protocol() -> impl IntoView {
    let active_tab = create_rw_signal("supply".to_string());
    let global = use_global_metrics();

    let tabs = vec![
        Tab { id: "supply", label: "Supply Dynamics" },
        Tab { id: "collateral", label: "Risk Analysis" },
        Tab { id: "stability", label: "Pool Activity" },
    ];

    view! {
        <div class="fade-in">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Protocol"</h1>
                    <p class="page-subtitle">"USDFC protocol insights"</p>
                </div>
                <button
                    class="btn btn-secondary"
                    on:click=move |_| global.refresh_all()
                >
                    <RefreshIcon />
                    "Refresh"
                </button>
            </div>

            <TabNav tabs=tabs active=active_tab />

            <TabContent id="supply" active=active_tab>
                <SupplyDynamicsTab />
            </TabContent>

            <TabContent id="collateral" active=active_tab>
                <RiskAnalysisTab />
            </TabContent>

            <TabContent id="stability" active=active_tab>
                <PoolActivityTab />
            </TabContent>
        </div>
    }
}

// ============================================================================
// Supply Dynamics Tab - Concentration, Distribution, Movements
// ============================================================================

#[component]
fn SupplyDynamicsTab() -> impl IntoView {
    let global = use_global_metrics();

    let top_holders = create_resource(
        || (),
        |_| async move { get_top_holders(Some(10), None).await }
    );

    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(50)).await }
    );

    view! {
        // Concentration Metrics
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 16px;">"Supply Concentration"</h3>
            <Suspense fallback=move || view! { <div class="skeleton" style="height: 100px;"></div> }>
                {move || {
                    let holders_data = top_holders.get();
                    let protocol_data = global.protocol.get();

                    match (holders_data, protocol_data) {
                        (Some(Ok(holders)), Some(Ok(metrics))) => {
                            let total_supply_f64 = decimal_to_f64(metrics.total_supply);
                            let top_10_total: f64 = holders.iter()
                                .map(|h| decimal_to_f64(h.balance))
                                .sum();
                            let top_10_pct = if total_supply_f64 > 0.0 {
                                top_10_total / total_supply_f64 * 100.0
                            } else { 0.0 };

                            let top_1 = holders.first()
                                .map(|h| decimal_to_f64(h.balance) / total_supply_f64 * 100.0)
                                .unwrap_or(0.0);
                            let top_5: f64 = holders.iter().take(5)
                                .map(|h| decimal_to_f64(h.balance))
                                .sum::<f64>() / total_supply_f64 * 100.0;

                            // Concentration indicator
                            let (concentration_status, concentration_color) = if top_10_pct > 80.0 {
                                ("Highly Concentrated", "var(--accent-red)")
                            } else if top_10_pct > 60.0 {
                                ("Moderately Concentrated", "var(--accent-yellow)")
                            } else {
                                ("Well Distributed", "var(--accent-green)")
                            };

                            view! {
                                <div class="grid-4">
                                    <div>
                                        <div class="metric-label">"Top 1 Holder"</div>
                                        <div class="metric-value purple">{format!("{:.1}%", normalize_zero(top_1))}</div>
                                    </div>
                                    <div>
                                        <div class="metric-label">"Top 5 Holders"</div>
                                        <div class="metric-value cyan">{format!("{:.1}%", normalize_zero(top_5))}</div>
                                    </div>
                                    <div>
                                        <div class="metric-label">"Top 10 Holders"</div>
                                        <div class="metric-value green">{format!("{:.1}%", normalize_zero(top_10_pct))}</div>
                                    </div>
                                    <div>
                                        <div class="metric-label">"Distribution"</div>
                                        <div class="metric-value" style=format!("color: {}", concentration_color)>
                                            {concentration_status}
                                        </div>
                                    </div>
                                </div>
                            }.into_view()
                        }
                        _ => view! {
                            <div style="color: var(--text-muted);">"Loading concentration data..."</div>
                        }.into_view()
                    }
                }}
            </Suspense>
        </div>

        // Supply Activity (Mint/Burn Analysis)
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 16px;">"Recent Supply Activity"</h3>
            <Suspense fallback=move || view! { <div class="skeleton" style="height: 100px;"></div> }>
                {move || {
                    recent_tx.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                let mints: Vec<_> = txs.iter()
                                    .filter(|tx| matches!(tx.tx_type, TransactionType::Mint))
                                    .collect();
                                let burns: Vec<_> = txs.iter()
                                    .filter(|tx| matches!(tx.tx_type, TransactionType::Burn))
                                    .collect();

                                let mint_volume: f64 = mints.iter()
                                    .map(|tx| decimal_to_f64(tx.amount))
                                    .sum();
                                let burn_volume: f64 = burns.iter()
                                    .map(|tx| decimal_to_f64(tx.amount))
                                    .sum();

                                let net_change = mint_volume - burn_volume;
                                let ratio = if burn_volume > 0.0 { mint_volume / burn_volume } else { 0.0 };

                                // Format volume with sign, handling zero properly
                                let format_signed_volume = |v: f64| -> String {
                                    if v.abs() < 0.01 {
                                        "0".to_string()
                                    } else if v > 0.0 {
                                        format!("+{}", format_volume(v))
                                    } else {
                                        format!("-{}", format_volume(v.abs()))
                                    }
                                };

                                let net_change_display = format_signed_volume(net_change);
                                let net_change_class = if net_change.abs() < 0.01 {
                                    "metric-value cyan"
                                } else if net_change > 0.0 {
                                    "metric-value green"
                                } else {
                                    "metric-value red"
                                };

                                view! {
                                    <div class="grid-4">
                                        <div>
                                            <div class="metric-label">"Mints"</div>
                                            <div class="metric-value green">{format_volume(mint_volume)}</div>
                                            <div style="color: var(--text-muted); font-size: 11px;">{format!("{} txs", mints.len())}</div>
                                        </div>
                                        <div>
                                            <div class="metric-label">"Burns"</div>
                                            <div class="metric-value red">{format_volume(burn_volume)}</div>
                                            <div style="color: var(--text-muted); font-size: 11px;">{format!("{} txs", burns.len())}</div>
                                        </div>
                                        <div>
                                            <div class="metric-label">"Net Change"</div>
                                            <div class=net_change_class>
                                                {net_change_display}
                                            </div>
                                        </div>
                                        <div>
                                            <div class="metric-label">"Mint/Burn Ratio"</div>
                                            <div class="metric-value cyan">
                                                {if ratio > 0.0 { format!("{:.2}:1", ratio) } else { "N/A".to_string() }}
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <div>"Error loading activity"</div> }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>

        // Top Holders Table
        <div class="card">
            <div class="card-header">
                <div>
                    <h3 class="card-title">"Holder Distribution"</h3>
                    <p class="card-subtitle">"Top 10 USDFC holders by balance"</p>
                </div>
                <button class="btn btn-secondary" on:click=move |_| top_holders.refetch()>
                    "Refresh"
                </button>
            </div>
            <div class="table-responsive">
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th class="hide-mobile">"Rank"</th>
                                <th>"Address"</th>
                                <th>"Balance"</th>
                                <th>"Share"</th>
                                <th>"Distribution"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="5" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    let holders_data = top_holders.get();
                                    let protocol_data = global.protocol.get();

                                    match (holders_data, protocol_data) {
                                        (Some(Ok(holders)), Some(Ok(metrics))) => {
                                            if holders.is_empty() {
                                                view! {
                                                    <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--text-muted);">"No holder data"</td></tr>
                                                }.into_view()
                                            } else {
                                                let total_supply = decimal_to_f64(metrics.total_supply).max(1.0);
                                                holders.iter().enumerate().map(|(i, holder)| {
                                                    let balance_f64 = decimal_to_f64(holder.balance);
                                                    let balance_display = format_usd(holder.balance);
                                                    let share = balance_f64 / total_supply * 100.0;
                                                    let bar_width = (share * 2.0).min(100.0);
                                                    let addr = holder.address.clone();
                                                    view! {
                                                        <tr>
                                                            <td class="hide-mobile" style="color: var(--text-muted);">{format!("#{}", i + 1)}</td>
                                                            <td style="font-family: monospace; font-size: 12px;">
                                                                <div style="display: flex; align-items: center; gap: 4px;">
                                                                    <a href=format!("/address/{}", addr) style="color: var(--accent-cyan); text-decoration: none;" title=addr.clone()>
                                                                        {shorten_hash(&addr)}
                                                                    </a>
                                                                    <a href=format!("https://filecoin.blockscout.com/address/{}", addr) target="_blank" style="color: var(--text-muted); text-decoration: none; font-size: 10px;" title="View on Blockscout">
                                                                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                            <polyline points="15 3 21 3 21 9"></polyline>
                                                                            <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                        </svg>
                                                                    </a>
                                                                </div>
                                                            </td>
                                                            <td style="font-family: monospace; color: var(--accent-cyan);">{balance_display}</td>
                                                            <td style="color: var(--text-muted);">{format!("{:.2}%", normalize_zero(share))}</td>
                                                            <td style="min-width: 80px; flex: 1;">
                                                                <div style="background: var(--bg-tertiary); border-radius: 4px; height: 8px; width: 100%;">
                                                                    <div style=format!("background: var(--accent-cyan); border-radius: 4px; height: 100%; width: {}%;", bar_width)></div>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()
                                            }
                                        }
                                        _ => view! {
                                            <tr><td colspan="5" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                                        }.into_view()
                                    }
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Risk Analysis Tab - Trove Health, Liquidation Risk
// ============================================================================

#[component]
fn RiskAnalysisTab() -> impl IntoView {
    let global = use_global_metrics();
    let troves = create_resource(
        || (),
        |_| async move { get_troves(Some(50), None).await }
    );

    view! {
        // TCR Gauge - Prominent display at top
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 16px; text-align: center;">"Total Collateral Ratio"</h3>
            <Suspense fallback=move || view! { <div class="skeleton" style="height: 180px;"></div> }>
                {move || {
                    global.protocol.get().map(|res| {
                        match res {
                            Ok(metrics) => {
                                let tcr = decimal_to_f64(metrics.tcr);
                                view! {
                                    <div style="display: flex; justify-content: center;">
                                        <GaugeChart
                                            value=tcr
                                            min=110.0
                                            max=200.0
                                            label="TCR"
                                            suffix="%"
                                        />
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <div style="text-align: center; color: var(--text-muted);">"Error loading TCR"</div> }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>

        // Risk Summary
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 16px;">"Liquidation Risk Summary"</h3>
            <Suspense fallback=move || view! { <div class="skeleton" style="height: 100px;"></div> }>
                {move || {
                    troves.get().map(|res| {
                        match res {
                            Ok(all_troves) => {
                                let total = all_troves.len();

                                // Show message if no active troves
                                if total == 0 {
                                    return view! {
                                        <div style="text-align: center; padding: 40px 20px; color: var(--text-muted);">
                                            <div style="font-size: 14px; margin-bottom: 8px;">"No active troves"</div>
                                            <div style="font-size: 12px;">"There are currently no open borrowing positions in the protocol."</div>
                                        </div>
                                    }.into_view();
                                }

                                let critical: Vec<_> = all_troves.iter()
                                    .filter(|t| decimal_to_f64(t.icr) < 115.0)
                                    .collect();
                                let at_risk: Vec<_> = all_troves.iter()
                                    .filter(|t| {
                                        let icr = decimal_to_f64(t.icr);
                                        icr >= 115.0 && icr < config().tcr_danger_threshold
                                    })
                                    .collect();
                                let healthy: Vec<_> = all_troves.iter()
                                    .filter(|t| decimal_to_f64(t.icr) >= config().tcr_danger_threshold)
                                    .collect();

                                // Calculate at-risk collateral
                                let critical_collateral: f64 = critical.iter()
                                    .map(|t| decimal_to_f64(t.collateral))
                                    .sum();
                                let at_risk_collateral: f64 = at_risk.iter()
                                    .map(|t| decimal_to_f64(t.collateral))
                                    .sum();

                                // Average and median ICR
                                let icrs: Vec<f64> = all_troves.iter()
                                    .map(|t| decimal_to_f64(t.icr))
                                    .collect();
                                let avg_icr = if !icrs.is_empty() {
                                    icrs.iter().sum::<f64>() / icrs.len() as f64
                                } else { 0.0 };
                                let median_icr = if !icrs.is_empty() {
                                    let mut sorted = icrs.clone();
                                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                                    sorted[sorted.len() / 2]
                                } else { 0.0 };

                                let format_fil_compact = |v: f64| {
                                    if v >= 1_000.0 { format!("{:.1}K FIL", v / 1_000.0) }
                                    else { format!("{:.1} FIL", v) }
                                };

                                view! {
                                    <div class="grid-2" style="margin-bottom: 24px;">
                                        <div class="grid-3">
                                            <div>
                                                <div class="metric-label">"Critical (< 115%)"</div>
                                                <div class="metric-value red">{critical.len().to_string()}</div>
                                                <div style="color: var(--text-muted); font-size: 11px;">{format_fil_compact(critical_collateral)}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"At Risk (< 150%)"</div>
                                                <div class="metric-value yellow">{at_risk.len().to_string()}</div>
                                                <div style="color: var(--text-muted); font-size: 11px;">{format_fil_compact(at_risk_collateral)}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Healthy (>= 150%)"</div>
                                                <div class="metric-value green">{healthy.len().to_string()}</div>
                                            </div>
                                        </div>
                                        <div class="grid-3">
                                            <div>
                                                <div class="metric-label">"Total Troves"</div>
                                                <div class="metric-value cyan">{total.to_string()}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Average ICR"</div>
                                                <div class="metric-value purple">{format!("{:.1}%", normalize_zero(avg_icr))}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Median ICR"</div>
                                                <div class="metric-value purple">{format!("{:.1}%", normalize_zero(median_icr))}</div>
                                            </div>
                                        </div>
                                    </div>

                                    // Risk distribution bar
                                    <div style="margin-top: 16px;">
                                        <div class="metric-label" style="margin-bottom: 8px;">"Trove Health Distribution"</div>
                                        <div style="display: flex; height: 24px; border-radius: 4px; overflow: hidden;">
                                            {if critical.len() > 0 {
                                                let pct = critical.len() as f64 / total as f64 * 100.0;
                                                view! {
                                                    <div style=format!("background: var(--accent-red); width: {}%; display: flex; align-items: center; justify-content: center;", pct)>
                                                        <span style="color: white; font-size: 10px; font-weight: 600;">{format!("{}%", pct as i32)}</span>
                                                    </div>
                                                }.into_view()
                                            } else { view! {}.into_view() }}
                                            {if at_risk.len() > 0 {
                                                let pct = at_risk.len() as f64 / total as f64 * 100.0;
                                                view! {
                                                    <div style=format!("background: var(--accent-yellow); width: {}%; display: flex; align-items: center; justify-content: center;", pct)>
                                                        <span style="color: black; font-size: 10px; font-weight: 600;">{format!("{}%", pct as i32)}</span>
                                                    </div>
                                                }.into_view()
                                            } else { view! {}.into_view() }}
                                            {if healthy.len() > 0 {
                                                let pct = healthy.len() as f64 / total as f64 * 100.0;
                                                view! {
                                                    <div style=format!("background: var(--accent-green); width: {}%; display: flex; align-items: center; justify-content: center;", pct)>
                                                        <span style="color: white; font-size: 10px; font-weight: 600;">{format!("{}%", pct as i32)}</span>
                                                    </div>
                                                }.into_view()
                                            } else { view! {}.into_view() }}
                                        </div>
                                        <div style="display: flex; justify-content: space-between; margin-top: 8px; font-size: 11px; color: var(--text-muted);">
                                            <span>"Critical < 115%"</span>
                                            <span>"At Risk < 150%"</span>
                                            <span>"Healthy ≥ 150%"</span>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <div>"Error loading troves"</div> }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>

        // Troves Table (sorted by risk)
        <div class="card">
            <div class="card-header">
                <div>
                    <h3 class="card-title">"Active Troves"</h3>
                    <p class="card-subtitle">"Sorted by collateral ratio (lowest first)"</p>
                </div>
                <button class="btn btn-secondary" on:click=move |_| troves.refetch()>
                    "Refresh"
                </button>
            </div>
            <div class="table-responsive">
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Address"</th>
                                <th>"FIL Collateral"</th>
                                <th>"USDFC Debt"</th>
                                <th>"ICR"</th>
                                <th class="hide-mobile">"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="5" style="text-align: center; padding: 20px;">"Loading troves..."</td></tr>
                            }>
                                {move || {
                                    troves.get().map(|res| {
                                        match res {
                                            Ok(mut all_troves) => {
                                                if all_troves.is_empty() {
                                                    view! {
                                                        <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--text-muted);">"No active troves"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    // Sort by ICR ascending (riskiest first)
                                                    all_troves.sort_by(|a, b| {
                                                        decimal_to_f64(a.icr)
                                                            .partial_cmp(&decimal_to_f64(b.icr))
                                                            .unwrap_or(std::cmp::Ordering::Equal)
                                                    });
                                                    all_troves.iter().take(20).map(|t| {
                                                        let icr = decimal_to_f64(t.icr);
                                                        let (status_class, status_text) = if icr < 115.0 {
                                                            ("color: var(--accent-red);", "CRITICAL")
                                                        } else if icr < config().tcr_danger_threshold {
                                                            ("color: var(--accent-yellow);", "AT RISK")
                                                        } else if icr < config().tcr_warning_threshold {
                                                            ("color: var(--accent-green);", "Healthy")
                                                        } else {
                                                            ("color: var(--accent-green);", "Safe")
                                                        };
                                                        let collateral = format_fil(t.collateral);
                                                        let debt = format_usdfc(t.debt);
                                                        let short_addr = format!("{}...{}", &t.address[..8], &t.address[t.address.len()-6..]);
                                                        view! {
                                                            <tr>
                                                                <td style="font-family: monospace; font-size: 12px;">
                                                                    <div style="display: flex; align-items: center; gap: 4px;">
                                                                        <a href=format!("/address/{}", t.address) style="color: var(--accent-cyan); text-decoration: none;" title=t.address.clone()>
                                                                            {short_addr}
                                                                        </a>
                                                                        <a href=format!("https://filecoin.blockscout.com/address/{}", t.address) target="_blank" style="color: var(--text-muted); text-decoration: none; font-size: 10px;" title="View on Blockscout">
                                                                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                                <polyline points="15 3 21 3 21 9"></polyline>
                                                                                <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                            </svg>
                                                                        </a>
                                                                    </div>
                                                                </td>
                                                                <td style="font-family: monospace;">{collateral}</td>
                                                                <td style="font-family: monospace;">{debt}</td>
                                                                <td style=status_class>{format!("{:.1}%", normalize_zero(icr))}</td>
                                                                <td class="hide-mobile"><span style=status_class>{status_text}</span></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Pool Activity Tab - Stability Pool Dynamics
// ============================================================================

#[component]
fn PoolActivityTab() -> impl IntoView {
    let global = use_global_metrics();

    let pool_transfers = create_resource(
        || (),
        |_| async move { get_stability_pool_transfers(Some(50)).await }
    );

    view! {
        // Pool Flow Analysis
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 16px;">"Pool Flow Analysis"</h3>
            <Suspense fallback=move || view! { <div class="skeleton" style="height: 100px;"></div> }>
                {move || {
                    let transfers = pool_transfers.get();
                    let protocol = global.protocol.get();

                    match (transfers, protocol) {
                        (Some(Ok(txs)), Some(Ok(metrics))) => {
                            let stability_pool_addr = config().stability_pool.to_lowercase();

                            let deposits: Vec<_> = txs.iter()
                                .filter(|tx| tx.to.to_lowercase() == stability_pool_addr)
                                .collect();
                            let withdrawals: Vec<_> = txs.iter()
                                .filter(|tx| tx.from.to_lowercase() == stability_pool_addr)
                                .collect();

                            let deposit_vol: f64 = deposits.iter()
                                .map(|tx| decimal_to_f64(tx.amount))
                                .sum();
                            let withdrawal_vol: f64 = withdrawals.iter()
                                .map(|tx| decimal_to_f64(tx.amount))
                                .sum();

                            let net_flow = deposit_vol - withdrawal_vol;
                            let ratio = if withdrawal_vol > 0.0 { deposit_vol / withdrawal_vol } else { 0.0 };

                            // Pool utilization
                            let pool_balance = decimal_to_f64(metrics.stability_pool_balance);
                            let total_supply = decimal_to_f64(metrics.total_supply).max(1.0);
                            let utilization = pool_balance / total_supply * 100.0;

                            // Flow direction indicator
                            let (flow_text, flow_color) = if net_flow > 1000.0 {
                                ("Inflow", "var(--accent-green)")
                            } else if net_flow < -1000.0 {
                                ("Outflow", "var(--accent-red)")
                            } else {
                                ("Stable", "var(--accent-cyan)")
                            };

                            // Format net flow with sign, handling zero properly
                            let net_flow_display = if net_flow.abs() < 0.01 {
                                "0".to_string()
                            } else if net_flow > 0.0 {
                                format!("+{}", format_volume(net_flow))
                            } else {
                                format!("-{}", format_volume(net_flow.abs()))
                            };

                            view! {
                                <div class="grid-2" style="margin-bottom: 24px;">
                                    <div class="grid-2">
                                        <div>
                                            <div class="metric-label">"Deposits"</div>
                                            <div class="metric-value green">{format_volume(deposit_vol)}</div>
                                            <div style="color: var(--text-muted); font-size: 11px;">{format!("{} txs", deposits.len())}</div>
                                        </div>
                                        <div>
                                            <div class="metric-label">"Withdrawals"</div>
                                            <div class="metric-value red">{format_volume(withdrawal_vol)}</div>
                                            <div style="color: var(--text-muted); font-size: 11px;">{format!("{} txs", withdrawals.len())}</div>
                                        </div>
                                    </div>
                                    <div class="grid-2">
                                        <div>
                                            <div class="metric-label">"Net Flow"</div>
                                            <div class="metric-value" style=format!("color: {}", flow_color)>
                                                {net_flow_display}
                                            </div>
                                            <div style=format!("color: {}; font-size: 11px;", flow_color)>{flow_text}</div>
                                        </div>
                                        <div>
                                            <div class="metric-label">"Deposit/Withdrawal"</div>
                                            <div class="metric-value cyan">
                                                {if ratio > 0.0 { format!("{:.2}:1", ratio) } else { "N/A".to_string() }}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // Pool utilization bar with gradient visualization
                                <div>
                                    <div class="metric-label" style="margin-bottom: 8px;">"Pool Utilization (% of Supply)"</div>
                                    <div style="display: flex; align-items: center; gap: 12px;">
                                        <div
                                            class="utilization-bar"
                                            style=format!(
                                                "flex: 1; height: 20px; border-radius: 4px; background: linear-gradient(to right, var(--accent-purple) 0%, var(--accent-cyan) {}%, rgba(255,255,255,0.1) {}%, rgba(255,255,255,0.1) 100%); display: flex; align-items: center; justify-content: flex-end; padding-right: 8px;",
                                                utilization.min(100.0), utilization.min(100.0)
                                            )
                                        >
                                            {if utilization > 10.0 {
                                                view! { <span style="color: white; font-size: 11px; font-weight: 600;">{format!("{:.1}%", normalize_zero(utilization))}</span> }.into_view()
                                            } else { view! {}.into_view() }}
                                        </div>
                                        {if utilization <= 10.0 {
                                            view! { <span style="color: var(--text-muted); font-size: 12px;">{format!("{:.1}%", normalize_zero(utilization))}</span> }.into_view()
                                        } else { view! {}.into_view() }}
                                    </div>
                                </div>
                            }.into_view()
                        }
                        _ => view! { <div style="color: var(--text-muted);">"Loading pool data..."</div> }.into_view()
                    }
                }}
            </Suspense>
        </div>

        // Pool Info Cards
        <div class="grid-2" style="margin-bottom: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"How It Works"</h3>
                <p style="color: var(--text-secondary); line-height: 1.8; font-size: 13px;">
                    "Deposit USDFC to the Stability Pool to absorb liquidations.
                    When a Trove is liquidated below 110% ICR, your USDFC repays the debt
                    and you receive the FIL collateral at a ~10% discount."
                </p>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Rewards"</h3>
                <p style="color: var(--text-secondary); line-height: 1.8; font-size: 13px;">
                    "Earn FIL from liquidations proportional to your pool share.
                    Higher pool utilization means more frequent liquidation opportunities
                    but also more competition for rewards."
                </p>
            </div>
        </div>

        // Recent Pool Activity
        <div class="card">
            <div class="card-header">
                <div>
                    <h3 class="card-title">"Recent Pool Activity"</h3>
                    <p class="card-subtitle">"Stability Pool deposits and withdrawals"</p>
                </div>
                <button class="btn btn-secondary" on:click=move |_| pool_transfers.refetch()>
                    "Refresh"
                </button>
            </div>
            <div class="table-responsive">
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th class="hide-mobile">"TX Hash"</th>
                                <th>"Action"</th>
                                <th>"Amount"</th>
                                <th>"Address"</th>
                                <th>"Time"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="5" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    pool_transfers.get().map(|res| {
                                        match res {
                                            Ok(txs) => {
                                                if txs.is_empty() {
                                                    view! {
                                                        <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--text-muted);">"No pool activity"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    let stability_pool_addr = "0x791ad78bbc58324089d3e0a8689e7d045b9592b5".to_lowercase();
                                                    txs.iter().take(15).map(|tx| {
                                                        let is_deposit = tx.to.to_lowercase() == stability_pool_addr;
                                                        let (action, action_class, user_addr) = if is_deposit {
                                                            ("Deposit", "color: var(--accent-green);", tx.from.clone())
                                                        } else {
                                                            ("Withdraw", "color: var(--accent-red);", tx.to.clone())
                                                        };
                                                        let time = format_timestamp(tx.timestamp);
                                                        let amount = format_amount(tx.amount);
                                                        view! {
                                                            <tr>
                                                                <td class="hide-mobile" style="font-family: monospace; font-size: 12px;">
                                                                    <a href=format!("https://filecoin.blockscout.com/tx/{}", tx.hash) target="_blank" style="color: var(--text-primary); text-decoration: none;" title=tx.hash.clone()>
                                                                        {shorten_hash(&tx.hash)}
                                                                    </a>
                                                                </td>
                                                                <td style=action_class>{action}</td>
                                                                <td style="font-family: monospace;">{amount}</td>
                                                                <td style="font-family: monospace; font-size: 12px;">
                                                                    <div style="display: flex; align-items: center; gap: 4px;">
                                                                        <a href=format!("/address/{}", user_addr) style="color: var(--accent-cyan); text-decoration: none;" title=user_addr.clone()>
                                                                            {shorten_hash(&user_addr)}
                                                                        </a>
                                                                        <a href=format!("https://filecoin.blockscout.com/address/{}", user_addr) target="_blank" style="color: var(--text-muted); text-decoration: none; font-size: 10px;" title="View on Blockscout">
                                                                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                                <polyline points="15 3 21 3 21 9"></polyline>
                                                                                <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                            </svg>
                                                                        </a>
                                                                    </div>
                                                                </td>
                                                                <td>{time}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Loading Component
// ============================================================================

#[component]
fn MetricsLoading() -> impl IntoView {
    view! {
        <div class="grid-4" style="margin-bottom: 24px;">
            {(0..4).map(|_| view! {
                <div class="card" style="min-height: 80px;">
                    <div class="skeleton" style="height: 14px; width: 80px; margin-bottom: 8px;"></div>
                    <div class="skeleton" style="height: 24px; width: 100px;"></div>
                </div>
            }).collect_view()}
        </div>
    }
}
