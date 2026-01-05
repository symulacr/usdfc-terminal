//! Error types for the USDFC Analytics Terminal
//! 
//! Provides typed errors instead of String for better error handling.

use thiserror::Error;
use std::time::Duration;

/// Main error type for API operations
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Failed to parse {field}: {value}")]
    Parse { field: &'static str, value: String },
    
    #[error("Resource not found: {resource} with id {id}")]
    NotFound { resource: &'static str, id: String },
    
    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimit { retry_after: Duration },
    
    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Timeout after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("WebSocket error: {message}")]
    WebSocket { message: String },
    
    #[error("Serialization error: {message}")]
    Serialization { message: String },
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    #[error("GraphQL error: {0}")]
    GraphQLError(String),
}

impl ApiError {
    /// Create a network error
    #[inline]
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network { message: message.into() }
    }
    
    /// Create a parse error
    #[inline]
    pub fn parse(field: &'static str, value: impl Into<String>) -> Self {
        Self::Parse { field, value: value.into() }
    }
    
    /// Create a not found error
    #[inline]
    pub fn not_found(resource: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound { resource, id: id.into() }
    }
    
    /// Create a rate limit error
    #[inline]
    pub fn rate_limit(retry_after: Duration) -> Self {
        Self::RateLimit { retry_after }
    }
    
    /// Check if error is retryable
    #[inline]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network { .. } | Self::RateLimit { .. } | Self::Timeout { .. })
    }
    
    /// Get suggested retry delay
    #[inline]
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::RateLimit { retry_after } => Some(*retry_after),
            Self::Network { .. } | Self::Timeout { .. } => Some(Duration::from_secs(1)),
            _ => None,
        }
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Validation errors for user input
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid transaction hash: {0}")]
    InvalidTxHash(String),
    
    #[error("Value out of range: {field} must be between {min} and {max}")]
    OutOfRange { field: &'static str, min: f64, max: f64 },
    
    #[error("Required field missing: {0}")]
    Required(&'static str),
}

impl ValidationError {
    /// Validate an Ethereum address
    #[inline]
    pub fn validate_address(address: &str) -> Result<(), Self> {
        let is_evm = address.len() == 42
            && address.starts_with("0x")
            && address[2..].chars().all(|c| c.is_ascii_hexdigit());
        let is_filecoin = (address.starts_with("f1") || address.starts_with("f4"))
            && address.len() > 2
            && address[2..].chars().all(|c| c.is_ascii_alphanumeric());
        if is_evm || is_filecoin {
            Ok(())
        } else if address.contains("...") {
            // Truncated address for display
            Ok(())
        } else {
            Err(Self::InvalidAddress(address.to_string()))
        }
    }
    
    /// Validate a transaction hash
    #[inline]
    pub fn validate_tx_hash(hash: &str) -> Result<(), Self> {
        if hash.len() == 66 && hash.starts_with("0x") {
            Ok(())
        } else if hash.contains("...") {
            // Truncated hash for display
            Ok(())
        } else {
            Err(Self::InvalidTxHash(hash.to_string()))
        }
    }
}
