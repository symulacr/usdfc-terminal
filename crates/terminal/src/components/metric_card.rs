use leptos::*;
use crate::components::icons::*;

#[component]
pub fn MetricCard(
    label: &'static str,
    value: String,
    #[prop(optional)] change: Option<f64>,
    #[prop(default = "cyan")] color: &'static str,
    #[prop(optional)] icon: Option<View>,
    #[prop(optional)] sparkline: Option<Vec<f64>>,
    #[prop(default = false)] is_loading: bool,
    #[prop(optional)] tooltip: Option<&'static str>,
) -> impl IntoView {
    // Memoize sparkline calculation
    let sparkline_points = sparkline.as_ref().map(|data| {
        compute_sparkline_points(data)
    });

    if is_loading {
        return view! {
            <div class="metric-card">
                <div class="skeleton" style="height: 14px; width: 80px; margin-bottom: 8px;"></div>
                <div class="skeleton" style="height: 28px; width: 120px;"></div>
            </div>
        }.into_view();
    }

    view! {
        <div class="metric-card">
            <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                <div class="metric-label-row">
                    <span class="metric-label">{label}</span>
                    {tooltip.map(|tip| view! {
                        <span class="info-tooltip" data-tooltip=tip>
                            <InfoIcon />
                        </span>
                    })}
                </div>
                {icon.map(|i| view! { <div style="color: var(--text-muted); width: 20px; height: 20px;">{i}</div> })}
            </div>
            <div class=format!("metric-value {}", color)>{value}</div>
            {change.map(|c| {
                let (css_class, icon_view) = if c > 0.01 {
                    ("metric-change positive", view! { <ArrowUpIcon /> }.into_view())
                } else if c < -0.01 {
                    ("metric-change negative", view! { <ArrowDownIcon /> }.into_view())
                } else {
                    ("metric-change neutral", view! { <MinusIcon /> }.into_view())
                };
                view! {
                    <div class=css_class>
                        {icon_view}
                        {format!("{:.2}%", c.abs())}
                    </div>
                }
            })}
            {sparkline_points.map(|(points, _color_var)| {
                let color_var = format!("var(--accent-{})", color);
                view! {
                    <div style="margin-top: 12px; height: 32px;">
                        <svg width="100%" height="32" viewBox="0 0 100 100" preserveAspectRatio="none">
                            <polyline 
                                points={points} 
                                fill="none" 
                                stroke={color_var}
                                stroke-width="2"
                                stroke-linecap="round"
                            />
                        </svg>
                    </div>
                }
            })}
        </div>
    }.into_view()
}

// Phase 4: Extract computation to non-generic function
#[inline]
fn compute_sparkline_points(data: &[f64]) -> (String, String) {
    match data.len() {
        0 => (String::new(), String::new()),
        1 => ("0,50 100,50".to_string(), String::new()),
        _ => {
            let max_val = data.iter().fold(0.0f64, |a, &b| f64::max(a, b));
            let min_val = data.iter().fold(f64::MAX, |a, &b| f64::min(a, b));
            let range = if max_val - min_val > 0.0 { max_val - min_val } else { 1.0 };
            let step = 100.0 / (data.len() - 1) as f64;

            let points: String = data
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let x = i as f64 * step;
                    let y = 100.0 - ((v - min_val) / range * 80.0 + 10.0);
                    format!("{},{}", x, y)
                })
                .collect::<Vec<_>>()
                .join(" ");

            (points, String::new())
        }
    }
}
