use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use reqwest::Client;
use tracing::instrument;

#[derive(Clone)]
pub struct ProxyState {
    pub client: Client,
    pub target_url: String,
}

/// Proxy handler that forwards requests to a downstream service.
///
/// It acts as a transparent gateway:
/// - **Buffering**: The request body is read into memory to ensure compatibility between Axum and Reqwest types.
/// - **Streaming**: The response from the downstream service is streamed back to the client to minimize memory usage.
/// - **Header Forwarding**: Headers are preserved and forwarded in both directions (request and response).
#[instrument(skip(state, req))]
pub async fn proxy_handler(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Result<impl IntoResponse, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("{}{}", state.target_url, path_query);

    let (parts, body) = req.into_parts();

    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let req_builder = state
        .client
        .request(parts.method, &uri)
        .headers(parts.headers)
        .body(bytes);

    match req_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let stream = resp.bytes_stream();
            let body = Body::from_stream(stream);

            let mut response = axum::response::Response::builder().status(status);
            *response.headers_mut().unwrap() = headers;

            Ok(response.body(body).unwrap())
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}
