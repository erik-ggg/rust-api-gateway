use axum::{Json, http::StatusCode, response::IntoResponse};
use tower_sessions::Session;

use crate::models::{login_request::LoginRequest, session_user::SessionUser};

/// Handles user login.
/// It accepts a JSON payload, creates a session, and stores the user info in the session store.
pub async fn login(session: Session, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let user = SessionUser {
        username: payload.username,
    };

    if session.insert("user", user).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create session",
        )
            .into_response();
    }

    (StatusCode::OK, "Logged in").into_response()
}

/// Handles user logout.
/// It destroys the session, effectively invalidating the cookie.
pub async fn logout(session: Session) -> impl IntoResponse {
    if session.delete().await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to logout").into_response();
    }

    (StatusCode::OK, "Logged out").into_response()
}
