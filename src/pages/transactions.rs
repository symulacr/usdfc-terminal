use leptos::*;
use crate::components::icons::*;
use crate::components::PaginationWithSize;
use crate::server_fn::{get_recent_transactions, get_address_info};
use crate::format::{format_timestamp, format_timestamp_full, format_balance, shorten_hash, format_volume, format_amount, decimal_to_f64, format_count};
use crate::types::Transaction;

const DEFAULT_PAGE_SIZE: u32 = 20;

#[component]
pub fn TransactionSearch() -> impl IntoView {
    // Search and filter state
    let (search_address, set_search_address) = create_signal(String::new());
    let (filter_address, set_filter_address) = create_signal(String::new());
    let (filter_type, set_filter_type) = create_signal(String::from("all"));
    let (min_amount, set_min_amount) = create_signal::<Option<f64>>(None);
    let (max_amount, set_max_amount) = create_signal::<Option<f64>>(None);
    let (input_error, set_input_error) = create_signal::<Option<String>>(None);

    // Pagination state
    let (current_page, set_current_page) = create_signal(1u32);
    let (total_pages, set_total_pages) = create_signal(1u32);
    let (page_size, set_page_size) = create_signal(DEFAULT_PAGE_SIZE);
    let (total_items, set_total_items) = create_signal(0u32);

    // Modal state for address detail
    let (selected_address, set_selected_address) = create_signal(None::<String>);

    // Modal state for transaction detail
    let (selected_tx, set_selected_tx) = create_signal(None::<Transaction>);

    // Fetch more transactions for pagination (100 instead of 50)
    let transactions = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(100)).await }
    );

    // Real-time address lookup
    let address_info = create_resource(
        move || search_address.get(),
        move |addr| async move {
            if addr.len() >= 10 && validate_address(&addr) {
                Some(get_address_info(addr).await)
            } else {
                None
            }
        }
    );

    // Validate on input change
    let on_input = move |ev| {
        let addr = event_target_value(&ev);
        set_search_address.set(addr.clone());

        if addr.is_empty() {
            set_input_error.set(None);
        } else if addr.len() >= 10 && !validate_address(&addr) {
            set_input_error.set(Some("Invalid address format. Use 0x... (EVM) or f0/f1/f2/f3/f4 (Filecoin).".to_string()));
        } else {
            set_input_error.set(None);
        }
    };

    // Clear filters
    let clear_filters = move |_| {
        set_filter_address.set(String::new());
        set_filter_type.set(String::from("all"));
        set_min_amount.set(None);
        set_max_amount.set(None);
        set_search_address.set(String::new());
        set_current_page.set(1); // Reset to first page when clearing filters
    };

    // Page change handler
    let on_page_change = Callback::new(move |page: u32| {
        set_current_page.set(page);
    });

    // Page size change handler
    let on_page_size_change = Callback::new(move |size: u32| {
        set_page_size.set(size);
        set_current_page.set(1); // Reset to first page when changing page size
    });

    view! {
        <div class="fade-in">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Explorer"</h1>
                    <p class="page-subtitle">"Search addresses and view USDFC transfers"</p>
                </div>
                <button
                    class="btn btn-secondary"
                    on:click=move |_| {
                        set_current_page.set(1);
                        transactions.refetch();
                    }
                >
                    <RefreshIcon />
                    "Refresh"
                </button>
            </div>

            // Transfer Stats Summary
            <Suspense fallback=move || view! {
                <div class="grid-4" style="margin-bottom: 24px;">
                    {(0..4).map(|_| view! {
                        <div class="card" style="min-height: 60px;">
                            <div class="skeleton" style="height: 14px; width: 80px; margin-bottom: 8px;"></div>
                            <div class="skeleton" style="height: 24px; width: 60px;"></div>
                        </div>
                    }).collect_view()}
                </div>
            }>
                {move || {
                    transactions.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                // Calculate stats
                                let total_count = txs.len();
                                let total_volume: f64 = txs.iter()
                                    .map(|tx| decimal_to_f64(tx.amount))
                                    .sum();
                                let transfer_count = txs.iter()
                                    .filter(|tx| matches!(tx.tx_type, crate::types::TransactionType::Transfer))
                                    .count();
                                let mint_count = txs.iter()
                                    .filter(|tx| matches!(tx.tx_type, crate::types::TransactionType::Mint))
                                    .count();

                                let volume_display = format_volume(total_volume);

                                view! {
                                    <div class="grid-4" style="margin-bottom: 24px;">
                                        <div class="card">
                                            <div class="metric-label">"Transactions "<span class="timeframe-badge">"24h"</span></div>
                                            <div class="metric-value cyan">{format_count(total_count)}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Volume "<span class="timeframe-badge">"24h"</span></div>
                                            <div class="metric-value green">{volume_display}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Transfers "<span class="timeframe-badge">"24h"</span></div>
                                            <div class="metric-value purple">{format_count(transfer_count)}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Mints "<span class="timeframe-badge">"24h"</span></div>
                                            <div class="metric-value yellow">{format_count(mint_count)}</div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <div></div> }.into_view()
                        }
                    })
                }}
            </Suspense>

            // Address Search
            <div class="card" style="margin-bottom: 24px;">
                <h3 style="margin-bottom: 16px;">"Address Lookup"</h3>
                <div class="input-with-icon">
                    <SearchIcon />
                    <input
                        type="text"
                        class="input"
                        placeholder="Search address (0x..., f0..., f1..., f2..., f3..., f4...)"
                        style="font-family: monospace;"
                        on:input=on_input
                        prop:value=search_address
                    />
                </div>

                {move || {
                    if let Some(err) = input_error.get() {
                        return view! {
                            <div style="margin-top: 16px; padding: 12px; text-align: center; color: var(--accent-red); font-size: 13px;">
                                {err}
                            </div>
                        }.into_view();
                    }
                    view! { <div></div> }.into_view()
                }}

                <Suspense fallback=move || {
                    if search_address.get().len() >= 10 {
                        view! { <div style="margin-top: 16px; padding: 20px; text-align: center; color: var(--text-muted);">"Searching..."</div> }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }>
                    {move || {
                        address_info.get().map(|info_opt| {
                            match info_opt {
                                Some(Ok(info)) => {
                                    let addr_for_filter = info.address.clone();
                                    view! {
                                        <div style="margin-top: 16px; padding: 20px; background: var(--bg-tertiary); border-radius: 8px;">
                                            <div class="grid-4" style="margin-bottom: 16px;">
                                                <div>
                                                    <div class="metric-label">"USDFC Balance"</div>
                                                    <div class="metric-value cyan">{format_balance(&info.usdfc_balance)}</div>
                                                </div>
                                                <div>
                                                    <div class="metric-label">"Transfers"</div>
                                                    <div class="metric-value green">{format_count(info.transfer_count as usize)}</div>
                                                </div>
                                                <div>
                                                    <div class="metric-label">"First Seen"</div>
                                                    <div class="metric-value purple">{&info.first_seen}</div>
                                                </div>
                                                <div>
                                                    <div class="metric-label">"Type"</div>
                                                    <div class="metric-value yellow">{&info.address_type}</div>
                                                </div>
                                            </div>
                                            <div style="display: flex; align-items: center; gap: 12px; flex-wrap: wrap;">
                                                <div style="font-family: monospace; font-size: 12px; color: var(--text-muted); word-break: break-all; flex: 1;">
                                                    {&info.address}
                                                </div>
                                                <button
                                                    class="btn btn-primary"
                                                    style="font-size: 12px; padding: 6px 12px;"
                                                    on:click=move |_| set_filter_address.set(addr_for_filter.clone())
                                                >
                                                    <FilterIcon />
                                                    "View Transactions"
                                                </button>
                                                <a
                                                    href=format!("https://filecoin.blockscout.com/address/{}", &info.address)
                                                    target="_blank"
                                                    class="btn btn-secondary"
                                                    style="font-size: 12px; padding: 6px 12px;"
                                                >
                                                    <ExternalLinkIcon />
                                                    "Blockscout"
                                                </a>
                                            </div>
                                        </div>
                                    }.into_view()
                                },
                                Some(Err(err)) => view! {
                                    <div style="margin-top: 16px; padding: 16px; text-align: center; color: var(--accent-red); font-size: 13px;">
                                        {err.to_string()}
                                    </div>
                                }.into_view(),
                                None => view! { <div></div> }.into_view(),
                            }
                        }).unwrap_or_else(|| view! { <div></div> }.into_view())
                    }}
                </Suspense>
            </div>

            // Filters
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header" style="margin-bottom: 16px;">
                    <h3 class="card-title">"Filters"</h3>
                    {move || {
                        let has_filters = !filter_address.get().is_empty()
                            || filter_type.get() != "all"
                            || min_amount.get().is_some()
                            || max_amount.get().is_some();
                        if has_filters {
                            view! {
                                <button class="btn btn-secondary" on:click=clear_filters>
                                    "Clear All"
                                </button>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}
                </div>
                <div class="filter-grid">
                    // Transaction Type Filter
                    <div class="filter-group">
                        <label class="filter-label">"Transaction Type"</label>
                        <select
                            class="input filter-select"
                            prop:value=filter_type
                            on:change=move |ev| {
                                set_filter_type.set(event_target_value(&ev));
                                set_current_page.set(1);
                            }
                        >
                            <option value="all">"All Transactions"</option>
                            <option value="transfer">"Transfers"</option>
                            <option value="mint">"Mints"</option>
                            <option value="burn">"Burns"</option>
                            <option value="deposit">"Deposits"</option>
                            <option value="withdraw">"Withdrawals"</option>
                            <option value="liquidation">"Liquidations"</option>
                            <option value="redemption">"Redemptions"</option>
                        </select>
                    </div>

                    // Amount Range Filters
                    <div class="filter-group">
                        <label class="filter-label">"Min Amount"</label>
                        <input
                            type="number"
                            class="input"
                            placeholder="0"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    set_min_amount.set(None);
                                } else {
                                    set_min_amount.set(val.parse().ok());
                                }
                                set_current_page.set(1);
                            }
                        />
                    </div>

                    <div class="filter-group">
                        <label class="filter-label">"Max Amount"</label>
                        <input
                            type="number"
                            class="input"
                            placeholder="Unlimited"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    set_max_amount.set(None);
                                } else {
                                    set_max_amount.set(val.parse().ok());
                                }
                                set_current_page.set(1);
                            }
                        />
                    </div>

                    // Address Filter Input
                    <div class="filter-group">
                        <label class="filter-label">"Address"</label>
                        <input
                            type="text"
                            class="input filter-input"
                            placeholder="Filter by address..."
                            prop:value=filter_address
                            on:input=move |ev| {
                                set_filter_address.set(event_target_value(&ev));
                                set_current_page.set(1);
                            }
                        />
                    </div>
                </div>

            </div>

            // Active Filter Chips
            {move || {
                let has_type = filter_type.get() != "all";
                let has_address = !filter_address.get().is_empty();
                let has_min = min_amount.get().is_some();
                let has_max = max_amount.get().is_some();
                let has_any_filter = has_type || has_address || has_min || has_max;

                if has_any_filter {
                    view! {
                        <div class="filter-chips" style="display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px;">
                            {move || {
                                if filter_type.get() != "all" {
                                    let type_label = match filter_type.get().as_str() {
                                        "transfer" => "Transfers",
                                        "mint" => "Mints",
                                        "burn" => "Burns",
                                        "deposit" => "Deposits",
                                        "withdraw" => "Withdrawals",
                                        "liquidation" => "Liquidations",
                                        "redemption" => "Redemptions",
                                        _ => "Unknown"
                                    };
                                    view! {
                                        <span class="filter-chip" style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: var(--accent-purple); color: white; border-radius: 16px; font-size: 12px;">
                                            "Type: " {type_label}
                                            <button
                                                style="background: none; border: none; color: white; cursor: pointer; padding: 0; font-size: 14px; line-height: 1;"
                                                on:click=move |_| {
                                                    set_filter_type.set(String::from("all"));
                                                    set_current_page.set(1);
                                                }
                                            >
                                                {'\u{00D7}'}
                                            </button>
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                            {move || {
                                if !filter_address.get().is_empty() {
                                    let addr = filter_address.get();
                                    let display_addr = if addr.len() > 12 {
                                        format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
                                    } else {
                                        addr.clone()
                                    };
                                    view! {
                                        <span class="filter-chip" style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: var(--accent-cyan); color: white; border-radius: 16px; font-size: 12px;">
                                            "Address: " {display_addr}
                                            <button
                                                style="background: none; border: none; color: white; cursor: pointer; padding: 0; font-size: 14px; line-height: 1;"
                                                on:click=move |_| {
                                                    set_filter_address.set(String::new());
                                                    set_current_page.set(1);
                                                }
                                            >
                                                {'\u{00D7}'}
                                            </button>
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                            {move || {
                                if let Some(min) = min_amount.get() {
                                    view! {
                                        <span class="filter-chip" style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: var(--accent-green); color: white; border-radius: 16px; font-size: 12px;">
                                            "Min: " {format!("${:.2}", min)}
                                            <button
                                                style="background: none; border: none; color: white; cursor: pointer; padding: 0; font-size: 14px; line-height: 1;"
                                                on:click=move |_| {
                                                    set_min_amount.set(None);
                                                    set_current_page.set(1);
                                                }
                                            >
                                                {'\u{00D7}'}
                                            </button>
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                            {move || {
                                if let Some(max) = max_amount.get() {
                                    view! {
                                        <span class="filter-chip" style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: var(--accent-yellow); color: var(--bg-primary); border-radius: 16px; font-size: 12px;">
                                            "Max: " {format!("${:.2}", max)}
                                            <button
                                                style="background: none; border: none; color: var(--bg-primary); cursor: pointer; padding: 0; font-size: 14px; line-height: 1;"
                                                on:click=move |_| {
                                                    set_max_amount.set(None);
                                                    set_current_page.set(1);
                                                }
                                            >
                                                {'\u{00D7}'}
                                            </button>
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}

            // Transactions Table
            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"USDFC Transfers"</h3>
                        <p class="card-subtitle">"Click on addresses to filter by sender/receiver"</p>
                    </div>
                </div>
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"Type"</th>
                                <th>"Amount"</th>
                                <th>"From"</th>
                                <th>"To"</th>
                                <th>"TX Hash"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="6" style="text-align: center; padding: 40px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    let type_filter = filter_type.get();
                                    let addr_filter = filter_address.get().to_lowercase();
                                    let min_amt = min_amount.get();
                                    let max_amt = max_amount.get();
                                    let page = current_page.get() as usize;
                                    let items_per_page = page_size.get() as usize;

                                    transactions.get().map(|res| {
                                        match res {
                                            Ok(txs) => {
                                                // Apply filters and clone to own the data
                                                let filtered: Vec<Transaction> = txs.iter().filter(|tx| {
                                                    // Type filter - exact match on lowercase
                                                    if type_filter != "all" {
                                                        let tx_type = tx.tx_type.as_str().to_lowercase();
                                                        if tx_type != type_filter {
                                                            return false;
                                                        }
                                                    }

                                                    // Address filter
                                                    if !addr_filter.is_empty() {
                                                        let from = tx.from.to_lowercase();
                                                        let to = tx.to.to_lowercase();
                                                        if !from.contains(&addr_filter) && !to.contains(&addr_filter) {
                                                            return false;
                                                        }
                                                    }

                                                    // Amount filters
                                                    let amount_f64 = decimal_to_f64(tx.amount);
                                                    if let Some(min) = min_amt {
                                                        if amount_f64 < min {
                                                            return false;
                                                        }
                                                    }
                                                    if let Some(max) = max_amt {
                                                        if amount_f64 > max {
                                                            return false;
                                                        }
                                                    }

                                                    true
                                                }).cloned().collect();

                                                // Calculate pagination
                                                let total_filtered = filtered.len();
                                                let total_pgs = ((total_filtered + items_per_page - 1) / items_per_page).max(1) as u32;
                                                set_total_pages.set(total_pgs);
                                                set_total_items.set(total_filtered as u32);

                                                // Get current page items
                                                let start_idx = (page - 1) * items_per_page;
                                                let page_items: Vec<Transaction> = filtered.into_iter().skip(start_idx).take(items_per_page).collect();

                                                if page_items.is_empty() {
                                                    view! {
                                                        <tr><td colspan="6" style="text-align: center; padding: 40px; color: var(--text-muted);">"No matching transactions"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    page_items.into_iter().map(|tx| {
                                                        let time_ago = format_timestamp(tx.timestamp);
                                                        let amount_str = format_amount(tx.amount);
                                                        let tx_type_class = tx.tx_type.css_class().to_string();
                                                        let tx_type_str = tx.tx_type.as_str().to_string();
                                                        let from_addr = tx.from.clone();
                                                        let to_addr = tx.to.clone();
                                                        let from_for_modal = tx.from.clone();
                                                        let to_for_modal = tx.to.clone();
                                                        let hash_display = tx.hash.clone();
                                                        let tx_for_modal = tx;

                                                        view! {
                                                            <tr>
                                                                <td>{time_ago}</td>
                                                                <td><span class={tx_type_class}>{tx_type_str}</span></td>
                                                                <td style="font-family: monospace;">{amount_str}</td>
                                                                <td style="font-family: monospace; font-size: 11px;">
                                                                    <div style="display: flex; align-items: center; gap: 4px;">
                                                                        <a
                                                                            href=format!("/address/{}", from_addr)
                                                                            style="color: var(--accent-cyan); text-decoration: none;"
                                                                            title=from_addr.clone()
                                                                        >
                                                                            {shorten_hash(&from_addr)}
                                                                        </a>
                                                                        <a
                                                                            href=format!("https://filecoin.blockscout.com/address/{}", from_for_modal)
                                                                            target="_blank"
                                                                            style="color: var(--text-muted); text-decoration: none; font-size: 10px;"
                                                                            title="View on Blockscout"
                                                                        >
                                                                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                                <polyline points="15 3 21 3 21 9"></polyline>
                                                                                <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                            </svg>
                                                                        </a>
                                                                    </div>
                                                                </td>
                                                                <td style="font-family: monospace; font-size: 11px;">
                                                                    <div style="display: flex; align-items: center; gap: 4px;">
                                                                        <a
                                                                            href=format!("/address/{}", to_addr)
                                                                            style="color: var(--accent-cyan); text-decoration: none;"
                                                                            title=to_addr.clone()
                                                                        >
                                                                            {shorten_hash(&to_addr)}
                                                                        </a>
                                                                        <a
                                                                            href=format!("https://filecoin.blockscout.com/address/{}", to_for_modal)
                                                                            target="_blank"
                                                                            style="color: var(--text-muted); text-decoration: none; font-size: 10px;"
                                                                            title="View on Blockscout"
                                                                        >
                                                                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                                                <polyline points="15 3 21 3 21 9"></polyline>
                                                                                <line x1="10" y1="14" x2="21" y2="3"></line>
                                                                            </svg>
                                                                        </a>
                                                                    </div>
                                                                </td>
                                                                <td style="font-family: monospace; font-size: 11px;">
                                                                    <span
                                                                        class="tx-hash-link clickable"
                                                                        title=hash_display.clone()
                                                                        on:click=move |_| {
                                                                            set_selected_tx.set(Some(tx_for_modal.clone()));
                                                                        }
                                                                    >
                                                                        {shorten_hash(&hash_display)}
                                                                    </span>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="6" style="text-align: center; padding: 40px; color: var(--accent-red);">{err.to_string()}</td></tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>

                // Pagination controls
                <PaginationWithSize
                    current_page=current_page
                    total_pages=total_pages
                    page_size=page_size
                    total_items=total_items
                    on_page_change=on_page_change
                    on_page_size_change=on_page_size_change
                />
            </div>

            // Address Detail Modal
            {move || selected_address.get().map(|addr| view! {
                <AddressDetailModal
                    address=addr
                    on_close=Callback::new(move |_| set_selected_address.set(None))
                />
            })}

            // Transaction Detail Modal
            {move || selected_tx.get().map(|tx| view! {
                <TransactionDetailModal
                    transaction=tx
                    on_close=Callback::new(move |_| set_selected_tx.set(None))
                    on_address_click=Callback::new(move |addr: String| {
                        set_selected_tx.set(None);
                        set_selected_address.set(Some(addr));
                    })
                />
            })}
        </div>
    }
}

fn validate_address(address: &str) -> bool {
    // EVM address: 0x + 40 hex chars
    let is_evm = address.len() == 42
        && address.starts_with("0x")
        && address[2..].chars().all(|c| c.is_ascii_hexdigit());

    // Filecoin addresses: f0, f1, f2, f3, f4 prefixes
    let is_filecoin = (address.starts_with("f0")
        || address.starts_with("f1")
        || address.starts_with("f2")
        || address.starts_with("f3")
        || address.starts_with("f4"))
        && address.len() > 2
        && address[2..].chars().all(|c| c.is_ascii_alphanumeric());

    is_evm || is_filecoin
}

#[component]
fn AddressDetailModal(
    address: String,
    on_close: Callback<()>,
) -> impl IntoView {
    let address_for_resource = address.clone();
    let address_for_header = address.clone();

    let address_info = create_resource(
        move || address_for_resource.clone(),
        |addr| async move { get_address_info(addr).await }
    );

    view! {
        <div
            class="address-modal-backdrop"
            on:click=move |_| on_close.call(())
        >
            <div
                class="address-modal-content"
                on:click=|e| e.stop_propagation()
            >
                <div class="address-modal-header">
                    <h2 style="font-family: monospace; font-size: 14px;">
                        {shorten_hash(&address_for_header)}
                    </h2>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| on_close.call(())
                    >
                        "×"
                    </button>
                </div>

                <Suspense fallback=move || view! {
                    <div style="padding: 40px; text-align: center;">"Loading address details..."</div>
                }>
                    {move || address_info.get().map(|res| match res {
                        Ok(info) => {
                            // Use the address from the response for the link
                            let link_addr = info.address.clone();
                            view! {
                                <div class="address-stats-grid">
                                    <div class="stat-card">
                                        <span class="stat-label">"USDFC Balance"</span>
                                        <span class="stat-value">{format_balance(&info.usdfc_balance)}</span>
                                    </div>
                                    <div class="stat-card">
                                        <span class="stat-label">"Transfers"</span>
                                        <span class="stat-value">{format_count(info.transfer_count as usize)}</span>
                                    </div>
                                    <div class="stat-card">
                                        <span class="stat-label">"First Seen"</span>
                                        <span class="stat-value">{info.first_seen.clone()}</span>
                                    </div>
                                    <div class="stat-card">
                                        <span class="stat-label">"Type"</span>
                                        <span class="stat-value">{info.address_type.clone()}</span>
                                    </div>
                                </div>

                                <div style="margin-top: 16px;">
                                    <a
                                        href=format!("https://filecoin.blockscout.com/address/{}", link_addr)
                                        target="_blank"
                                        class="btn btn-primary"
                                    >
                                        "View on Blockscout →"
                                    </a>
                                </div>
                            }.into_view()
                        },
                        Err(e) => view! {
                            <div style="color: var(--accent-red);">{e.to_string()}</div>
                        }.into_view()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

/// Transaction Detail Modal - displays full transaction information
#[component]
fn TransactionDetailModal(
    transaction: Transaction,
    on_close: Callback<()>,
    on_address_click: Callback<String>,
) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);

    let tx_hash = transaction.hash.clone();
    let tx_hash_for_copy = transaction.hash.clone();
    let tx_hash_for_link = transaction.hash.clone();
    let from_addr = transaction.from.clone();
    let to_addr = transaction.to.clone();
    let from_for_click = transaction.from.clone();
    let to_for_click = transaction.to.clone();

    // Copy to clipboard handler
    let on_copy = move |_| {
        let hash = tx_hash_for_copy.clone();
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = ["navigator", "clipboard"], js_name = writeText)]
                fn write_text(s: &str) -> js_sys::Promise;
            }

            let _ = write_text(&hash);
            set_copied.set(true);

            use gloo_timers::callback::Timeout;
            let timeout = Timeout::new(1500, move || {
                set_copied.set(false);
            });
            timeout.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = hash;
        }
    };

    // Lock body scroll and ESC key handler when modal opens
    #[cfg(feature = "hydrate")]
    {
        use leptos::*;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        // Lock body scroll on mount
        create_effect(move |_| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(body) = document.body() {
                let _ = body.style().set_property("overflow", "hidden");
            }
            // Scroll to top of page to ensure modal is visible
            window.scroll_to_with_x_and_y(0.0, 0.0);
        });

        // Unlock body scroll on cleanup
        on_cleanup(move || {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        let _ = body.style().remove_property("overflow");
                    }
                }
            }
        });

        // ESC key handler
        let close_callback = on_close.clone();
        create_effect(move |_| {
            let callback = close_callback.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if event.key() == "Escape" {
                    callback.call(());
                }
            }) as Box<dyn Fn(_)>);

            let window = web_sys::window().unwrap();
            let _ = window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());

            on_cleanup(move || {
                let window = web_sys::window().unwrap();
                let _ = window.remove_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            });
        });
    }

    // Format timestamp
    let time_display = format_timestamp_full(transaction.timestamp);
    let amount_display = format_amount(transaction.amount);

    // Status display
    let status_class = transaction.status.css_class();
    let status_text = transaction.status.as_str();
    let is_success = matches!(transaction.status, crate::types::TransactionStatus::Success);

    // Transaction type
    let tx_type = transaction.tx_type.as_str();
    let tx_type_class = transaction.tx_type.css_class();

    view! {
        <div
            class="address-modal-backdrop"
            on:click=move |_| on_close.call(())
        >
            <div
                class="address-modal-content tx-modal-content"
                on:click=|e| e.stop_propagation()
            >
                // Header
                <div class="address-modal-header">
                    <h2 style="color: var(--accent-purple); font-size: 16px; font-weight: 600;">
                        "Transaction Details"
                    </h2>
                    <button
                        class="btn btn-secondary"
                        style="padding: 4px 8px; font-size: 18px; line-height: 1;"
                        on:click=move |_| on_close.call(())
                    >
                        {'\u{00D7}'}
                    </button>
                </div>

                // Transaction Hash with copy button
                <div class="tx-modal-section">
                    <div class="tx-modal-label">"Transaction Hash"</div>
                    <div class="tx-modal-hash-row">
                        <span class="tx-modal-hash">{tx_hash.clone()}</span>
                        <button
                            class="tx-copy-btn"
                            on:click=on_copy
                            title=move || if copied.get() { "Copied!" } else { "Copy hash" }
                        >
                            {move || if copied.get() {
                                view! { <span style="color: var(--accent-green); font-size: 11px;">"Copied!"</span> }.into_view()
                            } else {
                                view! {
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                                    </svg>
                                }.into_view()
                            }}
                        </button>
                    </div>
                </div>

                // Status and Type row
                <div class="tx-modal-grid">
                    <div class="tx-modal-item">
                        <div class="tx-modal-label">"Status"</div>
                        <div class="tx-modal-value">
                            <span class={status_class}>
                                {if is_success {
                                    view! { <span style="color: var(--accent-green);">{'\u{2713}'}" "</span> }.into_view()
                                } else {
                                    view! { <span style="color: var(--accent-red);">{'\u{2717}'}" "</span> }.into_view()
                                }}
                                {status_text}
                            </span>
                        </div>
                    </div>
                    <div class="tx-modal-item">
                        <div class="tx-modal-label">"Type"</div>
                        <div class="tx-modal-value">
                            <span class={tx_type_class}>{tx_type}</span>
                        </div>
                    </div>
                </div>

                // Block and Time row
                <div class="tx-modal-grid">
                    <div class="tx-modal-item">
                        <div class="tx-modal-label">"Block Number"</div>
                        <div class="tx-modal-value" style="font-family: monospace;">
                            {format!("{}", transaction.block)}
                        </div>
                    </div>
                    <div class="tx-modal-item">
                        <div class="tx-modal-label">"Timestamp"</div>
                        <div class="tx-modal-value" style="font-size: 12px;">
                            {time_display}
                        </div>
                    </div>
                </div>

                // From Address
                <div class="tx-modal-section">
                    <div class="tx-modal-label">"From"</div>
                    <div class="tx-modal-address-row">
                        <span
                            class="tx-modal-address clickable"
                            on:click=move |_| on_address_click.call(from_for_click.clone())
                        >
                            {from_addr.clone()}
                        </span>
                    </div>
                </div>

                // To Address
                <div class="tx-modal-section">
                    <div class="tx-modal-label">"To"</div>
                    <div class="tx-modal-address-row">
                        <span
                            class="tx-modal-address clickable"
                            on:click=move |_| on_address_click.call(to_for_click.clone())
                        >
                            {to_addr.clone()}
                        </span>
                    </div>
                </div>

                // Value
                <div class="tx-modal-section">
                    <div class="tx-modal-label">"Value Transferred"</div>
                    <div class="tx-modal-value-large">
                        {amount_display}" USDFC"
                    </div>
                </div>

                // External Link
                <div style="margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color);">
                    <a
                        href=format!("https://filecoin.blockscout.com/tx/{}", tx_hash_for_link)
                        target="_blank"
                        class="btn btn-primary"
                        style="display: inline-flex; align-items: center; gap: 8px;"
                    >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                            <polyline points="15,3 21,3 21,9"/>
                            <line x1="10" y1="14" x2="21" y2="3"/>
                        </svg>
                        "View on Blockscout"
                    </a>
                </div>
            </div>
        </div>
    }
}
