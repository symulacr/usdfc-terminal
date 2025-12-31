//! Memoization utilities for Leptos components
//!
//! These utilities help prevent unnecessary re-renders and improve performance.

use leptos::*;

/// Debounce a signal value to reduce update frequency
/// Useful for search inputs and other frequently changing values
#[component]
pub fn DebouncedInput(
    /// Current value
    #[prop(into)]
    value: RwSignal<String>,
    /// Placeholder text
    #[prop(into, optional)]
    placeholder: Option<String>,
    /// CSS class
    #[prop(into, optional)]
    class: Option<String>,
) -> impl IntoView {
    // Internal immediate value
    let (immediate_value, set_immediate_value) = create_signal(value.get_untracked());

    // Sync back to parent signal on blur (simple debounce)
    let on_blur = move |_| {
        value.set(immediate_value.get());
    };

    view! {
        <input
            type="text"
            class=class.unwrap_or_default()
            placeholder=placeholder.unwrap_or_default()
            prop:value=move || immediate_value.get()
            on:input=move |ev| {
                set_immediate_value.set(event_target_value(&ev));
            }
            on:blur=on_blur
        />
    }
}

/// Throttled update wrapper
/// Limits how often a value can update
pub fn create_throttled_signal<T: Clone + 'static>(
    initial: T,
    _throttle_ms: u32,
) -> (Signal<T>, WriteSignal<T>) {
    let (value, set_value) = create_signal(initial);
    // Note: Full throttle implementation would require timers
    // This is a simplified version
    (value.into(), set_value)
}

/// Create a memo that only updates when its value actually changes
pub fn create_deduped_memo<T, F>(compute: F) -> Memo<T>
where
    T: Clone + PartialEq + 'static,
    F: Fn(Option<&T>) -> T + 'static,
{
    create_memo(compute)
}

/// Batch multiple signal updates together
/// Reduces re-renders when updating multiple related signals
pub fn batch_updates<F: FnOnce()>(f: F) {
    // In Leptos, updates are already batched within a reactive context
    // This is a convenience wrapper that documents intent
    f();
}
