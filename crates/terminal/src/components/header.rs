//! Header Component
//!
//! Fixed for hydration - uses Suspense with stable fallback.

use leptos::*;
use crate::app::{AppState, NetworkStatus};
use usdfc_api::{get_usdfc_price_data, get_holder_count, get_protocol_metrics};
use rust_decimal::prelude::ToPrimitive;

#[component]
pub fn Header() -> impl IntoView {
    let app_state = use_context::<AppState>().expect("AppState must be provided");
    let sidebar_expanded = app_state.sidebar_expanded;
    let network_status = app_state.network_status;
    let mobile_menu_open = app_state.mobile_menu_open;

    // Use regular resources for SSR compatibility
    let price_data = create_resource(
        || (),
        |_| async move { get_usdfc_price_data().await }
    );

    let holder_count = create_resource(
        || (),
        |_| async move { get_holder_count().await }
    );

    let protocol_metrics = create_resource(
        || (),
        |_| async move { get_protocol_metrics().await }
    );

    let toggle_sidebar = move |_| {
        mobile_menu_open.update(|v| *v = !*v);
        sidebar_expanded.update(|v| *v = !*v);
    };

    view! {
        <header class="header">
            <div class="header-left">
                <button
                    class="sidebar-toggle"
                    on:click=toggle_sidebar
                    title="Toggle sidebar"
                >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="3" y1="12" x2="21" y2="12"></line>
                        <line x1="3" y1="6" x2="21" y2="6"></line>
                        <line x1="3" y1="18" x2="21" y2="18"></line>
                    </svg>
                </button>
                <div class="logo">
                    <div class="logo-icon">"$"</div>
                    <span class="logo-text">"USDFC"</span>
                </div>
            </div>

            // Stats section - centered and distributed
            <div class="header-stats">
                // Price stat with change
                <Suspense fallback=move || view! {
                    <div class="stat">
                        <span class="label">"USDFC"</span>
                        <span class="value">"--"</span>
                    </div>
                }>
                    {move || {
                        price_data.get().map(|res| {
                            match res {
                                Ok(p) => {
                                    let price_display = p.price_usd
                                        .map(|v| format!("${:.4}", v))
                                        .unwrap_or_else(|| "--".to_string());
                                    let change = p.price_change_24h.unwrap_or(0.0);
                                    let change_class = if change >= 0.0 { "positive" } else { "negative" };
                                    let arrow = if change >= 0.0 { "+" } else { "" };
                                    let change_display = p.price_change_24h
                                        .map(|v| format!("{}{:.2}%", arrow, v))
                                        .unwrap_or_else(|| "--".to_string());
                                    view! {
                                        <div class="stat">
                                            <span class="label">"USDFC"</span>
                                            <span class="value">{price_display}</span>
                                            <span class={format!("change {}", change_class)}>
                                                {change_display}
                                            </span>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="stat">
                                        <span class="label">"USDFC"</span>
                                        <span class="value">"--"</span>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>

                // 24H Volume stat
                <Suspense fallback=move || view! {
                    <div class="stat">
                        <span class="label">"24H Vol"</span>
                        <span class="value">"--"</span>
                    </div>
                }>
                    {move || {
                        price_data.get().map(|res| {
                            match res {
                                Ok(p) => {
                                    let vol_display = p.volume_24h
                                        .map(format_compact)
                                        .unwrap_or_else(|| "--".to_string());
                                    view! {
                                        <div class="stat">
                                            <span class="label">"24H Vol"</span>
                                            <span class="value">{vol_display}</span>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="stat">
                                        <span class="label">"24H Vol"</span>
                                        <span class="value">"--"</span>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>

                // Liquidity stat
                <Suspense fallback=move || view! {
                    <div class="stat">
                        <span class="label">"Liquidity"</span>
                        <span class="value">"--"</span>
                    </div>
                }>
                    {move || {
                        price_data.get().map(|res| {
                            match res {
                                Ok(p) => {
                                    let liq_display = p.liquidity_usd
                                        .map(format_compact)
                                        .unwrap_or_else(|| "--".to_string());
                                    view! {
                                        <div class="stat">
                                            <span class="label">"Liquidity"</span>
                                            <span class="value">{liq_display}</span>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="stat">
                                        <span class="label">"Liquidity"</span>
                                        <span class="value">"--"</span>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>

                // Holders stat
                <Suspense fallback=move || view! {
                    <div class="stat">
                        <span class="label">"Holders"</span>
                        <span class="value">"--"</span>
                    </div>
                }>
                    {move || {
                        holder_count.get().map(|res| {
                            match res {
                                Ok(count) => {
                                    let display = format_number(count);
                                    view! {
                                        <div class="stat">
                                            <span class="label">"Holders"</span>
                                            <span class="value">{display}</span>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="stat">
                                        <span class="label">"Holders"</span>
                                        <span class="value">"--"</span>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>

                // TCR stat
                <Suspense fallback=move || view! {
                    <div class="stat">
                        <span class="label">"TCR"</span>
                        <span class="value">"--"</span>
                    </div>
                }>
                    {move || {
                        protocol_metrics.get().map(|res| {
                            match res {
                                Ok(metrics) => {
                                    let tcr_display = metrics.tcr.to_f64()
                                        .map(|v| format!("{:.1}%", v))
                                        .unwrap_or_else(|| "--".to_string());
                                    view! {
                                        <div class="stat">
                                            <span class="label">"TCR"</span>
                                            <span class="value">{tcr_display}</span>
                                        </div>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <div class="stat">
                                        <span class="label">"TCR"</span>
                                        <span class="value">"--"</span>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            <div class="header-right">
                <span
                    class="status-dot"
                    class:connected=move || network_status.get() == NetworkStatus::Connected
                    class:disconnected=move || network_status.get() == NetworkStatus::Disconnected
                    class:reconnecting=move || network_status.get() == NetworkStatus::Reconnecting
                ></span>
                <span class="network-label">
                    {move || match network_status.get() {
                        NetworkStatus::Connected => "Filecoin",
                        NetworkStatus::Disconnected => "Disconnected",
                        NetworkStatus::Reconnecting => "Reconnecting...",
                    }}
                </span>
            </div>
        </header>
    }
}

fn format_compact(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}

fn format_number(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        // Format with comma separator
        let thousands = value / 1_000;
        let remainder = value % 1_000;
        format!("{},{:03}", thousands, remainder)
    } else {
        value.to_string()
    }
}
