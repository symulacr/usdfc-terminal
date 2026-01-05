// Main pages
pub mod dashboard;
pub mod protocol;
pub mod transactions;
pub mod lending;
pub mod entities;
pub mod analytics;
pub mod advanced;
pub mod infrastructure;
pub mod tools;

// Address submodule
pub mod address;
pub use address::AddressDetail;

// Legacy modules (kept for compatibility, will redirect)
pub mod supply;
pub mod collateral;
pub mod stability;
pub mod flow;
pub mod network;
pub mod sankey;
pub mod contracts;
pub mod architecture;
pub mod api_reference;
pub mod export;
pub mod alerts;
