# Hardcoded and Placeholder Data

This document lists remaining hardcoded/static data and placeholders in the current codebase.
It replaces older notes about mock feature blocks, which no longer exist in server functions.

---

## 1. Supply and Circulating Supply

- `src/server_fn.rs`: `circulating_supply` is estimated as 94% of total supply.
- `src/blockscout.rs`: `get_circulating_supply()` is still TODO (returns zero).

---

## 2. Static Pages and Inline Tables

- `src/pages/contracts.rs`: Contract list is hardcoded (used for display and explorer links).
- `src/pages/api_reference.rs`: API endpoints and contract addresses are hardcoded in the view.
- `src/components/footer.rs`: Status text like "RPC: Connected" is static.

---

## 3. Textual Content

Descriptive paragraphs across pages (dashboard, collateral, stability, lending) are static copy and do not reflect live configuration changes.

---

## 4. Build Defaults

- `Cargo.toml`: default features are `["csr", "mock"]`. CSR-only builds render defaults for server functions and will not fetch live data unless `ssr` is enabled.

---

## Notes

- Server functions in `src/server_fn.rs` now use real APIs under `#[cfg(feature = "ssr")]`.
- "Coming Soon" sections were replaced with interim functional views using existing live data sources.
