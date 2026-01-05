use leptos::*;
use crate::components::gauge::GaugeChart;
use usdfc_api::{get_protocol_metrics, get_troves};
use usdfc_core::format::{format_value, format_fil, format_usdfc};

#[component]
pub fn CollateralHealth() -> impl IntoView {
    let metrics = create_resource(
        || (),
        |_| async move { get_protocol_metrics().await }
    );
    
    let troves = create_resource(
        || (),
        |_| async move { get_troves(Some(20), None).await }
    );

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Collateral Health"</h1>
                <p class="page-subtitle">"FIL collateral and trove health on Filecoin"</p>
            </div>

            <Suspense fallback=move || view! { <div class="card"><p>"Loading..."</p></div> }>
                {move || {
                    metrics.get().map(|res| {
                        match res {
                            Ok(m) => {
                                let total_collateral = format_value(m.total_collateral);
                                let tcr_f64: f64 = m.tcr.to_string().parse().unwrap_or(0.0);
                                let tcr = format!("{:.1}%", tcr_f64);
                                let active_troves = m.active_troves;
                                
                                view! {
                                    <div class="grid-3" style="margin-bottom: 24px;">
                                        <div class="card">
                                            <div class="metric-label">"Total FIL Collateral"</div>
                                            <div class="metric-value green">{total_collateral}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"System TCR"</div>
                                            <div class="metric-value cyan">{tcr}</div>
                                        </div>
                                        <div class="card">
                                            <div class="metric-label">"Active Troves"</div>
                                            <div class="metric-value purple">{active_troves.to_string()}</div>
                                        </div>
                                    </div>
                                    
                                    <div class="card" style="margin-bottom: 24px;">
                                        <h3 style="margin-bottom: 16px;">"Total Collateral Ratio"</h3>
                                        <GaugeChart value=tcr_f64 min=110.0 max=250.0 label="System TCR" suffix="%" />
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

            <div class="card" style="margin-bottom: 24px;">
                <h3 style="margin-bottom: 16px;">"Active Troves"</h3>
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Address"</th>
                                <th>"FIL Collateral"</th>
                                <th>"USDFC Debt"</th>
                                <th>"ICR"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! { 
                                <tr><td colspan="5" style="text-align: center; padding: 20px;">"Loading troves..."</td></tr> 
                            }>
                                {move || {
                                    troves.get().map(|res| {
                                        match res {
                                            Ok(all_troves) => {
                                                if all_troves.is_empty() {
                                                    view! {
                                                        <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--text-muted);">"No troves found"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    all_troves.iter().map(|t| {
                                                        let icr: f64 = t.icr.to_string().parse().unwrap_or(0.0);
                                                        let (status_class, status_text) = if icr < 115.0 {
                                                            ("color: var(--accent-red);", "Critical")
                                                        } else if icr < 150.0 {
                                                            ("color: var(--accent-yellow);", "At Risk")
                                                        } else {
                                                            ("color: var(--accent-green);", "Healthy")
                                                        };
                                                        let collateral = format_fil(t.collateral);
                                                        let debt = format_usdfc(t.debt);
                                                        let short_addr = format!("{}...{}", &t.address[..8], &t.address[t.address.len()-6..]);
                                                        view! {
                                                            <tr>
                                                                <td style="font-family: monospace; font-size: 12px;">
                                                                    <a
                                                                        href=format!("https://filecoin.blockscout.com/address/{}", t.address)
                                                                        target="_blank"
                                                                        style="color: var(--text-primary); text-decoration: none;"
                                                                        title=t.address.clone()
                                                                    >
                                                                        {short_addr}
                                                                    </a>
                                                                </td>
                                                                <td>{collateral}</td>
                                                                <td>{debt}</td>
                                                                <td style=status_class>{format!("{:.1}%", icr)}</td>
                                                                <td style=status_class>{status_text}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }
                                            Err(err) => view! {
                                                <tr><td colspan="5" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="grid-2">
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Collateral Info"</h3>
                    <p style="color: var(--text-secondary); line-height: 1.8;">
                        "USDFC is backed by FIL (Filecoin) collateral. Each Trove must maintain 
                        a minimum Individual Collateral Ratio (ICR) of 110%. Troves below this 
                        threshold can be liquidated."
                    </p>
                </div>
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Risk Thresholds"</h3>
                    <div style="color: var(--text-secondary); line-height: 2;">
                        <div>"Minimum ICR: "<span style="color: var(--accent-red);">"110%"</span></div>
                        <div>"Recovery Mode: "<span style="color: var(--accent-yellow);">"150%"</span></div>
                        <div>"Safe Zone: "<span style="color: var(--accent-green);">">200%"</span></div>
                    </div>
                </div>
            </div>
        </div>
    }
}

