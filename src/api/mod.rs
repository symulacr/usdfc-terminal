//! REST API module for USDFC Terminal
//!
//! Provides JSON API endpoints for external integrations.
//! These endpoints wrap the existing server functions to provide
//! a standard REST interface.

#[cfg(feature = "ssr")]
pub mod handlers;
pub mod models;

#[cfg(feature = "ssr")]
pub use handlers::*;
pub use models::*;
