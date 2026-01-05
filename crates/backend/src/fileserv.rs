//! File serving utilities for SSR
//!
//! Handles static file serving and error pages.

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
};
use leptos::*;
use tower::Service;
use tower_http::services::ServeDir;

use crate::state::AppState;

/// File and error handler for static files
pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<AppState>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.leptos_options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        // Return 404 page
        let handler = leptos_axum::render_app_to_stream(
            options.leptos_options.clone(),
            move || view! { <NotFound /> },
        );
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();

    // Use call instead of oneshot since tower version might differ
    let mut service = ServeDir::new(root);
    match Service::call(&mut service, req).await {
        Ok(res) => Ok(res.into_response()),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serving file: {}", err),
        )),
    }
}

/// 404 Not Found page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; background: #0a0f1a; color: #e0e6ed;">
            <h1 style="font-size: 72px; color: #00d4ff; margin: 0;">"404"</h1>
            <p style="font-size: 24px; margin: 16px 0;">"Page Not Found"</p>
            <a href="/" style="color: #00ff88; text-decoration: none;">"Return to Dashboard"</a>
        </div>
    }
}
