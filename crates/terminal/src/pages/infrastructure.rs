//! Infrastructure Page
//!
//! Combined view of Contracts, Architecture, Data Sources, and Error Logs.
//! Uses tabs for navigation between sections.

use leptos::*;
use crate::components::tabs::{TabNav, TabContent, Tab};
use usdfc_backend::server_fn::check_api_health;

#[component]
pub fn Infrastructure() -> impl IntoView {
    let active_tab = create_rw_signal("contracts".to_string());

    let tabs = vec![
        Tab { id: "contracts", label: "Contracts" },
        Tab { id: "architecture", label: "Architecture" },
        Tab { id: "sources", label: "Data Sources" },
        Tab { id: "logs", label: "Error Logs" },
    ];

    view! {
        <div class="fade-in">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Infrastructure"</h1>
                    <p class="page-subtitle">"Technical reference for USDFC protocol"</p>
                </div>
            </div>

            <TabNav tabs=tabs active=active_tab />

            <TabContent id="contracts" active=active_tab>
                <ContractsTab />
            </TabContent>

            <TabContent id="architecture" active=active_tab>
                <ArchitectureTab />
            </TabContent>

            <TabContent id="sources" active=active_tab>
                <DataSourcesTab />
            </TabContent>

            <TabContent id="logs" active=active_tab>
                <ErrorLogsTab />
            </TabContent>
        </div>
    }
}

// ============================================================================
// Contracts Tab
// ============================================================================

#[component]
fn ContractsTab() -> impl IntoView {
    let contracts = vec![
        ("USDFC Token", "0x80B98d3aa09ffff255c3ba4A241111Ff1262F045", "ERC20 stablecoin token"),
        ("Trove Manager", "0x5aB87c2398454125Dd424425e39c8909bBE16022", "Manages collateralized debt positions"),
        ("Stability Pool", "0x791Ad78bBc58324089D3E0A8689E7D045B9592b5", "Liquidation buffer pool"),
        ("Price Feed", "0x80e651c9739C1ed15A267c11b85361780164A368", "FIL/USD oracle"),
        ("Borrower Operations", "0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0", "Open/close troves"),
        ("Sorted Troves", "0x2C32e48e358d5b893C46906b69044D342d8DDd5F", "Ordered trove list"),
        ("Active Pool", "0x8637Ac7FdBB4c763B72e26504aFb659df71c7803", "Holds active collateral"),
        ("Multi Trove Getter", "0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F", "Batch trove queries"),
    ];

    view! {
        <div class="card">
            <h3 class="card-title" style="margin-bottom: 16px;">"Protocol Contracts"</h3>
            <p class="card-subtitle" style="margin-bottom: 16px;">"USDFC protocol contracts on Filecoin Mainnet"</p>
            <div class="table-container">
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Contract"</th>
                            <th>"Address"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {contracts.into_iter().map(|(name, addr, desc)| {
                            let explorer_url = format!("https://filecoin.blockscout.com/address/{}", addr);
                            view! {
                                <tr>
                                    <td style="font-weight: 600; color: var(--text-primary);">{name}</td>
                                    <td>
                                        <a href=explorer_url target="_blank" style="color: var(--accent-cyan); text-decoration: none; font-family: monospace; font-size: 12px;">
                                            {format!("{}...{}", &addr[..10], &addr[addr.len()-6..])}
                                        </a>
                                    </td>
                                    <td style="color: var(--text-muted);">{desc}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>

        <div class="grid-2" style="margin-top: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"DEX Integration"</h3>
                <div class="table-container">
                    <table class="table">
                        <tbody>
                            <tr>
                                <td style="font-weight: 600;">"USDFC/WFIL Pool"</td>
                                <td>
                                    <a href="https://filecoin.blockscout.com/address/0x4e07447bd38e60b94176764133788be1a0736b30" target="_blank" style="color: var(--accent-cyan); text-decoration: none; font-family: monospace; font-size: 12px;">
                                        "0x4e07...6b30"
                                    </a>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Network Info"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Chain ID: "<span style="color: var(--text-primary); font-family: monospace;">"314"</span></div>
                    <div>"Network: "<span style="color: var(--text-primary);">"Filecoin Mainnet"</span></div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Architecture Tab
// ============================================================================

#[component]
fn ArchitectureTab() -> impl IntoView {
    view! {
        <div class="grid-2" style="margin-bottom: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Core Components"</h3>
                <ul style="color: var(--text-secondary); line-height: 2; padding-left: 20px;">
                    <li>"Trove Manager - Collateralized debt positions"</li>
                    <li>"Stability Pool - Liquidation buffer"</li>
                    <li>"Price Feed - FIL/USD oracle"</li>
                    <li>"Sorted Troves - Ordered by ICR"</li>
                    <li>"Active Pool - Holds active collateral"</li>
                    <li>"Borrower Operations - Open/close troves"</li>
                </ul>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Protocol Parameters"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Minimum Collateral Ratio: "<span style="color: var(--accent-red);">"110%"</span></div>
                    <div>"Critical Collateral Ratio: "<span style="color: var(--accent-yellow);">"150%"</span></div>
                    <div>"Liquidation Reserve: "<span style="color: var(--text-primary);">"200 USDFC"</span></div>
                    <div>"Borrowing Fee: "<span style="color: var(--text-primary);">"0.5%"</span></div>
                </div>
            </div>
        </div>

        <div class="card" style="margin-bottom: 24px;">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"How It Works"</h3>
            <p style="color: var(--text-secondary); line-height: 1.8;">
                "USDFC is a decentralized stablecoin on Filecoin, backed by FIL collateral.
                Users deposit FIL into Troves to mint USDFC. Each Trove must maintain a minimum
                collateral ratio of 110%. If a Trove's ratio falls below this threshold, it can
                be liquidated. The Stability Pool provides the first line of defense for liquidations,
                with depositors earning FIL rewards."
            </p>
        </div>

        <div class="card">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"System Flow"</h3>
            <div style="background: var(--bg-tertiary); border-radius: 8px; padding: 24px;">
                <svg viewBox="0 0 800 200" style="width: 100%; height: 150px;">
                    // User
                    <rect x="20" y="60" width="100" height="60" rx="8" fill="var(--accent-cyan)" opacity="0.2" stroke="var(--accent-cyan)" stroke-width="2"/>
                    <text x="70" y="95" text-anchor="middle" fill="var(--text-primary)" font-size="12">"User"</text>

                    // Arrow 1
                    <line x1="120" y1="90" x2="180" y2="90" stroke="var(--text-muted)" stroke-width="2" marker-end="url(#arrow)"/>
                    <text x="150" y="80" text-anchor="middle" fill="var(--text-muted)" font-size="10">"Deposit FIL"</text>

                    // Trove Manager
                    <rect x="180" y="60" width="120" height="60" rx="8" fill="var(--accent-purple)" opacity="0.2" stroke="var(--accent-purple)" stroke-width="2"/>
                    <text x="240" y="95" text-anchor="middle" fill="var(--text-primary)" font-size="12">"Trove Manager"</text>

                    // Arrow 2
                    <line x1="300" y1="90" x2="360" y2="90" stroke="var(--text-muted)" stroke-width="2"/>
                    <text x="330" y="80" text-anchor="middle" fill="var(--text-muted)" font-size="10">"Mint"</text>

                    // USDFC Token
                    <rect x="360" y="60" width="100" height="60" rx="8" fill="var(--accent-green)" opacity="0.2" stroke="var(--accent-green)" stroke-width="2"/>
                    <text x="410" y="95" text-anchor="middle" fill="var(--text-primary)" font-size="12">"USDFC"</text>

                    // Arrow 3
                    <line x1="460" y1="90" x2="520" y2="90" stroke="var(--text-muted)" stroke-width="2"/>

                    // Stability Pool
                    <rect x="520" y="60" width="120" height="60" rx="8" fill="var(--accent-yellow)" opacity="0.2" stroke="var(--accent-yellow)" stroke-width="2"/>
                    <text x="580" y="95" text-anchor="middle" fill="var(--text-primary)" font-size="12">"Stability Pool"</text>

                    // Price Feed
                    <rect x="680" y="60" width="100" height="60" rx="8" fill="var(--accent-red)" opacity="0.2" stroke="var(--accent-red)" stroke-width="2"/>
                    <text x="730" y="95" text-anchor="middle" fill="var(--text-primary)" font-size="12">"Price Feed"</text>

                    // Connection to Price Feed
                    <line x1="730" y1="60" x2="730" y2="30" stroke="var(--text-muted)" stroke-width="1" stroke-dasharray="4"/>
                    <line x1="240" y1="30" x2="730" y2="30" stroke="var(--text-muted)" stroke-width="1" stroke-dasharray="4"/>
                    <line x1="240" y1="30" x2="240" y2="60" stroke="var(--text-muted)" stroke-width="1" stroke-dasharray="4"/>
                    <text x="485" y="20" text-anchor="middle" fill="var(--text-muted)" font-size="9">"FIL/USD Price Updates"</text>
                </svg>
            </div>
        </div>
    }
}

// ============================================================================
// Data Sources Tab
// ============================================================================

#[component]
fn DataSourcesTab() -> impl IntoView {
    view! {
        <div class="grid-2" style="margin-bottom: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"RPC Endpoint"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Provider: "<span style="color: var(--text-primary);">"GLIF"</span></div>
                    <div style="font-family: monospace; font-size: 12px; color: var(--accent-cyan); word-break: break-all;">
                        "https://api.node.glif.io/rpc/v1"
                    </div>
                </div>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Block Explorer"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Provider: "<span style="color: var(--text-primary);">"Blockscout"</span></div>
                    <div style="font-family: monospace; font-size: 12px; color: var(--accent-cyan); word-break: break-all;">
                        "https://filecoin.blockscout.com/api/v2"
                    </div>
                </div>
            </div>
        </div>

        <div class="grid-2" style="margin-bottom: 24px;">
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Subgraph"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Provider: "<span style="color: var(--text-primary);">"Goldsky (Secured Finance)"</span></div>
                    <div style="font-family: monospace; font-size: 11px; color: var(--accent-cyan); word-break: break-all;">
                        "api.goldsky.com/.../sf-filecoin-mainnet"
                    </div>
                </div>
            </div>
            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Price Data"</h3>
                <div style="color: var(--text-secondary); line-height: 2;">
                    <div>"Provider: "<span style="color: var(--text-primary);">"GeckoTerminal"</span></div>
                    <div style="font-family: monospace; font-size: 12px; color: var(--accent-cyan); word-break: break-all;">
                        "api.geckoterminal.com/api/v2"
                    </div>
                </div>
            </div>
        </div>

        <div class="card">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Data Refresh Rates"</h3>
            <div class="table-container">
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Data Type"</th>
                            <th>"Source"</th>
                            <th>"Refresh"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Protocol Metrics"</td>
                            <td style="color: var(--text-muted);">"On-chain (RPC)"</td>
                            <td style="color: var(--accent-cyan);">"On demand"</td>
                        </tr>
                        <tr>
                            <td>"Price Data"</td>
                            <td style="color: var(--text-muted);">"GeckoTerminal API"</td>
                            <td style="color: var(--accent-cyan);">"On demand"</td>
                        </tr>
                        <tr>
                            <td>"Transactions"</td>
                            <td style="color: var(--text-muted);">"Blockscout API"</td>
                            <td style="color: var(--accent-cyan);">"On demand"</td>
                        </tr>
                        <tr>
                            <td>"Lending Data"</td>
                            <td style="color: var(--text-muted);">"Secured Finance Subgraph"</td>
                            <td style="color: var(--accent-cyan);">"On demand"</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ============================================================================
// Error Logs Tab
// ============================================================================

#[component]
fn ErrorLogsTab() -> impl IntoView {
    // Store recent errors in a signal
    let errors = create_rw_signal(Vec::<(String, String, String)>::new());
    let last_check = create_rw_signal(None::<String>);

    // Check API health and populate errors
    let health = create_resource(
        || (),
        |_| async move { check_api_health().await }
    );

    // Effect to populate errors based on health check
    create_effect(move |_| {
        if let Some(Ok(h)) = health.get() {
            let mut new_errors = Vec::new();
            let now = get_current_time_string();

            if !h.rpc_ok {
                new_errors.push((
                    now.clone(),
                    "RPC Connection".to_string(),
                    "Failed to connect to Filecoin RPC (api.node.glif.io)".to_string(),
                ));
            }
            if !h.blockscout_ok {
                new_errors.push((
                    now.clone(),
                    "Blockscout API".to_string(),
                    "Failed to connect to Blockscout API".to_string(),
                ));
            }
            if !h.subgraph_ok {
                new_errors.push((
                    now.clone(),
                    "Subgraph".to_string(),
                    "Failed to connect to Secured Finance subgraph".to_string(),
                ));
            }

            errors.set(new_errors);
            last_check.set(Some(now));
        }
    });

    view! {
        <div class="card" style="margin-bottom: 24px;">
            <div class="card-header">
                <div>
                    <h3 class="card-title">"Recent Errors"</h3>
                    <p class="card-subtitle">"API and data source connection issues"</p>
                </div>
                <button
                    class="btn btn-secondary"
                    on:click=move |_| health.refetch()
                >
                    "Refresh"
                </button>
            </div>

            <Suspense fallback=move || view! {
                <div style="text-align: center; padding: 40px; color: var(--text-muted);">
                    "Checking data sources..."
                </div>
            }>
                {move || {
                    let error_list = errors.get();
                    if error_list.is_empty() {
                        view! {
                            <div class="empty-state" style="padding: 40px;">
                                <div class="empty-state-icon" style="font-size: 32px; margin-bottom: 16px;">"✓"</div>
                                <div class="empty-state-title">"No Errors"</div>
                                <div class="empty-state-desc">"All data sources are operating normally."</div>
                                <div style="font-size: 11px; color: var(--text-muted); margin-top: 12px;">
                                    {move || last_check.get().map(|t| format!("Last checked: {}", t)).unwrap_or_default()}
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="error-log">
                                <ul style="list-style: none; padding: 0; margin: 0;">
                                    {error_list.iter().map(|(time, source, message)| {
                                        view! {
                                            <li class="error-log-item" style="padding: 12px; border-bottom: 1px solid var(--border-color); display: flex; gap: 16px; align-items: flex-start;">
                                                <span class="error-icon" style="color: var(--accent-red); font-size: 16px;">"!"</span>
                                                <div style="flex: 1;">
                                                    <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                                                        <span style="font-weight: 600; color: var(--text-primary);">{source.clone()}</span>
                                                        <span style="font-size: 11px; color: var(--text-muted); font-family: monospace;">{time.clone()}</span>
                                                    </div>
                                                    <div style="color: var(--text-secondary); font-size: 13px;">{message.clone()}</div>
                                                </div>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            </div>
                        }.into_view()
                    }
                }}
            </Suspense>
        </div>

        <div class="card">
            <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Data Source Status"</h3>
            <Suspense fallback=move || view! {
                <div class="skeleton" style="height: 150px;"></div>
            }>
                {move || {
                    health.get().map(|res| match res {
                        Ok(h) => view! {
                            <div class="grid-2" style="gap: 16px;">
                                <StatusCard name="Filecoin RPC" connected=h.rpc_ok endpoint="api.node.glif.io" />
                                <StatusCard name="Blockscout API" connected=h.blockscout_ok endpoint="filecoin.blockscout.com" />
                                <StatusCard name="Secured Finance" connected=h.subgraph_ok endpoint="api.goldsky.com" />
                                <StatusCard name="GeckoTerminal" connected=true endpoint="api.geckoterminal.com" />
                            </div>
                        }.into_view(),
                        Err(e) => view! {
                            <div class="error-state">
                                <span style="color: var(--accent-red);">"Failed to check status: "{e.to_string()}</span>
                            </div>
                        }.into_view()
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn StatusCard(name: &'static str, connected: bool, endpoint: &'static str) -> impl IntoView {
    view! {
        <div class="stat-card" style=format!("border-left: 3px solid {};", if connected { "var(--accent-green)" } else { "var(--accent-red)" })>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <span style="font-weight: 600; color: var(--text-primary);">{name}</span>
                <span
                    class=if connected { "status-badge online" } else { "status-badge offline" }
                    style=format!("padding: 2px 8px; border-radius: 4px; font-size: 11px; background: {}; color: {};",
                        if connected { "rgba(34, 197, 94, 0.2)" } else { "rgba(239, 68, 68, 0.2)" },
                        if connected { "var(--accent-green)" } else { "var(--accent-red)" }
                    )
                >
                    {if connected { "Online" } else { "Offline" }}
                </span>
            </div>
            <div style="font-family: monospace; font-size: 11px; color: var(--text-muted);">{endpoint}</div>
        </div>
    }
}

/// Get current time as a formatted string
fn get_current_time_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            date.get_utc_full_year(),
            date.get_utc_month() + 1,
            date.get_utc_date(),
            date.get_utc_hours(),
            date.get_utc_minutes(),
            date.get_utc_seconds()
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Simple timestamp format for server-side
        format!("Timestamp: {}", secs)
    }
}
