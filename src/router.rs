use axum::{
    Router, middleware,
    routing::{any, get, post},
};
use time::Duration;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

use crate::{
    config::AppConfig,
    handlers::{auth, health, proxy},
    middleware::auth_guard::auth_guard,
};

/// Configures the Axum application router.
///
/// This involves:
/// - **Session Layer**: Manages user sessions via cookies with a 30-minute inactivity expiry.
/// - **CORS Layer**: Configures Cross-Origin Resource Sharing to allow credentials (cookies) and mirror the request origin for development convenience.
/// - **Proxy State**: Initialize the HTTP client and target URL for the gateway logic.
/// - **Routes**: Sets up public (health, login), protected (logout), and proxy routes.
pub fn app(config: AppConfig, store: MemoryStore) -> Router {
    // Parse SameSite from config
    let same_site = match config.security.same_site.as_str() {
        "Lax" => tower_sessions::cookie::SameSite::Lax,
        "Strict" => tower_sessions::cookie::SameSite::Strict,
        "None" => tower_sessions::cookie::SameSite::None,
        _ => tower_sessions::cookie::SameSite::Lax,
    };

    let session_layer = SessionManagerLayer::new(store)
        .with_secure(config.security.secure_cookies)
        .with_http_only(true)
        .with_same_site(same_site)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30)));

    let cors_layer = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
        .allow_credentials(true)
        .allow_methods(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(vec![
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ]);

    let proxy_state = proxy::ProxyState {
        client: reqwest::Client::new(),
        target_url: config.proxy.target_url,
    };

    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/login", post(auth::login));

    let protected_routes = Router::new()
        .route("/logout", post(auth::logout))
        .layer(middleware::from_fn(auth_guard));

    let proxy_routes = Router::new()
        .route("/{*path}", any(proxy::proxy_handler))
        .with_state(proxy_state)
        .layer(middleware::from_fn(auth_guard));

    let mut router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(proxy_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .layer(session_layer);

    // Add HSTS if enabled
    if config.security.enable_hsts {
        router = router.layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    // Add CSP if enabled
    if config.security.enable_csp {
        router = router.layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static("default-src 'self'"),
        ));
    }

    let reliability_layer = tower::ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(
            |err: axum::BoxError| async move {
                (
                    axum::http::StatusCode::REQUEST_TIMEOUT,
                    format!("Request timed out: {}", err),
                )
            },
        ))
        .layer(tower::buffer::BufferLayer::new(1024))
        .layer(tower::limit::RateLimitLayer::new(
            config.reliability.rate_limit_per_sec,
            std::time::Duration::from_secs(1),
        ))
        .layer(tower::timeout::TimeoutLayer::new(
            std::time::Duration::from_millis(config.reliability.timeout_ms),
        ));

    router.layer(reliability_layer)
}
