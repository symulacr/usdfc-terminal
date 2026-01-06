//! Shared UI Control Components
//!
//! Reusable components for time range selection, status indicators,
//! filters, and metric displays used across all pages.

use leptos::*;
use leptos::html::Div;

/// Time range options used across the application
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Hour1,
    Hour6,
    Hour24,
    Day7,
    Day30,
    Day90,
    All,
}

impl TimeRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hour1 => "1h",
            Self::Hour6 => "6h",
            Self::Hour24 => "24h",
            Self::Day7 => "7d",
            Self::Day30 => "30d",
            Self::Day90 => "90d",
            Self::All => "all",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Hour1 => "1H",
            Self::Hour6 => "6H",
            Self::Hour24 => "24H",
            Self::Day7 => "7D",
            Self::Day30 => "30D",
            Self::Day90 => "90D",
            Self::All => "ALL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "1h" => Self::Hour1,
            "6h" => Self::Hour6,
            "24h" => Self::Hour24,
            "7d" => Self::Day7,
            "30d" => Self::Day30,
            "90d" => Self::Day90,
            _ => Self::All,
        }
    }

    /// Returns seconds for this time range (for filtering)
    pub fn seconds(&self) -> Option<i64> {
        match self {
            Self::Hour1 => Some(3600),
            Self::Hour6 => Some(21600),
            Self::Hour24 => Some(86400),
            Self::Day7 => Some(604800),
            Self::Day30 => Some(2592000),
            Self::Day90 => Some(7776000),
            Self::All => None,
        }
    }

    /// Returns days for API calls
    pub fn days(&self) -> Option<i32> {
        match self {
            Self::Hour1 => Some(1),
            Self::Hour6 => Some(1),
            Self::Hour24 => Some(1),
            Self::Day7 => Some(7),
            Self::Day30 => Some(30),
            Self::Day90 => Some(90),
            Self::All => None,
        }
    }
}

/// Time Range Selector Component
///
/// A row of pill buttons for selecting time ranges.
/// Updates the provided signal when user clicks.
#[component]
pub fn TimeRangeSelector(
    /// The current selected time range
    selected: RwSignal<TimeRange>,
    /// Which options to show (default: all)
    #[prop(default = vec![TimeRange::Hour1, TimeRange::Hour6, TimeRange::Hour24, TimeRange::Day7, TimeRange::Day30, TimeRange::All])]
    options: Vec<TimeRange>,
    /// Compact mode (smaller buttons)
    #[prop(default = false)]
    compact: bool,
) -> impl IntoView {
    view! {
        <div class=if compact { "time-range-selector compact" } else { "time-range-selector" }>
            {options.into_iter().map(|range| {
                view! {
                    <button
                        class=move || if selected.get() == range { "time-btn active" } else { "time-btn" }
                        on:click=move |_| selected.set(range)
                    >
                        {range.label()}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Status indicator dot colors
#[derive(Clone, Copy, PartialEq)]
pub enum StatusLevel {
    Online,   // Green
    Warning,  // Yellow
    Offline,  // Red
    Unknown,  // Gray
}

impl StatusLevel {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Online => "status-dot online",
            Self::Warning => "status-dot warning",
            Self::Offline => "status-dot offline",
            Self::Unknown => "status-dot unknown",
        }
    }
}

/// Status Dots Component
///
/// Shows a row of status indicator dots for data sources.
#[component]
pub fn StatusDots(
    /// List of (name, status) pairs
    sources: Signal<Vec<(&'static str, StatusLevel)>>,
) -> impl IntoView {
    view! {
        <div class="status-dots">
            {move || {
                sources.get().into_iter().map(|(name, status)| {
                    view! {
                        <div class=status.class() title=name></div>
                    }
                }).collect_view()
            }}
        </div>
    }
}

/// Single metric item for MetricBar
#[derive(Clone)]
pub struct MetricItem {
    pub label: &'static str,
    pub value: String,
    pub change: Option<f64>,
}

/// Metric Bar Component
///
/// Shows metrics in a single horizontal line.
/// Overflow handled with "+N more" dropdown.
#[component]
pub fn MetricBar(
    /// Metrics to display
    metrics: Signal<Vec<MetricItem>>,
    /// Max visible before overflow
    #[prop(default = 5)]
    max_visible: usize,
) -> impl IntoView {
    let show_overflow = create_rw_signal(false);
    let _dropdown_ref = create_node_ref::<Div>();

    // Click-outside handler to close dropdown
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        create_effect(move |_| {
            if show_overflow.get() {
                // Add click listener to document when dropdown is open
                let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    if let Some(target) = event.target() {
                        if let Some(dropdown_el) = _dropdown_ref.get() {
                            let target_node: web_sys::Node = target.unchecked_into();
                            // Check if click is outside the dropdown container
                            if !dropdown_el.contains(Some(&target_node)) {
                                show_overflow.set(false);
                            }
                        }
                    }
                }) as Box<dyn Fn(_)>);

                let document = web_sys::window()
                    .expect("window")
                    .document()
                    .expect("document");

                let _ = document.add_event_listener_with_callback(
                    "click",
                    closure.as_ref().unchecked_ref(),
                );

                // Store closure to clean up on next effect run or component unmount
                on_cleanup({
                    let document = document.clone();
                    let closure_ref = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
                    move || {
                        let _ = document.remove_event_listener_with_callback("click", &closure_ref);
                    }
                });

                // Prevent closure from being dropped
                closure.forget();
            }
        });
    }

    view! {
        <div class="metric-bar">
            {move || {
                let items = metrics.get();
                let visible: Vec<_> = items.iter().take(max_visible).cloned().collect();
                let overflow: Vec<_> = items.iter().skip(max_visible).cloned().collect();
                let overflow_count = overflow.len();

                view! {
                    <>
                        {visible.into_iter().map(|item| {
                            view! {
                                <div class="metric-bar-item">
                                    <span class="metric-bar-label">{item.label}</span>
                                    <span class="metric-bar-value">{item.value.clone()}</span>
                                    {item.change.map(|c| {
                                        let class = if c >= 0.0 { "metric-bar-change up" } else { "metric-bar-change down" };
                                        view! {
                                            <span class=class>{format!("{:+.1}%", c)}</span>
                                        }
                                    })}
                                </div>
                            }
                        }).collect_view()}

                        {if overflow_count > 0 {
                            view! {
                                <div node_ref=dropdown_ref class="metric-bar-overflow">
                                    <button
                                        class="metric-bar-more"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            show_overflow.update(|v| *v = !*v);
                                        }
                                    >
                                        {format!("+{} more", overflow_count)}
                                    </button>
                                    <div class=move || if show_overflow.get() { "metric-bar-dropdown open" } else { "metric-bar-dropdown" }>
                                        {overflow.clone().into_iter().map(|item| {
                                            view! {
                                                <div class="metric-bar-dropdown-item">
                                                    <span class="metric-bar-label">{item.label}</span>
                                                    <span class="metric-bar-value">{item.value.clone()}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! { <></> }.into_view()
                        }}
                    </>
                }
            }}
        </div>
    }
}

/// Filter Panel Component
///
/// Collapsible filter panel with active count badge.
#[component]
pub fn FilterPanel(
    /// Whether panel is expanded
    expanded: RwSignal<bool>,
    /// Number of active filters
    active_count: Signal<usize>,
    /// Panel content
    children: Children,
) -> impl IntoView {
    view! {
        <div class="filter-panel-wrapper">
            <button
                class="filter-toggle"
                on:click=move |_| expanded.update(|v| *v = !*v)
            >
                <span class="filter-toggle-icon">{move || if expanded.get() { "▼" } else { "▶" }}</span>
                "Filters"
                {move || {
                    let count = active_count.get();
                    if count > 0 {
                        view! { <span class="filter-badge">{count.to_string()}</span> }.into_view()
                    } else {
                        view! { <></> }.into_view()
                    }
                }}
            </button>
            <div class=move || if expanded.get() { "filter-panel-content open" } else { "filter-panel-content" }>
                {children()}
            </div>
        </div>
    }
}

/// Chart Type Selector
#[component]
pub fn ChartTypeSelector(
    selected: RwSignal<String>,
    #[prop(default = vec!["area", "line", "bars"])]
    options: Vec<&'static str>,
) -> impl IntoView {
    view! {
        <div class="chart-type-selector">
            {options.into_iter().map(|ct| {
                let chart_type = ct;
                view! {
                    <button
                        class=move || if selected.get() == chart_type { "chart-type-btn active" } else { "chart-type-btn" }
                        on:click=move |_| selected.set(chart_type.to_string())
                        title=chart_type
                    >
                        {match chart_type {
                            "area" => view! {
                                <svg viewBox="0 0 20 20" fill="currentColor">
                                    <path d="M2 16L6 10L10 13L14 7L18 11V16H2Z" opacity="0.3"/>
                                    <path d="M2 16L6 10L10 13L14 7L18 11" fill="none" stroke="currentColor" stroke-width="1.5"/>
                                </svg>
                            },
                            "line" => view! {
                                <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <polyline points="2,14 6,8 10,11 14,5 18,9"/>
                                    <circle cx="6" cy="8" r="1.5" fill="currentColor"/>
                                    <circle cx="10" cy="11" r="1.5" fill="currentColor"/>
                                    <circle cx="14" cy="5" r="1.5" fill="currentColor"/>
                                </svg>
                            },
                            "bars" => view! {
                                <svg viewBox="0 0 20 20" fill="currentColor">
                                    <rect x="2" y="10" width="3" height="6"/>
                                    <rect x="7" y="6" width="3" height="10"/>
                                    <rect x="12" y="8" width="3" height="8"/>
                                    <rect x="17" y="4" width="3" height="12"/>
                                </svg>
                            },
                            _ => view! { <svg viewBox="0 0 20 20"></svg> }
                        }}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Inline Stat Component (for header)
#[component]
pub fn InlineStat(
    label: &'static str,
    value: Signal<String>,
    #[prop(optional)]
    change: Option<Signal<f64>>,
    #[prop(optional)]
    status: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="inline-stat">
            <span class="inline-stat-label">{label}</span>
            <span class=format!("inline-stat-value {}", status.unwrap_or(""))>{move || value.get()}</span>
            {change.map(|c| {
                let change_class = move || if c.get() >= 0.0 { "inline-stat-change up" } else { "inline-stat-change down" };
                view! {
                    <span class=change_class>
                        {move || format!("{:+.2}%", c.get())}
                    </span>
                }
            })}
        </div>
    }
}

/// Refresh Button
#[component]
pub fn RefreshButton(
    on_click: impl Fn() + 'static,
    #[prop(default = false)]
    loading: bool,
) -> impl IntoView {
    view! {
        <button
            class=if loading { "refresh-btn loading" } else { "refresh-btn" }
            on:click=move |_| on_click()
            disabled=loading
        >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M23 4v6h-6"/>
                <path d="M1 20v-6h6"/>
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
            </svg>
        </button>
    }
}
