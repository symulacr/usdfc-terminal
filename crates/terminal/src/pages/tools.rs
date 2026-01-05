//! Tools Page
//!
//! Combined view of Export, API Reference, Gas Estimator, and Alerts.
//! Uses tabs for navigation between sections.

use leptos::*;
use crate::components::tabs::{TabNav, TabContent, Tab};
use crate::components::icons::*;
use usdfc_core::config::config;
use usdfc_api::get_recent_transactions;
use crate::global_metrics::use_global_metrics;
use usdfc_core::format::{format_amount, shorten_hash, decimal_to_f64};

/// Estimate gas for common USDFC protocol operations
fn estimate_gas(operation: &str) -> u64 {
    match operation {
        "open_trove" => 500_000,
        "adjust_trove" => 300_000,
        "close_trove" => 200_000,
        "provide_sp" => 150_000,
        "withdraw_sp" => 150_000,
        "transfer" => 65_000,
        "approve" => 46_000,
        "swap" => 200_000,
        _ => 100_000,
    }
}

/// Format number with thousands separators
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let formatted: String = chars
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join(",")
        .chars()
        .rev()
        .collect();
    formatted
}

#[component]
pub fn Tools() -> impl IntoView {
    let active_tab = create_rw_signal("export".to_string());

    let tabs = vec![
        Tab { id: "export", label: "Export" },
        Tab { id: "api", label: "API Reference" },
        Tab { id: "gas", label: "Gas Estimator" },
        Tab { id: "alerts", label: "Alerts" },
    ];

    view! {
        <div class="fade-in">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Tools"</h1>
                    <p class="page-subtitle">"Data export, API access, and monitoring"</p>
                </div>
            </div>

            <TabNav tabs=tabs active=active_tab />

            <TabContent id="export" active=active_tab>
                <ExportTab />
            </TabContent>

            <TabContent id="api" active=active_tab>
                <ApiTab />
            </TabContent>

            <TabContent id="gas" active=active_tab>
                <GasEstimatorTab />
            </TabContent>

            <TabContent id="alerts" active=active_tab>
                <AlertsTab />
            </TabContent>
        </div>
    }
}

// ============================================================================
// Export Tab
// ============================================================================

#[component]
fn ExportTab() -> impl IntoView {
    let cfg = config();
    let usdfc_token = cfg.usdfc_token.clone();
    let blockscout_url = "https://filecoin.blockscout.com";
    let subgraph_url = cfg.subgraph_url.clone();

    view! {
        // Blockscout Exports
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 8px;">"Blockscout CSV Exports"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Direct download from Filecoin Blockscout"</p>
            <div class="grid-2" style="gap: 16px;">
                <div class="export-card">
                    <h4>"Token Transfers"</h4>
                    <p>"All USDFC transfers in CSV format"</p>
                    <a
                        href=format!("{}/api/v2/tokens/{}/transfers?csv=true", blockscout_url, usdfc_token)
                        target="_blank"
                        class="btn btn-primary"
                    >
                        <DownloadIcon />
                        "Download CSV"
                    </a>
                </div>
                <div class="export-card">
                    <h4>"Token Holders"</h4>
                    <p>"All USDFC holders with balances"</p>
                    <a
                        href=format!("{}/api/v2/tokens/{}/holders?csv=true", blockscout_url, usdfc_token)
                        target="_blank"
                        class="btn btn-primary"
                    >
                        <DownloadIcon />
                        "Download CSV"
                    </a>
                </div>
            </div>
        </div>

        // JSON API Endpoints
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 8px;">"JSON API Endpoints"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Programmatic access to USDFC data"</p>
            <div class="api-endpoints">
                <ApiEndpointRow
                    method="GET"
                    path=format!("/api/v2/tokens/{}/transfers", usdfc_token)
                    description="Token transfer history"
                    base_url=blockscout_url.to_string()
                />
                <ApiEndpointRow
                    method="GET"
                    path=format!("/api/v2/tokens/{}/holders", usdfc_token)
                    description="Current token holders"
                    base_url=blockscout_url.to_string()
                />
                <ApiEndpointRow
                    method="GET"
                    path=format!("/api/v2/tokens/{}/counters", usdfc_token)
                    description="Token statistics"
                    base_url=blockscout_url.to_string()
                />
                <ApiEndpointRow
                    method="GET"
                    path=format!("/api/v2/tokens/{}", usdfc_token)
                    description="Token metadata"
                    base_url=blockscout_url.to_string()
                />
            </div>
        </div>

        // Subgraph
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 8px;">"Subgraph (GraphQL)"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Secured Finance lending market data"</p>
            <div class="code-block">
                <code>{subgraph_url.clone()}</code>
            </div>
            <p style="color: var(--text-muted); font-size: 12px; margin: 12px 0;">
                "Use a GraphQL client or the playground to query lending markets, transactions, and daily volumes."
            </p>
            <a
                href=subgraph_url.clone()
                target="_blank"
                class="btn btn-secondary"
            >
                <ExternalLinkIcon />
                "Open GraphQL Playground"
            </a>
        </div>

        // Advanced Tools
        <div class="card">
            <h3 class="card-title" style="margin-bottom: 16px;">"Advanced Tools"</h3>
            <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                <a
                    href=format!("{}/advanced-filters?token_contract_address_hashes_to_include={}", blockscout_url, usdfc_token)
                    target="_blank"
                    class="btn btn-secondary"
                >
                    "Advanced Filters"
                </a>
                <a
                    href=format!("{}/api-docs", blockscout_url)
                    target="_blank"
                    class="btn btn-secondary"
                >
                    "API Documentation"
                </a>
                <a
                    href=format!("{}/token/{}", blockscout_url, usdfc_token)
                    target="_blank"
                    class="btn btn-secondary"
                >
                    "View on Blockscout"
                </a>
            </div>
        </div>
    }
}

#[component]
fn ApiEndpointRow(
    method: &'static str,
    path: String,
    description: &'static str,
    base_url: String,
) -> impl IntoView {
    let full_url = format!("{}{}", base_url, path);

    view! {
        <div class="api-endpoint-row">
            <div class="api-endpoint-info">
                <span class="method-badge">{method}</span>
                <code class="api-path">{path}</code>
                <span class="api-desc">{description}</span>
            </div>
            <a
                href=full_url
                target="_blank"
                class="btn btn-secondary btn-sm"
            >
                "Open"
            </a>
        </div>
    }
}

// ============================================================================
// API Tab
// ============================================================================

#[component]
fn ApiTab() -> impl IntoView {
    view! {
        <div class="card" style="margin-bottom: 24px;">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Data Sources"</h3>
            <div class="table-container">
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Source"</th>
                            <th>"Endpoint"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Filecoin RPC"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"api.node.glif.io"</td>
                            <td style="color: var(--text-muted);">"On-chain contract data"</td>
                        </tr>
                        <tr>
                            <td>"Blockscout"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"filecoin.blockscout.com/api"</td>
                            <td style="color: var(--text-muted);">"Token transfers and transactions"</td>
                        </tr>
                        <tr>
                            <td>"GeckoTerminal"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"api.geckoterminal.com"</td>
                            <td style="color: var(--text-muted);">"DEX prices and pools"</td>
                        </tr>
                        <tr>
                            <td>"Secured Finance"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"api.goldsky.com"</td>
                            <td style="color: var(--text-muted);">"Lending market subgraph"</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <div class="card">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Contract Addresses (Mainnet)"</h3>
            <pre style="background: var(--bg-primary); padding: 16px; border-radius: 8px; overflow-x: auto; font-size: 12px; color: var(--text-secondary);">
{r#"USDFC Token:         0x80B98d3aa09ffff255c3ba4A241111Ff1262F045
Trove Manager:       0x5aB87c2398454125Dd424425e39c8909bBE16022
Stability Pool:      0x791Ad78bBc58324089D3E0A8689E7D045B9592b5
Price Feed:          0x80e651c9739C1ed15A267c11b85361780164A368
Active Pool:         0x8637Ac7FdBB4c763B72e26504aFb659df71c7803
Sorted Troves:       0x2C32e48e358d5b893C46906b69044D342d8DDd5F
Borrower Operations: 0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0
Multi Trove Getter:  0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F
USDFC/WFIL Pool:     0x4e07447bd38e60b94176764133788be1a0736b30"#}
            </pre>
        </div>
    }
}

// ============================================================================
// Gas Estimator Tab
// ============================================================================

#[component]
fn GasEstimatorTab() -> impl IntoView {
    let selected_operation = create_rw_signal("transfer".to_string());

    let gas_estimate = create_memo(move |_| {
        estimate_gas(&selected_operation.get())
    });

    // Approximate gas price in nanoFIL (this is a placeholder - in v2 could be fetched dynamically)
    let gas_price_nanofil: f64 = 100.0;

    let estimated_cost = create_memo(move |_| {
        let gas = gas_estimate.get() as f64;
        let cost_nanofil = gas * gas_price_nanofil;
        let cost_fil = cost_nanofil / 1_000_000_000.0;
        cost_fil
    });

    const OPERATIONS: &[(&str, &str, &str)] = &[
        ("transfer", "Transfer USDFC", "Standard ERC20 transfer"),
        ("approve", "Approve Spending", "Token approval for contract"),
        ("open_trove", "Open Trove", "Create new collateralized position"),
        ("adjust_trove", "Adjust Trove", "Modify existing trove"),
        ("close_trove", "Close Trove", "Repay debt and close position"),
        ("provide_sp", "Deposit to Stability Pool", "Provide USDFC to earn rewards"),
        ("withdraw_sp", "Withdraw from Stability Pool", "Remove USDFC from pool"),
        ("swap", "Swap (DEX)", "Exchange tokens on DEX"),
    ];

    view! {
        <div class="card" style="margin-bottom: 24px;">
            <h3 class="card-title" style="margin-bottom: 8px;">"Gas Estimator"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"Estimate transaction costs for USDFC operations"</p>

            <div style="display: flex; flex-direction: column; gap: 16px;">
                <div>
                    <label style="display: block; font-size: 12px; color: var(--text-muted); margin-bottom: 8px;">"Select Operation"</label>
                    <select
                        class="input"
                        style="width: 100%; max-width: 400px;"
                        on:change=move |ev| selected_operation.set(event_target_value(&ev))
                    >
                        {OPERATIONS.iter().map(|(id, label, _)| {
                            view! {
                                <option value=*id attr:selected=move || selected_operation.get() == *id>{*label}</option>
                            }
                        }).collect_view()}
                    </select>
                </div>

                <div class="grid-3">
                    <div class="stat-card">
                        <div class="metric-label">"Estimated Gas"</div>
                        <div class="metric-value cyan">{move || format_with_commas(gas_estimate.get())}</div>
                        <div class="metric-sub">"gas units"</div>
                    </div>
                    <div class="stat-card">
                        <div class="metric-label">"Est. Cost"</div>
                        <div class="metric-value green">{move || format!("{:.2} FIL", estimated_cost.get())}</div>
                        <div class="metric-sub">"at 100 nanoFIL/gas"</div>
                    </div>
                    <div class="stat-card">
                        <div class="metric-label">"Gas Price"</div>
                        <div class="metric-value">"~100"</div>
                        <div class="metric-sub">"nanoFIL/gas (estimate)"</div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Gas Reference Table"</h3>
            <div class="table-container">
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Operation"</th>
                            <th>"Est. Gas"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {OPERATIONS.iter().map(|(id, label, desc)| {
                            let gas = estimate_gas(id);
                            view! {
                                <tr>
                                    <td style="font-weight: 600; color: var(--text-primary);">{*label}</td>
                                    <td style="font-family: monospace; color: var(--accent-cyan);">{format_with_commas(gas)}</td>
                                    <td style="color: var(--text-muted);">{*desc}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
            <p style="color: var(--text-muted); font-size: 11px; margin-top: 12px;">
                "* Gas estimates are approximate and may vary based on network conditions and contract state."
            </p>
        </div>
    }
}

// ============================================================================
// Alerts Tab
// ============================================================================

#[component]
fn AlertsTab() -> impl IntoView {
    let global = use_global_metrics();

    let transactions = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(25)).await }
    );

    view! {
        <div class="card">
            <div class="card-header">
                <div>
                    <h3 class="card-title">"Live Alert Feed"</h3>
                    <p class="card-subtitle">"Auto-generated from protocol metrics and recent transfers"</p>
                </div>
                <button
                    class="btn btn-secondary"
                    on:click=move |_| {
                        global.refresh_all();
                        transactions.refetch();
                    }
                >
                    "Refresh"
                </button>
            </div>
            <div style="display: flex; flex-direction: column; gap: 12px;">
                <Suspense fallback=move || view! { <div style="text-align: center; padding: 20px;">"Loading alerts..."</div> }>
                    {move || {
                        let mut alerts: Vec<View> = Vec::new();

                        // Check TCR from global metrics
                        if let Some(Ok(m)) = global.protocol.get() {
                            let tcr_f64 = decimal_to_f64(m.tcr);
                            if tcr_f64 < 125.0 {
                                alerts.push(view! {
                                    <div class="alert-card danger">
                                        <div class="alert-icon">"!"</div>
                                        <div class="alert-content">
                                            <div class="alert-title">"Critical TCR"</div>
                                            <div class="alert-desc">"System TCR is below 125%. Liquidation risk is elevated."</div>
                                        </div>
                                        <div class="alert-time">{format!("{:.1}%", tcr_f64)}</div>
                                    </div>
                                }.into_view());
                            } else if tcr_f64 < 150.0 {
                                alerts.push(view! {
                                    <div class="alert-card warning">
                                        <div class="alert-icon">"!"</div>
                                        <div class="alert-content">
                                            <div class="alert-title">"TCR Warning"</div>
                                            <div class="alert-desc">"System TCR is below 150%. Monitor collateral health."</div>
                                        </div>
                                        <div class="alert-time">{format!("{:.1}%", tcr_f64)}</div>
                                    </div>
                                }.into_view());
                            }
                        }

                        transactions.get().map(|res| {
                            match res {
                                Ok(txs) => {
                                    // Large transfers
                                    let large_tx = txs
                                        .iter()
                                        .filter(|tx| decimal_to_f64(tx.amount) >= 1_000_000.0)
                                        .take(3)
                                        .collect::<Vec<_>>();
                                    for tx in large_tx {
                                        alerts.push(view! {
                                            <div class="alert-card info">
                                                <div class="alert-icon">"$"</div>
                                                <div class="alert-content">
                                                    <div class="alert-title">"Large Transfer"</div>
                                                    <div class="alert-desc">{format!("{} USDFC transfer detected.", format_amount(tx.amount))}</div>
                                                </div>
                                                <div class="alert-time">{shorten_hash(&tx.hash)}</div>
                                            </div>
                                        }.into_view());
                                    }

                                    if alerts.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-title">"No active alerts"</div>
                                                <div class="empty-state-desc">"Protocol metrics and transfers are within normal ranges."</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        alerts.into_iter().collect_view()
                                    }
                                }
                                Err(err) => view! {
                                    <div class="empty-state">
                                        <div class="empty-state-title">"Alert Error"</div>
                                        <div class="empty-state-desc">{err.to_string()}</div>
                                    </div>
                                }.into_view()
                            }
                        }).unwrap_or_else(|| view! { <div></div> }.into_view())
                    }}
                </Suspense>
            </div>
        </div>

        // Alert Configuration Info
        <div class="card" style="margin-top: 24px;">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Alert Thresholds"</h3>
            <div class="grid-3">
                <div>
                    <div class="metric-label">"Critical TCR"</div>
                    <div class="metric-value red">"< 125%"</div>
                </div>
                <div>
                    <div class="metric-label">"Warning TCR"</div>
                    <div class="metric-value yellow">"< 150%"</div>
                </div>
                <div>
                    <div class="metric-label">"Large Transfer"</div>
                    <div class="metric-value cyan">"> 1M USDFC"</div>
                </div>
            </div>
        </div>
    }
}
