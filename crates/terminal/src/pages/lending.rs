use leptos::*;
use crate::components::icons::*;
use crate::components::charts::BarChart;
use usdfc_backend::server_fn::{get_lending_markets, get_order_book, get_recent_lending_trades, get_daily_volumes};
use usdfc_core::format::{shorten_hash, format_date};
use std::collections::HashMap;

#[component]
pub fn LendingMarkets() -> impl IntoView {
    let markets = create_resource(
        || (),
        |_| async move { get_lending_markets().await }
    );

    let order_book = create_resource(
        || (),
        |_| async move { get_order_book(None).await }
    );

    let recent_trades = create_resource(
        || (),
        |_| async move { get_recent_lending_trades(Some(10)).await }
    );

    let daily_volumes = create_resource(
        || (),
        |_| async move { get_daily_volumes(Some(14)).await }
    );

    // Signals for order book expand/collapse
    let lend_orders_expanded = create_rw_signal(false);
    let borrow_orders_expanded = create_rw_signal(false);

    view! {
        <div class="fade-in">
            <div class="page-header">
                <h1 class="page-title">"Lending Markets"</h1>
                <p class="page-subtitle">"Secured Finance fixed-rate lending on Filecoin"</p>
            </div>

            // Best APR Summary Card
            <Suspense fallback=move || view! { <div></div> }>
                {move || {
                    markets.get().map(|res| {
                        match res {
                            Ok(markets) if !markets.is_empty() => {
                                let mut best_lend_apr: Option<f64> = None;
                                let mut best_borrow_apr: Option<f64> = None;

                                for market in &markets {
                                    if market.is_active {
                                        if market.lend_apr > 0.0 {
                                            best_lend_apr = Some(best_lend_apr.map_or(market.lend_apr, |v| v.max(market.lend_apr)));
                                        }
                                        if market.borrow_apr > 0.0 {
                                            best_borrow_apr = Some(best_borrow_apr.map_or(market.borrow_apr, |v| v.min(market.borrow_apr)));
                                        }
                                    }
                                }

                                view! {
                                    <div class="grid-2" style="margin-bottom: 24px;">
                                        <div class="card" style="background: var(--bg-secondary); border: 1px solid var(--border-color);">
                                            <div style="display: flex; align-items: center; justify-content: space-between;">
                                                <div>
                                                    <p style="color: var(--text-muted); font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px;">"Best Lend APR"</p>
                                                    <p style="color: var(--text-primary); font-size: 28px; font-weight: 600; font-family: monospace;">
                                                        {best_lend_apr.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string())}
                                                    </p>
                                                </div>
                                                <div style="color: var(--accent-green); width: 28px; height: 28px; stroke-width: 3;">
                                                    <TrendingUpIcon />
                                                </div>
                                            </div>
                                        </div>
                                        <div class="card" style="background: var(--bg-secondary); border: 1px solid var(--border-color);">
                                            <div style="display: flex; align-items: center; justify-content: space-between;">
                                                <div>
                                                    <p style="color: var(--text-muted); font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px;">"Best Borrow APR"</p>
                                                    <p style="color: var(--text-primary); font-size: 28px; font-weight: 600; font-family: monospace;">
                                                        {best_borrow_apr.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string())}
                                                    </p>
                                                </div>
                                                <div style="color: var(--accent-red); width: 28px; height: 28px; stroke-width: 3;">
                                                    <TrendingDownIcon />
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            _ => view! { <div></div> }.into_view()
                        }
                    })
                }}
            </Suspense>

            <div class="card">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"USDFC/FIL Market Pairs"</h3>
                        <p class="card-subtitle">"Grouped by maturity date"</p>
                    </div>
                    <button 
                        class="btn btn-secondary"
                        on:click=move |_| markets.refetch()
                    >
                        <RefreshIcon />
                        "Refresh"
                    </button>
                </div>
                <div class="table-container table-responsive">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Maturity"</th>
                                <th class="hide-mobile">"FIL Lend"</th>
                                <th class="hide-mobile">"FIL Borrow"</th>
                                <th>"USDFC Lend"</th>
                                <th>"USDFC Borrow"</th>
                                <th class="hide-mobile">"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! { 
                                <tr><td colspan="6" style="text-align: center; padding: 20px;">"Loading markets..."</td></tr> 
                            }>
                                {move || {
                                    markets.get().map(|res| {
                                        match res {
                                            Ok(markets) if !markets.is_empty() => {
                                                // Group markets by maturity
                                                let mut pairs: HashMap<String, MarketPair> = HashMap::new();
                                                
                                                for market in &markets {
                                                    let maturity_date = format_maturity(&market.maturity);
                                                    let currency = decode_currency(&market.currency);
                                                    let lend_apr = format_apr(market.lend_apr);
                                                    let borrow_apr = format_apr(market.borrow_apr);
                                                    
                                                    let entry = pairs.entry(maturity_date.clone()).or_insert(MarketPair {
                                                        maturity: maturity_date,
                                                        maturity_ts: market.maturity.parse().unwrap_or(0),
                                                        fil_lend: "—".to_string(),
                                                        fil_borrow: "—".to_string(),
                                                        usdfc_lend: "—".to_string(),
                                                        usdfc_borrow: "—".to_string(),
                                                        is_active: market.is_active,
                                                    });
                                                    
                                                    if currency == "FIL" {
                                                        entry.fil_lend = lend_apr;
                                                        entry.fil_borrow = borrow_apr;
                                                    } else if currency == "USDFC" {
                                                        entry.usdfc_lend = lend_apr;
                                                        entry.usdfc_borrow = borrow_apr;
                                                    }
                                                    
                                                    if market.is_active {
                                                        entry.is_active = true;
                                                    }
                                                }
                                                
                                                // Sort by maturity timestamp
                                                let mut sorted_pairs: Vec<_> = pairs.into_values().collect();
                                                sorted_pairs.sort_by_key(|p| p.maturity_ts);
                                                
                                                sorted_pairs.iter().map(|pair| {
                                                    let status = if pair.is_active { "Active" } else { "Matured" };
                                                    let status_class = if pair.is_active { "color: var(--accent-green);" } else { "color: var(--text-muted);" };
                                                    let fil_color = if pair.fil_lend == "—" { "color: var(--text-muted);" } else { "color: var(--accent-cyan);" };
                                                    let usdfc_color = if pair.usdfc_lend == "—" { "color: var(--text-muted);" } else { "color: var(--accent-green);" };
                                                    
                                                    view! {
                                                        <tr>
                                                            <td style="font-weight: 500;">{&pair.maturity}</td>
                                                            <td class="hide-mobile" style=fil_color>{&pair.fil_lend}</td>
                                                            <td class="hide-mobile" style=fil_color>{&pair.fil_borrow}</td>
                                                            <td style=usdfc_color>{&pair.usdfc_lend}</td>
                                                            <td style=usdfc_color>{&pair.usdfc_borrow}</td>
                                                            <td class="hide-mobile" style=status_class>{status}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()
                                            },
                                            Ok(_) => view! {
                                                <tr>
                                                    <td colspan="6" style="text-align: center; padding: 20px; color: var(--text-muted);">
                                                        "No markets found"
                                                    </td>
                                                </tr>
                                            }.into_view(),
                                            Err(err) => view! {
                                                <tr>
                                                    <td colspan="6" style="text-align: center; padding: 20px; color: var(--accent-red);">
                                                        {err.to_string()}
                                                    </td>
                                                </tr>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            // Order Book Section
            <div class="grid-2" style="margin-top: 24px;">
                <div class="card">
                    <div class="card-header">
                        <div>
                            <h3 class="card-title">"USDFC Lend Orders"</h3>
                            <p class="card-subtitle">"Open orders to lend USDFC"</p>
                        </div>
                        <button
                            class="btn btn-secondary"
                            style="font-size: 12px; padding: 6px 12px;"
                            on:click=move |_| lend_orders_expanded.update(|v| *v = !*v)
                        >
                            {move || if lend_orders_expanded.get() { "Collapse" } else { "Expand" }}
                        </button>
                    </div>
                    <div
                        class="table-container"
                        style=move || if lend_orders_expanded.get() {
                            "overflow-y: auto;".to_string()
                        } else {
                            "max-height: 300px; overflow-y: auto;".to_string()
                        }
                    >
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>"Price"</th>
                                    <th>"APR"</th>
                                    <th>"Amount"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <Suspense fallback=move || view! {
                                    <tr><td colspan="3" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                                }>
                                    {move || {
                                        let is_expanded = lend_orders_expanded.get();
                                        order_book.get().map(|res| {
                                            match res {
                                                Ok(book) => {
                                                    if book.lend_orders.is_empty() {
                                                        view! {
                                                            <tr><td colspan="3" style="text-align: center; padding: 20px; color: var(--text-muted);">"No open lend orders"</td></tr>
                                                        }.into_view()
                                                    } else {
                                                        let orders_to_show: Vec<_> = if is_expanded {
                                                            book.lend_orders.iter().collect()
                                                        } else {
                                                            book.lend_orders.iter().take(10).collect()
                                                        };
                                                        orders_to_show.iter().map(|order| {
                                                            view! {
                                                                <tr>
                                                                    <td style="color: var(--accent-green); font-family: monospace;">{format!("{:.2}", order.price)}</td>
                                                                    <td style="color: var(--text-primary);">{format!("{:.2}%", order.apr)}</td>
                                                                    <td style="font-family: monospace;">{format!("{:.2}", order.amount)}</td>
                                                                </tr>
                                                            }
                                                        }).collect_view()
                                                    }
                                                }
                                                Err(err) => view! {
                                                    <tr><td colspan="3" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
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
                            <h3 class="card-title">"USDFC Borrow Orders"</h3>
                            <p class="card-subtitle">"Open orders to borrow USDFC"</p>
                        </div>
                        <button
                            class="btn btn-secondary"
                            style="font-size: 12px; padding: 6px 12px;"
                            on:click=move |_| borrow_orders_expanded.update(|v| *v = !*v)
                        >
                            {move || if borrow_orders_expanded.get() { "Collapse" } else { "Expand" }}
                        </button>
                    </div>
                    <div
                        class="table-container"
                        style=move || if borrow_orders_expanded.get() {
                            "overflow-y: auto;".to_string()
                        } else {
                            "max-height: 300px; overflow-y: auto;".to_string()
                        }
                    >
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>"Price"</th>
                                    <th>"APR"</th>
                                    <th>"Amount"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <Suspense fallback=move || view! {
                                    <tr><td colspan="3" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                                }>
                                    {move || {
                                        let is_expanded = borrow_orders_expanded.get();
                                        order_book.get().map(|res| {
                                            match res {
                                                Ok(book) => {
                                                    if book.borrow_orders.is_empty() {
                                                        view! {
                                                            <tr><td colspan="3" style="text-align: center; padding: 20px; color: var(--text-muted);">"No open borrow orders"</td></tr>
                                                        }.into_view()
                                                    } else {
                                                        let orders_to_show: Vec<_> = if is_expanded {
                                                            book.borrow_orders.iter().collect()
                                                        } else {
                                                            book.borrow_orders.iter().take(10).collect()
                                                        };
                                                        orders_to_show.iter().map(|order| {
                                                            view! {
                                                                <tr>
                                                                    <td style="color: var(--accent-red); font-family: monospace;">{format!("{:.2}", order.price)}</td>
                                                                    <td style="color: var(--text-primary);">{format!("{:.2}%", order.apr)}</td>
                                                                    <td style="font-family: monospace;">{format!("{:.2}", order.amount)}</td>
                                                                </tr>
                                                            }
                                                        }).collect_view()
                                                    }
                                                }
                                                Err(err) => view! {
                                                    <tr><td colspan="3" style="text-align: center; padding: 20px; color: var(--accent-red);">{err.to_string()}</td></tr>
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

            // Recent Trades Section
            <div class="card" style="margin-top: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Recent Trades"</h3>
                        <p class="card-subtitle">"Latest lending market transactions"</p>
                    </div>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| recent_trades.refetch()
                    >
                        <RefreshIcon />
                        "Refresh"
                    </button>
                </div>
                <div class="table-container">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"ID"</th>
                                <th>"Side"</th>
                                <th>"Currency"</th>
                                <th>"Amount"</th>
                                <th>"APR"</th>
                                <th>"Time"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Suspense fallback=move || view! {
                                <tr><td colspan="6" style="text-align: center; padding: 20px;">"Loading..."</td></tr>
                            }>
                                {move || {
                                    recent_trades.get().map(|res| {
                                        match res {
                                            Ok(trades) => {
                                                if trades.is_empty() {
                                                    view! {
                                                        <tr><td colspan="6" style="text-align: center; padding: 20px; color: var(--text-muted);">"No recent trades"</td></tr>
                                                    }.into_view()
                                                } else {
                                                    trades.iter().map(|trade| {
                                                        let time = format_trade_timestamp(trade.timestamp);
                                                        let side_color = if trade.side == "Lend" { "color: var(--accent-green);" } else { "color: var(--accent-red);" };
                                                        view! {
                                                            <tr>
                                                                <td style="font-family: monospace; font-size: 11px;">{shorten_hash(&trade.id)}</td>
                                                                <td style=side_color>{&trade.side}</td>
                                                                <td>{&trade.currency}</td>
                                                                <td style="font-family: monospace;">{format!("{:.2}", trade.amount)}</td>
                                                                <td>{format!("{:.2}%", trade.apr)}</td>
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

            // Daily Volume Chart
            <div class="card" style="margin-top: 24px;">
                <div class="card-header">
                    <div>
                        <h3 class="card-title">"Daily Trading Volume"</h3>
                        <p class="card-subtitle">"14-day lending market volume"</p>
                    </div>
                </div>
                <Suspense fallback=move || view! {
                    <div style="height: 200px; display: flex; align-items: center; justify-content: center; color: var(--text-muted);">
                        "Loading volume data..."
                    </div>
                }>
                    {move || {
                        daily_volumes.get().map(|res| {
                            match res {
                                Ok(volumes) => {
                                    if volumes.is_empty() {
                                        view! {
                                            <div style="height: 200px; display: flex; align-items: center; justify-content: center; color: var(--text-muted);">
                                                "No volume data available"
                                            </div>
                                        }.into_view()
                                    } else {
                                        // Convert to chart data format
                                        let chart_data: Vec<(String, f64)> = volumes.iter()
                                            .rev()
                                            .take(14)
                                            .map(|v| (v.day.clone(), v.volume))
                                            .collect();
                                        view! {
                                            <BarChart data=chart_data color="#00d4ff" height=200 />
                                        }.into_view()
                                    }
                                }
                                Err(err) => view! {
                                    <div style="height: 200px; display: flex; align-items: center; justify-content: center; color: var(--accent-red);">
                                        {err.to_string()}
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            <div class="grid-2" style="margin-top: 24px;">
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"How It Works"</h3>
                    <p style="color: var(--text-secondary); line-height: 1.8;">
                        "Secured Finance provides fixed-rate lending markets on Filecoin. 
                        Each maturity date has two markets: FIL and USDFC. Lenders earn 
                        fixed yields, borrowers pay predictable rates."
                    </p>
                </div>
                <div class="card">
                    <h3 style="color: var(--text-primary); margin-bottom: 16px;">"Market Pairs"</h3>
                    <p style="color: var(--text-secondary); line-height: 1.8;">
                        "FIL and USDFC markets share the same maturity dates. You can 
                        lend FIL to earn yield or borrow USDFC against FIL collateral. 
                        APR shown is annualized based on current prices."
                    </p>
                </div>
            </div>
        </div>
    }
}

struct MarketPair {
    maturity: String,
    maturity_ts: i64,
    fil_lend: String,
    fil_borrow: String,
    usdfc_lend: String,
    usdfc_borrow: String,
    is_active: bool,
}

fn format_maturity(timestamp: &str) -> String {
    timestamp.parse::<u64>()
        .ok()
        .map(format_date)
        .unwrap_or_else(|| "Invalid date".to_string())
}

fn decode_currency(bytes32: &str) -> String {
    let hex = bytes32.trim_start_matches("0x");
    let mut result = String::new();
    
    let chars: Vec<char> = hex.chars().collect();
    for chunk in chars.chunks(2) {
        if chunk.len() == 2 {
            let byte_str: String = chunk.iter().collect();
            if let Ok(byte) = u8::from_str_radix(&byte_str, 16) {
                if byte > 0 && byte < 128 {
                    result.push(byte as char);
                }
            }
        }
    }
    
    let decoded = result.trim().to_string();
    if decoded.is_empty() { "Unknown".to_string() } else { decoded }
}

fn format_apr(apr: f64) -> String {
    if apr <= 0.0 {
        "N/A".to_string()
    } else {
        format!("{:.2}%", apr)
    }
}

fn format_trade_timestamp(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%b %d %H:%M").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
