# Clean Code Plan - USDFC Analytics Terminal

This document outlines code cleanup tasks. **DO NOT APPLY** - for reference only.

## 1. Consolidate GraphQL Clients

**Current State:** Two separate GraphQL implementations:
- `src/subgraph.rs` - Secured Finance subgraph
- `src/blockscout.rs` - Blockscout API (REST + GraphQL)

**Proposed Change:**
Create a shared `src/graphql.rs` module:
```rust
pub struct GraphQLClient {
    client: reqwest::Client,
    url: String,
}

impl GraphQLClient {
    pub async fn query<T: DeserializeOwned>(&self, query: &str) -> Result<T, ApiError>;
}
```

Then wrap in specialized clients:
```rust
pub struct SubgraphClient(GraphQLClient);
pub struct BlockscoutGqlClient(GraphQLClient);
```

**Files Affected:**
- `src/subgraph.rs` - Extract generic GraphQL logic
- `src/blockscout.rs` - Use shared client
- `src/lib.rs` - Add graphql module

---

## 2. Fix Cargo Build Warnings

**Current Warnings to Fix:**

1. **Unused imports** - Remove or cfg-gate:
   - Various `#[allow(unused)]` should be reviewed

2. **Dead code** - Items marked `#[allow(dead_code)]`:
   - Review if truly unused or needed for future

3. **Deprecated patterns**:
   - Check for any deprecated Leptos 0.6 patterns

**Commands to identify:**
```bash
cargo check --features ssr 2>&1 | grep -E "(warning|unused|dead_code)"
cargo clippy --features ssr -- -W clippy::all
```

---

## 3. Remove Duplicate Type Definitions

**Files to Review:**
- `src/types.rs` - Main type definitions
- `src/server_fn.rs` - Some types defined inline
- `src/blockscout.rs` - API response types

**Proposed:**
- Move all shared types to `src/types.rs`
- Keep API-specific response types in their modules
- Use type aliases where appropriate

---

## 4. Standardize Error Handling

**Current State:**
- Mix of `.map_err()` and `?` operator
- Some unwrap/expect still present
- Inconsistent error messages

**Proposed Pattern:**
```rust
// Use thiserror for all custom errors
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("API error: {0}")]
    Api(String),
    // ...
}

// Consistent error propagation
fn example() -> Result<T, TerminalError> {
    rpc_call().await.map_err(|e| TerminalError::Rpc(e.to_string()))?;
    Ok(result)
}
```

---

## 5. Extract Common UI Components

**Candidates for Extraction:**
1. **LoadingState** - Consistent loading indicator
2. **ErrorDisplay** - Consistent error display
3. **AddressLink** - Blockscout address link with formatting
4. **TxHashLink** - Blockscout tx link with formatting

**Example:**
```rust
#[component]
pub fn AddressLink(address: String, #[prop(default = true)] shorten: bool) -> impl IntoView {
    let display = if shorten { shorten_hash(&address) } else { address.clone() };
    view! {
        <a href=format!("https://filecoin.blockscout.com/address/{}", address)
           target="_blank"
           style="font-family: monospace; font-size: 12px;">
            {display}
        </a>
    }
}
```

---

## 6. CSS Cleanup

**Tasks:**
1. Remove unused CSS classes
2. Consolidate duplicate style rules
3. Use CSS custom properties consistently
4. Remove legacy color vars if truly unused

**Files:**
- `src/styles.css` - Main stylesheet

---

## 7. Configuration Cleanup

**Current:** All config in `src/config.rs` via env vars

**Improvements:**
1. Add validation for required env vars at startup
2. Add sensible defaults where appropriate
3. Document all env vars in `.env.example`

---

## 8. Test Coverage (Future)

**Areas to Test:**
1. Address format validation
2. Value formatting functions
3. Unit price to APR conversion
4. API response parsing

---

## Priority Order

1. **High:** Fix cargo warnings (blocks clean builds)
2. **Medium:** Consolidate GraphQL clients
3. **Medium:** Extract common UI components
4. **Low:** CSS cleanup
5. **Low:** Config improvements

---

## Notes

- Run `cargo fmt` before any cleanup
- Run `cargo clippy` to catch issues
- Test WASM build after any changes: `cargo leptos build --release`
