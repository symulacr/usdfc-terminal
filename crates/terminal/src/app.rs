use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::header::Header;
use crate::components::sidebar::Sidebar;
use crate::components::footer::Footer;
use crate::components::error_boundary::ErrorFallback;
use crate::global_metrics::GlobalMetrics;
use crate::pages::*;

/// 404 Not Found page component
#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="not-found-page">
            <div class="not-found-content">
                <h1 class="not-found-code">"404"</h1>
                <h2 class="not-found-title">"Page Not Found"</h2>
                <p class="not-found-message">
                    "The page you're looking for doesn't exist or has been moved."
                </p>
                <a href="/" class="not-found-link">"Return to Dashboard"</a>
            </div>
        </div>
    }
}

/// Loading spinner component for route transitions
#[component]
fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading-overlay">
            <div class="loading-spinner"></div>
            <span>"Loading..."</span>
        </div>
    }
}

/// Theme mode for the application
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Network connection status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NetworkStatus {
    #[default]
    Connected,
    Disconnected,
    Reconnecting,
}

/// Global app state provided via context
#[derive(Clone)]
pub struct AppState {
    /// Whether the sidebar is expanded or collapsed
    pub sidebar_expanded: RwSignal<bool>,
    /// Current theme mode (dark/light)
    pub theme_mode: RwSignal<ThemeMode>,
    /// Network connection status
    pub network_status: RwSignal<NetworkStatus>,
    /// Number of unread alerts
    pub unread_alerts: RwSignal<u32>,
    /// Mobile menu open state
    pub mobile_menu_open: RwSignal<bool>,
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        Self {
            sidebar_expanded: create_rw_signal(true),
            theme_mode: create_rw_signal(ThemeMode::Dark),
            network_status: create_rw_signal(NetworkStatus::Connected),
            unread_alerts: create_rw_signal(3), // Default from sidebar badge
            mobile_menu_open: create_rw_signal(false),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context for meta tags
    provide_meta_context();

    // Global app state with all reactive signals
    let app_state = AppState::new();
    let sidebar_expanded = app_state.sidebar_expanded;
    let theme_mode = app_state.theme_mode;
    let mobile_menu_open = app_state.mobile_menu_open;
    provide_context(app_state);

    // Global metrics context - shared across all pages
    let global_metrics = GlobalMetrics::new();
    provide_context(global_metrics);

    view! {
        <Stylesheet href="/pkg/usdfc-terminal.css"/>
        <Meta name="description" content="Real-time analytics terminal for the USDFC stablecoin protocol"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Title text="USDFC Analytics Terminal"/>
        <Script src="https://cdn.jsdelivr.net/npm/echarts@5.5.0/dist/echarts.min.js"/>

        <Router>
            <div
                class="app-container"
                class:sidebar-collapsed=move || !sidebar_expanded.get()
                class:theme-light=move || theme_mode.get() == ThemeMode::Light
            >
                <style>{include_str!("styles.css")}</style>
                <Header />
                <div class="main-wrapper">
                    // Mobile overlay - closes menu when clicked
                    <div
                        class="mobile-overlay"
                        class:active=move || mobile_menu_open.get()
                        on:click=move |_| mobile_menu_open.set(false)
                    ></div>
                    <Sidebar />
                    <main class="main-content">
                        <ErrorBoundary fallback=|errors| {
                            let error_msg = errors.get()
                                .into_iter()
                                .map(|(_, e)| e.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            view! {
                                <ErrorFallback error=error_msg />
                            }
                        }>
                            <Routes>
                                // Main routes
                                <Route path="/" view=dashboard::Dashboard />
                                <Route path="/dashboard" view=dashboard::Dashboard />
                                <Route path="/protocol" view=protocol::Protocol />
                                <Route path="/transactions" view=transactions::TransactionSearch />
                                <Route path="/address" view=transactions::TransactionSearch />
                                <Route path="/address/:addr" view=AddressDetail />
                                <Route path="/lending" view=lending::LendingMarkets />
                                <Route path="/entities" view=entities::EntityRegistry />
                                <Route path="/analytics" view=analytics::Analytics />
                                <Route path="/advanced" view=advanced::AdvancedAnalytics />
                                <Route path="/infrastructure" view=infrastructure::Infrastructure />
                                <Route path="/tools" view=tools::Tools />

                                // Redirects for old routes
                                <Route path="/supply" view=|| view! { <Redirect path="/protocol" /> } />
                                <Route path="/collateral" view=|| view! { <Redirect path="/protocol" /> } />
                                <Route path="/stability" view=|| view! { <Redirect path="/protocol" /> } />
                                <Route path="/flow" view=|| view! { <Redirect path="/analytics" /> } />
                                <Route path="/network" view=|| view! { <Redirect path="/analytics" /> } />
                                <Route path="/sankey" view=|| view! { <Redirect path="/analytics" /> } />
                                <Route path="/contracts" view=|| view! { <Redirect path="/infrastructure" /> } />
                                <Route path="/architecture" view=|| view! { <Redirect path="/infrastructure" /> } />
                                <Route path="/api" view=|| view! { <Redirect path="/tools" /> } />
                                <Route path="/export" view=|| view! { <Redirect path="/tools" /> } />
                                <Route path="/alerts" view=|| view! { <Redirect path="/tools" /> } />

                                // 404 fallback route - must be last
                                <Route path="/*any" view=NotFoundPage />
                            </Routes>
                        </ErrorBoundary>
                    </main>
                </div>
                <Footer />
            </div>
        </Router>
    }
}

#[component]
pub fn Shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"USDFC Analytics Terminal"</title>
                <link rel="stylesheet" href="/pkg/usdfc-terminal.css"/>
                <script src="https://cdn.jsdelivr.net/npm/echarts@5.5.0/dist/echarts.min.js"></script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
