use leptos::*;
use crate::components::icons::*;
use usdfc_core::config::config;

#[component]
pub fn DataExport() -> impl IntoView {
    let cfg = config();
    let usdfc_token = cfg.usdfc_token.clone();
    let blockscout_url = "https://filecoin.blockscout.com";
    let subgraph_url = cfg.subgraph_url.clone();

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Data Export"</h1>
                <p class="page-subtitle">"Export USDFC protocol data from multiple sources"</p>
            </div>

            // Blockscout Exports
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Blockscout CSV Exports"</h3>
                        <p class="card-subtitle">"Direct download from Filecoin Blockscout"</p>
                    </div>
                </div>
                <div class="grid-2" style="padding: 16px; gap: 16px;">
                    <div class="export-card">
                        <h4>"Token Transfers"</h4>
                        <p>"All USDFC transfers in CSV format"</p>
                        <a
                            href=format!("{}/api/v2/tokens/{}/transfers?csv=true", blockscout_url, usdfc_token)
                            target="_blank"
                            class="btn btn-primary"
                        >
                            <DownloadIcon />
                            "Download CSV"
                        </a>
                    </div>
                    <div class="export-card">
                        <h4>"Token Holders"</h4>
                        <p>"All USDFC holders with balances"</p>
                        <a
                            href=format!("{}/api/v2/tokens/{}/holders?csv=true", blockscout_url, usdfc_token)
                            target="_blank"
                            class="btn btn-primary"
                        >
                            <DownloadIcon />
                            "Download CSV"
                        </a>
                    </div>
                </div>
            </div>

            // JSON API Endpoints
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"JSON API Endpoints"</h3>
                        <p class="card-subtitle">"Programmatic access to USDFC data"</p>
                    </div>
                </div>
                <div class="api-endpoints">
                    <ApiEndpointRow
                        method="GET"
                        path=format!("/api/v2/tokens/{}/transfers", usdfc_token)
                        description="Token transfer history"
                        base_url=blockscout_url.to_string()
                    />
                    <ApiEndpointRow
                        method="GET"
                        path=format!("/api/v2/tokens/{}/holders", usdfc_token)
                        description="Current token holders"
                        base_url=blockscout_url.to_string()
                    />
                    <ApiEndpointRow
                        method="GET"
                        path=format!("/api/v2/tokens/{}/counters", usdfc_token)
                        description="Token statistics"
                        base_url=blockscout_url.to_string()
                    />
                    <ApiEndpointRow
                        method="GET"
                        path=format!("/api/v2/tokens/{}", usdfc_token)
                        description="Token metadata"
                        base_url=blockscout_url.to_string()
                    />
                </div>
            </div>

            // Subgraph (GraphQL)
            <div class="card" style="margin-bottom: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Subgraph (GraphQL)"</h3>
                        <p class="card-subtitle">"Secured Finance lending market data"</p>
                    </div>
                </div>
                <div style="padding: 16px;">
                    <div class="code-block">
                        <code>{subgraph_url.clone()}</code>
                    </div>
                    <p style="color: var(--text-muted); font-size: 12px; margin: 12px 0;">
                        "Use a GraphQL client or the playground to query lending markets, transactions, and daily volumes."
                    </p>
                    <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                        <a
                            href=subgraph_url.clone()
                            target="_blank"
                            class="btn btn-secondary"
                        >
                            <ExternalLinkIcon />
                            "Open GraphQL Playground"
                        </a>
                    </div>
                </div>
            </div>

            // Advanced Tools
            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Advanced Tools"</h3>
                        <p class="card-subtitle">"Additional resources for data analysis"</p>
                    </div>
                </div>
                <div style="padding: 16px;">
                    <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                        <a
                            href=format!("{}/advanced-filters?token_contract_address_hashes_to_include={}", blockscout_url, usdfc_token)
                            target="_blank"
                            class="btn btn-secondary"
                        >
                            "Advanced Filters"
                        </a>
                        <a
                            href=format!("{}/api-docs", blockscout_url)
                            target="_blank"
                            class="btn btn-secondary"
                        >
                            "API Documentation"
                        </a>
                        <a
                            href=format!("{}/token/{}", blockscout_url, usdfc_token)
                            target="_blank"
                            class="btn btn-secondary"
                        >
                            "View on Blockscout"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ApiEndpointRow(
    method: &'static str,
    path: String,
    description: &'static str,
    base_url: String,
) -> impl IntoView {
    let full_url = format!("{}{}", base_url, path);

    view! {
        <div class="api-endpoint-row">
            <div class="api-endpoint-info">
                <span class="method-badge">{method}</span>
                <code class="api-path">{path}</code>
                <span class="api-desc">{description}</span>
            </div>
            <a
                href=full_url
                target="_blank"
                class="btn btn-secondary btn-sm"
            >
                "Open"
            </a>
        </div>
    }
}
