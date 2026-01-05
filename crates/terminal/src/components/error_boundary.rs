//! Error boundary components for graceful error handling
//!
//! These components catch errors and display user-friendly fallback UI
//! instead of crashing the application.

use leptos::*;

/// A reusable error fallback component
#[component]
pub fn ErrorFallback(
    /// The error that occurred
    #[prop(into)]
    error: String,
    /// Optional retry callback
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="error-fallback">
            <div class="error-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" y1="8" x2="12" y2="12"></line>
                    <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
            </div>
            <h3 class="error-title">"Something went wrong"</h3>
            <p class="error-message">{error}</p>
            {on_retry.map(|retry| {
                view! {
                    <button
                        class="error-retry-btn"
                        on:click=move |_| retry.call(())
                    >
                        "Try Again"
                    </button>
                }
            })}
        </div>
    }
}

/// Compact error indicator for inline use
#[component]
pub fn InlineError(
    #[prop(into)]
    message: String,
) -> impl IntoView {
    view! {
        <span class="inline-error" title=message.clone()>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            " "{message}
        </span>
    }
}

/// Data loading error with retry capability
#[component]
pub fn DataLoadError<F>(
    /// Error message to display
    #[prop(into)]
    error: String,
    /// Callback to retry the data fetch
    on_retry: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <div class="data-load-error">
            <div class="error-content">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="error-icon-small">
                    <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                    <line x1="12" y1="9" x2="12" y2="13"></line>
                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                </svg>
                <span class="error-text">{error}</span>
            </div>
            <button class="retry-btn" on:click=move |_| on_retry()>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                    <polyline points="23 4 23 10 17 10"></polyline>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
                </svg>
                " Retry"
            </button>
        </div>
    }
}

/// Connection error indicator
#[component]
pub fn ConnectionError() -> impl IntoView {
    view! {
        <div class="connection-error">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="1" y1="1" x2="23" y2="23"></line>
                <path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55"></path>
                <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39"></path>
                <path d="M10.71 5.05A16 16 0 0 1 22.58 9"></path>
                <path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88"></path>
                <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
                <line x1="12" y1="20" x2="12.01" y2="20"></line>
            </svg>
            <span>"Connection lost. Attempting to reconnect..."</span>
        </div>
    }
}
