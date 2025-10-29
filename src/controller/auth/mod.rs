//! NOTE: The authentication controller module could probably be consider an abomination of this project.

mod error;

pub use error::Error;
use sqlx::types::Uuid;
use tower_cookies::Cookies;

use crate::{AppState, database::DatabaseConnection};
use axum::{Json, Router, extract::State, routing::get};

pub(in crate::controller::auth) type Result<T> = std::result::Result<T, self::Error>;

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/session", get(get_auth_session))
}

//  i32, // That should be unsigned, but it fails converting to u32, as postgres does not have unsigned, like a [1, 2^31 - 1]
//     abbreviation: String,
//     company: String,
//     since: chrono::NaiveDate, // DATE
//     price: f32,
//     delta: f32,
//     last_update: chrono::NaiveDate, // TIMESTAMP
//     created_at: chrono::NaiveDate,  // TIMESTAMP

// TODO: Delegate the database schemas to separate module/file.
pub struct Session {
    id: sqlx::types::uuid::Uuid,
    user_id: i32,
    created_at: chrono::NaiveDate,
    expires_at: chrono::NaiveDate,
}

pub struct User {
    // TODO: Map the full user schema here.
}

/// Stripped from sensitive info about the user
#[derive(serde::Serialize)]
pub struct SessionUser {
    id: i32,
    balance: f32,
    delta: f32,
}

pub struct UserSessionsJunction {
    user_id: i32,
    session_id: sqlx::types::uuid::Uuid,
    // Primary key is (user_id, session_id), not sure if we need to represent that here.
}

#[axum::debug_handler]
pub async fn get_auth_session(
    State(AppState {
        database: DatabaseConnection(conn),
    }): State<AppState>,
    cookies: Cookies,
) -> self::Result<Json<SessionUser>> {
    // Validates the JWT from the request "Authorization" header or session id.
    let Some(cookie_ssid) = cookies.get("SSID") else {
        return Err(self::Error::MissingSessionCookie);
    };

    let cookie_ssid = cookie_ssid.value();
    let cookie_ssid = Uuid::parse_str(cookie_ssid)
        .map_err(|e| self::Error::InvalidSessionCookie(e.to_string()))?;

    let Some((expires_at, user_id)) = 
        // Check if the sessions exists for the ssid cookie.
        // NOTE: Not sure why I have to cast the $1 to uuid, but without it it fails.
        sqlx::query!("SELECT expires_at, user_id FROM sessions WHERE sessions.id = $1::uuid", cookie_ssid)
            .fetch_optional(&conn)
            .await?.map(|r| {
                (r.expires_at, r.user_id)
            })
    else {
        // NOTE: Not sure if that error is appropriate to auth::Error, but it is also not database::Error,
        // the database is working fine, it's just the client sent the non-existing session id.
        return Err(self::Error::MissingSessionInDatabase);
    };

    if expires_at < chrono::Utc::now().naive_utc() {
        return Err(self::Error::SessionExpired(expires_at.to_string()));
    }

    // TODO: Those columns of that table are definitely not extensive, we need to think about what we will
    // actually need when returning user information to the client.
    let Some(user) = sqlx::query_as!(SessionUser, "SELECT id, balance, delta FROM users WHERE users.id = $1", user_id)
        .fetch_optional(&conn)
        .await? else {
        // That should not happen, as we have a valid session with a user_id.
        return Err(self::Error::UserNotFound);
    };

    return Ok(Json(user));
}
