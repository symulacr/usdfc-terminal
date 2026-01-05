use leptos::*;
use crate::components::Pagination;
use usdfc_backend::server_fn::{get_top_holders, get_usdfc_price_data};
use usdfc_core::format::{format_amount, shorten_hash, format_usd_compact};
use usdfc_core::config::config;

const HOLDERS_PER_PAGE: usize = 25;

#[component]
pub fn EntityRegistry() -> impl IntoView {
    // Pagination state for holders
    let (holders_page, set_holders_page) = create_signal(1u32);
    let (holders_total_pages, set_holders_total_pages) = create_signal(1u32);

    // Fetch more holders for pagination (50 instead of 20)
    let holders = create_resource(
        || (),
        |_| async move { get_top_holders(Some(50), None).await }
    );

    let pool_data = create_resource(
        || (),
        |_| async move { get_usdfc_price_data().await }
    );

    // Page change handler for holders
    let on_holders_page_change = Callback::new(move |page: u32| {
        set_holders_page.set(page);
    });

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Entity Registry"</h1>
                <p class="page-subtitle">"Known entities and contracts in the USDFC ecosystem"</p>
            </div>

            // Protocol Contracts Section
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Protocol Contracts"</h3>
                        <p class="card-subtitle">"Core USDFC protocol smart contracts on Filecoin"</p>
                    </div>
                </div>
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
                            <ProtocolContracts />
                        </tbody>
                    </table>
                </div>
            </div>

            // DEX Pools Section
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"DEX Liquidity Pools"</h3>
                        <p class="card-subtitle">"USDFC trading pools with live metrics from GeckoTerminal"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| pool_data.refetch()
                    >
                        "Refresh"
                    </button>
                </div>
                <Suspense fallback=move || view! {
                    <div style="padding: 20px; text-align: center; color: var(--text-muted);">"Loading pool data..."</div>
                }>
                    {move || {
                        pool_data.get().map(|res| {
                            match res {
                                Ok(data) => {
                                    let cfg = config();
                                    view! {
                                        <div class="table-container">
                                            <table class="table">
                                                <thead>
                                                    <tr>
                                                        <th>"Pool"</th>
                                                        <th>"DEX"</th>
                                                        <th>"Price"</th>
                                                        <th>"24h Change"</th>
                                                        <th>"24h Volume"</th>
                                                        <th>"Liquidity"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <tr>
                                                        <td>
                                                            <a
                                                                href=format!("https://filecoin.blockscout.com/address/{}", cfg.pool_usdfc_wfil)
                                                                target="_blank"
                                                                style="color: var(--accent-cyan); text-decoration: none; font-weight: 500;"
                                                            >
                                                                "USDFC/WFIL"
                                                            </a>
                                                        </td>
                                                        <td style="color: var(--text-muted);">"Swap"</td>
                                                        <td style="font-family: monospace;">{data.price_usd.map(|v| format!("${:.4}", v)).unwrap_or_else(|| "Error".to_string())}</td>
                                                        <td style={if data.price_change_24h.unwrap_or(0.0) >= 0.0 { "color: var(--accent-green);" } else { "color: var(--accent-red);" }}>
                                                            {data.price_change_24h.map(|v| format!("{:+.2}%", v)).unwrap_or_else(|| "--".to_string())}
                                                        </td>
                                                        <td style="font-family: monospace;">{data.volume_24h.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</td>
                                                        <td style="font-family: monospace; color: var(--accent-purple);">{data.liquidity_usd.map(format_usd_compact).unwrap_or_else(|| "--".to_string())}</td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                }
                                Err(err) => view! {
                                    <div style="padding: 20px; text-align: center; color: var(--accent-red);">{err.to_string()}</div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Top Holders Section
            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Top USDFC Holders"</h3>
                        <p class="card-subtitle">"Live balances from Blockscout"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| {
                            set_holders_page.set(1);
                            holders.refetch();
                        }
                    >
                        "Refresh"
                    </button>
                </div>
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Rank"</th>
                                <th>"Address"</th>
                                <th>"Balance"</th>
                                <th>"Entity"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="4" style="text-align: center; padding: 20px;">"Loading holders..."</td></tr>
                            }>
                                {move || {
                                    let page = holders_page.get() as usize;

                                    holders.get().map(|res| {
                                        match res {
                                            Ok(items) => {
                                                if items.is_empty() {
                                                    view! {
                                                        <tr><td colspan="4" style="text-align: center; padding: 20px; color: var(--text-muted);">"No holders found"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    // Calculate pagination
                                                    let total_holders = items.len();
                                                    let total_pgs = ((total_holders + HOLDERS_PER_PAGE - 1) / HOLDERS_PER_PAGE).max(1) as u32;
                                                    set_holders_total_pages.set(total_pgs);

                                                    // Get current page items
                                                    let start_idx = (page - 1) * HOLDERS_PER_PAGE;
                                                    let page_items: Vec<_> = items.iter().skip(start_idx).take(HOLDERS_PER_PAGE).collect();

                                                    page_items.iter().enumerate().map(|(i, item)| {
                                                        let rank = start_idx + i + 1;
                                                        let entity = identify_entity(&item.address);
                                                        let badge_class = entity_badge_class(&entity);
                                                        view! {
                                                            <tr>
                                                                <td style="color: var(--text-muted);">{format!("#{}", rank)}</td>
                                                                <td style="font-family: monospace; font-size: 12px;">
                                                                    <div style="display: flex; align-items: center; gap: 8px;">
                                                                        <a
                                                                            href=format!("/address/{}", item.address)
                                                                            style="color: var(--accent-cyan); text-decoration: none;"
                                                                            title="View address details"
                                                                        >
                                                                            {shorten_hash(&item.address)}
                                                                        </a>
                                                                        <a
                                                                            href=format!("https://filecoin.blockscout.com/address/{}", item.address)
                                                                            target="_blank"
                                                                            style="color: var(--text-muted); text-decoration: none; font-size: 10px;"
                                                                            title="View on Blockscout"
                                                                        >
                                                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                                <polyline points="15 3 21 3 21 9"></polyline>
                                                                                <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                            </svg>
                                                                        </a>
                                                                        <CopyButton address=item.address.clone() />
                                                                    </div>
                                                                </td>
                                                                <td style="font-family: monospace;">{format_amount(item.balance)}" USDFC"</td>
                                                                <td><span class=badge_class>{entity}</span></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="4" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>

                // Pagination controls for holders
                <Pagination
                    current_page=holders_page
                    total_pages=holders_total_pages
                    on_page_change=on_holders_page_change
                />
            </div>
        </div>
    }
}

/// Copy-to-clipboard button with feedback
#[component]
fn CopyButton(address: String) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);
    let address_clone = address.clone();

    let on_click = move |_| {
        let addr = address_clone.clone();
        #[cfg(feature = "hydrate")]
        {
            // Use JavaScript to copy to clipboard
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = navigator, js_name = clipboard)]
                static CLIPBOARD: JsValue;

                #[wasm_bindgen(js_namespace = ["navigator", "clipboard"], js_name = writeText)]
                fn write_text(s: &str) -> js_sys::Promise;
            }

            let _ = write_text(&addr);
            set_copied.set(true);

            // Reset after 1.5 seconds
            use gloo_timers::callback::Timeout;
            let timeout = Timeout::new(1500, move || {
                set_copied.set(false);
            });
            timeout.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = addr;
        }
    };

    view! {
        <button
            on:click=on_click
            style="background: none; border: none; cursor: pointer; padding: 2px 4px; color: var(--text-muted); display: flex; align-items: center;"
            title=move || if copied.get() { "Copied!" } else { "Copy address" }
        >
            {move || if copied.get() {
                view! {
                    <span style="color: var(--accent-green); font-size: 10px; font-weight: 500;">"Copied!"</span>
                }.into_view()
            } else {
                view! {
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                }.into_view()
            }}
        </button>
    }
}

#[component]
fn ProtocolContracts() -> impl IntoView {
    let cfg = config();
    let contracts = vec![
        ("USDFC Token", &cfg.usdfc_token, "ERC-20 stablecoin token"),
        ("Trove Manager", &cfg.trove_manager, "Manages individual troves"),
        ("Stability Pool", &cfg.stability_pool, "Liquidation backstop pool"),
        ("Active Pool", &cfg.active_pool, "Holds active collateral"),
        ("Borrower Operations", &cfg.borrower_operations, "User borrowing interface"),
        ("Price Feed", &cfg.price_feed, "FIL/USD price oracle"),
        ("Sorted Troves", &cfg.sorted_troves, "Troves sorted by ICR"),
    ];

    contracts.into_iter().map(|(name, addr, desc)| {
        let addr_owned = addr.to_string();
        view! {
            <tr>
                <td style="font-weight: 500;">{name}</td>
                <td style="font-family: monospace; font-size: 11px;">
                    <div style="display: flex; align-items: center; gap: 8px;">
                        <a
                            href=format!("/address/{}", addr)
                            style="color: var(--accent-cyan); text-decoration: none;"
                            title="View address details"
                        >
                            {shorten_hash(addr)}
                        </a>
                        <a
                            href=format!("https://filecoin.blockscout.com/address/{}", addr)
                            target="_blank"
                            style="color: var(--text-muted); text-decoration: none; font-size: 10px;"
                            title="View on Blockscout"
                        >
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                <polyline points="15 3 21 3 21 9"></polyline>
                                <line x1="10" y1="14" x2="21" y2="3"></line>
                            </svg>
                        </a>
                        <CopyButton address=addr_owned />
                    </div>
                </td>
                <td style="color: var(--text-muted); font-size: 12px;">{desc}</td>
            </tr>
        }
    }).collect_view()
}

fn identify_entity(address: &str) -> &'static str {
    let addr = address.to_lowercase();
    let cfg = config();

    // Check known protocol contracts
    if addr == cfg.stability_pool.to_lowercase() { return "Stability Pool"; }
    if addr == cfg.active_pool.to_lowercase() { return "Active Pool"; }
    if addr == cfg.trove_manager.to_lowercase() { return "Trove Manager"; }
    if addr == cfg.borrower_operations.to_lowercase() { return "Borrower Ops"; }

    // Check known DEX pools
    if addr == cfg.pool_usdfc_wfil.to_lowercase() { return "USDFC/WFIL Pool"; }

    // Default
    "Unknown"
}

fn entity_badge_class(entity: &str) -> &'static str {
    match entity {
        "Stability Pool" | "Active Pool" | "Trove Manager" | "Borrower Ops" => "entity-badge protocol",
        "USDFC/WFIL Pool" => "entity-badge dex",
        _ => "entity-badge",
    }
}

