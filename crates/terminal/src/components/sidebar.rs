use leptos::*;
use leptos_router::*;
use crate::app::AppState;

#[component]
pub fn Sidebar() -> impl IntoView {
    let app_state = use_context::<AppState>().expect("AppState must be provided");
    let sidebar_expanded = app_state.sidebar_expanded;
    let mobile_menu_open = app_state.mobile_menu_open;

    view! {
        <aside
            class="sidebar"
            class:collapsed=move || !sidebar_expanded.get()
            class:mobile-open=move || mobile_menu_open.get()
        >
            // Overview Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Overview"</div>
                <SidebarItem href="/dashboard" label="Dashboard">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="7" height="7"></rect>
                        <rect x="14" y="3" width="7" height="7"></rect>
                        <rect x="14" y="14" width="7" height="7"></rect>
                        <rect x="3" y="14" width="7" height="7"></rect>
                    </svg>
                </SidebarItem>
            </div>

            // Protocol Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Protocol"</div>
                <SidebarItem href="/protocol" label="Protocol">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
                    </svg>
                </SidebarItem>
            </div>

            // Markets Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Markets"</div>
                <SidebarItem href="/lending" label="Lending">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="12" y1="1" x2="12" y2="23"></line>
                        <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
                    </svg>
                </SidebarItem>
            </div>

            // Data Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Data"</div>
                <SidebarItem href="/transactions" label="Transactions">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8"></circle>
                        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    </svg>
                </SidebarItem>
                <SidebarItem href="/entities" label="Entities">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                        <line x1="3" y1="9" x2="21" y2="9"></line>
                        <line x1="9" y1="21" x2="9" y2="9"></line>
                    </svg>
                </SidebarItem>
            </div>

            // Analytics Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Analytics"</div>
                <SidebarItem href="/analytics" label="Analytics">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                        <polyline points="12 5 19 12 12 19"></polyline>
                    </svg>
                </SidebarItem>
                <SidebarItem href="/advanced" label="Advanced">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
                    </svg>
                </SidebarItem>
            </div>

            // Reference Section
            <div class="sidebar-section">
                <div class="sidebar-section-title">"Reference"</div>
                <SidebarItem href="/infrastructure" label="Infrastructure">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="16 18 22 12 16 6"></polyline>
                        <polyline points="8 6 2 12 8 18"></polyline>
                    </svg>
                </SidebarItem>
                <SidebarItem href="/tools" label="Tools">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                        <polyline points="14 2 14 8 20 8"></polyline>
                        <line x1="16" y1="13" x2="8" y2="13"></line>
                        <line x1="16" y1="17" x2="8" y2="17"></line>
                    </svg>
                </SidebarItem>
            </div>
        </aside>
    }
}

#[component]
fn SidebarItem(
    href: &'static str,
    label: &'static str,
    #[prop(optional)] badge: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    let is_active = move || {
        let current = pathname();
        if href == "/" || href == "/dashboard" {
            current == "/" || current == "/dashboard"
        } else {
            current == href || current.starts_with(&format!("{}/", href))
        }
    };

    view! {
        <A
            href=href
            class=move || if is_active() { "sidebar-item active" } else { "sidebar-item" }
        >
            {children()}
            <span>{label}</span>
            {badge.map(|b| view! { <span class="sidebar-badge">{b}</span> })}
        </A>
    }
}
