//! Address Detail Page
//!
//! Displays comprehensive information about a specific address including:
//! - USDFC and FIL balances
//! - Activity statistics
//! - Transaction history with filtering and pagination

use leptos::*;
use leptos_router::*;
use crate::components::icons::*;
use crate::components::PaginationWithSize;
use crate::server_fn::{get_address_info, get_recent_transactions};
use crate::format::{
    format_timestamp, format_balance, shorten_hash, format_count,
    format_amount, decimal_to_f64, format_date,
};
use crate::types::Transaction;

const DEFAULT_PAGE_SIZE: u32 = 25;

/// Main Address Detail component - accessed via /address/:addr route
#[component]
pub fn AddressDetail() -> impl IntoView {
    let params = use_params_map();

    // Extract address from URL params
    let address = move || {
        params.with(|p| p.get("addr").cloned().unwrap_or_default())
    };

    // Pagination state
    let (current_page, set_current_page) = create_signal(1u32);
    let (total_pages, set_total_pages) = create_signal(1u32);
    let (page_size, set_page_size) = create_signal(DEFAULT_PAGE_SIZE);
    let (total_items, set_total_items) = create_signal(0u32);

    // Filter state
    let (tx_filter, set_tx_filter) = create_signal(String::from("all"));

    // Timeframe state for activity stats
    let (timeframe, set_timeframe) = create_signal(String::from("24h"));

    // Copy state
    let (copied, set_copied) = create_signal(false);

    // Fetch address info
    let address_info = create_resource(
        address,
        |addr| async move {
            if addr.is_empty() {
                return Err(ServerFnError::ServerError("No address provided".to_string()));
            }
            get_address_info(addr).await
        }
    );

    // Fetch transactions (we'll filter client-side for the address)
    let transactions = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(200)).await }
    );

    // Copy address to clipboard handler
    let on_copy = move |_| {
        let addr = address();
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = ["navigator", "clipboard"], js_name = writeText)]
                fn write_text(s: &str) -> js_sys::Promise;
            }

            let _ = write_text(&addr);
            set_copied.set(true);

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

    // Page change handler
    let on_page_change = Callback::new(move |page: u32| {
        set_current_page.set(page);
    });

    // Page size change handler
    let on_page_size_change = Callback::new(move |size: u32| {
        set_page_size.set(size);
        set_current_page.set(1);
    });

    view! {
        <div class="fade-in">
            // Header with back navigation
            <div class="page-header" style="margin-bottom: 24px;">
                <div style="display: flex; align-items: center; gap: 16px;">
                    <a
                        href="/transactions"
                        class="btn btn-secondary"
                        style="padding: 8px 12px; display: flex; align-items: center; gap: 6px;"
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width: 16px; height: 16px;">
                            <polyline points="15,18 9,12 15,6"/>
                        </svg>
                        "Back"
                    </a>
                    <div>
                        <h1 class="page-title">"Address Details"</h1>
                        <p class="page-subtitle">"View address balances and transaction history"</p>
                    </div>
                </div>
                <a
                    href=move || format!("https://filecoin.blockscout.com/address/{}", address())
                    target="_blank"
                    class="btn btn-secondary"
                    style="display: flex; align-items: center; gap: 6px;"
                >
                    <ExternalLinkIcon />
                    "View on Blockscout"
                </a>
            </div>

            // Address Header Card
            <div class="card" style="margin-bottom: 24px;">
                <div style="display: flex; align-items: flex-start; justify-content: space-between; flex-wrap: wrap; gap: 16px;">
                    <div style="flex: 1; min-width: 300px;">
                        <div class="metric-label" style="margin-bottom: 8px;">"Address"</div>
                        <div style="display: flex; align-items: center; gap: 12px; flex-wrap: wrap;">
                            <code style="font-size: 14px; color: var(--accent-cyan); word-break: break-all; background: var(--bg-tertiary); padding: 8px 12px; border-radius: 6px; flex: 1; min-width: 280px;">
                                {address}
                            </code>
                            <button
                                class="btn btn-secondary"
                                style="padding: 8px 12px;"
                                on:click=on_copy
                                title="Copy address"
                            >
                                {move || if copied.get() {
                                    view! { <CheckIcon /> }.into_view()
                                } else {
                                    view! { <CopyIcon /> }.into_view()
                                }}
                            </button>
                        </div>
                    </div>
                </div>

                // Balance Display
                <Suspense fallback=move || view! {
                    <div class="grid-3" style="margin-top: 24px;">
                        {(0..3).map(|_| view! {
                            <div style="padding: 16px; background: var(--bg-tertiary); border-radius: 8px;">
                                <div class="skeleton" style="height: 14px; width: 80px; margin-bottom: 8px;"></div>
                                <div class="skeleton" style="height: 32px; width: 120px;"></div>
                            </div>
                        }).collect_view()}
                    </div>
                }>
                    {move || {
                        address_info.get().map(|res| match res {
                            Ok(info) => view! {
                                <div class="grid-3" style="margin-top: 24px;">
                                    <div style="padding: 16px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"USDFC Balance"</div>
                                        <div class="metric-value cyan" style="font-size: 28px;">
                                            {format_balance(&info.usdfc_balance)}
                                        </div>
                                    </div>
                                    <div style="padding: 16px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"Address Type"</div>
                                        <div class="metric-value purple" style="font-size: 28px;">
                                            {info.address_type.clone()}
                                        </div>
                                    </div>
                                    <div style="padding: 16px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"Total Transfers"</div>
                                        <div class="metric-value green" style="font-size: 28px;">
                                            {format_count(info.transfer_count as usize)}
                                        </div>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(e) => view! {
                                <div style="margin-top: 24px; padding: 16px; color: var(--accent-red); text-align: center;">
                                    "Error loading address: " {e.to_string()}
                                </div>
                            }.into_view()
                        })
                    }}
                </Suspense>
            </div>

            // Activity Stats Grid
            <div class="card" style="margin-bottom: 24px;">
                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
                    <h3 class="card-title" style="margin: 0;">"Activity Stats"</h3>
                    <div class="timeframe-tabs" style="display: flex; gap: 4px; background: var(--bg-tertiary); padding: 4px; border-radius: 8px;">
                        <button
                            class=move || if timeframe.get() == "24h" { "timeframe-btn active" } else { "timeframe-btn" }
                            style=move || format!(
                                "padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; {}",
                                if timeframe.get() == "24h" { "background: var(--accent-cyan); color: var(--bg-primary);" } else { "background: transparent; color: var(--text-secondary);" }
                            )
                            on:click=move |_| set_timeframe.set(String::from("24h"))
                        >
                            "24h"
                        </button>
                        <button
                            class=move || if timeframe.get() == "7d" { "timeframe-btn active" } else { "timeframe-btn" }
                            style=move || format!(
                                "padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; {}",
                                if timeframe.get() == "7d" { "background: var(--accent-cyan); color: var(--bg-primary);" } else { "background: transparent; color: var(--text-secondary);" }
                            )
                            on:click=move |_| set_timeframe.set(String::from("7d"))
                        >
                            "7d"
                        </button>
                        <button
                            class=move || if timeframe.get() == "30d" { "timeframe-btn active" } else { "timeframe-btn" }
                            style=move || format!(
                                "padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; {}",
                                if timeframe.get() == "30d" { "background: var(--accent-cyan); color: var(--bg-primary);" } else { "background: transparent; color: var(--text-secondary);" }
                            )
                            on:click=move |_| set_timeframe.set(String::from("30d"))
                        >
                            "30d"
                        </button>
                    </div>
                </div>
                <Suspense fallback=move || view! {
                    <div class="grid-4">
                        {(0..4).map(|_| view! {
                            <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                <div class="skeleton" style="height: 12px; width: 60px; margin-bottom: 8px;"></div>
                                <div class="skeleton" style="height: 20px; width: 80px;"></div>
                            </div>
                        }).collect_view()}
                    </div>
                }>
                    {move || {
                        let addr = address().to_lowercase();
                        let current_timeframe = timeframe.get();

                        // Get both address info and transactions
                        let info_opt = address_info.get().and_then(|r| r.ok());
                        let txs_opt = transactions.get().and_then(|r| r.ok());

                        // Calculate cutoff time based on timeframe (as Unix timestamp)
                        // Use js_sys::Date to get current time in WASM-compatible way
                        #[cfg(feature = "hydrate")]
                        let now = (js_sys::Date::now() / 1000.0) as u64;
                        #[cfg(not(feature = "hydrate"))]
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        let cutoff_secs: u64 = match current_timeframe.as_str() {
                            "24h" => 24 * 60 * 60,
                            "7d" => 7 * 24 * 60 * 60,
                            "30d" => 30 * 24 * 60 * 60,
                            _ => 24 * 60 * 60,
                        };
                        let cutoff = now.saturating_sub(cutoff_secs);

                        // Timeframe label for display
                        let timeframe_label = match current_timeframe.as_str() {
                            "24h" => "24h",
                            "7d" => "7d",
                            "30d" => "30d",
                            _ => "24h",
                        };

                        if let (Some(info), Some(txs)) = (info_opt, txs_opt.clone()) {
                            // Filter transactions for this address within timeframe
                            let addr_txs: Vec<&Transaction> = txs.iter()
                                .filter(|tx| {
                                    let is_addr_tx = tx.from.to_lowercase() == addr || tx.to.to_lowercase() == addr;
                                    // Compare Unix timestamps (tx.timestamp is u64 seconds)
                                    let is_in_timeframe = tx.timestamp >= cutoff;
                                    is_addr_tx && is_in_timeframe
                                })
                                .collect();

                            // Calculate stats
                            let incoming: Vec<&&Transaction> = addr_txs.iter()
                                .filter(|tx| tx.to.to_lowercase() == addr)
                                .collect();
                            let outgoing: Vec<&&Transaction> = addr_txs.iter()
                                .filter(|tx| tx.from.to_lowercase() == addr)
                                .collect();

                            let total_incoming: f64 = incoming.iter()
                                .map(|tx| decimal_to_f64(tx.amount))
                                .sum();
                            let total_outgoing: f64 = outgoing.iter()
                                .map(|tx| decimal_to_f64(tx.amount))
                                .sum();

                            let first_seen = info.first_seen.clone();

                            // Get last active from all transactions (not filtered by timeframe)
                            let all_addr_txs: Vec<&Transaction> = txs.iter()
                                .filter(|tx| tx.from.to_lowercase() == addr || tx.to.to_lowercase() == addr)
                                .collect();
                            let last_active = all_addr_txs.first()
                                .map(|tx| format_date(tx.timestamp))
                                .unwrap_or_else(|| "Never".to_string());

                            // Check if all activity stats are zero
                            let has_no_activity = incoming.is_empty() && outgoing.is_empty();

                            // Labels with timeframe
                            let incoming_label = format!("Incoming Transfers ({})", timeframe_label);
                            let outgoing_label = format!("Outgoing Transfers ({})", timeframe_label);
                            let received_label = format!("Total Received ({})", timeframe_label);
                            let sent_label = format!("Total Sent ({})", timeframe_label);

                            view! {
                                // Show message if no activity in timeframe
                                {if has_no_activity {
                                    view! {
                                        <div style="padding: 16px; background: var(--bg-tertiary); border-radius: 8px; text-align: center; margin-bottom: 16px; border: 1px dashed var(--border-color);">
                                            <span style="color: var(--text-muted);">
                                                {format!("No USDFC activity for this address in the last {}",
                                                    match current_timeframe.as_str() {
                                                        "24h" => "24 hours",
                                                        "7d" => "7 days",
                                                        "30d" => "30 days",
                                                        _ => "24 hours",
                                                    }
                                                )}
                                            </span>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}
                                <div class="grid-4">
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{incoming_label}</div>
                                        {if incoming.is_empty() {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"No activity"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value green">{format_count(incoming.len())}</div> }.into_view()
                                        }}
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{outgoing_label}</div>
                                        {if outgoing.is_empty() {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"No activity"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value red">{format_count(outgoing.len())}</div> }.into_view()
                                        }}
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"First Seen"</div>
                                        {if first_seen.is_empty() || first_seen == "-" {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"Never"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value purple">{first_seen}</div> }.into_view()
                                        }}
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"Last Active"</div>
                                        {if last_active == "Never" {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"Never"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value yellow">{last_active}</div> }.into_view()
                                        }}
                                    </div>
                                </div>
                                <div class="grid-2" style="margin-top: 16px;">
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{received_label}</div>
                                        {if total_incoming == 0.0 {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"No activity"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value green">{format!("{:.2} USDFC", total_incoming)}</div> }.into_view()
                                        }}
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{sent_label}</div>
                                        {if total_outgoing == 0.0 {
                                            view! { <div class="metric-value" style="color: var(--text-muted);">"No activity"</div> }.into_view()
                                        } else {
                                            view! { <div class="metric-value red">{format!("{:.2} USDFC", total_outgoing)}</div> }.into_view()
                                        }}
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="grid-4">
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{format!("Incoming Transfers ({})", timeframe_label)}</div>
                                        <div class="metric-value" style="color: var(--text-muted);">"-"</div>
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">{format!("Outgoing Transfers ({})", timeframe_label)}</div>
                                        <div class="metric-value" style="color: var(--text-muted);">"-"</div>
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"First Seen"</div>
                                        <div class="metric-value" style="color: var(--text-muted);">"-"</div>
                                    </div>
                                    <div style="padding: 12px; background: var(--bg-tertiary); border-radius: 8px;">
                                        <div class="metric-label">"Last Active"</div>
                                        <div class="metric-value" style="color: var(--text-muted);">"-"</div>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }}
                </Suspense>
            </div>

            // Transaction History
            <div class="card">
                <div class="card-header" style="margin-bottom: 16px;">
                    <div>
                        <h3 class="card-title">"Transaction History"</h3>
                        <p class="card-subtitle">"USDFC transfers involving this address"</p>
                    </div>
                    <div style="display: flex; align-items: center; gap: 12px;">
                        // Direction filter
                        <select
                            class="input"
                            style="width: auto; min-width: 140px;"
                            prop:value=tx_filter
                            on:change=move |ev| {
                                set_tx_filter.set(event_target_value(&ev));
                                set_current_page.set(1);
                            }
                        >
                            <option value="all">"All Transfers"</option>
                            <option value="incoming">"Incoming"</option>
                            <option value="outgoing">"Outgoing"</option>
                        </select>
                        <button
                            class="btn btn-secondary"
                            on:click=move |_| transactions.refetch()
                        >
                            <RefreshIcon />
                            "Refresh"
                        </button>
                    </div>
                </div>

                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"Direction"</th>
                                <th>"Amount"</th>
                                <th>"Counterparty"</th>
                                <th>"TX Hash"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="5" style="text-align: center; padding: 40px;">"Loading transactions..."</td></tr>
                            }>
                                {move || {
                                    let addr = address().to_lowercase();
                                    let filter = tx_filter.get();
                                    let page = current_page.get() as usize;
                                    let items_per_page = page_size.get() as usize;

                                    transactions.get().map(|res| match res {
                                        Ok(txs) => {
                                            // Filter transactions for this address
                                            let filtered: Vec<Transaction> = txs.iter()
                                                .filter(|tx| {
                                                    let from = tx.from.to_lowercase();
                                                    let to = tx.to.to_lowercase();
                                                    let is_related = from == addr || to == addr;

                                                    if !is_related {
                                                        return false;
                                                    }

                                                    match filter.as_str() {
                                                        "incoming" => to == addr,
                                                        "outgoing" => from == addr,
                                                        _ => true,
                                                    }
                                                })
                                                .cloned()
                                                .collect();

                                            // Calculate pagination
                                            let total_filtered = filtered.len();
                                            let total_pgs = ((total_filtered + items_per_page - 1) / items_per_page).max(1) as u32;
                                            set_total_pages.set(total_pgs);
                                            set_total_items.set(total_filtered as u32);

                                            // Get current page items
                                            let start_idx = (page - 1) * items_per_page;
                                            let page_items: Vec<Transaction> = filtered.into_iter()
                                                .skip(start_idx)
                                                .take(items_per_page)
                                                .collect();

                                            if page_items.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="5" style="text-align: center; padding: 40px; color: var(--text-muted);">
                                                            "No transactions found for this address"
                                                        </td>
                                                    </tr>
                                                }.into_view()
                                            } else {
                                                page_items.into_iter().map(|tx| {
                                                    let time_display = format_timestamp(tx.timestamp);
                                                    let amount_str = format_amount(tx.amount);
                                                    let to_lower = tx.to.to_lowercase();

                                                    // Determine direction and counterparty
                                                    let (direction, direction_class, counterparty) = if to_lower == addr {
                                                        ("Received", "type-badge deposit", tx.from.clone())
                                                    } else {
                                                        ("Sent", "type-badge withdraw", tx.to.clone())
                                                    };

                                                    let tx_hash = tx.hash.clone();
                                                    let tx_hash_link = tx.hash.clone();
                                                    let counterparty_link = counterparty.clone();

                                                    view! {
                                                        <tr>
                                                            <td>{time_display}</td>
                                                            <td><span class=direction_class>{direction}</span></td>
                                                            <td style="font-family: monospace;">{amount_str}" USDFC"</td>
                                                            <td style="font-family: monospace; font-size: 11px;">
                                                                <a
                                                                    href=format!("/address/{}", counterparty_link)
                                                                    class="address-link"
                                                                    title=counterparty.clone()
                                                                >
                                                                    {shorten_hash(&counterparty)}
                                                                </a>
                                                            </td>
                                                            <td style="font-family: monospace; font-size: 11px;">
                                                                <a
                                                                    href=format!("https://filecoin.blockscout.com/tx/{}", tx_hash_link)
                                                                    target="_blank"
                                                                    class="tx-hash-link"
                                                                    title=tx_hash.clone()
                                                                >
                                                                    {shorten_hash(&tx_hash)}
                                                                    <ExternalLinkIcon />
                                                                </a>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()
                                            }
                                        }
                                        Err(e) => view! {
                                            <tr>
                                                <td colspan="5" style="text-align: center; padding: 40px; color: var(--accent-red);">
                                                    {e.to_string()}
                                                </td>
                                            </tr>
                                        }.into_view()
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>

                // Pagination
                <PaginationWithSize
                    current_page=current_page
                    total_pages=total_pages
                    page_size=page_size
                    total_items=total_items
                    on_page_change=on_page_change
                    on_page_size_change=on_page_size_change
                />
            </div>
        </div>
    }
}

/// Legacy AddressLookup component - redirects to Explorer
#[component]
pub fn AddressLookup() -> impl IntoView {
    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Address Lookup"</h1>
                <p class="page-subtitle">"Search for address balances and history"</p>
            </div>

            <div class="card" style="text-align: center; padding: 40px;">
                <p style="color: var(--text-secondary); margin-bottom: 16px;">
                    "Address lookup has been merged with Transaction Search."
                </p>
                <a
                    href="/transactions"
                    class="btn btn-primary"
                    style="text-decoration: none;"
                >
                    "Go to Explorer"
                </a>
            </div>
        </div>
    }
}
