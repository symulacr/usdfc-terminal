//! Chart Series Components - Different visualization types

use leptos::*;
use usdfc_core::types::TVCandle;

/// Area series component for smooth area charts
#[component]
pub fn AreaSeries(
    #[prop(into)] data: MaybeSignal<Vec<(f64, f64)>>,
    #[prop(default = "#00d4ff")] color: &'static str,
    #[prop(default = 0.3)] opacity: f64,
) -> impl IntoView {
    let data_for_path = data.clone();
    let path = create_memo(move |_| {
        let points = data_for_path.get();
        if points.is_empty() {
            return String::new();
        }

        let mut path = String::new();

        // Find Y baseline (max Y value = bottom of chart)
        let baseline = points.iter().map(|(_, y)| *y).fold(0.0f64, f64::max) + 50.0;

        // Start from baseline
        path.push_str(&format!("M{},{}", points[0].0, baseline));

        // Line up to first point
        path.push_str(&format!(" L{},{}", points[0].0, points[0].1));

        // Draw top line through all points
        for (x, y) in points.iter() {
            path.push_str(&format!(" L{:.2},{:.2}", x, y));
        }

        // Close back to baseline
        if let Some((last_x, _)) = points.last() {
            path.push_str(&format!(" L{},{}", last_x, baseline));
        }
        path.push_str(" Z");

        path
    });

    let line_path = create_memo(move |_| {
        let points = data.get();
        if points.is_empty() {
            return String::new();
        }

        let mut path = String::new();
        for (i, (x, y)) in points.iter().enumerate() {
            if i == 0 {
                path.push_str(&format!("M{:.2},{:.2}", x, y));
            } else {
                path.push_str(&format!(" L{:.2},{:.2}", x, y));
            }
        }
        path
    });

    let gradient_id = format!("area-gradient-{}", color.replace('#', ""));

    view! {
        <g class="area-series">
            <defs>
                <linearGradient id=gradient_id.clone() x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color=color stop-opacity=opacity.to_string()/>
                    <stop offset="100%" stop-color=color stop-opacity="0.05"/>
                </linearGradient>
            </defs>
            <path
                d=move || path.get()
                fill=format!("url(#{})", gradient_id)
            />
            <path
                d=move || line_path.get()
                fill="none"
                stroke=color
                stroke-width="2"
            />
        </g>
    }
}

/// Line series component
#[component]
pub fn LineSeries(
    #[prop(into)] data: MaybeSignal<Vec<(f64, f64)>>,
    #[prop(default = "#00d4ff")] color: &'static str,
    #[prop(default = 2.0)] stroke_width: f64,
    #[prop(default = false)] dashed: bool,
) -> impl IntoView {
    let path = create_memo(move |_| {
        let points = data.get();
        if points.is_empty() {
            return String::new();
        }

        let mut path = String::new();
        for (i, (x, y)) in points.iter().enumerate() {
            if i == 0 {
                path.push_str(&format!("M{:.2},{:.2}", x, y));
            } else {
                path.push_str(&format!(" L{:.2},{:.2}", x, y));
            }
        }
        path
    });

    let dash_array = if dashed { "5,5" } else { "" };

    view! {
        <path
            d=move || path.get()
            fill="none"
            stroke=color
            stroke-width=stroke_width.to_string()
            stroke-dasharray=dash_array
            class="line-series"
        />
    }
}

/// Candlestick series component
#[component]
pub fn CandlestickSeries(
    #[prop(into)] candles: MaybeSignal<Vec<TVCandle>>,
    #[prop(into)] x_scale: MaybeSignal<fn(usize, usize) -> f64>,
    #[prop(into)] y_scale: MaybeSignal<fn(f64) -> f64>,
    #[prop(default = "#22c55e")] bullish_color: &'static str,
    #[prop(default = "#ef4444")] bearish_color: &'static str,
) -> impl IntoView {
    let candles_for_each = candles.clone();
    let candles_for_children = candles.clone();
    let x_scale_clone = x_scale.clone();
    let y_scale_clone = y_scale.clone();

    view! {
        <g class="candlestick-series">
            <For
                each={move || {
                    let data = candles_for_each.get();
                    data.into_iter().enumerate().collect::<Vec<_>>()
                }}
                key=|(i, _)| *i
                children=move |(i, candle)| {
                    let total = candles_for_children.get().len();
                    let x_fn = x_scale_clone.get();
                    let y_fn = y_scale_clone.get();

                    let x = x_fn(i, total);
                    let candle_width = if total > 1 { 600.0 / total as f64 * 0.8 } else { 10.0 };

                    let is_bullish = candle.close >= candle.open;
                    let body_top = y_fn(if is_bullish { candle.close } else { candle.open });
                    let body_bottom = y_fn(if is_bullish { candle.open } else { candle.close });
                    let body_height = (body_bottom - body_top).max(1.0);

                    let wick_top = y_fn(candle.high);
                    let wick_bottom = y_fn(candle.low);

                    let color = if is_bullish { bullish_color } else { bearish_color };

                    view! {
                        <g class="candle">
                            <line
                                x1=x.to_string()
                                y1=wick_top.to_string()
                                x2=x.to_string()
                                y2=wick_bottom.to_string()
                                stroke=color
                                stroke-width="1"
                            />
                            <rect
                                x=(x - candle_width / 2.0).to_string()
                                y=body_top.to_string()
                                width=candle_width.to_string()
                                height=body_height.to_string()
                                fill=color
                            />
                        </g>
                    }
                }
            />
        </g>
    }
}

/// Bar series component for volume
#[component]
pub fn BarSeries(
    #[prop(into)] data: MaybeSignal<Vec<(f64, f64)>>,
    #[prop(default = "#8b5cf6")] color: &'static str,
    #[prop(default = 0.6)] opacity: f64,
    #[prop(default = 10.0)] bar_width: f64,
    #[prop(into)] baseline: MaybeSignal<f64>,
) -> impl IntoView {
    view! {
        <g class="bar-series" opacity=opacity.to_string()>
            <For
                each={move || data.get().into_iter().enumerate().collect::<Vec<_>>()}
                key=|(i, _)| *i
                children=move |(_, (x, y))| {
                    let base = baseline.get();
                    let height = (base - y).abs();
                    let top_y = y.min(base);

                    view! {
                        <rect
                            x=(x - bar_width / 2.0).to_string()
                            y=top_y.to_string()
                            width=bar_width.to_string()
                            height=height.to_string()
                            fill=color
                        />
                    }
                }
            />
        </g>
    }
}
