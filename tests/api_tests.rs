use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rust_api_gateway::{config::AppConfig, router};
use tower::ServiceExt; // for oneshot
use tower_sessions::MemoryStore;

#[tokio::test]
async fn health_check_works() {
    let config = AppConfig::new().unwrap();
    let store = MemoryStore::default();
    let app = router::app(config, store);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn protected_routes_require_login() {
    let config = AppConfig::new().unwrap();
    let store = MemoryStore::default();
    let app = router::app(config, store);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/logout")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_flow_works() {
    let config = AppConfig::new().unwrap();
    let store = MemoryStore::default();
    let app = router::app(config, store);

    // 1. Login
    let login_body = serde_json::to_string(&serde_json::json!({
        "username": "testuser"
    }))
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Extract cookie
    let cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // 2. Access Protected Route (Logout) with cookie
    let response = app
        .oneshot(
            Request::builder()
                .uri("/logout")
                .method("POST")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn security_headers_in_production() {
    // Manually construct prod config
    let config = AppConfig {
        server: rust_api_gateway::config::ServerConfig {
            port: 0,
            log_level: "info".into(),
            log_format: "text".into(),
        },
        proxy: rust_api_gateway::config::ProxyConfig {
            target_url: "http://example.com".into(),
        },
        security: rust_api_gateway::config::SecurityConfig {
            https: true,
            secure_cookies: true,
            same_site: "Strict".into(),
            enable_hsts: true,
            enable_csp: true,
        },
        reliability: rust_api_gateway::config::ReliabilityConfig {
            rate_limit_per_sec: 100,
            timeout_ms: 1000,
        },
    };

    let store = MemoryStore::default();
    let app = router::app(config, store);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.headers().contains_key("strict-transport-security"));
    assert!(response.headers().contains_key("content-security-policy"));
}

#[tokio::test]
async fn security_headers_missing_in_dev() {
    // Manually construct dev config
    let config = AppConfig {
        server: rust_api_gateway::config::ServerConfig {
            port: 0,
            log_level: "info".into(),
            log_format: "text".into(),
        },
        proxy: rust_api_gateway::config::ProxyConfig {
            target_url: "http://example.com".into(),
        },
        security: rust_api_gateway::config::SecurityConfig {
            https: false,
            secure_cookies: false,
            same_site: "Lax".into(),
            enable_hsts: false,
            enable_csp: false,
        },
        reliability: rust_api_gateway::config::ReliabilityConfig {
            rate_limit_per_sec: 100,
            timeout_ms: 1000,
        },
    };

    let store = MemoryStore::default();
    let app = router::app(config, store);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(!response.headers().contains_key("strict-transport-security"));
    assert!(!response.headers().contains_key("content-security-policy"));
}
