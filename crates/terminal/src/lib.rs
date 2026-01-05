//! USDFC Analytics Terminal - Frontend Application
//!
//! Leptos-based analytics dashboard for the USDFC protocol

pub mod app;
pub mod components;
pub mod pages;
pub mod global_metrics;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
