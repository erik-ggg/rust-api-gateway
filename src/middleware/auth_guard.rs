use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use time;
use tower_sessions::Session;

use crate::models::session_user::SessionUser;

/// Middleware to protect routes by ensuring a valid session exists.
/// Returns `401 Unauthorized` if the session is missing or invalid.
pub async fn auth_guard(
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = session
        .get::<SessionUser>("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if user.is_some() {
        // Force session update to extend the cookie lifetime.
        // We re-set the expiry to sliding window logic.
        session.set_expiry(Some(tower_sessions::Expiry::OnInactivity(
            time::Duration::minutes(30),
        )));
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
