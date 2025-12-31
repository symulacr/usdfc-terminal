//! Application state for SSR
//! 
//! This module defines the shared state used by the Axum server.

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use leptos::LeptosOptions;

/// Application state shared across requests
#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    // Add additional state here:
    // pub db_pool: PgPool,
    // pub redis_client: redis::Client,
    // pub rpc_endpoint: String,
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}
