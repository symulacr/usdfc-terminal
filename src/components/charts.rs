use leptos::*;

/// Compute chart points from data - extracted for reuse
#[inline]
fn compute_chart_points(data: &[(String, f64)]) -> (String, String, f64, f64) {
    if data.is_empty() {
        return (String::new(), String::new(), 0.0, 0.0);
    }

    let max_value = data.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
    let min_value = data.iter().map(|(_, v)| *v).fold(f64::MAX, f64::min);
    let range = if max_value - min_value > 0.0 { max_value - min_value } else { 1.0 };

    // For a single point, render it in the middle of the chart.
    let width = if data.len() > 1 {
        100.0 / data.len() as f64
    } else {
        100.0
    };

    let points: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, (_, v))| {
            let x = i as f64 * width + width / 2.0;
            let y = 100.0 - ((v - min_value) / range * 80.0 + 10.0);
            format!("{},{}", x, y)
        })
        .collect();

    let line_points = points.join(" ");
    let area_points = format!("0,100 {} 100,100", points.join(" "));

    (line_points, area_points, min_value, max_value)
}

#[component]
pub fn AreaChart(
    data: Vec<(String, f64)>,
    #[prop(default = "#00d4ff")] color: &'static str,
    #[prop(default = 200)] height: i32,
) -> impl IntoView {
    if data.is_empty() {
        return view! {
            <div class="chart-container" style=format!("height: {}px; display: flex; align-items: center; justify-content: center;", height)>
                <span style="color: var(--text-muted);">"No data available"</span>
            </div>
        }
        .into_view();
    }

    let (line_points, area_points, _, _) = compute_chart_points(&data);
    let gradient_id = format!("gradient-{}", color.replace("#", ""));

    view! {
        <div class="chart-container" style=format!("height: {}px", height)>
            <svg class="chart-svg" viewBox="0 0 100 100" preserveAspectRatio="none">
                <defs>
                    <linearGradient id={gradient_id.clone()} x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" stop-color={color} stop-opacity="0.3"/>
                        <stop offset="100%" stop-color={color} stop-opacity="0"/>
                    </linearGradient>
                </defs>
                <polygon
                    points={area_points}
                    fill={format!("url(#{})", gradient_id)}
                />
                <polyline
                    points={line_points}
                    fill="none"
                    stroke={color}
                    stroke-width="0.5"
                />
            </svg>
            <div style="display: flex; justify-content: space-between; padding: 8px 0; font-size: 10px; color: var(--text-muted);">
                {data.iter().map(|(label, _)| {
                    view! { <span>{label.clone()}</span> }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn BarChart(
    data: Vec<(String, f64)>,
    #[prop(default = "#00d4ff")] color: &'static str,
    #[prop(default = 200)] height: i32,
) -> impl IntoView {
    let max_value = data.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
    let min_value = data.iter().map(|(_, v)| *v).fold(f64::MAX, f64::min);
    let range = max_value - min_value;

    // Check if all values are zero (no volume data)
    let all_zero = max_value == 0.0;

    if all_zero {
        return view! {
            <div class="chart-container" style=format!("height: {}px; display: flex; align-items: center; justify-content: center;", height)>
                <span style="color: var(--text-muted);">"No volume data available"</span>
            </div>
        }.into_view();
    }

    view! {
        <div class="chart-container" style=format!("height: {}px", height)>
            <div style="display: flex; align-items: flex-end; height: 100%; gap: 4px; padding-bottom: 24px;">
                {data.iter().map(|(label, value)| {
                    // Calculate bar height with proper range scaling
                    let height_pct = if range > 0.0 {
                        // Scale based on min/max range, with 10% minimum to show small differences
                        let normalized = (value - min_value) / range;
                        // Map to 10%-100% range so all bars are visible but show relative differences
                        10.0 + normalized * 90.0
                    } else if max_value > 0.0 {
                        // All values are the same non-zero value, show half height
                        50.0
                    } else {
                        0.0
                    };
                    view! {
                        <div style="flex: 1; display: flex; flex-direction: column; align-items: center;">
                            <div
                                style=format!(
                                    "width: 100%; height: {}%; background: {}; border-radius: 4px 4px 0 0; min-height: 4px;",
                                    height_pct,
                                    color
                                )
                            ></div>
                            <span style="font-size: 10px; color: var(--text-muted); margin-top: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%;">
                                {label.clone()}
                            </span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }.into_view()
}

#[component]
pub fn DonutChart(
    data: Vec<(String, f64, &'static str)>,
    #[prop(default = 120)] size: i32,
) -> impl IntoView {
    let total: f64 = data.iter().map(|(_, v, _)| *v).sum();
    let radius = 40.0;
    let stroke_width = 12.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;

    // Avoid division by zero and show a friendly placeholder when there is no data.
    if total <= 0.0 {
        return view! {
            <div class="donut-container empty">
                <svg width={size} height={size} viewBox="0 0 100 100">
                    <circle
                        cx="50"
                        cy="50"
                        r={radius}
                        fill="none"
                        stroke="var(--border-color)"
                        stroke-width={stroke_width}
                        stroke-dasharray={circumference.to_string()}
                        stroke-dashoffset="0"
                    />
                    <text x="50" y="50" text-anchor="middle" dy="0.3em" fill="var(--text-muted)" font-size="12" font-weight="500">
                        "No data"
                    </text>
                </svg>
            </div>
        }
        .into_view();
    }

    let segments = compute_donut_segments(&data, total, circumference);

    view! {
        <div class="donut-container">
            <svg width={size} height={size} viewBox="0 0 100 100">
                {segments.into_iter().map(|(dash, offset, color)| {
                    view! {
                        <circle
                            cx="50"
                            cy="50"
                            r={radius}
                            fill="none"
                            stroke={color}
                            stroke-width={stroke_width}
                            stroke-dasharray={format!("{} {}", dash, circumference - dash)}
                            stroke-dashoffset={-offset}
                            transform="rotate(-90 50 50)"
                        />
                    }
                }).collect_view()}
                <text x="50" y="50" text-anchor="middle" dy="0.3em" fill="var(--text-primary)" font-size="14" font-weight="600">
                    {format!("{:.0}%", 100.0)}
                </text>
            </svg>
            <div class="donut-legend">
                {data.iter().map(|(label, value, color)| {
                    let pct = value / total * 100.0;
                    view! {
                        <div class="legend-item">
                            <div class="legend-color" style=format!("background: {}", color)></div>
                            <span class="legend-label">{label.clone()}</span>
                            <span class="legend-value">{format!("{:.1}%", pct)}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[inline]
fn compute_donut_segments(data: &[(String, f64, &str)], total: f64, circumference: f64) -> Vec<(f64, f64, String)> {
    if total <= 0.0 {
        return Vec::new();
    }

    let mut current_offset = 0.0;
    data.iter()
        .map(|(_, value, color)| {
            let pct = value / total;
            let dash = pct * circumference;
            let offset = current_offset;
            current_offset += dash;
            (dash, offset, color.to_string())
        })
        .collect()
}

#[component]
pub fn SparklineChart(
    data: Vec<f64>,
    #[prop(default = "#00d4ff")] color: &'static str,
    #[prop(default = 60)] width: i32,
    #[prop(default = 24)] height: i32,
) -> impl IntoView {
    let points = compute_sparkline_svg_points(&data);

    view! {
        <svg width={width} height={height} viewBox="0 0 100 100" preserveAspectRatio="none">
            <polyline
                points={points}
                fill="none"
                stroke={color}
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    }
}

#[inline]
fn compute_sparkline_svg_points(data: &[f64]) -> String {
    match data.len() {
        0 => String::new(),
        1 => "0,50 100,50".to_string(),
        _ => {
            let max_value = data.iter().fold(0.0f64, |a, &b| f64::max(a, b));
            let min_value = data.iter().fold(f64::MAX, |a, &b| f64::min(a, b));
            let range = if max_value - min_value > 0.0 {
                max_value - min_value
            } else {
                1.0
            };

            let step = 100.0 / (data.len() - 1) as f64;
            data.iter()
                .enumerate()
                .map(|(i, v)| {
                    let x = i as f64 * step;
                    let y = 100.0 - ((v - min_value) / range * 80.0 + 10.0);
                    format!("{},{}", x, y)
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}
