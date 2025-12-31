//! USDFC Analytics Terminal - Main Entry Point

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
#[cfg(feature = "ssr")]
use std::time::Instant;

/// Application start time for uptime tracking
#[cfg(feature = "ssr")]
static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

/// Get application uptime in seconds
#[cfg(feature = "ssr")]
fn get_uptime_secs() -> u64 {
    START_TIME.elapsed().as_secs()
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::get, Router, Json};
    use axum::body::Body;
    use axum::http::header::{HeaderName, HeaderValue};
    use axum::http::{Method, Request};
    use axum::middleware::{self, Next};
    use axum::response::Response;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use serde::Serialize;
    use tower_http::compression::CompressionLayer;
    use tower_http::cors::{CorsLayer, Any};
    use tower_http::services::ServeDir;
    use usdfc_analytics_terminal::{app::App, fileserv::file_and_error_handler, state::AppState};
    use usdfc_analytics_terminal::api::handlers;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    // Load .env file
    let _ = dotenvy::dotenv();

    // Get Leptos configuration
    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let mut leptos_options = conf.leptos_options;

    // Derive bind address from unified Config (env-backed on server, defaults on client)
    let cfg = usdfc_analytics_terminal::config::config();
    let addr_str = format!("{}:{}", cfg.host, cfg.port);
    let addr = addr_str.parse().unwrap_or_else(|_| {
        tracing::warn!("Invalid address '{}', falling back to 0.0.0.0:3000", addr_str);
        "0.0.0.0:3000"
            .parse()
            .expect("default address should always parse")
    });
    leptos_options.site_addr = addr;

    // Explicitly register all server functions (required on some platforms)
    use server_fn::axum::register_explicit;
    use usdfc_analytics_terminal::server_fn::*;

    register_explicit::<GetProtocolMetrics>();
    register_explicit::<GetRecentTransactions>();
    register_explicit::<GetTroves>();
    register_explicit::<GetLendingMarkets>();
    register_explicit::<GetDailyVolumes>();
    register_explicit::<GetAddressInfo>();
    register_explicit::<GetNormalizedAddress>();
    register_explicit::<GetTopHolders>();
    register_explicit::<GetStabilityPoolTransfers>();
    register_explicit::<GetUSDFCPriceData>();
    register_explicit::<CheckApiHealth>();
    register_explicit::<GetHolderCount>();
    register_explicit::<GetOrderBook>();
    register_explicit::<GetRecentLendingTrades>();
    register_explicit::<GetAdvancedChartData>();

    tracing::info!("Registered {} server functions", 15);

    // Generate route list from App component for SSR
    let routes = generate_route_list(App);

    // Build application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
    };

    // Security headers middleware for production deployment
    async fn security_headers(request: Request<Body>, next: Next) -> Response {
        let mut response = next.run(request).await;
        let headers = response.headers_mut();

        // Prevent MIME type sniffing
        headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );

        // Prevent clickjacking
        headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );

        // XSS protection (legacy but still useful)
        headers.insert(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        );

        // Control referrer information
        headers.insert(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );

        // HSTS - enforces HTTPS for 1 year including subdomains
        headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );

        // Content Security Policy
        headers.insert(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self'; \
                 connect-src 'self' https://api.node.glif.io https://filecoin.blockscout.com https://api.goldsky.com https://api.geckoterminal.com; \
                 frame-ancestors 'none';",
            ),
        );

        response
    }

    // Health check response types
    #[derive(Serialize)]
    struct HealthStatus {
        status: String,
        version: String,
        uptime_secs: u64,
        checks: HealthChecks,
    }

    #[derive(Serialize)]
    struct HealthChecks {
        rpc: CheckResult,
        blockscout: CheckResult,
        subgraph: CheckResult,
        gecko: CheckResult,
        database: CheckResult,
    }

    #[derive(Serialize)]
    struct CheckResult {
        status: String,
        latency_ms: Option<u64>,
        error: Option<String>,
    }

    // Health check handler - comprehensive health status
    async fn health_handler() -> Json<HealthStatus> {
        use usdfc_analytics_terminal::rpc::RpcClient;
        use usdfc_analytics_terminal::blockscout::BlockscoutClient;
        use usdfc_analytics_terminal::subgraph::SubgraphClient;
        use usdfc_analytics_terminal::gecko::GeckoClient;
        use usdfc_analytics_terminal::config::config;

        // Check RPC
        let rpc_check = {
            let start = std::time::Instant::now();
            let rpc = RpcClient::new();
            match rpc.get_fil_price().await {
                Ok(_) => CheckResult {
                    status: "ok".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                },
                Err(e) => CheckResult {
                    status: "error".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(e.to_string()),
                },
            }
        };

        // Check Blockscout
        let blockscout_check = {
            let start = std::time::Instant::now();
            let blockscout = BlockscoutClient::new();
            match blockscout.gql_get_token_info(&config().usdfc_token).await {
                Ok(_) => CheckResult {
                    status: "ok".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                },
                Err(e) => CheckResult {
                    status: "error".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(e.to_string()),
                },
            }
        };

        // Check Subgraph
        let subgraph_check = {
            let start = std::time::Instant::now();
            let subgraph = SubgraphClient::new();
            match subgraph.get_lending_markets().await {
                Ok(_) => CheckResult {
                    status: "ok".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                },
                Err(e) => CheckResult {
                    status: "error".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(e.to_string()),
                },
            }
        };

        // Check GeckoTerminal
        let gecko_check = {
            let start = std::time::Instant::now();
            let gecko = GeckoClient::new();
            match gecko.get_pool_info(&config().pool_usdfc_wfil).await {
                Ok(_) => CheckResult {
                    status: "ok".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                },
                Err(e) => CheckResult {
                    status: "error".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(e.to_string()),
                },
            }
        };

        // Check SQLite database
        let db_check = {
            let start = std::time::Instant::now();
            match usdfc_analytics_terminal::historical::check_db_health() {
                Ok(_) => CheckResult {
                    status: "ok".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: None,
                },
                Err(e) => CheckResult {
                    status: "error".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    error: Some(e),
                },
            }
        };

        // Determine overall status - degraded if any non-critical service fails
        // Critical services: RPC, Blockscout, Database
        let all_critical_healthy = rpc_check.status == "ok"
            && blockscout_check.status == "ok"
            && db_check.status == "ok";

        let status = if all_critical_healthy {
            "healthy"
        } else {
            "degraded"
        };

        Json(HealthStatus {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: get_uptime_secs(),
            checks: HealthChecks {
                rpc: rpc_check,
                blockscout: blockscout_check,
                subgraph: subgraph_check,
                gecko: gecko_check,
                database: db_check,
            },
        })
    }

    // Simple readiness check - returns ok if server is running
    async fn ready_handler() -> &'static str {
        // Initialize START_TIME on first call if not already done
        let _ = *START_TIME;
        "ok"
    }

    // Build REST API router with CORS support
    // CORS layer allows cross-origin requests to API endpoints
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);

    // Create API router with all REST endpoints
    let api_routes = Router::new()
        .route("/v1/price", get(handlers::get_price))
        .route("/v1/metrics", get(handlers::get_metrics))
        .route("/v1/health", get(handlers::get_health))
        .route("/v1/history", get(handlers::get_history))
        .route("/v1/troves", get(handlers::get_troves_list))
        .route("/v1/troves/:addr", get(handlers::get_trove_by_address))
        .route("/v1/transactions", get(handlers::get_transactions))
        .route("/v1/address/:addr", get(handlers::get_address))
        .route("/v1/lending", get(handlers::get_lending))
        .route("/v1/holders", get(handlers::get_holders))
        .layer(cors);

    // Build Axum router with leptos_router integration
    // Note: In Leptos 0.6, server functions are automatically handled by leptos_routes()
    let app = Router::new()
        // REST API endpoints - MUST be before leptos_routes to take precedence
        .nest("/api", api_routes)
        // Health check endpoints for Kubernetes/Docker
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        // Static files - MUST be before leptos_routes to prevent /*any from catching them
        .nest_service("/pkg", ServeDir::new(format!("{}/pkg", leptos_options.site_root)))
        // Leptos routes with SSR (this also handles server functions automatically)
        .leptos_routes(&app_state, routes, App)
        // Static file serving and 404 handler
        .fallback(file_and_error_handler)
        // Add security headers
        .layer(middleware::from_fn(security_headers))
        // Add compression
        .layer(CompressionLayer::new())
        // Add state
        .with_state(app_state);

    // Initialize SQLite database for metric history persistence
    match usdfc_analytics_terminal::historical::init_db() {
        Ok(()) => tracing::info!("Initialized metrics database"),
        Err(e) => tracing::error!("Failed to initialize metrics database: {}", e),
    }

    // Start background metric snapshot collector
    usdfc_analytics_terminal::historical::start_snapshot_collector();
    tracing::info!("Started background metric snapshot collector (60s interval)");

    // Start server
    tracing::info!("Starting USDFC Analytics Terminal on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutting down...");
}

#[cfg(all(feature = "csr", not(feature = "ssr")))]
fn main() {
    use leptos::*;
    use usdfc_analytics_terminal::app::App;
    
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

#[cfg(all(feature = "hydrate", not(feature = "ssr"), not(feature = "csr")))]
fn main() {
    use leptos::*;
    use usdfc_analytics_terminal::app::App;

    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

#[cfg(all(not(feature = "ssr"), not(feature = "csr"), not(feature = "hydrate")))]
fn main() {
    println!("No rendering mode selected. Use --features csr, ssr, or hydrate");
}
