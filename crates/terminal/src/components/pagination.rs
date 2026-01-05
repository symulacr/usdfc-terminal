use leptos::*;

/// Reusable pagination component for list views
///
/// # Arguments
/// * `current_page` - Signal containing the current page number (1-indexed)
/// * `total_pages` - Signal containing the total number of pages
/// * `on_page_change` - Callback called when a page change is requested
#[component]
pub fn Pagination(
    current_page: ReadSignal<u32>,
    total_pages: ReadSignal<u32>,
    on_page_change: Callback<u32>,
) -> impl IntoView {
    view! {
        <div class="pagination">
            <button
                class="pagination-btn"
                attr:disabled=move || current_page.get() <= 1
                on:click=move |_| on_page_change.call(current_page.get() - 1)
            >
                "Previous"
            </button>
            <span class="pagination-info">
                "Page " {move || current_page.get()} " of " {move || total_pages.get()}
            </span>
            <button
                class="pagination-btn"
                attr:disabled=move || current_page.get() >= total_pages.get()
                on:click=move |_| on_page_change.call(current_page.get() + 1)
            >
                "Next"
            </button>
        </div>
    }
}

/// Pagination with page size selector
#[component]
pub fn PaginationWithSize(
    current_page: ReadSignal<u32>,
    total_pages: ReadSignal<u32>,
    page_size: ReadSignal<u32>,
    total_items: ReadSignal<u32>,
    on_page_change: Callback<u32>,
    on_page_size_change: Callback<u32>,
) -> impl IntoView {
    // Pre-compute disabled states to avoid closure serialization issues in SSR
    let is_first_page = move || current_page.get() <= 1;
    let is_last_page = move || current_page.get() >= total_pages.get();

    // Pre-compute page values
    let prev_page = move || current_page.get().saturating_sub(1).max(1);
    let next_page = move || (current_page.get() + 1).min(total_pages.get());
    let last_page = move || total_pages.get();

    view! {
        <div class="pagination">
            <div class="pagination-size-selector">
                <label class="pagination-size-label">"Show:"</label>
                <select
                    class="pagination-size-select"
                    on:change=move |ev| {
                        let val = event_target_value(&ev).parse::<u32>().unwrap_or(20);
                        on_page_size_change.call(val);
                    }
                >
                    <option value="10" selected=move || page_size.get() == 10>"10"</option>
                    <option value="20" selected=move || page_size.get() == 20>"20"</option>
                    <option value="50" selected=move || page_size.get() == 50>"50"</option>
                    <option value="100" selected=move || page_size.get() == 100>"100"</option>
                </select>
            </div>

            <div class="pagination-controls">
                <button
                    class="pagination-btn"
                    disabled=is_first_page
                    on:click=move |_| on_page_change.call(1)
                    title="First page"
                >
                    "«"
                </button>
                <button
                    class="pagination-btn"
                    disabled=is_first_page
                    on:click=move |_| on_page_change.call(prev_page())
                >
                    "Previous"
                </button>
                <span class="pagination-info">
                    "Page " {move || current_page.get()} " of " {move || total_pages.get()}
                    " (" {move || total_items.get()} " items)"
                </span>
                <button
                    class="pagination-btn"
                    disabled=is_last_page
                    on:click=move |_| on_page_change.call(next_page())
                >
                    "Next"
                </button>
                <button
                    class="pagination-btn"
                    disabled=is_last_page
                    on:click=move |_| on_page_change.call(last_page())
                    title="Last page"
                >
                    "»"
                </button>
            </div>
        </div>
    }
}
