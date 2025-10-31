//! NOTE: The authentication controller module could probably be consider an abomination of this project.

mod error;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub use error::Error;
use sqlx::types::Uuid;
use tower_cookies::{Cookie, Cookies};
use tracing::info;

use crate::{
    AppState,
    database::{
        DatabaseConnection,
        types::{ClientUser, DatabaseUser},
    },
};
use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};

pub(in crate::controller::auth) type Result<T> = std::result::Result<T, self::Error>;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/session", get(get_auth_session))
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
}

#[axum::debug_handler]
pub async fn get_auth_session(
    State(AppState {
        database: DatabaseConnection(conn),
        ..
    }): State<AppState>,
    cookies: Cookies,
) -> self::Result<Json<ClientUser>> {
    // Validates the JWT from the request "Authorization" header or session id.
    let Some(cookie_ssid) = cookies.get("SSID") else {
        return Err(self::Error::MissingSessionCookie);
    };

    let cookie_ssid = cookie_ssid.value();
    let cookie_ssid = Uuid::parse_str(cookie_ssid)
        .map_err(|e| self::Error::InvalidSessionCookie(e.to_string()))?;

    // Check if the sessions exists for the ssid cookie.
    // NOTE: Not sure why I have to cast the $1 to uuid, but without it it fails.
    let Some((expires_at, user_id)) = sqlx::query!(
        "SELECT expires_at, user_id FROM sessions WHERE sessions.id = $1::uuid",
        cookie_ssid
    )
    .fetch_optional(&conn)
    .await?
    .map(|r| (r.expires_at, r.user_id)) else {
        // NOTE: Not sure if that error is appropriate to auth::Error, but it is also not database::Error,
        // the database is working fine, it's just the client sent the non-existing session id.
        return Err(self::Error::MissingSessionInDatabase);
    };

    if expires_at < chrono::Utc::now().naive_utc() {
        return Err(self::Error::SessionExpired(expires_at.to_string()));
    }

    // TODO: Those columns of that table are definitely not extensive, we need to think about what we will
    // actually need when returning user information to the client.

    let Some(user) = sqlx::query_as!(
        ClientUser,
        "SELECT id, balance, delta FROM users WHERE users.id = $1",
        user_id
    )
    .fetch_optional(&conn)
    .await?
    else {
        // That should not happen, as we have a valid session with a user_id.
        return Err(self::Error::UserNotFound);
    };

    return Ok(Json(user));
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClientRegisterPayload {
    email: String,
    password: String,
}

// We are doing the module trick to isolate the const flags, which, bear with me, are useless.
mod password_policy {
    // We are not restricting any letters or symbols for the password, just enforcing some policies.
    // Although I am not sure if that is a good idea.

    // TODO: Port that to client-side.
    const MIN_LENGTH: usize = 8;
    const MAX_LENGTH: usize = 128;
    const SPECIAL_CHARACTERS: &str = "!@#$%^&*()-+";
    const REQUIRE_SPECIAL_CHARACTERS: bool = true;
    const REQUIRE_UPPERCASE: bool = true;
    const REQUIRE_DIGIT: bool = true;
    const REQUIRE_LOWERCASE: bool = true;

    pub fn validate_password_policy(password: &str) -> bool {
        // NOTE: That is not the length, that is the size in bytes as this is how the len function works, it may behave unexpectedly with the grapheme rich symbols,
        // leave that be.
        let size = password.len();

        if !(MIN_LENGTH..MAX_LENGTH).contains(&size) {
            return false;
        }

        let (mut has_uppercase, mut has_lowercase, mut has_digit, mut has_special) = (
            !REQUIRE_UPPERCASE,
            !REQUIRE_LOWERCASE,
            !REQUIRE_DIGIT,
            !REQUIRE_SPECIAL_CHARACTERS,
        );

        for char in password.chars() {
            if !has_uppercase && char.is_uppercase() {
                has_uppercase = true;
            } else if !has_lowercase && char.is_lowercase() {
                has_lowercase = true;
            } else if !has_digit && char.is_ascii_digit() {
                has_digit = true;
            } else if !has_special && SPECIAL_CHARACTERS.contains(char) {
                has_special = true;
            }

            // Early exit if all requirements are met, size is already early satisfied .
            if has_uppercase && has_lowercase && has_digit && has_special {
                return true;
            }
        }

        return has_uppercase && has_lowercase && has_digit && has_special;
    }

    #[cfg(test)]
    mod tests {
        use super::validate_password_policy;

        #[test]
        fn test_password_policy() {
            assert!(validate_password_policy("Password1!"));
            assert!(!validate_password_policy("weakpass"));
            assert!(!validate_password_policy("Short1!"));
            assert!(!validate_password_policy("NoSpecialChar1"));
            assert!(!validate_password_policy("NOLOWERCASE1!"));
            assert!(!validate_password_policy("nouppercase1!"));
            assert!(!validate_password_policy("NoDigit!"));
        }
    }
}

#[axum::debug_handler]
pub async fn register_user(
    State(AppState {
        database: DatabaseConnection(conn),
        ..
    }): State<AppState>,
    cookies: Cookies,
    mut headers: HeaderMap,
    Json(payload): Json<ClientRegisterPayload>,
) -> self::Result<Json<ClientUser>> {
    // To register a user we need to:
    // 1. Take the email and password from the user, send it over HTTP, ideally that would be HTTPS
    // but we are not doing that.
    // 2. We would hash the password using Argon2 algorithm
    //  2.5 We could also enforce some password policies here, like minimum length, special characters, etc.
    // 3. We would take that pair of the email and password (ideally email would also be validated, but
    // we are not doing that, or actually I do not know if that is ideal, I have heard that it is kind a tricky
    // and maybe not necessary) check if that exists in the database, note: I have heard not to do that
    // as it leaks information about existing users, but most services that I have seen are still using
    // this approach so I am not sure why would I not do that. So we check if the email and password
    // exists in the database, I would prompt the user if the email is taken, but actually if we
    // check the pair simultaneously we cannot prompt that, so maybe we should check the email first,
    // but then it is prone to timing attacks. I guess I will just check the email first and pair
    // Actually I have mixed up the login and registration flow here, for registration we just need to check
    // if the email is taken, not the pair.
    // So we need to query the database for the email and see if it exists.
    // Then we would just take the hashed password and email and insert it into the database.
    // alongside with the unique salt for that user. The salt will be probably prefixing the password
    // for the login logic.
    // 4. Then we would create a session for that user, note: we still do not have CSRF protection.
    // 5. We would save that session in the database and set the session cookie in the response.
    // 6. Next we would save the session cookie and the user_id generate into the junction table
    // as there could be multiple sessions for a single user.
    // 7. Finally we would return a success response to the client.

    // TODO: The user can register multiple times, even if already logged in, decide what to do about that.
    // We could redirect to the root page.

    let ClientRegisterPayload { email, password } = payload;

    // We are not doing email validation, just rely on the client side validation.

    if !password_policy::validate_password_policy(&password) {
        return Err(self::Error::WeakPassword(password));
    }

    let password = password.as_bytes();

    // We are storing the password as a hash using Argon2 algorithm with salt.
    // We store the salt to verify the password later during login.

    // NOTE: As per documentation, that may block the OS, maybe that should be put inside the tokio::task::spawn_blocking
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    let parsed_hash = PasswordHash::new(&password_hash)?;

    assert!(
        Argon2::default()
            .verify_password(password, &parsed_hash)
            .is_ok()
    );

    let mut tx = conn.begin().await?;

    // Check if email is already taken.
    let is_email_taken = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)",
        email
    )
    .fetch_one(tx.as_mut())
    .await?
    .exists;

    if let Some(is_email_taken) = is_email_taken
        && is_email_taken
    {
        return Err(self::Error::EmailTaken(email));
    }

    let account = sqlx::query!("INSERT INTO accounts (created_at) VALUES (DEFAULT) RETURNING id")
        .fetch_one(tx.as_mut())
        .await?;

    let user = sqlx::query_as!(
        DatabaseUser,
        "INSERT INTO users (email, password_hash, password_salt, account_id) 
        VALUES ($1, $2, $3, $4) RETURNING *",
        email,
        password_hash,
        salt.as_str(),
        account.id
    )
    .map(ClientUser::from)
    .fetch_one(tx.as_mut())
    .await?;

    let session = sqlx::query!(
        "INSERT INTO sessions (user_id, created_at, expires_at) 
        VALUES ($1, DEFAULT, DEFAULT) RETURNING id",
        user.id
    )
    .fetch_one(tx.as_mut())
    .await?;

    // Save the user-session into the junction table.
    // sqlx::query!(
    //     "INSERT INTO user_sessions (user_id, session_id) VALUES ($1, $2)",
    //     user.id,
    //     session.id
    // )
    // .execute(&conn)
    // .await?;

    tx.commit().await?;

    let cookie = Cookie::build(("SSID", session.id.to_string()))
        .http_only(true)
        .path("/")
        .max_age(time::Duration::days(7));

    cookies.add(cookie.into());

    return Ok(Json(user));
}

#[axum::debug_handler]
pub async fn login_user() {
    unimplemented!()
}

#[axum::debug_handler]
pub async fn logout_user() {
    unimplemented!()
}
