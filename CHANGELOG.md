# Changelog

All notable changes to the USDFC Analytics Terminal will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Railway deployment configuration
- Production deployment at https://usdfc-terminal-cleaned-production.up.railway.app/
- MultiTroveGetter aggregation for accurate total debt calculation
- Resource usage metrics in documentation
- Comprehensive architecture documentation (`docs/ARCHITECTURE.md`)
  - Complete tech stack with all dependencies
  - Cargo workspace structure diagrams
  - System architecture and data flow
  - Caching strategy and TTL configuration
  - Dual API architecture (REST + Server Functions)
  - Build profiles explanation (railway, ci, production)
  - Deployment flow documentation

### Fixed
- cargo-leptos version compatibility (upgraded to v0.2.47 from v0.2.5)
- cargo install performance (switched to pre-compiled binary installer)
- wasm-bindgen version mismatch (downgraded project to 0.2.105 to match cargo-leptos bundled version)
- ConfigSectionNotFound panic (corrected Cargo.toml path to crates/terminal/Cargo.toml)
- RPC contract revert errors (exit code 33)
- Build warnings across terminal and API crates
- Inaccurate server function count in diagrams (16, not 15)

### Changed
- Updated documentation with production URLs
- Simplified API reference examples
- Updated README.md tech stack section with accurate dependencies
- Moved detailed diagrams from README to ARCHITECTURE.md
- Improved README conciseness while maintaining completeness

## [0.1.0] - 2024-12-31

### Added
- Initial release
- REST API with 10 endpoints
- Real-time USDFC analytics
- Integration with Filecoin RPC, Blockscout, Secured Finance, GeckoTerminal
- SSR + WASM hydration with Leptos 0.6
- SQLite-based metrics history
- Docker support
- Comprehensive API documentation
