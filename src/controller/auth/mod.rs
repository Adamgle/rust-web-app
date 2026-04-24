mod error;
mod password_policy;

use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub use error::Error;
use hmac::{Hmac, Mac};
use sqlx::{Executor, Pool, types::Uuid};
use tower_cookies::{Cookie, Cookies};

use crate::{
    AppState,
    config::{self, Env, EnvError},
    controller::{cookies, error::GenericControllerError, types::ApiStatusResponse},
    database::{
        DatabaseConnection,
        types::{ClientUser, DatabaseSession, DatabaseUser},
    },
};
use axum::{
    Json, Router,
    extract::{FromRef, FromRequest, State, rejection::JsonRejection},
    routing::{get, post},
};

pub(in crate::controller::auth) type Result<T> = std::result::Result<T, self::Error>;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClientAuthenticationCredentials {
    pub email: String,
    pub password: String,
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

// <S: Clone + Send + Sync + 'static>
pub fn router() -> axum::Router<AppState>
// where
    // DatabaseConnection: FromRef<S>,
{
    Router::new()
        .route("/auth/session", get(get_auth_session))
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
}

/// Generate HMAC Mac for the given bytes using the HMAC secret key from environment variable.
pub fn generate_hmac_mac(bytes: &[u8]) -> self::Result<Hmac<sha2::Sha256>> {
    let hmac_key = dotenvy::var(Env::HmacSecretKey.as_ref())
        .map_err(|e| config::Error::Env(EnvError::MissingEnv(Arc::from(e))))?;

    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(hmac_key.as_bytes())
        // That would be server error as the HMAC key stored on the server is invalid.
        .map_err(|e| self::Error::Other(Arc::new(anyhow::Error::new(e))))?;

    mac.update(bytes);
    Ok(mac)
}

/// Parse and verify the SSID cookie using HMAC signature.
pub fn parse_ssid_cookie(cookie: Cookie) -> self::Result<Uuid> {
    let (ssid, signature) = match cookie.value().split_once(cookies::SSID_SEPARATOR) {
        Some((ssid, signature)) => (ssid, signature),
        None => {
            // NOTE: In theory, there could be no signature part or there could be no ssid.
            return Err(self::Error::InvalidSessionCookieWrongFormat {
                ssid: Some(cookie.to_string()),
            });
        }
    };

    let ssid =
        Uuid::parse_str(ssid).map_err(|e| self::Error::InvalidSessionCookieWrongUuidFormat {
            ssid: Some(ssid.to_string()),
            source: Arc::new(anyhow::Error::new(e)),
        })?;

    // Decode the signature
    let signature = hex::decode(signature).map_err(|e| {
        self::Error::InvalidSessionCookieHmacVerificationFailed {
            ssid: Some(cookie.to_string()),
            source: Arc::new(anyhow::Error::new(e)),
        }
    })?;

    let mac = self::generate_hmac_mac(ssid.as_bytes())?;

    mac.verify_slice(&signature).map_err(|e| {
        self::Error::InvalidSessionCookieHmacVerificationFailed {
            ssid: Some(cookie.to_string()),
            source: Arc::new(anyhow::Error::new(e)),
        }
    })?;

    Ok(ssid)
}

pub async fn get_server_side_session(
    conn: &Pool<sqlx::Postgres>,
    cookies: &Cookies,
) -> self::Result<ClientUser> {
    let Some(cookie) = cookies.get(cookies::SSID) else {
        return Err(self::Error::MissingSessionCookie);
    };

    let cookie_ssid = self::parse_ssid_cookie(cookie.clone())?;

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

    // This does not make sens because the session would no exist without the user,
    let Some(user) = sqlx::query_as!(
        ClientUser,
        "SELECT id, balance, delta, created_at, email FROM users WHERE users.id = $1",
        user_id
    )
    .fetch_optional(conn)
    .await?
    else {
        // That should not happen, as we have a valid session with a user_id.
        return Err(self::Error::Other(Arc::new(anyhow::anyhow!(
            "User not found for valid session"
        ))));
    };

    return Ok(user);
}

pub async fn get_auth_session(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
) -> self::Result<Json<ClientUser>> {
    return Ok(Json(self::get_server_side_session(&conn, &cookies).await?));
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
pub fn create_ssid_cookie<T: TryInto<Uuid>>(ssid: T) -> self::Result<Cookie<'static>>
where
    <T as std::convert::TryInto<sqlx::types::Uuid>>::Error: std::fmt::Debug,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let ssid = ssid
        .try_into()
        .map_err(|e| self::Error::InvalidSessionCookieWrongUuidFormat {
            // Cannot get the ssid string here unfortunately.
            ssid: None,
            source: Arc::new(anyhow::Error::new(e)),
        })?;

    let mac = self::generate_hmac_mac(ssid.as_bytes())?;

    let result = mac.finalize();
    let signature = result.into_bytes();

    let signature = hex::encode(signature);

    let value = format!("{}{}{}", ssid, cookies::SSID_SEPARATOR, signature);

    Ok(Cookie::build((cookies::SSID, value))
        .http_only(true)
        .path("/")
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(time::Duration::days(7))
        .into())
}

pub fn hash_password(password: &str) -> self::Result<String> {
    // NOTE: As per documentation OsRng use may block the OS, maybe that should be put inside the tokio::task::spawn_blocking

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}

pub async fn register_user(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
    ExtractClientAuthenticationCredentials(credentials): ExtractClientAuthenticationCredentials<
        ClientAuthenticationCredentials,
    >,
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

    return Ok(Json(user));
}

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
        return Err(self::Error::InvalidCredentials {
            source: Some(Arc::new(anyhow::anyhow!("No user exists for that email."))),
        });
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

pub async fn logout_user(
    State(DatabaseConnection(conn)): State<DatabaseConnection>,
    cookies: Cookies,
) -> self::Result<Json<ApiStatusResponse>> {
    // 1. Check if there is a user, there is a session cookie, that is valid and exists in db.
    // 2. Remove the cookie server-side sending appropriate Set-Cookie header.
    // 3. Remove the session from the database.

    // NOTE: Maybe that should be a transaction.

    // That should not happen as the endpoint should not be called when user is not logged in the first place.
    // Practically speaking, the only error that can happen is that error, when user modified the cookie and is invalid,
    // is expired, was removed from the database or does not exist in the first place and endpoint was still called.
    // We are wrapping each as the client error and does not even need to test for those possibilities, just wrapping with self::Error::ClientError.

    if let Err(e) = self::get_server_side_session(&conn, &cookies).await {
        // We are wrapping that in the ClientError to avoid bloating client with the error message
        // that is not relevant to them, also we just want to log BAD_REQUEST to them and this provides the formatting.
        return Err(self::Error::from(GenericControllerError::ClientError {
            source: Some(Arc::new(anyhow::Error::new(e))),
        }));
    }

    match cookies.get(cookies::SSID) {
        Some(cookie) => {
            // Client sent invalid ssid cookie, that should not happen as we already validated the session above.
            let ssid = self::parse_ssid_cookie(cookie)?;

            // Delete the session from the database.
            sqlx::query!("DELETE FROM sessions WHERE id = $1::uuid", ssid)
                .execute(&conn)
                .await?;

            // To properly remove the cookie it has to be of the same name, path and domain.
            // let cookie = Cookie::build((cookies::SSID, "")).http_only(true).path("/");

            // We are recreating the cookie to remove it to be sure it has the same attributes, specifically
            // the same name, path and domain.
            let cookie =
                create_ssid_cookie(ssid).map_err(|e| GenericControllerError::ClientError {
                    source: Some(Arc::new(anyhow::Error::new(e))),
                })?;
            cookies.remove(cookie);
        }
        None => {
            // NOTE: This should not happen as call for server side session already validates that.
            let error = self::Error::MissingSessionCookie;

            return Err(self::Error::GenericControllerError(
                GenericControllerError::ClientError {
                    source: Some(Arc::new(anyhow::anyhow!(error))),
                },
            ));
        }
    };

    return Ok(Json(ApiStatusResponse { status: true }));
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use axum::{
        body::Body,
        extract::Request,
        http::{self, header},
    };
    use http::method::Method;
    use http_body_util::BodyExt;
    use sqlx::types::uuid;
    use tower::ServiceExt;

    use crate::{AppState, app};

    use super::*;

    // Generally we have to use serial_test::serial in each test that interacts with the environment that other tests
    // can affect, specifically that include the environment variables as they are mutated while testing.
    // If though each test that changes the env would restore it back, while tests run in parallel they may see the
    // mutated env from other tests that was not yet restored.
    // We may also run the tests on single threads doing cargo test -- --test-threads 1 but then each tests, even the
    // ones that are not have to run serially are and it ends up slower.

    // NOTE: It is worth noting, that if some test fails, try running it with cargo test -- --test-threads 1, or add [serial_test::serial]

    #[test]
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

        let cookie = super::create_ssid_cookie(uuid);

        assert!(cookie.is_ok());
        assert!(super::create_ssid_cookie(uuid.to_string()).is_ok());

        // Cookie ssid and signature are valid
        let ssid = super::parse_ssid_cookie(cookie.unwrap());
        assert!(ssid.is_ok());
    }

    #[test]
    fn test_parse_cookie_invalid_cookie_format() -> anyhow::Result<()> {
        let valid_signed_signature = super::create_ssid_cookie(Uuid::new_v4())?
            .value()
            .split_once(cookies::SSID_SEPARATOR)
            .unwrap()
            .1
            .to_string();

        let invalid_cookie_values = vec![
            "just-a-random-string-without-separator".to_string(),
            format!("random-string{}random-string", cookies::SSID_SEPARATOR), // valid format with invalid ssid and signature.
            uuid::Uuid::new_v4().to_string(), // valid UUID but no signature
            format!("{}{}", cookies::SSID_SEPARATOR, valid_signed_signature), // Valid signature but no UUID
            format!(
                "{}{}{}",
                uuid::Uuid::new_v4().to_string(),
                cookies::SSID_SEPARATOR,
                valid_signed_signature
            ), // valid format, valid uuid and valid signature, but signature not for the registered ssid cookie
            format!(
                "{}{}",
                uuid::Uuid::new_v4().to_string().as_str(),
                cookies::SSID_SEPARATOR,
            ), // valid UUID with separator but no signature
            format!(
                "{}{}{}",
                uuid::Uuid::new_v4().to_string().as_str(),
                cookies::SSID_SEPARATOR,
                "a".repeat(64)
            ), // valid UUID with separator but invalid signature
            "".to_string(),                      // empty string
            cookies::SSID_SEPARATOR.to_string(), // just separator
        ];

        for invalid_value in invalid_cookie_values {
            let cookie = Cookie::new(cookies::SSID, invalid_value.to_string());
            let result = super::parse_ssid_cookie(cookie);

            // All we care about here is that it errors out.
            assert!(result.is_err());
        }

        Ok(())
    }

    #[test]
    fn test_parse_cookie_signature_wrong_format() -> anyhow::Result<()> {
        let ssid = Uuid::new_v4();
        let mut cookie = super::create_ssid_cookie(ssid)?;
        let mut parts = cookie
            .value()
            .split(cookies::SSID_SEPARATOR)
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(parts.len(), 2);

        // We would take the signature, decode the hex value, take the bytes, concatenate those and write it to tamper the cookie.
        let signature = hex::decode(&parts[1])?
            .iter()
            .map(|b| format!("{:x}", b))
            .collect::<String>();

        parts[1] = signature;
        cookie.set_value(parts.join(cookies::SSID_SEPARATOR));

        let result = super::parse_ssid_cookie(cookie.clone());

        // NOTE: This sometimes fails the assertion, do not know why.
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            self::Error::InvalidSessionCookieHmacVerificationFailed { .. }
        ));

        Ok(())
    }

    /// This tests whether the parsing fails if we would supply the cookie with valid signature, meaning signed
    /// with correct HMAC key, but for different ssid value.
    #[test]
    fn test_parse_cookie_correct_signature_for_wrong_ssid_value() -> anyhow::Result<()> {
        let mut cookie = super::create_ssid_cookie(Uuid::new_v4())?;
        let mut parts = cookie
            .value()
            .split(cookies::SSID_SEPARATOR)
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(parts.len(), 2);

        let different_cookie = super::create_ssid_cookie(Uuid::new_v4())?
            .value()
            .split(cookies::SSID_SEPARATOR)
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(different_cookie.len(), 2);

        let different_signature = &different_cookie[1];

        // Change the signature for valid, but for different cookie.
        parts[1] = different_signature.to_string();
        cookie.set_value(parts.join(cookies::SSID_SEPARATOR));

        let parsed = super::parse_ssid_cookie(cookie);
        assert!(parsed.is_err());
        assert!(matches!(
            parsed.unwrap_err(),
            self::Error::InvalidSessionCookieHmacVerificationFailed { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_parse_cookie_invalid_signature() {
        let ssid = Uuid::new_v4();
        let mut cookie = super::create_ssid_cookie(ssid).expect("Failed to create SSID cookie");

        // Tamper with the cookie value to invalidate the HMAC signature.
        let mut parts = cookie
            .value()
            .split(cookies::SSID_SEPARATOR)
            .map(String::from)
            .collect::<Vec<String>>();

        assert_eq!(parts.len(), 2);

        let invalid_signature_invalid_size = [1u8; 16];
        let invalid_signature_valid_size: [u8; 32] = [0u8; 32];

        for invalid_signature in [
            hex::encode(invalid_signature_invalid_size),
            hex::encode(invalid_signature_valid_size),
        ] {
            parts[1] = invalid_signature;
            let tampered_value = parts.join(cookies::SSID_SEPARATOR);
            cookie.set_value(tampered_value);

            let result = super::parse_ssid_cookie(cookie.clone());
            assert!(result.is_err());

            let result = result.unwrap_err();

            matches!(
                result,
                self::Error::InvalidSessionCookieHmacVerificationFailed { .. }
            );
        }
    }

    #[test]
    fn test_hmac_signature_encoding_decoding() -> anyhow::Result<()> {
        // Generate the HMAC signature with the private key using uuid.
        // Encode the signature to hex, decode to bytes and check if it matches the original signature bytes.

        let ssid = Uuid::new_v4();

        // NOTE: Uuid::as_bytes() is different from doing Uuid::to_string().as_bytes() and that will fail.
        let mac = super::generate_hmac_mac(ssid.as_bytes())?;
        let result = mac.finalize();
        let signature = result.into_bytes();

        let hex_signature = hex::encode(signature);
        assert!(hex::decode(&hex_signature)? == signature.to_vec());

        Ok(())
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
