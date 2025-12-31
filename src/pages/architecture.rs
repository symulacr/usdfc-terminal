use leptos::*;

#[component]
pub fn Architecture() -> impl IntoView {
    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Architecture"</h1>
                <p class="page-subtitle">"USDFC protocol architecture overview"</p>
            </div>

            <div class="grid-2" style="margin-bottom: 24px;">
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Core Components"</h3>
                    <ul style="color: var(--text-secondary); line-height: 2;">
                        <li>"Trove Manager - Collateralized debt positions"</li>
                        <li>"Stability Pool - Liquidation buffer"</li>
                        <li>"Price Feed - FIL/USD oracle"</li>
                        <li>"Sorted Troves - Ordered by ICR"</li>
                    </ul>
                </div>
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Key Parameters"</h3>
                    <ul style="color: var(--text-secondary); line-height: 2;">
                        <li>"Minimum Collateral Ratio: 110%"</li>
                        <li>"Critical Collateral Ratio: 150%"</li>
                        <li>"Liquidation Reserve: 200 USDFC"</li>
                        <li>"Borrowing Fee: 0.5%"</li>
                    </ul>
                </div>
            </div>

            <div class="card">
                <h3 style="color: var(--text-primary); margin-bottom: 16px;">"How It Works"</h3>
                <p style="color: var(--text-secondary); line-height: 1.8;">
                    "USDFC is a decentralized stablecoin on Filecoin, backed by FIL collateral. 
                    Users deposit FIL into Troves to mint USDFC. Each Trove must maintain a minimum 
                    collateral ratio of 110%. If a Trove's ratio falls below this threshold, it can 
                    be liquidated. The Stability Pool provides the first line of defense for liquidations, 
                    with depositors earning FIL rewards."
                </p>
            </div>
        </div>
    }
}
