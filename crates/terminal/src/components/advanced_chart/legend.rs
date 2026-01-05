//! Chart Legend Component

use leptos::*;
use usdfc_core::types::ChartMetric;
use std::collections::HashSet;

/// Chart legend with toggleable series
#[component]
pub fn ChartLegend(
    visible_metrics: ReadSignal<HashSet<ChartMetric>>,
    #[prop(into)] on_toggle: Callback<ChartMetric>,
) -> impl IntoView {
    view! {
        <div class="chart-legend">
            <For
                each={move || ChartMetric::all().iter().copied().collect::<Vec<_>>()}
                key=|m| m.label()
                children=move |metric| {
                    let is_visible = move || visible_metrics.get().contains(&metric);
                    let color = metric.color();

                    view! {
                        <button
                            class="legend-item"
                            class:active=is_visible
                            on:click=move |_| on_toggle.call(metric)
                            style=move || if is_visible() {
                                format!("--legend-color: {};", color)
                            } else {
                                "--legend-color: var(--text-muted);".to_string()
                            }
                        >
                            <span
                                class="legend-indicator"
                                style=format!("background-color: {};", color)
                            ></span>
                            <span class="legend-label">{metric.label()}</span>
                        </button>
                    }
                }
            />
        </div>
    }
}

/// Compact legend for wallet charts
#[component]
pub fn CompactLegend(
    #[prop(into)] items: MaybeSignal<Vec<(String, String, bool)>>,
    #[prop(into)] on_toggle: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="compact-legend">
            <For
                each={move || items.get()}
                key=|(label, _, _)| label.clone()
                children=move |(label, color, active)| {
                    let label_clone = label.clone();
                    view! {
                        <button
                            class="legend-chip"
                            class:active=active
                            on:click=move |_| on_toggle.call(label_clone.clone())
                        >
                            <span
                                class="legend-dot"
                                style=format!("background-color: {};", color)
                            ></span>
                            {label.clone()}
                        </button>
                    }
                }
            />
        </div>
    }
}

/// Operation type legend for wallet analytics
#[component]
pub fn OperationLegend(
    #[prop(into)] breakdown: MaybeSignal<Vec<(String, usize, String)>>,
) -> impl IntoView {
    view! {
        <div class="operation-legend">
            <div class="operation-legend-title">"Operations"</div>
            <div class="operation-legend-items">
                <For
                    each={move || breakdown.get()}
                    key=|(label, _, _)| label.clone()
                    children=move |(label, count, color)| {
                        view! {
                            <div class="operation-item">
                                <span
                                    class="operation-dot"
                                    style=format!("background-color: {};", color)
                                ></span>
                                <span class="operation-label">{label}</span>
                                <span class="operation-count">{count}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
