//! Chart Header Component with controls

use leptos::*;
use crate::types::{ChartResolution, ChartLookback, ChartType};

/// Chart header with resolution, lookback selectors, and address input
#[allow(unused_variables)]
#[component]
pub fn ChartHeader(
    resolution: RwSignal<ChartResolution>,
    lookback: RwSignal<ChartLookback>,
    chart_type: RwSignal<ChartType>,
    wallet_address: RwSignal<Option<String>>,
    is_loading: ReadSignal<bool>,
) -> impl IntoView {
    // Local state for address input
    let address_input = create_rw_signal(String::new());

    // Handle address search - the actual logic
    let do_search = move || {
        let addr = address_input.get();
        if addr.starts_with("0x") && addr.len() >= 10 {
            wallet_address.set(Some(addr));
        } else if addr.is_empty() {
            wallet_address.set(None);
        }
    };

    // Click handler
    let on_search_click = move |_: ev::MouseEvent| {
        do_search();
    };

    // Keypress handler
    let on_search_keypress = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_search();
        }
    };

    // Clear wallet mode
    let clear_wallet = move |_: ev::MouseEvent| {
        wallet_address.set(None);
        address_input.set(String::new());
    };

    view! {
        <div class="chart-header">
            <div class="header-left">
                <span class="chart-title">"Advanced Analytics"</span>

                // Loading indicator
                <Show when=move || is_loading.get()>
                    <span class="loading-indicator">
                        <span class="loading-dot"></span>
                        <span class="loading-dot"></span>
                        <span class="loading-dot"></span>
                    </span>
                </Show>
            </div>

            <div class="header-center">
                // Address search input
                <div class="address-search">
                    <input
                        type="text"
                        placeholder="Enter wallet address (0x...)"
                        class="address-input"
                        prop:value=move || address_input.get()
                        on:input=move |ev| address_input.set(event_target_value(&ev))
                        on:keypress=on_search_keypress
                    />
                    <button class="search-btn" on:click=on_search_click>
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                        </svg>
                    </button>
                </div>

                // Show wallet address if in wallet mode
                <Show when=move || wallet_address.get().is_some()>
                    <div class="wallet-badge">
                        <span class="wallet-addr">
                            {move || {
                                wallet_address.get().map(|a| {
                                    if a.len() > 12 {
                                        format!("{}...{}", &a[..6], &a[a.len()-4..])
                                    } else {
                                        a
                                    }
                                }).unwrap_or_default()
                            }}
                        </span>
                        <button class="clear-wallet" on:click=clear_wallet>
                            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                            </svg>
                        </button>
                    </div>
                </Show>
            </div>

            <div class="header-right">
                // Lookback selector
                <div class="lookback-selector">
                    <For
                        each={move || [
                            ChartLookback::Hour1,
                            ChartLookback::Hour4,
                            ChartLookback::Day1,
                            ChartLookback::Week1,
                            ChartLookback::Month1,
                            ChartLookback::All,
                        ]}
                        key=|lb| lb.label()
                        children=move |lb| {
                            let is_active = move || lookback.get() == lb;
                            view! {
                                <button
                                    class:active=is_active
                                    class="lookback-btn"
                                    on:click=move |_| lookback.set(lb)
                                >
                                    {lb.label()}
                                </button>
                            }
                        }
                    />
                </div>

                // Refresh button
                <button class="refresh-btn" title="Refresh data">
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
