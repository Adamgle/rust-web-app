//! NOTE: The authentication controller module could probably be consider an abomination of this project.

mod error;
pub use error::Error;
use tower_cookies::Cookies;

use crate::AppState;
use axum::{Json, Router, response::IntoResponse, routing::get};

pub(in crate::controller::auth) type Result<T> = std::result::Result<T, self::Error>;

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/session", get(get_auth_session))
}

#[axum::debug_handler]
pub async fn get_auth_session(cookies: Cookies) -> self::Result<impl IntoResponse> {
    // Validates the JWT from the request "Authorization" header or session id.
    let Some(cookie) = cookies.get("SSID") else {
        return Err(self::Error::MissingSessionCookie);
    };

    // We are doing session based authentication for now.
    // NOTE: Review that strategy against CSRF attacks, as it is surely vulnerable.
    // 1. Check the cookies for a session_id.
    // 2. Query the database for the session.
    // 3. Return the user associated with the session, if any.

    //     interface SessionUser {
    //   name: string;
    //   image?: string;
    //   balance: string;
    // }

    Ok(Json(serde_json::json!({
        "name": "John Doe",
        "image": "https://example.com/avatar.jpg",
        "balance": "1000.00"
    })))
}
