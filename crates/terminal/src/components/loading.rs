//! Loading state components for consistent UI patterns
//!
//! These components provide consistent loading indicators across the application.
//! Includes skeleton loaders for visual loading placeholders.

use leptos::*;

/// Generic skeleton loader component
/// Use this for creating custom skeleton placeholders
#[component]
pub fn Skeleton(
    #[prop(default = "100%")]
    width: &'static str,
    #[prop(default = "1rem")]
    height: &'static str,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let class_name = if class.is_empty() {
        "skeleton".to_string()
    } else {
        format!("skeleton {}", class)
    };
    view! {
        <div
            class=class_name
            style=format!("width: {}; height: {};", width, height)
        />
    }
}

/// Skeleton for a full card/panel
#[component]
pub fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="card skeleton-card">
            <div class="skeleton" style="width: 40%; height: 1.25rem;"/>
            <div class="skeleton" style="width: 100%; height: 200px; margin-top: 1rem;"/>
        </div>
    }
}

/// Skeleton for activity/transaction list items
#[component]
pub fn ActivityItemSkeleton() -> impl IntoView {
    view! {
        <div class="activity-item skeleton-activity">
            <div class="skeleton" style="width: 60px; height: 1rem;"/>
            <div class="skeleton" style="width: 80px; height: 1rem;"/>
            <div class="skeleton" style="width: 40px; height: 0.875rem;"/>
        </div>
    }
}

/// Skeleton for metric rows (label + value pairs)
#[component]
pub fn MetricRowSkeleton() -> impl IntoView {
    view! {
        <div class="metric-row skeleton-metric-row">
            <div class="skeleton" style="width: 60%; height: 0.875rem;"/>
            <div class="skeleton" style="width: 30%; height: 1rem;"/>
        </div>
    }
}

/// Full-page loading spinner
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading-spinner-container">
            <div class="loading-spinner"></div>
        </div>
    }
}

/// Inline loading spinner for buttons and smaller elements
#[component]
pub fn InlineSpinner(
    #[prop(default = 16)]
    size: u32,
) -> impl IntoView {
    view! {
        <div class="inline-spinner" style=format!("width: {}px; height: {}px;", size, size)></div>
    }
}

/// Skeleton loader for metric cards
#[component]
pub fn MetricCardSkeleton() -> impl IntoView {
    view! {
        <div class="metric-card skeleton">
            <div class="skeleton-line skeleton-title"></div>
            <div class="skeleton-line skeleton-value"></div>
            <div class="skeleton-line skeleton-subtitle"></div>
        </div>
    }
}

/// Skeleton loader for table rows
#[component]
pub fn TableRowSkeleton(
    #[prop(default = 5)]
    columns: usize,
) -> impl IntoView {
    view! {
        <tr class="skeleton-row">
            {(0..columns).map(|_| {
                view! {
                    <td>
                        <div class="skeleton-line"></div>
                    </td>
                }
            }).collect_view()}
        </tr>
    }
}

/// Skeleton loader for data tables
#[component]
pub fn TableSkeleton(
    #[prop(default = 5)]
    rows: usize,
    #[prop(default = 4)]
    columns: usize,
) -> impl IntoView {
    view! {
        <div class="table-skeleton">
            {(0..rows).map(|_| {
                view! {
                    <TableRowSkeleton columns=columns />
                }
            }).collect_view()}
        </div>
    }
}

/// Skeleton loader for charts
#[component]
pub fn ChartSkeleton() -> impl IntoView {
    view! {
        <div class="chart-skeleton">
            <div class="chart-skeleton-bars">
                {(0..8).map(|i| {
                    let height = 30 + (i * 10) % 60;
                    view! {
                        <div
                            class="chart-skeleton-bar"
                            style=format!("height: {}%;", height)
                        ></div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Loading overlay for sections being refreshed
#[component]
pub fn LoadingOverlay(
    #[prop(into)]
    show: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="loading-overlay-container">
            {children()}
            <Show when=move || show.get()>
                <div class="loading-overlay">
                    <LoadingSpinner />
                </div>
            </Show>
        </div>
    }
}

/// Empty state for no data scenarios
#[component]
pub fn EmptyState(
    #[prop(into)]
    title: String,
    #[prop(into, optional)]
    description: Option<String>,
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M8 15h8M9 9h.01M15 9h.01"></path>
                </svg>
            </div>
            <h3 class="empty-state-title">{title}</h3>
            {description.map(|desc| view! {
                <p class="empty-state-description">{desc}</p>
            })}
            {children.map(|c| c())}
        </div>
    }
}

/// Pulse animation indicator for live data
#[component]
pub fn LiveIndicator() -> impl IntoView {
    view! {
        <span class="live-indicator" title="Live data">
            <span class="live-dot"></span>
            "LIVE"
        </span>
    }
}

/// Progress bar component
#[component]
pub fn ProgressBar(
    /// Current progress (0-100)
    #[prop(into)]
    progress: Signal<f64>,
    /// Optional label
    #[prop(into, optional)]
    label: Option<String>,
) -> impl IntoView {
    view! {
        <div class="progress-bar-container">
            {label.map(|l| view! { <span class="progress-label">{l}</span> })}
            <div class="progress-bar">
                <div
                    class="progress-fill"
                    style=move || format!("width: {}%", progress.get().clamp(0.0, 100.0))
                ></div>
            </div>
        </div>
    }
}
