# Workspace Migration Status

## Completed (Phase 1 & Partial Phase 2)

### ✅ Phase 1: Quick Wins - DONE
1. Removed unused dependencies:
   - ethers (~45 crates)
   - graphql_client (~8 crates)
   - hex, inventory

2. Optimized reqwest:
   - Now uses rustls-tls instead of native-tls
   - Removed OpenSSL dependency

3. Created railway build profile:
   - debug = false (saves ~1.5min)
   - lto = "thin" (saves ~2-3min vs full LTO)
   - codegen-units = 4
   - incremental = false

4. Optimized Dockerfile:
   - Install cargo-leptos from binary (~10s vs ~3min)
   - Added dependency pre-caching layer
   - Uses railway profile

**Savings: ~7-8 minutes compile time**

### 🔄 Phase 2: Workspace Split - IN PROGRESS

#### Completed:
1. ✅ Created workspace structure:
   ```
   crates/
   ├── core/      (types, error, config, format)
   ├── backend/   (rpc, blockscout, subgraph, api, etc.)
   └── terminal/  (app, components, pages)
   ```

2. ✅ Created Cargo.toml for workspace root
3. ✅ Created Cargo.toml for each crate
4. ✅ Copied modules to appropriate crates
5. ✅ Updated most imports (sed replacements done)

#### Remaining Work:

1. **Fix remaining import issues:**
   ```bash
   # Need manual fixes in:
   - crates/backend/src/*.rs (check for remaining `use crate::` that should be `use usdfc_core::`)
   - crates/terminal/src/main.rs (verify all imports correct)
   - crates/terminal/src/app.rs (update imports)
   - crates/terminal/src/global_metrics.rs
   ```

2. **Update Dockerfile for workspace:**
   ```dockerfile
   # Change line 31:
   COPY Cargo.toml Cargo.lock ./
   # To:
   COPY Cargo.toml Cargo.lock ./
   COPY crates/core/Cargo.toml crates/core/Cargo.toml
   COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
   COPY crates/terminal/Cargo.toml crates/terminal/Cargo.toml

   # Change line 34-36:
   RUN mkdir -p src && \
       echo "fn main() {}" > src/main.rs && \
       echo "pub fn dummy() {}" > src/lib.rs
   # To:
   RUN mkdir -p crates/core/src crates/backend/src crates/terminal/src && \
       echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
       echo "pub fn dummy() {}" > crates/backend/src/lib.rs && \
       echo "pub fn dummy() {}" > crates/terminal/src/lib.rs

   # Change line 40:
   RUN cargo build --profile railway --features ssr --lib
   # To:
   RUN cargo build --profile railway -p usdfc-core -p usdfc-backend

   # Change line 52-53:
   COPY src ./src
   # To:
   COPY crates/core/src crates/core/src
   COPY crates/backend/src crates/backend/src
   COPY crates/terminal/src crates/terminal/src

   # Change line 57:
   RUN cargo leptos build --profile railway
   # To:
   RUN cargo leptos build --profile railway -p usdfc-analytics-terminal

   # Update binary path line 75:
   COPY --from=builder /app/target/railway/usdfc-analytics-terminal
   # Keep as is (workspace still outputs to same location)
   ```

3. **Test workspace build:**
   ```bash
   cargo build --profile railway -p usdfc-core
   cargo build --profile railway -p usdfc-backend
   cargo build --profile railway -p usdfc-analytics-terminal
   ```

4. **Fix any compilation errors** that arise from import changes

5. **Update .gitignore** if needed for workspace structure

6. **Verify cargo-leptos works** with workspace layout

## TODO: Phase 3 - Monomorphization (After Phase 2 works)

1. Add type aliases in usdfc-core/src/error.rs:
   ```rust
   pub type ApiResult<T> = Result<T, ApiError>;
   ```

2. Update all `Result<T, ApiError>` to `ApiResult<T>` in backend

3. Move serde derives to DTO layer:
   - Remove `#[derive(Serialize, Deserialize)]` from core types
   - Create DTOs in `usdfc-backend/src/dto.rs`
   - Implement `From` conversions

4. Optional: Box trait objects at HTTP client boundaries

## Next Steps

1. **Immediate:** Fix Dockerfile for workspace
2. **Then:** Test workspace build locally
3. **Then:** Fix any compilation errors
4. **Then:** Deploy to Railway
5. **Then:** Phase 3 optimizations

## Expected Final Results

- Phase 1 alone: **8-9 min builds** (under Railway timeout ✅)
- Phase 2 complete: **6-7 min builds** (with workspace caching)
- Phase 3 complete: **5-6 min builds** (optimized monomorphization)

## Files Modified

```
Cargo.toml (replaced with workspace manifest)
Cargo.toml.original (backup)
Dockerfile (needs workspace updates)
crates/core/Cargo.toml (new)
crates/core/src/lib.rs (new)
crates/core/src/{types,error,config,format}.rs (copied)
crates/backend/Cargo.toml (new)
crates/backend/src/lib.rs (new)
crates/backend/src/*.rs (copied, imports updated)
crates/terminal/Cargo.toml (new)
crates/terminal/src/lib.rs (new)
crates/terminal/src/main.rs (copied, imports partially updated)
crates/terminal/src/*.rs (copied, imports updated via sed)
```

## Testing Checklist

- [ ] `cargo check -p usdfc-core` passes
- [ ] `cargo check -p usdfc-backend` passes
- [ ] `cargo check -p usdfc-analytics-terminal --features ssr` passes
- [ ] `cargo check -p usdfc-analytics-terminal --features hydrate` passes
- [ ] `cargo leptos build --profile railway` works
- [ ] Dockerfile builds successfully
- [ ] Railway deployment completes
- [ ] Application runs correctly
- [ ] All endpoints work
- [ ] Cursor pagination works
