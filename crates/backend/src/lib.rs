//! USDFC Backend - Server-side API services
//!
//! This crate contains all SSR-only functionality including:
//! - RPC clients (Filecoin, Blockscout, Subgraph, GeckoTerminal)
//! - Server functions for Leptos
//! - REST API handlers
//! - Caching and persistence

pub mod rpc;
pub mod blockscout;
pub mod subgraph;
pub mod gecko;
pub mod cache;
pub mod circuit_breaker;
pub mod historical;
pub mod server_fn;
pub mod fileserv;
pub mod state;
pub mod address_conv;
pub mod api;

// Re-export commonly used items
pub use server_fn::*;
pub use state::AppState;
