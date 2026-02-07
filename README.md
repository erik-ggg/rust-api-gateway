# Rust API Gateway Demo

A production-ready demonstration of an API Gateway built with **Rust** and **Axum**. This project showcases how to implement essential gateway patterns including authentication, proxying, observability, and security hardening using a modern async Rust stack.

## 🚀 Features

- **Web Framework**: Built on [Axum](https://github.com/tokio-rs/axum), leveraging the Tokio ecosystem.
- **Authentication**: Session-based authentication via cookies (`tower-sessions`) with **auto-refresh mechanism**.
- **Gateway/Proxy**: Transparent request forwarding with streaming response support (`reqwest`).
- **Observability**:
  - **Tracing**: Structured logging (JSON in Prod, Text in Dev).
  - **Metrics**: Ready for aggregation.
- **Reliability**:
  - **Rate Limiting**: Token bucket algorithm to prevent abuse.
  - **Timeouts**: Enforced timeouts for upstream requests.
- **Security Hardening**:
  - **Cookies**: `Secure`, `HttpOnly`, and `SameSite` enforcement.
  - **Headers**: HSTS (`Strict-Transport-Security`) and CSP (`Content-Security-Policy`).
- **Configuration**: Multi-environment support (Dev, Pre, Prod) via TOML files.

## 🛠️ Prerequisites

- **Rust**: Latest stable version. [Install Rust](https://www.rust-lang.org/tools/install).
- **Target Service**: A backend to proxy to (defaults to `https://httpbin.org`).

## 🏃‍♂️ How to Run

### 1. Development Mode
Optimized for local testing. Security features are relaxed (HTTP allowed), and logs are human-readable.

```bash
# Default behavior
cargo run
# OR explicitly
RUN_MODE=development cargo run
```

### 2. Production Mode
Enables all security features, JSON logging, and stricter reliability limits.

```bash
RUN_MODE=production cargo run
```

> **Note**: In production mode, cookie security settings (`Secure: true`) require HTTPS. Accessing via `http://localhost` may result in cookies being rejected by the browser.

## ⚙️ Configuration

The application uses layered configuration in the `config/` directory:

| Environment | File | Description |
|---|---|---|
| **Base** | `default.toml` | Common settings for all environments. |
| **Dev** | `development.toml` | Overrides for local dev (Debug logs, relaxed limits). |
| **Prod** | `production.toml` | Overrides for prod (JSON logs, strict limits, security enabled). |

### Key Settings
- **`[security]`**: Toggle HTTPS, HSTS, CSP, and Cookie policy.
- **`[reliability]`**: Configure Rate Limits (`rate_limit_per_sec`) and Timeouts (`timeout_ms`).
- **`[server]`**: Set Port, Log Level, and Log Format.

## 🧪 Testing

### Automated Tests
Run integration tests to verify authentication, proxy flows, and **security configuration**:
```bash
cargo test
```

### Manual Verification (Curl)
**1. Login**
```bash
curl -c cookies.txt -X POST -H "Content-Type: application/json" \
    -d '{"username": "demo"}' \
    http://localhost:3000/login
```

**2. Access Proxy (Protected)**
accessing `/get` will be proxied to `https://httpbin.org/get`:
```bash
curl -b cookies.txt http://localhost:3000/get
```

## 📂 Project Structure

- `src/main.rs`: Entry point & Config loading.
- `src/router.rs`: Router & Middleware assembly (Auth, CORS, RateLimit, Timeout).
- `src/config.rs`: Strongly typed configuration structs.
- `src/handlers/`: Request business logic.
  - `proxy.rs`: The core gateway forwarding logic (streaming).
- `src/middleware/`: Custom middleware (Auth Guard).
