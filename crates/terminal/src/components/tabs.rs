//! Reusable Tab Component
//!
//! Provides tab navigation for combined pages.

use leptos::*;

/// Tab definition
#[derive(Clone, PartialEq)]
pub struct Tab {
    pub id: &'static str,
    pub label: &'static str,
}

/// Tab navigation component
#[component]
pub fn TabNav(
    tabs: Vec<Tab>,
    active: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="tab-nav">
            {tabs.into_iter().map(|tab| {
                let tab_id = tab.id.to_string();
                let tab_id_click = tab_id.clone();
                let label = tab.label;
                view! {
                    <button
                        class=move || if active.get() == tab_id { "tab-btn active" } else { "tab-btn" }
                        on:click=move |_| active.set(tab_id_click.clone())
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Tab content wrapper - only shows when active
#[component]
pub fn TabContent(
    id: &'static str,
    active: RwSignal<String>,
    children: Children,
) -> impl IntoView {
    let id_owned = id.to_string();
    view! {
        <div
            class=move || {
                let is_active = active.get() == id_owned;
                if is_active {
                    "tab-content tab-content-active"
                } else {
                    "tab-content tab-content-inactive"
                }
            }
        >
            {children()}
        </div>
    }
}
