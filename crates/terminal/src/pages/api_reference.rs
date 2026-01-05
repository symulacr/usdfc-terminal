use leptos::*;

#[component]
pub fn ApiReference() -> impl IntoView {
    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"API Reference"</h1>
                <p class="page-subtitle">"USDFC data APIs and endpoints"</p>
            </div>

            <div class="card" style="margin-bottom: 24px;">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Data Sources"</h3>
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Source"</th>
                            <th>"Endpoint"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Filecoin RPC"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"api.node.glif.io"</td>
                            <td>"On-chain contract data"</td>
                        </tr>
                        <tr>
                            <td>"Blockscout"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"filecoin.blockscout.com/api"</td>
                            <td>"Token transfers and transactions"</td>
                        </tr>
                        <tr>
                            <td>"GeckoTerminal"</td>
                            <td style="font-family: monospace; color: var(--accent-cyan);">"api.geckoterminal.com"</td>
                            <td>"DEX prices and pools"</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Contract Addresses (Mainnet)"</h3>
                <pre style="background: var(--bg-primary); padding: 16px; border-radius: 8px; overflow-x: auto; font-size: 12px;">
{r#"USDFC Token:         0x80B98d3aa09ffff255c3ba4A241111Ff1262F045
Trove Manager:       0x5aB87c2398454125Dd424425e39c8909bBE16022
Stability Pool:      0x791Ad78bBc58324089D3E0A8689E7D045B9592b5
Price Feed:          0x80e651c9739C1ed15A267c11b85361780164A368
Active Pool:         0x8637Ac7FdBB4c763B72e26504aFb659df71c7803
Sorted Troves:       0x2C32e48e358d5b893C46906b69044D342d8DDd5F
Borrower Operations: 0x1dE3c2e21DD5AF7e5109D2502D0d570D57A1abb0
Multi Trove Getter:  0x5065b1F44fEF55Df7FD91275Fcc2D7567F8bf98F"#}
                </pre>
            </div>
        </div>
    }
}
