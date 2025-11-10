//! NOTE: The authentication controller module could probably be consider an abomination of this project.

mod error;

use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub use error::Error;
use sqlx::{Executor, Pool, types::Uuid};
use tower_cookies::{Cookie, Cookies};

use crate::{
    controller::{cookies, types::ApiStatusResponse},
    database::{
        DatabaseConnection,
        types::{ClientUser, DatabaseSession, DatabaseUser},
    },
};
use axum::{
    Json, Router,
    extract::{FromRef, FromRequest, State, rejection::JsonRejection},
    response::IntoResponse,
    routing::{get, post},
};

pub(in crate::controller::auth) type Result<T> = std::result::Result<T, self::Error>;

pub fn router<S: Clone + Send + Sync + 'static>() -> axum::Router<S>
where
    DatabaseConnection: FromRef<S>,
{
    Router::new()
        .route("/auth/session", get(get_auth_session))
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
}

pub async fn get_server_side_session(
    conn: &Pool<sqlx::Postgres>,
    cookies: &Cookies,
) -> self::Result<ClientUser> {
    let Some(cookie_ssid) = cookies.get(cookies::SSID) else {
        return Err(self::Error::MissingSessionCookie);
    };

    let cookie_ssid = cookie_ssid.value();

    // TODO: Test the error, how it behaves when the UUID is invalid.
    // let cookie_ssid: Uuid = cookie_ssid.try_into()
    let cookie_ssid: Uuid = Uuid::parse_str(cookie_ssid).map_err(|e| {
        self::Error::InvalidSessionCookieWrongUuidFormat {
            ssid: Some(cookie_ssid.to_string()),
            source: Arc::new(anyhow::Error::new(e)),
        }
    })?;

    // Check if the sessions exists for the ssid cookie.
    // NOTE: Not sure why I have to cast the $1 to uuid, but without it it fails.
    let Some((expires_at, user_id)) = sqlx::query!(
        "SELECT expires_at, user_id FROM sessions WHERE sessions.id = $1::uuid",
        cookie_ssid
    )
    .fetch_optional(conn)
    .await?
    .map(|r| (r.expires_at, r.user_id)) else {
        // NOTE: Not sure if that error is appropriate to auth::Error, but it is also not database::Error,
        // the database is working fine, it's just the client sent the non-existing session id.
        return Err(self::Error::MissingSessionInDatabase);
    };

    if expires_at < chrono::Utc::now().naive_utc() {
        // Delete the expired session
        sqlx::query!("DELETE FROM sessions WHERE id = $1::uuid", cookie_ssid)
            .execute(conn)
            .await?;

        return Err(self::Error::SessionExpired(expires_at.to_string()));
    }

    let Some(user) = sqlx::query_as!(
        ClientUser,
        "SELECT id, balance, delta, created_at, email FROM users WHERE users.id = $1",
        user_id
    )
    .fetch_optional(conn)
    .await?
    else {
        // That should not happen, as we have a valid session with a user_id.
        return Err(self::Error::UserNotFound);
    };

    return Ok(user);
}

#[axum::debug_handler]
pub async fn get_auth_session(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
) -> self::Result<Json<ClientUser>> {
    return Ok(Json(self::get_server_side_session(&conn, &cookies).await?));
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClientAuthenticationCredentials {
    pub email: String,
    pub password: String,
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

/// NOTE: I am not sure if I want to isolate such logic into separate functions as it's not very flexible.
pub async fn create_database_session(
    executor: impl Executor<'_, Database = sqlx::Postgres>,
    user_id: i32,
) -> self::Result<DatabaseSession> {
    Ok(sqlx::query_as!(
        DatabaseSession,
        "INSERT INTO sessions (user_id, created_at, expires_at)
        VALUES ($1, DEFAULT, DEFAULT) RETURNING *",
        user_id
    )
    .fetch_one(executor)
    .await?)
}

// I want it to take the ssid as a string or uuid, if string then that should be convertible to uuid,
fn create_ssid_cookie<T: TryInto<Uuid>>(ssid: T) -> self::Result<Cookie<'static>>
where
    <T as std::convert::TryInto<sqlx::types::Uuid>>::Error: std::fmt::Debug,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let s = ssid
        .try_into()
        .map_err(|e| self::Error::InvalidSessionCookieWrongUuidFormat {
            // Cannot get the ssid string here unfortunately.
            ssid: None,
            source: Arc::new(anyhow::Error::new(e)),
        })?;

    Ok(Cookie::build((cookies::SSID, s.to_string()))
        .http_only(true)
        .path("/")
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(time::Duration::days(7))
        .into())
}

/// A custom extractor to normalize the email to lowercase, obvious overkill.
/// I think I could achieve that with deserialize attributes to serde, but not sure.
pub struct ExtractClientAuthenticationCredentials<T>(pub T);

impl<S> FromRequest<S> for ExtractClientAuthenticationCredentials<ClientAuthenticationCredentials>
where
    axum::Json<ClientAuthenticationCredentials>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        match axum::Json::<ClientAuthenticationCredentials>::from_request(req, state).await {
            Ok(mut value) => {
                value.email = value.email.to_lowercase();

                Ok(Self(value.0))
            }
            Err(rejection) => Err(rejection),
        }
    }
}

fn hash_password(password: &str) -> self::Result<String> {
    // NOTE: As per documentation OsRng use may block the OS, maybe that should be put inside the tokio::task::spawn_blocking

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}

#[axum::debug_handler]
pub async fn register_user(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
    ExtractClientAuthenticationCredentials(credentials): ExtractClientAuthenticationCredentials<
        ClientAuthenticationCredentials,
    >,
) -> self::Result<impl IntoResponse> {
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

    // That is kind off weird, but the Err is returned when there is no session, which is what we want.
    // That could technically be more idiomatic if that would be an Option, but then we would not be
    // able propagate the errors easily, so leave that be.
    if self::get_server_side_session(&conn, &cookies).await.is_ok() {
        // Frontend edge runtime would redirect the user to homepage if already authenticated.
        return Err(self::Error::AlreadyAuthenticated);
    };

    let ClientAuthenticationCredentials { email, password } = credentials;

    // We are not doing email validation, just rely on the client side validation.

    if !password_policy::validate_password_policy(&password) {
        return Err(self::Error::PasswordRequirementsNotMet(password));
    }

    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = conn.begin().await?;

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

    let password_hash = self::hash_password(&password)?;

    let user = sqlx::query_as!(
        DatabaseUser,
        "INSERT INTO users (email, password_hash, account_id) 
        VALUES ($1, $2, $3) RETURNING *",
        email,
        password_hash,
        account.id
    )
    .map(ClientUser::from)
    .fetch_one(tx.as_mut())
    .await?;

    let DatabaseSession { id: ssid, .. } =
        self::create_database_session(tx.as_mut(), user.id).await?;

    let cookie = self::create_ssid_cookie(ssid)?;
    cookies.add(cookie);

    tx.commit().await?;

    return Ok(axum::response::IntoResponse::into_response(Json(user)));
}

#[axum::debug_handler]
pub async fn login_user(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
    ExtractClientAuthenticationCredentials(credentials): ExtractClientAuthenticationCredentials<
        ClientAuthenticationCredentials,
    >,
) -> self::Result<Json<ClientUser>> {
    // Logging the user we need to do:
    // 1. Check if the user is already authenticated, if so, return an error.
    // 2. Take the email and password from the user, send it over HTTP, ideally that would be HTTPS
    // 3. We would have to take that email and query the database for the user, I have heard that is not ideal
    // as it exposes timing attacks, but I don't see the way we would hash the password without the salt.
    // We have to take the email, match the user, take the salt and password, hash it and compare the hashes against the one in database.
    // 4. Then we would save the ssid cookie and create a session for that user in the database.

    // We cannot just propagate the error here, as they are not relevant to that endpoint.
    // We could log it though.
    let Err(_) = self::get_server_side_session(&conn, &cookies).await else {
        return Err(self::Error::AlreadyAuthenticated);
    };

    let ClientAuthenticationCredentials { email, password } = credentials;

    let mut tx = conn.begin().await?;

    let Some(user) = sqlx::query_as!(DatabaseUser, "SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(tx.as_mut())
        .await?
    else {
        return Err(self::Error::InvalidCredentials { source: None });
    };

    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password_hash)?,
        )
        .map_err(|e| self::Error::InvalidCredentials {
            source: Some(Arc::new(anyhow::Error::new(e))),
        })?;

    let DatabaseSession { id: ssid, .. } =
        self::create_database_session(tx.as_mut(), user.id).await?;

    let cookie = self::create_ssid_cookie(ssid)?;
    cookies.add(cookie);

    tx.commit().await?;

    return Ok(Json(ClientUser::from(user)));
}

#[axum::debug_handler]
pub async fn logout_user(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
) -> self::Result<Json<ApiStatusResponse>> {
    // 1. Check if there is a user, there is a session cookie, that is valid and exists in db.
    // 2. Remove the cookie server-side sending appropriate Set-Cookie header.
    // 3. Remove the session from the database.

    // NOTE: Maybe that should be a transaction.

    self::get_server_side_session(&conn, &cookies)
        .await
        .map_err(|e| self::Error::ClientError {
            source: Some(Arc::new(e.into())),
        })?;

    match cookies.get(cookies::SSID).map(|c| c.value().to_owned()) {
        Some(ssid) => {
            let ssid = Uuid::parse_str(&ssid).map_err(|e| self::Error::ClientError {
                source: Some(Arc::new(e.into())),
            })?;

            // Delete the session from the database.
            sqlx::query!(
                "DELETE FROM sessions WHERE id = $1::uuid",
                // That should not happen
                ssid
            )
            .execute(&conn)
            .await?;

            // To properly remove the cookie it has to be of the same name, path and domain.
            // let cookie = Cookie::build((cookies::SSID, "")).http_only(true).path("/");
            let cookie = create_ssid_cookie(ssid)?;
            cookies.remove(cookie);
        }
        None => {
            // NOTE: This should not happen as call for server side session already validates that.
            return Err(self::Error::ClientError {
                source: Some(Arc::new(anyhow::anyhow!(self::Error::MissingSessionCookie))),
            });
        }
    };

    return Ok(Json(ApiStatusResponse { status: true }));
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use axum::{body::Body, extract::Request, http};
    use http::method::Method;
    use http_body_util::BodyExt;
    use reqwest::header;
    use sqlx::types::uuid;
    use tower::ServiceExt;

    use crate::{AppState, app};

    use super::*;

    // Router::new()
    // .route("/auth/session", get(get_auth_session))
    // .route("/auth/register", post(register_user))
    // .route("/auth/login", post(login_user))
    // .route("/auth/logout", post(logout_user))

    // Generally we have to use serial_test::serial in each test that interacts with the environment that other tests
    // can affect, specifically that include the environment variables as they are mutated while testing.
    // If though each test that changes the env would restore it back, while tests run in parallel they may see the
    // mutated env from other tests that was not yet restored.
    // We may also run the tests on single threads doing cargo test -- --test-threads 1 but then each tests, even the
    // ones that are not have to run serially are and it ends up slower.

    // NOTE: It is worth noting, that if some test fails, try running it with cargo test -- --test-threads 1, or add [serial_test::serial]

    #[sqlx::test]
    #[serial_test::serial]
    async fn test(pool: sqlx::Pool<sqlx::Postgres>) -> sqlx::Result<()> {
        sqlx::query!("SELECT * FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await?;

        Ok(())
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_password_hash() {
        let password = "Password1!";
        let hash = hash_password(password).expect("Failed to hash password");

        let parsed_hash = PasswordHash::new(&hash).expect("Failed to parse password hash");

        let argon2 = Argon2::default();

        assert_eq!(
            argon2.verify_password(password.as_bytes(), &parsed_hash),
            Ok(())
        );

        assert_ne!(
            argon2.verify_password(b"WrongPassword", &parsed_hash),
            Ok(())
        );

        // Different salt
        let salt = SaltString::generate(&mut OsRng);
        let wrong_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password with different salt");

        assert_ne!(
            wrong_hash,
            PasswordHash::new(&hash).expect("Failed to parse seemingly valid password hash")
        );
        assert_ne!(wrong_hash.to_string(), hash);
    }

    #[test]
    fn test_create_ssid_cookie_invalid_uuid() {
        assert!(create_ssid_cookie("invalid-uuid-string").is_err());
        let valid_size = "a".repeat(uuid::Uuid::new_v4().to_string().len());
        assert!(create_ssid_cookie(valid_size).is_err())
    }

    #[test]
    fn test_create_ssid_cookie_valid_uuid() {
        let uuid = Uuid::new_v4();

        assert!(create_ssid_cookie(uuid).is_ok());
        assert!(create_ssid_cookie(uuid.to_string()).is_ok());
    }

    /// Asserts that the ExtractClientAuthenticationCredentials extractor lowercases the email field.
    #[sqlx::test]
    #[serial_test::serial]
    async fn test_credentials_extractor_normalizes_email(
        pool: sqlx::Pool<sqlx::Postgres>,
    ) -> anyhow::Result<()> {
        // Register test handler that echos back the extracted credentials.
        let handler = async |ExtractClientAuthenticationCredentials(credentials): ExtractClientAuthenticationCredentials<
        ClientAuthenticationCredentials>| {
            Json(credentials)
        };

        let app = app(AppState::new(DatabaseConnection(pool)))
            .await
            .context("failed to build app")?
            .route("/test", post(handler));

        let credentials = ClientAuthenticationCredentials {
            email: "UPPERCASE_VALID_EMAIL@gmail.com".into(),
            password: "Password1!".into(),
        };

        let body = serde_json::to_string(&credentials)
            .context("failed to serialize credentials to JSON")?;

        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .context("failed to build request")?;

        let response = app
            .oneshot(request)
            .await
            .context("request failed in app.oneshot")?;

        let response_body = response
            .into_body()
            .collect()
            .await
            .context("failed to collect response body")?
            .to_bytes();

        let response_credentials = serde_json::from_str::<ClientAuthenticationCredentials>(
            std::str::from_utf8(&response_body)?,
        )?;

        assert!(response_credentials.email == credentials.email.to_lowercase());

        Ok(())
    }
}
