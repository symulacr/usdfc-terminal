//! REST API module for USDFC Terminal
//!
//! Provides JSON API endpoints for external integrations.
//! These endpoints wrap the existing server functions to provide
//! a standard REST interface.

pub mod handlers;
pub mod models;

pub use handlers::*;
pub use models::*;
