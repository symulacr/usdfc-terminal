//! USDFC Core - Shared types and utilities
//!
//! This crate contains lightweight, dependency-free types and utilities
//! shared across the USDFC Analytics Terminal backend and frontend.

pub mod types;
pub mod error;
pub mod config;
pub mod format;

// Re-export commonly used types
pub use types::*;
pub use error::*;
pub use config::*;
