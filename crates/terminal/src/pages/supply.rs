use leptos::*;
use rust_decimal::prelude::ToPrimitive;
use usdfc_backend::server_fn::{get_protocol_metrics, get_recent_transactions, get_holder_count, get_top_holders};
use usdfc_core::format::{format_value, format_amount, format_timestamp, shorten_hash};

#[component]
pub fn SupplyMetrics() -> impl IntoView {
    let metrics = create_resource(
        || (),
        |_| async move { get_protocol_metrics().await }
    );
    
    let recent_tx = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(25)).await }
    );

    let holder_count = create_resource(
        || (),
        |_| async move { get_holder_count().await }
    );

    let top_holders = create_resource(
        || (),
        |_| async move { get_top_holders(Some(10), None).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Supply Metrics"</h1>
                <p class="page-subtitle">"USDFC supply analysis on Filecoin"</p>
            </div>

            <Suspense fallback=move || view! { <div class="card"><p>"Loading..."</p></div> }>
                {move || {
                    metrics.get().map(|res| {
                        match res {
                            Ok(m) => {
                                let total_supply = format_value(m.total_supply);
                                let circulating = format_value(m.circulating_supply);
                                let treasury = format_value(m.treasury_balance);

                                // Get holder count
                                let holders = holder_count.get()
                                    .and_then(|r| r.ok())
                                    .map(|c| c.to_string())
                                    .unwrap_or_else(|| "-".to_string());

                                view! {
                                    <div class="grid-4" style="margin-bottom: 24px;">
                                        <div class="card">
                                            <div class="metric-label">"Total Supply"</div>
                                            <div class="metric-value cyan">{total_supply}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Circulating Supply"</div>
                                            <div class="metric-value green">{circulating}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Treasury"</div>
                                            <div class="metric-value purple">{treasury}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Token Holders"</div>
                                            <div class="metric-value yellow">{holders}</div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(err) => view! {
                                <div class="card">
                                    <div class="metric-label">"Metrics Error"</div>
                                    <div class="metric-value red">{err.to_string()}</div>
                                </div>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>

            // Top Holders Table
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Top Holders"</h3>
                        <p class="card-subtitle">"Largest USDFC token holders"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| top_holders.refetch()
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
                                <th>"Share"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="4" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    top_holders.get().map(|res| {
                                        match res {
                                            Ok(holders) => {
                                                if holders.is_empty() {
                                                    view! {
                                                        <tr><td colspan="4" style="text-align: center; padding: 20px; color: var(--text-muted);">"No holder data available"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    // Calculate total for share percentage
                                                    let total: f64 = holders.iter().map(|h| {
                                                        h.balance.to_f64().unwrap_or(0.0)
                                                    }).sum();

                                                    holders.iter().enumerate().map(|(i, holder)| {
                                                        let balance_f64 = holder.balance.to_f64().unwrap_or(0.0);
                                                        let balance_display = format_value(holder.balance);
                                                        let share = if total > 0.0 { balance_f64 / total * 100.0 } else { 0.0 };
                                                        let addr = holder.address.clone();
                                                        view! {
                                                            <tr>
                                                                <td style="color: var(--text-muted);">{format!("#{}", i + 1)}</td>
                                                                <td style="font-family: monospace; font-size: 12px;">
                                                                    <a
                                                                        href=format!("https://filecoin.blockscout.com/address/{}", addr)
                                                                        target="_blank"
                                                                        style="color: var(--text-primary); text-decoration: none;"
                                                                        title=addr.clone()
                                                                    >
                                                                        {shorten_hash(&addr)}
                                                                    </a>
                                                                </td>
                                                                <td style="font-family: monospace; color: var(--accent-cyan);">{balance_display}</td>
                                                                <td style="color: var(--text-muted);">{format!("{:.1}%", share)}</td>
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
            </div>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Recent Supply Events"</h3>
                        <p class="card-subtitle">"Latest mint/burn activity from Blockscout"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| recent_tx.refetch()
                    >
                        "Refresh"
                    </button>
                </div>
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"TX Hash"</th>
                                <th>"Type"</th>
                                <th>"Amount"</th>
                                <th>"Time"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="4" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    recent_tx.get().map(|res| {
                                        match res {
                                            Ok(txs) => {
                                                let events: Vec<_> = txs
                                                    .into_iter()
                                                    .filter(|tx| matches!(tx.tx_type, crate::types::TransactionType::Mint | crate::types::TransactionType::Burn))
                                                    .collect();
                                                if events.is_empty() {
                                                    view! {
                                                        <tr><td colspan="4" style="text-align: center; padding: 20px; color: var(--text-muted);">"No recent mint/burn events"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    events.iter().map(|tx| {
                                                        let time = format_timestamp(tx.timestamp);
                                                        let amount = format_amount(tx.amount);
                                                        view! {
                                                            <tr>
                                                                <td style="font-family: monospace;">
                                                                    <a
                                                                        href=format!("https://filecoin.blockscout.com/tx/{}", tx.hash)
                                                                        target="_blank"
                                                                        style="color: var(--text-primary); text-decoration: none;"
                                                                        title=tx.hash.clone()
                                                                    >
                                                                        {shorten_hash(&tx.hash)}
                                                                    </a>
                                                                </td>
                                                                <td><span class={tx.tx_type.css_class()}>{tx.tx_type.as_str()}</span></td>
                                                                <td style="font-family: monospace;">{amount}</td>
                                                                <td>{time}</td>
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
            </div>
        </div>
    }
}

