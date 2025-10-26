use axum::{Router, response::IntoResponse, routing::get};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/session", get(get_auth_session))
}

#[axum::debug_handler]
pub async fn get_auth_session() -> impl IntoResponse {
    // Validates the JWT from the request "Authorization" header.

    axum::http::StatusCode::OK
}
