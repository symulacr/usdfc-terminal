# API Integration Guide

This guide explains how the USDFC Analytics Terminal connects to real data sources in SSR + hydrate mode.

---

## Overview

The codebase is designed around SSR server functions:
- **Server functions** (in `src/server_fn.rs`) run only under `#[cfg(feature = "ssr")]`.
- **CSR-only builds** return defaults for server functions (zeros/empty lists).
- **Real API calls** are implemented directly in server functions and API clients.

---

## Step 1: Enable SSR + Hydrate for Production

Production builds should run SSR with hydration enabled so the browser receives server-fetched data:

```toml
[features]
default = ["ssr", "hydrate"]  # Recommended for production
ssr = ["leptos/ssr", ...]
hydrate = ["leptos/hydrate", ...]
http = ["reqwest"]
subgraph = ["graphql-client"]
```

---

If you prefer not to change defaults, build with:

```
cargo build --features ssr,hydrate
```

## Step 2: Choose Your Data Source

### Option A: HTTP REST API

Add `reqwest` to dependencies:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
```

Implement in `src/server_fn.rs`:

```rust
#[server(GetTotalSupply, "/api")]
pub async fn get_total_supply() -> Result<Decimal, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://your-api.com/v1/supply/total")
            .send()
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        let supply = Decimal::from_str(&data["total_supply"].to_string())
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(supply)
    }

    #[cfg(not(feature = "ssr"))]
    { Ok(Decimal::ZERO) }
}
```

---

## Address Conversion (f4 <-> 0x)

For Blockscout queries, normalize addresses on the server:
- `0x...` (EVM) is used directly.
- `f4...` is converted to `0x...` using the EAM namespace (32).
- `f1/f3` are not supported by Blockscout without an indexer or conversion.

The converter is implemented in `src/address_conv.rs` using `fvm_shared` and `hex`.
