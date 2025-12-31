use leptos::*;
use rust_decimal::prelude::ToPrimitive;
use crate::server_fn::{get_protocol_metrics, get_stability_pool_transfers};
use crate::format::{format_value, format_amount, format_timestamp, shorten_hash};

#[component]
pub fn StabilityPool() -> impl IntoView {
    let metrics = create_resource(
        || (),
        |_| async move { get_protocol_metrics().await }
    );
    
    let pool_transfers = create_resource(
        || (),
        |_| async move { get_stability_pool_transfers(Some(20)).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Stability Pool"</h1>
                <p class="page-subtitle">"USDFC stability pool on Filecoin"</p>
            </div>

            <Suspense fallback=move || view! { <div class="card"><p>"Loading..."</p></div> }>
                {move || {
                    metrics.get().map(|res| {
                        match res {
                            Ok(m) => {
                                let pool_balance = format_value(m.stability_pool_balance);
                                let total_supply = format_value(m.total_supply);
                                
                                let supply_f64: f64 = m.total_supply.to_string().parse().unwrap_or(0.0);
                                let pool_f64: f64 = m.stability_pool_balance.to_string().parse().unwrap_or(0.0);
                                let coverage = if supply_f64 > 0.0 { pool_f64 / supply_f64 * 100.0 } else { 0.0 };
                                
                                view! {
                                    <div class="grid-3" style="margin-bottom: 24px;">
                                        <div class="card">
                                            <div class="metric-label">"Pool Balance"</div>
                                            <div class="metric-value cyan">{pool_balance}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Total USDFC Supply"</div>
                                            <div class="metric-value green">{total_supply}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Coverage Ratio"</div>
                                            <div class="metric-value yellow">{format!("{:.1}%", coverage)}</div>
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

            <div class="grid-2" style="margin-bottom: 24px;">
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"How It Works"</h3>
                    <p style="color: var(--text-secondary); line-height: 1.8;">
                        "The Stability Pool is the first line of defense in maintaining system solvency. 
                        USDFC holders can deposit their tokens to the pool. When a Trove is liquidated, 
                        the deposited USDFC is used to repay the debt, and depositors receive the 
                        liquidated FIL collateral at a discount."
                    </p>
                </div>
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Rewards"</h3>
                    <p style="color: var(--text-secondary); line-height: 1.8;">
                        "Stability Pool depositors earn FIL from liquidations. When a Trove is 
                        liquidated below 110% collateral ratio, depositors receive the FIL collateral 
                        proportional to their share of the pool, typically at a ~10% discount."
                    </p>
                </div>
            </div>

            // Pool Activity Summary
            <Suspense fallback=move || view! { <div></div> }>
                {move || {
                    pool_transfers.get().map(|res| {
                        match res {
                            Ok(txs) => {
                                // Calculate deposits (transfers TO pool) and withdrawals (transfers FROM pool)
                                let stability_pool_addr = "0x791ad78bbc58324089d3e0a8689e7d045b9592b5".to_lowercase();

                                let deposits: f64 = txs.iter()
                                    .filter(|tx| tx.to.to_lowercase() == stability_pool_addr)
                                    .map(|tx| tx.amount.to_f64().unwrap_or(0.0))
                                    .sum();
                                let withdrawals: f64 = txs.iter()
                                    .filter(|tx| tx.from.to_lowercase() == stability_pool_addr)
                                    .map(|tx| tx.amount.to_f64().unwrap_or(0.0))
                                    .sum();
                                let deposit_count = txs.iter()
                                    .filter(|tx| tx.to.to_lowercase() == stability_pool_addr)
                                    .count();
                                let withdrawal_count = txs.iter()
                                    .filter(|tx| tx.from.to_lowercase() == stability_pool_addr)
                                    .count();

                                let format_volume = |v: f64| {
                                    if v >= 1_000_000.0 { format!("{:.2}M", v / 1_000_000.0) }
                                    else if v >= 1_000.0 { format!("{:.1}K", v / 1_000.0) }
                                    else { format!("{:.0}", v) }
                                };

                                view! {
                                    <div class="card" style="margin-bottom: 24px;">
                                        <h3 class="card-title" style="margin-bottom: 16px;">"Recent Activity"</h3>
                                        <div class="grid-4">
                                            <div>
                                                <div class="metric-label">"Deposits"</div>
                                                <div class="metric-value green">{format_volume(deposits)}</div>
                                                <div style="color: var(--text-muted); font-size: 12px;">{format!("{} txs", deposit_count)}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Withdrawals"</div>
                                                <div class="metric-value red">{format_volume(withdrawals)}</div>
                                                <div style="color: var(--text-muted); font-size: 12px;">{format!("{} txs", withdrawal_count)}</div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Net Flow"</div>
                                                <div class={if deposits >= withdrawals { "metric-value green" } else { "metric-value red" }}>
                                                    {if deposits >= withdrawals { "+" } else { "" }}{format_volume(deposits - withdrawals)}
                                                </div>
                                            </div>
                                            <div>
                                                <div class="metric-label">"Total Activity"</div>
                                                <div class="metric-value cyan">{format_volume(deposits + withdrawals)}</div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            Err(_) => view! { <div></div> }.into_view()
                        }
                    })
                }}
            </Suspense>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Stability Pool Transfers"</h3>
                        <p class="card-subtitle">"Recent USDFC transfers involving the Stability Pool"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| pool_transfers.refetch()
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
                                <th>"From"</th>
                                <th>"To"</th>
                                <th>"Time"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="6" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    pool_transfers.get().map(|res| {
                                        match res {
                                            Ok(txs) => {
                                                if txs.is_empty() {
                                                    view! {
                                                        <tr><td colspan="6" style="text-align: center; padding: 20px; color: var(--text-muted);">"No transfers found"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    txs.iter().map(|tx| {
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
                                                                <td style="font-family: monospace;">
                                                                    <a
                                                                        href=format!("https://filecoin.blockscout.com/address/{}", tx.from)
                                                                        target="_blank"
                                                                        style="color: var(--text-primary); text-decoration: none;"
                                                                        title=tx.from.clone()
                                                                    >
                                                                        {shorten_hash(&tx.from)}
                                                                    </a>
                                                                </td>
                                                                <td style="font-family: monospace;">
                                                                    <a
                                                                        href=format!("https://filecoin.blockscout.com/address/{}", tx.to)
                                                                        target="_blank"
                                                                        style="color: var(--text-primary); text-decoration: none;"
                                                                        title=tx.to.clone()
                                                                    >
                                                                        {shorten_hash(&tx.to)}
                                                                    </a>
                                                                </td>
                                                                <td>{time}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="6" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
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

