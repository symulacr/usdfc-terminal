use leptos::*;
use crate::server_fn::{get_protocol_metrics, get_recent_transactions};
use crate::format::{format_amount, shorten_hash};

#[component]
pub fn Alerts() -> impl IntoView {
    let metrics = create_resource(
        || (),
        |_| async move { get_protocol_metrics().await }
    );
    let transactions = create_resource(
        || (),
        |_| async move { get_recent_transactions(Some(25)).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Alerts"</h1>
                <p class="page-subtitle">"Protocol monitoring and notifications"</p>
            </div>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Live Alert Feed"</h3>
                        <p class="card-subtitle">"Auto-generated from protocol metrics and recent transfers"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| {
                            metrics.refetch();
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
                            if let Some(Ok(m)) = metrics.get() {
                                let tcr_f64: f64 = m.tcr.to_string().parse().unwrap_or(0.0);
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
                                        let large_tx = txs
                                            .iter()
                                            .filter(|tx| tx.amount.to_string().parse::<f64>().unwrap_or(0.0) >= 1_000_000.0)
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
        </div>
    }
}

