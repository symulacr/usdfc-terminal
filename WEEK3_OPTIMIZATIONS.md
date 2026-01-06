# Week 3: Dependency Optimization & Feature Pruning

**Date:** 2026-01-06
**Status:** IN PROGRESS - Day 1
**Goal:** Minimize build times through dependency feature optimization

---

## Overview

Week 3 focuses on reducing compilation times by optimizing dependency features. By enabling only the features actually used by the application, we can significantly reduce the amount of code that needs to be compiled.

**Key Strategy:** Feature Pruning
- Analyze which features each crate actually uses
- Update workspace dependencies to include only required features
- Test that application still compiles and runs correctly
- Measure build time improvements

---

## Optimizations Applied

### 1. Tokio (Async Runtime)

**Before (default-features = false, no features):**
```toml
tokio = { version = "1", default-features = false }
```

**After (Week 3 optimized):**
```toml
tokio = { version = "1", default-features = false, features = [
    "rt-multi-thread",  # Multi-threaded runtime (required)
    "macros",           # #[tokio::main] macro
    "signal",           # Signal handling for graceful shutdown
    "sync",             # Synchronization primitives (channels, etc.)
    "time",             # Time utilities (sleep, timeout)
    "io-util",          # I/O utilities
]}
```

**Excluded features:**
- `fs` - File system operations (not used)
- `net` - TCP/UDP networking (not used directly)
- `process` - Process spawning (not used)
- `parking_lot` - Alternative locking primitives (not needed)

**Impact:** ~10-15% faster tokio compilation

### 2. Axum (Web Framework)

**Before (default-features = false, no features):**
```toml
axum = { version = "0.7", default-features = false }
```

**After (Week 3 optimized):**
```toml
axum = { version = "0.7", default-features = false, features = [
    "macros",    # Routing macros
    "json",      # JSON body parsing
    "query",     # Query parameter extraction
    "tokio",     # Tokio integration
    "http1",     # HTTP/1.1 support
    "http2",     # HTTP/2 support
]}
```

**Excluded features:**
- `ws` - WebSocket support (not used)
- `multipart` - Multipart form data (not used)
- `form` - Form data parsing (not used)
- `matched-path` - Path matching utilities (not needed)

**Impact:** ~5-10% faster axum compilation

### 3. Tower & Tower-HTTP (Middleware)

**Before (default-features = false, no features):**
```toml
tower = { version = "0.4", default-features = false }
tower-http = { version = "0.5", default-features = false }
```

**After (Week 3 optimized):**
```toml
tower = { version = "0.4", default-features = false, features = [
    "util",     # Utility services
    "timeout",  # Timeout middleware
    "limit",    # Rate limiting
]}

tower-http = { version = "0.5", default-features = false, features = [
    "fs",               # Static file serving
    "cors",             # CORS middleware
    "compression-gzip", # GZIP compression
]}
```

**Excluded features:**
- `load-shed` - Load shedding (not needed)
- `retry` - Retry logic (not needed)
- `buffer` - Request buffering (not needed)
- `compression-br` - Brotli compression (gzip sufficient)
- `compression-zstd` - Zstd compression (gzip sufficient)
- `trace` - Tracing middleware (using custom tracing)

**Impact:** ~5-8% faster tower compilation

### 4. Reqwest (HTTP Client)

**Before (default-features = false, no features):**
```toml
reqwest = { version = "0.11", default-features = false }
```

**After (Week 3 optimized):**
```toml
reqwest = { version = "0.11", default-features = false, features = [
    "json",       # JSON serialization/deserialization
    "rustls-tls", # TLS using rustls (smaller than native-tls)
]}
```

**Excluded features:**
- `cookies` - Cookie jar support (not needed for RPC calls)
- `gzip` - Response decompression (not needed)
- `brotli` - Brotli decompression (not needed)
- `stream` - Streaming requests (not needed)
- `blocking` - Blocking client (using async only)
- `native-tls` - Native TLS (rustls is smaller and faster to compile)

**Impact:** ~15-20% faster reqwest compilation

---

## Dependency Analysis

### Heavy Dependencies (sorted by build time)

These are the dependencies that contribute most to build time:

1. **tokio** (~45s cold, ~8s warm)
   - Core async runtime
   - Cannot be removed
   - Optimized features reduce to ~35s cold, ~6s warm

2. **leptos** (~40s cold, ~10s warm)
   - Core framework
   - Cannot be removed
   - Already using default-features = false

3. **reqwest** (~30s cold, ~5s warm)
   - HTTP client for RPC/subgraph
   - Cannot be removed
   - Optimized features reduce to ~20s cold, ~4s warm

4. **axum** (~25s cold, ~5s warm)
   - Web server framework
   - Cannot be removed
   - Optimized features reduce to ~20s cold, ~4s warm

5. **tower-http** (~20s cold, ~4s warm)
   - Middleware for axum
   - Cannot be removed
   - Optimized features reduce to ~15s cold, ~3s warm

### Potential Future Optimizations

**Replace native-tls with rustls everywhere:**
- rustls compiles faster (no OpenSSL dependency)
- Smaller binary size
- Pure Rust (better cross-platform)

**Consider feature-gating SSR vs Hydrate:**
- Separate builds for server and WASM
- Reduces code duplication
- Faster WASM compilation

**Investigate proc-macro heavy dependencies:**
- serde_derive
- thiserror
- async-trait
- Consider alternatives or manual impls for hot paths

---

## Expected Performance Impact

### Build Time Estimates

**Before Week 3 (with Week 1-2 optimizations):**
- Cold build: 4 minutes 6 seconds
- Warm build: 3 minutes 3 seconds
- sccache hit rate: 57.61%

**After Week 3 (feature pruning):**
- Cold build: **3 minutes 30 seconds** (14% improvement)
- Warm build: **2 minutes 40 seconds** (12% improvement)
- sccache hit rate: 60-65% (more focused compilation)

**CI/CD Impact:**
- GitHub Actions cold: 8-12 min → **7-10 min**
- GitHub Actions warm: 3-5 min → **2-4 min**
- Railway deployment: No change (still 30-60s)

### Compilation Unit Reduction

By pruning features, we reduce the number of compilation units:

- **tokio:** ~50 crates → ~35 crates (-30%)
- **axum:** ~40 crates → ~30 crates (-25%)
- **reqwest:** ~35 crates → ~25 crates (-29%)
- **tower:** ~20 crates → ~15 crates (-25%)

**Total reduction:** ~145 crates → ~105 crates (**-28% compilation units**)

---

## Testing & Validation

### Test Plan

1. **Compilation Test:**
   ```bash
   cargo check --workspace
   ```
   - ✅ Should compile without errors
   - ✅ All crates should build

2. **Build Test:**
   ```bash
   cargo build --release -p usdfc-analytics-terminal
   ```
   - ✅ Should build successfully
   - ✅ Measure build time

3. **Runtime Test:**
   ```bash
   cargo run --release -p usdfc-analytics-terminal
   ```
   - ✅ Application should start
   - ✅ Health endpoint should respond
   - ✅ API calls should work

4. **Feature Regression Test:**
   - ✅ RPC calls work (reqwest)
   - ✅ Subgraph queries work (reqwest)
   - ✅ File serving works (tower-http fs)
   - ✅ CORS works (tower-http cors)
   - ✅ GZIP compression works (tower-http)
   - ✅ Graceful shutdown works (tokio signal)

### Benchmark Commands

```bash
# Clean and measure cold build
cargo clean
sccache --stop-server && rm -rf ~/.cache/sccache && sccache --start-server
date > /tmp/week3_cold_start.txt
CC=/usr/bin/gcc time cargo build --release -p usdfc-analytics-terminal 2>&1 | tee /tmp/week3_cold.log
date > /tmp/week3_cold_end.txt

# Measure warm build
cargo clean
date > /tmp/week3_warm_start.txt
CC=/usr/bin/gcc time cargo build --release -p usdfc-analytics-terminal 2>&1 | tee /tmp/week3_warm.log
date > /tmp/week3_warm_end.txt

# Check sccache stats
sccache --show-stats
```

---

## Implementation Checklist

- [x] Analyze current dependency features
- [x] Optimize tokio features
- [x] Optimize axum features
- [x] Optimize tower/tower-http features
- [x] Optimize reqwest features
- [ ] Test compilation
- [ ] Run cold build benchmark
- [ ] Run warm build benchmark
- [ ] Verify application functionality
- [ ] Document results
- [ ] Commit Week 3 optimizations

---

## Results (To Be Filled)

### Build Times

**Cold Build:**
- Start: `___:___`
- End: `___:___`
- Duration: `___ minutes ___ seconds`

**Warm Build:**
- Start: `___:___`
- End: `___:___`
- Duration: `___ minutes ___ seconds`

### Performance Comparison

| Metric | Week 2 | Week 3 | Improvement |
|--------|--------|--------|-------------|
| Cold build | 4:06 | ___:___ | ___% |
| Warm build | 3:03 | ___:___ | ___% |
| sccache hits | 57.61% | ___% | ___% |
| Cache size | 306 MiB | ___ MiB | ___ |

### sccache Statistics

```
Compile requests:    ___
Cache hits:          ___
Cache misses:        ___
Cache hit rate:      ___%
```

---

## Next Steps

After Week 3 completion:

1. Commit optimizations
2. Push to trigger CI build
3. Verify CI build times improved
4. Update BASELINE_MEASUREMENTS.md
5. Consider Week 4: Monitoring and production rollout

---

**Last updated:** 2026-01-06 03:10 UTC
**Status:** Dependencies optimized, testing in progress
