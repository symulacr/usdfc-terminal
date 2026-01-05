# USDFC Analytics Terminal - Build Optimization Plan
**Target: Reduce Railway build time from 12+ minutes to under 10 minutes**
**Focus: Backend/SSR only - preserve WASM client as-is**

---

## Executive Summary

**Current Issues:**
- Single monolithic crate compiles both SSR backend AND WASM client
- Heavy dependencies (ethers, axum, reqwest, leptos) all in one Cargo.toml
- Build timeout on Railway (~12 min needed, ~10 min limit)
- Dependencies recompile even when unchanged (poor cache utilization)

**Proposed Solution:**
Split into workspace with 3 crates:
- `usdfc-core` - Lightweight types, no heavy deps (100ms compile)
- `usdfc-backend` - SSR-only API servers (6-7 min compile)
- `usdfc-terminal` - Leptos frontend + workspace integration (thin)

**Expected Improvements:**
- Parallel compilation across crates
- Better CI caching (core deps cached separately)
- Faster incremental builds
- Railway build: **8-9 minutes** (under timeout)

---

## 1. Workspace Structure Refactor

### Current Structure
```
usdfc-analytics-terminal/
├── Cargo.toml (single crate - 203 lines)
├── src/
│   ├── lib.rs (SSR + hydrate + csr features)
│   ├── main.rs (SSR entry point)
│   ├── rpc.rs (432 lines)
│   ├── blockscout.rs (1283 lines)
│   ├── subgraph.rs (494 lines)
│   ├── gecko.rs (414 lines)
│   ├── server_fn.rs (1545 lines)
│   ├── historical.rs (398 lines)
│   ├── cache.rs (177 lines)
│   ├── types.rs (1246 lines)
│   ├── api/handlers.rs
│   ├── components/
│   └── pages/
└── target/ (single build tree)
```

### Proposed Workspace Structure
```
usdfc-analytics-terminal/
├── Cargo.toml (workspace manifest)
├── crates/
│   ├── core/           # Lightweight shared types
│   │   ├── Cargo.toml (minimal deps: serde, rust_decimal, chrono)
│   │   └── src/
│   │       ├── types.rs (ProtocolMetrics, Transaction, etc.)
│   │       ├── error.rs (ApiError, ValidationError)
│   │       ├── config.rs (Config struct, no dotenvy)
│   │       └── format.rs (display helpers)
│   │
│   ├── backend/        # SSR-only API clients
│   │   ├── Cargo.toml (heavy deps: reqwest, axum, rusqlite)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rpc.rs
│   │       ├── blockscout.rs
│   │       ├── subgraph.rs
│   │       ├── gecko.rs
│   │       ├── cache.rs
│   │       ├── circuit_breaker.rs
│   │       ├── historical.rs
│   │       ├── api/handlers.rs
│   │       ├── fileserv.rs
│   │       ├── state.rs
│   │       └── server_fn.rs (re-exports)
│   │
│   └── terminal/       # Leptos SSR + hydrate
│       ├── Cargo.toml (depends on core + backend)
│       └── src/
│           ├── lib.rs (app.rs + components + pages)
│           ├── main.rs (thin - just axum setup)
│           ├── components/
│           ├── pages/
│           └── server_fn.rs (server function definitions)
│
└── target/
    └── release/
        ├── usdfc-core/ (cached)
        ├── usdfc-backend/ (cached)
        └── usdfc-terminal/ (rebuilds on changes)
```

**Benefits:**
- ✅ Core types compile once, cached for all crates
- ✅ Backend API clients compile independently
- ✅ Changes to UI don't recompile backend
- ✅ Parallel compilation with `cargo build -p core -p backend`
- ✅ Railway caches `usdfc-core` and `usdfc-backend` deps separately

---

## 2. Dependency Optimization by Crate

### Crate 1: `usdfc-core` (Shared Types)
**Goal**: Zero proc-macros, minimal deps, fast compile (<100ms)

**Dependencies (5 total)**:
```toml
[dependencies]
serde = { version = "1", default-features = false }  # No derive
rust_decimal = { version = "1.39", default-features = false }  # No macros
chrono = { version = "0.4", default-features = false, features = ["alloc"] }
thiserror = "1.0"  # Lightweight error derive
once_cell = "1.21"  # For lazy statics
```

**No proc-macros** = faster builds, better caching

**Modules**:
- `types.rs` - Manual serde impl OR move derives to `backend` wrapper types
- `error.rs` - ApiError, ValidationError (thiserror only)
- `config.rs` - Config struct (values set by backend via constructor)
- `format.rs` - Display/formatting helpers (no external deps)

**Compile time**: ~80-100ms (types only, no codegen)

---

### Crate 2: `usdfc-backend` (API Services)
**Goal**: All heavy SSR deps, but optimized features

**Current Heavy Dependencies to Optimize**:

#### **Remove Completely**:
1. **`ethers = "2.0"`** ← Not used anywhere, remove entirely
   - Savings: ~45 crates, ~2-3 minutes compile time
   - Action: Delete from Cargo.toml

2. **`graphql_client = "0.14"`** ← Declared but unused
   - Current: GraphQL sent via `reqwest::Client` manually
   - Action: Remove dependency, keep manual queries
   - Savings: ~8 crates, ~30 seconds

#### **Optimize Features**:

3. **`reqwest = "0.11"`** - Used in 4 modules (rpc, blockscout, subgraph, gecko)
   - Current features: `["json"]`
   - Optimized features: `["json", "rustls-tls"]` (remove native-tls)
   - Disable: `default-features = false`
   - Savings: Removes OpenSSL dependency, ~20 seconds

4. **`axum = "0.7"`**
   - Current features: `["macros"]`
   - Keep: Essential for REST API
   - Optimize: Already minimal

5. **`rusqlite = "0.31"`**
   - Current features: `["bundled"]`
   - Keep: Needed for SQLite without system dependency
   - Optimize: Already optimal

6. **`leptos_axum = "0.6"`**
   - Current: Full SSR integration
   - Keep: Core SSR functionality
   - Note: Can't reduce without breaking SSR

**Optimized `backend/Cargo.toml`**:
```toml
[dependencies]
usdfc-core = { path = "../core" }

# HTTP client - optimized
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }

# Web framework
axum = { version = "0.7", features = ["macros"] }
tower-http = { version = "0.5", features = ["fs", "cors", "compression-gzip"] }

# Async runtime
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }

# Persistence
rusqlite = { version = "0.31", features = ["bundled"] }

# Rate limiting
governor = "0.6"

# Leptos SSR (backend only)
leptos_axum = "0.6"

# Serialization (with derives - allowed in backend)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utils
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Removed**:
- ❌ `ethers` (45 crates)
- ❌ `graphql_client` (8 crates)
- ❌ `fvm_shared` (not used)
- ❌ `inventory` (use explicit registration instead)

**Total dependency reduction**: ~50-55 crates removed

---

### Crate 3: `usdfc-terminal` (Leptos UI)
**Goal**: Thin wrapper, depends on core + backend

**Dependencies**:
```toml
[dependencies]
usdfc-core = { path = "../core" }
usdfc-backend = { path = "../backend", optional = true }

# Leptos framework
leptos = { version = "0.6", default-features = false }
leptos_meta = { version = "0.6", default-features = false }
leptos_router = { version = "0.6", default-features = false }

# WASM
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"

# Browser APIs (hydrate only)
web-sys = { version = "0.3", features = [...], optional = true }
js-sys = { version = "0.3", optional = true }
gloo-timers = { version = "0.3", optional = true }

[features]
ssr = ["dep:usdfc-backend", "leptos/ssr", ...]
hydrate = ["leptos/hydrate", "dep:web-sys", ...]
```

**Modules**:
- `lib.rs` - App component
- `components/` - UI components
- `pages/` - Page components
- `server_fn.rs` - Re-export from `usdfc-backend::server_fn`

---

## 3. Reduce Generic Monomorphization

### Problem
Heavy generic types cause code bloat and slow compilation:
- `Result<T, E>` with complex error types
- `Vec<ComplexStruct>` duplicated for each type
- Generic API clients duplicated per endpoint

### Solutions

#### 3.1. Type Aliases for Common Results
**Before**:
```rust
// Duplicated error handling for each function
pub async fn get_token_holders() -> Result<Vec<TokenHolder>, ApiError>
pub async fn get_pool_info() -> Result<PoolInfo, ApiError>
// ... 30+ functions with Result<T, ApiError>
```

**After** (in `usdfc-core`):
```rust
// Single ApiResult type reduces monomorphization
pub type ApiResult<T> = Result<T, ApiError>;

pub async fn get_token_holders() -> ApiResult<Vec<TokenHolder>>
pub async fn get_pool_info() -> ApiResult<PoolInfo>
```

**Savings**: ~10-15% reduction in Result<T,E> codegen

---

#### 3.2. Boxed Trait Objects at Boundaries
**Before**:
```rust
// Each HTTP client has different generic implementations
impl RpcClient { ... }    // reqwest::Client with specific middleware
impl BlockscoutClient { ... }  // reqwest::Client with different config
// ... each client monomorphizes reqwest differently
```

**After**:
```rust
// Shared HTTP client trait
pub trait HttpClient {
    async fn get(&self, url: &str) -> ApiResult<String>;
    async fn post(&self, url: &str, body: &str) -> ApiResult<String>;
}

// Single implementation
pub struct ReqwestClient(reqwest::Client);

// All API clients use Box<dyn HttpClient>
pub struct RpcClient {
    http: Box<dyn HttpClient>,
}
```

**Savings**: ~20-25% reduction in reqwest codegen duplication

---

#### 3.3. Move Serde Derives to Wrapper Types
**Before** (in types.rs - core module):
```rust
#[derive(Serialize, Deserialize)]  // Proc-macro slows build
pub struct ProtocolMetrics {
    pub tcr: f64,
    pub total_supply: Decimal,
    // ... 10+ fields
}
```

**After**:
**`usdfc-core/src/types.rs`** (no derives):
```rust
// Plain struct, no proc-macros
pub struct ProtocolMetrics {
    pub tcr: f64,
    pub total_supply: Decimal,
    // ...
}
```

**`usdfc-backend/src/dto.rs`** (with derives):
```rust
use usdfc_core::types::ProtocolMetrics;

#[derive(Serialize, Deserialize)]
pub struct ProtocolMetricsDto {
    pub tcr: f64,
    pub total_supply: String,  // Decimal -> String for JSON
    // ...
}

impl From<ProtocolMetrics> for ProtocolMetricsDto {
    fn from(m: ProtocolMetrics) -> Self { ... }
}
```

**Benefits**:
- Core types compile instantly (no proc-macros)
- Serde only in backend (doesn't slow core builds)
- JSON conversion isolated to API boundary

---

## 4. CI/Build Optimizations

### 4.1. Cargo Build Profile for CI
**New profile in workspace `Cargo.toml`**:
```toml
# Optimized for Railway CI builds (not local dev)
[profile.railway]
inherits = "release"
lto = "thin"           # Thin LTO instead of full (2-3x faster)
codegen-units = 4      # Parallel codegen (4-8 units optimal)
opt-level = 2          # O2 instead of Oz (faster build, similar size)
strip = true
panic = "abort"
debug = false          # ⭐ Disable debug info (saves 1-2 min)
incremental = false    # ⭐ Disable incremental (not useful in CI)
```

**Build command for Railway**:
```bash
cargo leptos build --profile railway --release
```

**Savings**:
- Debug info removal: **-1.5 to 2 minutes**
- Thin LTO: **-2 to 3 minutes** (vs full LTO)
- Total: **~4 minutes faster** than current `release` profile

---

### 4.2. Dockerfile Optimizations

**Current Dockerfile Issues**:
1. Installs `cargo-leptos` from source (~3 min compile)
2. No dependency pre-caching
3. Copies all files at once (breaks cache on any change)
4. Builds everything in single RUN command

**Optimized Dockerfile**:
```dockerfile
# =============================================================================
# Stage 1: Dependency Cache Layer
# =============================================================================
FROM rustlang/rust:nightly-slim AS deps

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

# Install WASM target
RUN rustup target add wasm32-unknown-unknown

# ⭐ Install cargo-leptos from binary (not source)
RUN curl -L https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.2/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xz -C /usr/local/cargo/bin

# ⭐ Copy only Cargo files first (better caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
COPY crates/terminal/Cargo.toml crates/terminal/Cargo.toml

# ⭐ Create dummy source files to cache dependencies
RUN mkdir -p crates/core/src crates/backend/src crates/terminal/src && \
    echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/backend/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/terminal/src/lib.rs

# ⭐ Build dependencies only (cached layer)
RUN cargo build --profile railway --features ssr -p usdfc-core -p usdfc-backend

# =============================================================================
# Stage 2: Application Build
# =============================================================================
FROM deps AS builder

# Copy real source code
COPY crates/core/src crates/core/src
COPY crates/backend/src crates/backend/src
COPY crates/terminal/src crates/terminal/src
COPY public ./public

# ⭐ Rebuild only application code (deps already cached)
RUN cargo leptos build --profile railway

# =============================================================================
# Stage 3: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash appuser

# Copy artifacts
COPY --from=builder /app/target/railway/usdfc-terminal /app/usdfc-terminal
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/public /app/public
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

RUN mkdir -p /app/data && chown -R appuser:appuser /app

USER appuser

EXPOSE ${PORT:-3000}

ENV LEPTOS_OUTPUT_NAME="usdfc-terminal"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_ENV="PROD"
ENV RUST_LOG="info"

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-3000}/health || exit 1

CMD ["/app/usdfc-terminal"]
```

**Key Optimizations**:
1. ⭐ Install `cargo-leptos` from **binary** (~10 seconds vs ~3 minutes)
2. ⭐ **Dependency pre-caching** layer (only rebuilds when Cargo.toml changes)
3. ⭐ Use `railway` profile (debug=false, incremental=false, thin LTO)
4. ⭐ Separate cache layers for Cargo files and source code

**Expected Savings**: **-4 to 5 minutes** total

---

### 4.3. Build Timings Profiling

**Add to `.cargo/config.toml`**:
```toml
[build]
# Enable build time profiling
timings = true

[term]
# Quiet mode for cleaner CI logs
quiet = false
verbose = false
```

**Generate build timing report**:
```bash
cargo build --profile railway --timings
# Outputs: target/cargo-timings/cargo-timing.html
```

**CI Integration** (in Dockerfile or Railway build command):
```bash
# Build with timings
cargo leptos build --profile railway --timings

# Upload timing report as artifact (optional)
cp target/cargo-timings/cargo-timing.html /tmp/build-timings.html || true
```

**Benefits**:
- Identifies slowest crates
- Visualizes parallel compilation
- Tracks build time regressions

---

## 5. Implementation Roadmap

### Phase 1: Quick Wins (30 min)
**Goal**: Get Railway build under 10 minutes without refactoring

1. ✅ Create `railway` build profile (Cargo.toml)
   - debug = false
   - incremental = false
   - lto = "thin"
   - codegen-units = 4

2. ✅ Remove unused dependencies
   - Delete `ethers` from Cargo.toml
   - Delete `graphql_client`
   - Delete `fvm_shared`

3. ✅ Optimize Dockerfile
   - Install cargo-leptos from binary
   - Add dependency caching layer
   - Use `--profile railway`

**Expected result**: **8-9 minute builds** (under Railway timeout)

---

### Phase 2: Workspace Split (2-3 hours)
**Goal**: Long-term build performance and maintainability

1. ✅ Create workspace structure
   - `crates/core/` - types, errors, config
   - `crates/backend/` - API clients, server functions
   - `crates/terminal/` - Leptos UI

2. ✅ Move modules to appropriate crates
   - Core: types.rs, error.rs, config.rs, format.rs
   - Backend: rpc.rs, blockscout.rs, subgraph.rs, gecko.rs, cache.rs, historical.rs
   - Terminal: app.rs, components/, pages/

3. ✅ Update imports across codebase
   - `use usdfc_core::types::*`
   - `use usdfc_backend::rpc::RpcClient`

4. ✅ Test workspace builds
   - `cargo build -p usdfc-core`
   - `cargo build -p usdfc-backend`
   - `cargo build -p usdfc-terminal`

**Expected result**: **6-7 minute builds** with better caching

---

### Phase 3: Monomorphization Reduction (1-2 hours)
**Goal**: Further reduce compile times

1. ✅ Add type aliases for common patterns
   - `type ApiResult<T> = Result<T, ApiError>`

2. ✅ Move serde derives to DTO layer
   - Plain structs in `core`
   - Serde derives in `backend/dto.rs`

3. ✅ Boxed traits at boundaries (optional)
   - `trait HttpClient`
   - Reduce reqwest duplication

**Expected result**: **5-6 minute builds**

---

## 6. Before/After Comparison

### Build Time Breakdown

**Before (Current)**:
```
Total: 12-13 minutes (TIMEOUT on Railway)
├── Dependencies compilation: ~8-9 min
│   ├── cargo-leptos install: ~3 min
│   ├── ethers ecosystem: ~2 min
│   ├── leptos + axum + tower: ~2 min
│   └── other deps: ~2 min
├── Application compilation: ~2-3 min
│   ├── Server binary: ~1.5 min
│   └── WASM client: ~1.5 min
└── Docker image creation: ~1 min
```

**After Phase 1 (Quick Wins)**:
```
Total: 8-9 minutes (UNDER Railway timeout ✅)
├── Dependencies compilation: ~5-6 min
│   ├── cargo-leptos binary: ~10 sec
│   ├── leptos + axum: ~2 min
│   ├── reqwest (optimized): ~1.5 min
│   └── other deps: ~1.5 min
├── Application compilation: ~2 min
│   ├── Server binary: ~1 min (railway profile)
│   └── WASM client: ~1 min
└── Docker image creation: ~1 min
```

**After Phase 2 (Workspace Split)**:
```
Total: 6-7 minutes (with cached core)
├── usdfc-core: ~80ms (cached)
├── usdfc-backend deps: ~3 min (cached)
├── usdfc-backend compile: ~1.5 min
├── usdfc-terminal compile: ~1.5 min
├── WASM client: ~1 min
└── Docker image: ~1 min
```

**After Phase 3 (Monomorphization)**:
```
Total: 5-6 minutes (optimized)
├── Workspace deps: ~2.5 min (cached)
├── Core + Backend: ~1 min
├── Terminal + WASM: ~1.5 min
└── Docker image: ~1 min
```

---

## 7. Risk Mitigation

### Potential Issues

1. **Leptos + Workspace Compatibility**
   - **Risk**: `cargo-leptos` expects single crate structure
   - **Mitigation**: Keep `terminal` as the build target, it depends on `backend` and `core`
   - **Testing**: Verify `cargo leptos serve` works with workspace

2. **Server Function Registration**
   - **Risk**: Moving server functions to separate crate might break registration
   - **Mitigation**: Use explicit registration in `main.rs` instead of `inventory` crate
   - **Testing**: Verify all endpoints work after split

3. **Cache Invalidation**
   - **Risk**: Docker layer caching might not work as expected
   - **Mitigation**: Test Dockerfile locally before Railway deploy
   - **Testing**: Build twice, verify second build uses cache

4. **Import Path Changes**
   - **Risk**: Moving modules breaks existing imports
   - **Mitigation**: Update all imports systematically with find+replace
   - **Testing**: `cargo check` passes for all features (ssr, hydrate, csr)

---

## 8. Success Metrics

### Build Performance
- ✅ Railway build completes in **<10 minutes** (vs 12+ min timeout)
- ✅ Dependency cache hit rate: **>80%** on repeat builds
- ✅ Core crate compile time: **<100ms**
- ✅ Full clean build: **<6 minutes** (after Phase 2)

### Code Quality
- ✅ Zero new compiler warnings
- ✅ All features compile (ssr, hydrate, csr)
- ✅ Tests pass (if any)
- ✅ Deployed app functionality unchanged

### Developer Experience
- ✅ Incremental builds **<30 seconds** for small changes
- ✅ Clear module boundaries
- ✅ Better IDE performance (fewer dependencies per crate)

---

## 9. Next Steps

**Immediate (User Approval Required)**:
1. Review this plan
2. Confirm Phase 1 (Quick Wins) approach
3. Get approval for dependency removal (ethers, graphql_client)

**Implementation Priority**:
1. **Phase 1 first** - Get Railway builds working NOW
2. **Phase 2 later** - Long-term architecture improvement
3. **Phase 3 optional** - If still need more performance

**Timeline Estimate**:
- Phase 1: 30 minutes
- Phase 2: 2-3 hours
- Phase 3: 1-2 hours (optional)

---

## 10. Alternative: Skip Phase 2/3, Just Fix Railway

**If workspace split is too complex**, we can achieve Railway success with just Phase 1:

**Minimal Changes** (15 minutes):
1. Remove `ethers` and `graphql_client` from Cargo.toml
2. Add `railway` profile to Cargo.toml
3. Update Dockerfile to:
   - Install cargo-leptos from binary
   - Use `--profile railway`

**Expected Result**: **~8.5 minutes** (under Railway timeout)

**Trade-off**: Won't get long-term benefits of workspace split, but solves immediate problem.

---

## Appendix A: Dependency Analysis

### Heavy Crates by Compile Time (estimated)

| Crate | Compile Time | Used By | Action |
|-------|--------------|---------|--------|
| `ethers` | ~2-3 min | NONE | ❌ **REMOVE** |
| `leptos` | ~1.5 min | Terminal | ✅ Keep (required) |
| `axum` | ~1 min | Backend | ✅ Keep (required) |
| `reqwest` | ~1 min | 4 modules | ✅ Optimize features |
| `tower-http` | ~45 sec | Axum middleware | ✅ Keep |
| `graphql_client` | ~30 sec | NONE | ❌ **REMOVE** |
| `rusqlite` | ~25 sec | historical.rs | ✅ Keep |
| `tokio` | ~20 sec | Async runtime | ✅ Keep |
| `serde` | ~15 sec | Everywhere | ✅ Keep |

**Total removable**: ~3-4 minutes of compile time

---

## Appendix B: Configuration Files

### Workspace Cargo.toml
```toml
[workspace]
members = [
    "crates/core",
    "crates/backend",
    "crates/terminal",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["USDFC Team"]
license = "MIT"

[workspace.dependencies]
serde = { version = "1", default-features = false }
serde_json = "1"
rust_decimal = { version = "1.39", default-features = false }
chrono = { version = "0.4", default-features = false }
thiserror = "1.0"
once_cell = "1.21"

# Workspace-wide profiles
[profile.railway]
inherits = "release"
lto = "thin"
codegen-units = 4
opt-level = 2
strip = true
panic = "abort"
debug = false
incremental = false
```

---

**End of Build Optimization Plan**
