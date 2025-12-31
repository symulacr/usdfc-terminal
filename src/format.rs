//! Formatting utilities for displaying values
//!
//! Centralized formatting to ensure consistency across the UI.
//! All pages should import from this module instead of defining local helpers.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

// ============================================================================
// Core Conversion
// ============================================================================

/// Convert Decimal to f64 safely (use this instead of to_string().parse())
#[inline]
pub fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

// ============================================================================
// Amount Formatting (Tokens - no currency symbol)
// ============================================================================

/// Format a Decimal token amount with thousand separators
/// Output: "1,234.56" (no symbol)
#[inline]
pub fn format_amount(amount: Decimal) -> String {
    format_readable(decimal_to_f64(amount), 2)
}

/// Format a Decimal as FIL amount
/// Output: "1,234.56 FIL"
#[inline]
pub fn format_fil(value: Decimal) -> String {
    format!("{} FIL", format_readable(decimal_to_f64(value), 2))
}

/// Format a Decimal as USDFC amount
/// Output: "1,234.56 USDFC"
#[inline]
pub fn format_usdfc(value: Decimal) -> String {
    format!("{} USDFC", format_readable(decimal_to_f64(value), 2))
}

// ============================================================================
// Currency Formatting (USD values - with $ symbol)
// ============================================================================

/// Format a Decimal value as USD currency
/// Output: "$1,234.56"
#[inline]
pub fn format_usd(value: Decimal) -> String {
    format!("${}", format_readable(decimal_to_f64(value), 2))
}

/// Format f64 as USD currency
/// Output: "$1,234.56"
#[inline]
pub fn format_usd_f64(value: f64) -> String {
    format!("${}", format_readable(value, 2))
}

/// Format a large USD value in compact form
/// Output: "$1.5M", "$2.3K", "$500"
#[inline]
pub fn format_usd_compact(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("${:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}

// ============================================================================
// Volume Formatting (consistent across all pages)
// ============================================================================

/// Format volume in compact form (no symbol)
/// Output: "1.5M", "2.3K", "500"
#[inline]
pub fn format_volume(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("{:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}

/// Format volume with USD prefix
/// Output: "$1.5M", "$2.3K", "$500"
#[inline]
pub fn format_volume_usd(value: f64) -> String {
    format_usd_compact(value)
}

// ============================================================================
// Percentage Formatting
// ============================================================================

/// Format a percentage value
/// Output: "12.34%"
#[inline]
pub fn format_percentage(value: f64) -> String {
    format!("{:.2}%", value)
}

/// Format a percentage with sign
/// Output: "+12.34%" or "-5.67%"
#[inline]
pub fn format_percentage_change(value: f64) -> String {
    if value >= 0.0 {
        format!("+{:.2}%", value)
    } else {
        format!("{:.2}%", value)
    }
}

/// Format TCR/ICR with 1 decimal
/// Output: "156.7%"
#[inline]
pub fn format_ratio(value: f64) -> String {
    format!("{:.1}%", value)
}

// ============================================================================
// Timestamp Formatting (consistent across all pages)
// ============================================================================

/// Format a unix timestamp as short relative time
/// Output: "2 mins ago", "3 hours ago", "5 days ago"
#[inline]
pub fn format_time_ago(timestamp: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{} mins ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

/// Format a unix timestamp as short datetime
/// Output: "Jan 15, 14:30"
#[inline]
pub fn format_timestamp(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(seconds as i64, 0)
        .map(|dt| dt.format("%b %d, %H:%M").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Format a unix timestamp as full datetime with UTC
/// Output: "2025-01-15 14:30:00 UTC"
#[inline]
pub fn format_timestamp_full(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(seconds as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Invalid time".to_string())
}

/// Format a unix timestamp as date only
/// Output: "Jan 15, 2025"
#[inline]
pub fn format_date(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(seconds as i64, 0)
        .map(|dt| dt.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "Invalid date".to_string())
}

/// Format a unix timestamp as hour:minute only
/// Output: "14:30"
#[inline]
pub fn format_time(seconds: u64) -> String {
    chrono::DateTime::from_timestamp(seconds as i64, 0)
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "--:--".to_string())
}

// ============================================================================
// Address/Hash Formatting (consistent 6+4 truncation)
// ============================================================================

/// Shorten a hash or address for display
/// Output: "0xAbCd...1234" (6 prefix + 4 suffix)
#[inline]
pub fn shorten_hash(value: &str) -> String {
    if value.len() > 13 {
        format!("{}...{}", &value[0..6], &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}

/// Shorten an address (alias for shorten_hash)
#[inline]
pub fn shorten_address(value: &str) -> String {
    shorten_hash(value)
}

// ============================================================================
// Number Formatting Helpers
// ============================================================================

/// Format number with thousand separators and specified decimals
/// Output: "1,234,567.89"
#[inline]
pub fn format_readable(value: f64, decimals: usize) -> String {
    let formatted = format!("{:.1$}", value, decimals);
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer_part = parts[0];

    let mut result = String::with_capacity(integer_part.len() + integer_part.len() / 3 + 4);
    let chars: Vec<char> = integer_part.chars().collect();
    let start = if chars.first() == Some(&'-') { 1 } else { 0 };

    if start == 1 {
        result.push('-');
    }

    for (i, c) in chars[start..].iter().enumerate() {
        if i > 0 && (chars.len() - start - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    if parts.len() > 1 && decimals > 0 {
        result.push('.');
        result.push_str(parts[1]);
    }

    result
}

/// Format large numbers in compact form (for charts/headers)
/// Output: "1.5B", "2.3M", "4.5K", "500"
#[inline]
pub fn format_compact(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("{:.1}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}

/// Format a number with thousand separators (integer)
#[inline]
pub fn format_number(value: f64) -> String {
    format_readable(value, 0)
}

/// Format a count (integer) with thousand separators
/// Output: "1,234", "56,789"
#[inline]
pub fn format_count(value: usize) -> String {
    format_readable(value as f64, 0)
}

/// Format a number with specified decimal places
#[inline]
pub fn format_number_decimals(value: f64, decimals: usize) -> String {
    format_readable(value, decimals)
}

// ============================================================================
// Legacy/Compatibility (kept for existing code)
// ============================================================================

/// Format a balance string with thousand separators
#[inline]
pub fn format_balance(balance: &str) -> String {
    let val: f64 = balance.parse().unwrap_or(0.0);
    format_readable(val, 2)
}

/// Format a Decimal value with $ prefix (legacy - prefer format_usd)
#[inline]
pub fn format_value(value: Decimal) -> String {
    format_usd(value)
}

/// Format a large number as currency with suffix (legacy)
#[inline]
pub fn format_currency(value: f64) -> String {
    format_usd_compact(value)
}

/// Truncate an address for display (legacy - use shorten_hash)
#[inline]
pub fn truncate_address(address: &str) -> String {
    shorten_hash(address)
}

/// Truncate a transaction hash for display (legacy - use shorten_hash)
#[inline]
pub fn truncate_tx_hash(hash: &str) -> String {
    shorten_hash(hash)
}

/// Format a timestamp as relative time (legacy - use format_time_ago)
#[inline]
pub fn format_relative_time(seconds_ago: u64) -> String {
    if seconds_ago < 60 {
        format!("{}s ago", seconds_ago)
    } else if seconds_ago < 3600 {
        format!("{} mins ago", seconds_ago / 60)
    } else if seconds_ago < 86400 {
        format!("{} hours ago", seconds_ago / 3600)
    } else {
        format!("{} days ago", seconds_ago / 86400)
    }
}

/// Format bytes as human-readable size
#[inline]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format short date (legacy - use format_date)
#[inline]
pub fn format_date_short(seconds: u64) -> String {
    format_date(seconds)
}
