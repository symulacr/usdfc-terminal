use leptos::*;
use usdfc_backend::server_fn::check_api_health;

#[component]
pub fn Footer() -> impl IntoView {
    // Resource for API health (initial load, SSR)
    let health = create_resource(
        || (),
        |_| async move { check_api_health().await }
    );

    view! {
        <footer class="app-footer">
            <div class="footer-left">
                <span class="version">"USDFC Terminal v0.1.6"</span>
            </div>
            <div class="footer-right">
                <span class="status-label">"Live Data"</span>
                <Suspense fallback=move || view! {
                    <span class="status-item">
                        "Checking..."
                        <span class="status-dot warning"></span>
                    </span>
                }>
                    {move || {
                        health.get().map(|res| {
                            match res {
                                Ok(h) => {
                                    let rpc_dot = if h.rpc_ok { "status-dot healthy" } else { "status-dot disconnected" };
                                    let blockscout_dot = if h.blockscout_ok { "status-dot healthy" } else { "status-dot disconnected" };
                                    let subgraph_dot = if h.subgraph_ok { "status-dot healthy" } else { "status-dot disconnected" };

                                    view! {
                                        <span class="status-item">
                                            "RPC"
                                            <span class=rpc_dot></span>
                                        </span>
                                        <span class="status-item">
                                            "Blockscout"
                                            <span class=blockscout_dot></span>
                                        </span>
                                        <span class="status-item">
                                            "Subgraph"
                                            <span class=subgraph_dot></span>
                                        </span>
                                    }.into_view()
                                }
                                Err(_) => view! {
                                    <span class="status-item">
                                        "Status: Error"
                                        <span class="status-dot disconnected"></span>
                                    </span>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </footer>
    }
}
