//! Chart Tooltip Component

use leptos::*;

/// Chart tooltip that shows OHLCV data on hover
#[component]
pub fn ChartTooltip(
    #[prop(into)] visible: MaybeSignal<bool>,
    #[prop(into)] x: MaybeSignal<f64>,
    #[prop(into)] y: MaybeSignal<f64>,
    #[prop(into)] open: MaybeSignal<f64>,
    #[prop(into)] high: MaybeSignal<f64>,
    #[prop(into)] low: MaybeSignal<f64>,
    #[prop(into)] close: MaybeSignal<f64>,
    #[prop(into)] volume: MaybeSignal<f64>,
) -> impl IntoView {
    view! {
        <Show when=move || visible.get()>
            <div
                class="chart-tooltip"
                style=move || format!(
                    "left: {}px; top: {}px;",
                    x.get() + 15.0,
                    y.get() - 80.0
                )
            >
                <div class="tooltip-body">
                    <div class="tooltip-row">
                        <span class="tooltip-label">"O:"</span>
                        <span class="tooltip-value">{move || format!("${:.4}", open.get())}</span>
                    </div>
                    <div class="tooltip-row">
                        <span class="tooltip-label">"H:"</span>
                        <span class="tooltip-value highlight-high">{move || format!("${:.4}", high.get())}</span>
                    </div>
                    <div class="tooltip-row">
                        <span class="tooltip-label">"L:"</span>
                        <span class="tooltip-value highlight-low">{move || format!("${:.4}", low.get())}</span>
                    </div>
                    <div class="tooltip-row">
                        <span class="tooltip-label">"C:"</span>
                        <span class="tooltip-value">{move || format!("${:.4}", close.get())}</span>
                    </div>
                    <div class="tooltip-row">
                        <span class="tooltip-label">"Vol:"</span>
                        <span class="tooltip-value">{move || format_volume(volume.get())}</span>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Format volume with appropriate suffix
fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("${:.2}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format!("${:.2}K", volume / 1_000.0)
    } else {
        format!("${:.2}", volume)
    }
}

/// Simple value tooltip for single metrics
#[component]
pub fn ValueTooltip(
    #[prop(into)] visible: MaybeSignal<bool>,
    #[prop(into)] x: MaybeSignal<f64>,
    #[prop(into)] y: MaybeSignal<f64>,
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into)] value: MaybeSignal<String>,
    #[prop(default = "#00d4ff")] color: &'static str,
) -> impl IntoView {
    // Store the signals so they can be accessed from multiple closures
    let label_stored = store_value(label);
    let value_stored = store_value(value);

    view! {
        <Show when=move || visible.get()>
            <div
                class="value-tooltip"
                style=move || format!(
                    "left: {}px; top: {}px; border-left: 3px solid {};",
                    x.get() + 15.0,
                    y.get() - 30.0,
                    color
                )
            >
                <span class="tooltip-label">{move || label_stored.with_value(|s| s.get())}</span>
                <span class="tooltip-value">{move || value_stored.with_value(|s| s.get())}</span>
            </div>
        </Show>
    }
}
