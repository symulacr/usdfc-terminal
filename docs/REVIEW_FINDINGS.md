# USDFC Terminal Review Findings

Scope:
- API connections and config, SSR/hydrate data flow, request/response handling, and browser display.
- Focus on server <-> API <-> UI data integrity, fallbacks, and observability.

Assumptions (from user):
- Production target is SSR + hydrate.
- User inputs should accept EVM 0x addresses and Filecoin f4/f1 formats.
- Full hashes/addresses should be preserved in data model and only truncated at render time.

Findings (ordered by severity):

High:
- Default build is csr + mock, and non-SSR server fns return defaults without making API calls. This makes the browser look "live" but uses zeros/empty data in production unless SSR is enabled. (Cargo.toml:105-146, src/server_fn.rs:18-53, src/server_fn.rs:79-93, src/components/header.rs:9-66)
- Server fns swallow upstream errors with unwrap_or/unwrap_or_default, then render defaults as valid data. Outages/rate limits become silent data corruption. (src/server_fn.rs:24-35, src/server_fn.rs:84-86, src/server_fn.rs:117-130, src/server_fn.rs:240-248, src/server_fn.rs:283-291)
- Trove ABI decoding likely incorrect for dynamic array returns (offset ignored), and negative start index encoding is 64-bit, not 256-bit. This risks corrupted trove/ICR data. (src/rpc.rs:195-287)

Medium:
- Address input is not validated against 0x/f4/f1 formats before sending to Blockscout. Invalid input produces misleading "no activity." (src/pages/transactions.rs:41-56, src/server_fn.rs:277-305, src/error.rs:93-133)
- Blockscout endpoints use raw user input in URL paths without encoding. Malformed input can alter paths/queries. (src/blockscout.rs:224-265, src/blockscout.rs:433-476)
- Amount parsing assumes 18 decimals and ignores API-provided decimals. If token decimals differ, values are wrong. (src/blockscout.rs:37-41, src/blockscout.rs:97-103, src/blockscout.rs:287-291)
- Transaction hashes and addresses are truncated at fetch time, so data exports are lossy and not linkable. Truncation should be UI-only. (src/blockscout.rs:118-123, src/blockscout.rs:306-310, src/blockscout.rs:424-430)
- "Time ago" formatting treats epoch timestamps as a duration; displayed times are incorrect. (src/pages/dashboard.rs:180-188, src/pages/transactions.rs:164-173)
- Lending maturity dates are computed with 30-day month/365-day year approximation; displayed dates are wrong. (src/pages/lending.rs:156-173)
- search_transactions ignores filters and get_recent_transfers ignores limit, so UI controls are misleading. (src/server_fn.rs:96-100, src/blockscout.rs:62-65)
- Circulating supply is a hardcoded 94% estimate, not derived from on-chain or indexer data. (src/server_fn.rs:36-47)
- HTTP clients lack timeouts/retries; slow upstreams can hang requests. (src/rpc.rs:39-61, src/subgraph.rs:96-116, src/blockscout.rs:51-76, src/gecko.rs:25-45)

Low:
- get_holder_count does not check HTTP status before JSON parsing, leading to confusing errors on non-200 responses. (src/blockscout.rs:134-152)
- SSR static file handler unwraps file serve errors and can panic the server. (src/fileserv.rs:24-31)
- Docs describe mock data paths that do not exist in current code, which obscures real data flow. (docs/HARDCODED_DATA.md:1-80)

Notes:
- For SSR + hydrate, ensure server functions are enabled in production builds and that client hydration uses the server-provided initial state where applicable.
- For address validation, extend validation to accept 0x (EVM) and Filecoin f4/f1 forms before calling Blockscout or other upstreams.
- Preserve full hashes/addresses in data models; truncate only in UI rendering.
