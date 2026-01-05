use leptos::*;

#[component]
pub fn SmartContracts() -> impl IntoView {
    // Real USDFC contract addresses on Filecoin
    let contracts = vec![
        ("USDFC Token", "0xb3F018422D7A03d56B380a7574635F8BE71eE01D", "ERC20 stablecoin token"),
        ("Trove Manager", "0x5779E9d3387d9617Be6A44d191d51C8E346bB94b", "Manages collateralized debt positions"),
        ("Stability Pool", "0x63E3eE138E8540aD4b33AF67ed5b8A9F70C76fFd", "Liquidation buffer pool"),
        ("Price Feed", "0x01F94C1323178060a06B67354DCef2Ba1a7E7F8c", "FIL/USD oracle"),
        ("Borrower Operations", "0x0D9b9A70A16F42C01A84d65aF8828685e4c012aB", "Open/close troves"),
        ("Sorted Troves", "0x21bD276bbbDCB7ba8de3A291Db66006A5CE3e926", "Ordered trove list"),
    ];

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Smart Contracts"</h1>
                <p class="page-subtitle">"USDFC protocol contracts on Filecoin Mainnet"</p>
            </div>

            <div class="card">
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Contract"</th>
                            <th>"Address"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {contracts.into_iter().map(|(name, addr, desc)| {
                            let explorer_url = format!("https://filfox.info/en/address/{}", addr);
                            view! {
                                <tr>
                                    <td style="font-weight: 600;">{name}</td>
                                    <td>
                                        <a href=explorer_url target="_blank" style="color: var(--accent-cyan); text-decoration: none;">
                                            {format!("{}...{}", &addr[..8], &addr[addr.len()-6..])}
                                        </a>
                                    </td>
                                    <td style="color: var(--text-muted);">{desc}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
