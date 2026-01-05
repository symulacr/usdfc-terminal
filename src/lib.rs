//! USDFC Analytics Terminal
//! 
//! A real-time analytics dashboard for the USDFC stablecoin protocol.
//! Built with Leptos + Axum for full-stack Rust.
//!
//! ## Features
//! 
//! - **SSR**: Server-side rendering with Axum (3-5ms latency)
//! - **Hydration**: Progressive enhancement for interactivity
//! - **Server Functions**: Type-safe client-server communication
//! - **Real-time**: WebSocket support for live updates

pub mod app;
pub mod components;
pub mod pages;
pub mod types;
pub mod api;
pub mod error;
pub mod format;
pub mod data;
pub mod server_fn;
pub mod config;
pub mod address_conv;
pub mod global_metrics;

#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod blockscout;
#[cfg(feature = "ssr")]
pub mod subgraph;
#[cfg(feature = "ssr")]
pub mod gecko;
#[cfg(feature = "ssr")]
pub mod cache;
#[cfg(feature = "ssr")]
pub mod circuit_breaker;
#[cfg(feature = "ssr")]
pub mod historical;

// Re-export main app component for hydration
pub use app::App;

// WASM entry point for hydration
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    // In Leptos 0.6 with hydrate feature, mount_to_body automatically hydrates SSR content
    leptos::mount_to_body(App);
}

// WASM entry point for CSR (client-side rendering only)
#[cfg(all(feature = "csr", not(feature = "hydrate")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
