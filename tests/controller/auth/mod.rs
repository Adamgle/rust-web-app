// Integration tests for the config/mod.rs
// TODO: We need to figure out a better way for structuring the integration tests
// If I put the files in separate directories the analyzer does not link them.

// #![allow(unused)]

// Router::new()
// .route("/auth/session", get(get_auth_session))
// .route("/auth/register", post(register_user))
// .route("/auth/login", post(login_user))
// .route("/auth/logout", post(logout_user))

// #[tokio::test]

use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    http::{self, Method, Request, request::Builder},
};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use reqwest::header;
use rust_web_app::{
    AppState, Error,
    controller::{
        self,
        auth::{self, ClientAuthenticationCredentials},
        cookies,
    },
    database::types::{ClientUser, DatabaseAccount, DatabaseSession, DatabaseUser},
};

use sqlx::types::Uuid;
use tower::ServiceExt;
use tracing::info;

#[derive(Debug)]
struct TestRequest {
    pool: sqlx::Pool<sqlx::Postgres>,
    builder: Builder,
}

#[derive(Debug)]
struct TestResponse {
    // pool: sqlx::Pool<sqlx::Postgres>,
    // app: Router,
    response: http::Response<Body>,
    error: Option<rust_web_app::Error>,
}

impl TestRequest {
    fn new(pool: sqlx::Pool<sqlx::Postgres>, builder: Builder) -> Self {
        Self { pool, builder }
    }

    async fn send<T>(self, payload: T) -> anyhow::Result<TestResponse>
    where
        T: serde::Serialize,
    {
        // NOTE: Maybe we should return that router.
        let app = rust_web_app::app(AppState::new(self.pool)).await?;

        let request = self
            .builder
            .body(Body::from(serde_json::to_string(&payload)?))?;

        // This would give you the response after serialization.
        let response = app.oneshot(request).await?;
        let error = response
            .extensions()
            .get::<Arc<rust_web_app::Error>>()
            // We can afford that clone when testing.
            .map(|e| e.as_ref().clone());

        Ok(TestResponse { response, error })
    }
}

/// It helps to have reproducible Request Builder setups for different auth endpoints.
///
/// We can think about doing something like that for each controller module.
enum AuthEndpoint {
    Register,
    Login,
    Logout,
    Session,
}

impl AuthEndpoint {
    const EMAIL: &'static str = "first@email.com";
    const PASSWORD: &'static str = "Password1!";

    /// Builds default test request for the given builder, returning a `TestRequest` that contain
    /// that builder so it can be modified .
    fn build(&self, pool: sqlx::Pool<sqlx::Postgres>) -> TestRequest {
        match self {
            Self::Register => TestRequest::new(
                pool,
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/register")
                    .header(header::CONTENT_TYPE, "application/json"),
            ),
            Self::Login => TestRequest::new(
                pool,
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/login")
                    .header(header::CONTENT_TYPE, "application/json"),
            ),
            Self::Logout => TestRequest::new(
                pool,
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/logout"),
            ),
            Self::Session => TestRequest::new(
                pool,
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/auth/session"),
            ),
        }
    }

    // NOTE: That is an overkill and strictly wrong as it repeats code inside the variant,
    // but I prefer this instead of ambiguous payload being returned for different endpoints.
    //
    // Also, it only wastes 1 bytes of memory so who cares.
    fn payload(&self) -> TestAuthPayload {
        match self {
            AuthEndpoint::Register => TestAuthPayload::Register(ClientAuthenticationCredentials {
                email: Self::EMAIL.to_string(),
                password: Self::PASSWORD.to_string(),
            }),
            AuthEndpoint::Login => TestAuthPayload::Login(ClientAuthenticationCredentials {
                email: Self::EMAIL.to_string(),
                password: Self::PASSWORD.to_string(),
            }),
            _ => unimplemented!(),
        }
    }

    /// Creates the state of the database after the endpoint is called and succeeds.
    ///
    /// For example, for registration it would create the account, user and session in the database.
    async fn create(&self, pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<TestAuthState> {
        match self {
            Self::Register => {
                // Create dummy session, user and account  in the database to fill the cookies with ssid.
                let account = sqlx::query_as!(
                    DatabaseAccount,
                    "INSERT INTO accounts (created_at) VALUES (DEFAULT) RETURNING *"
                )
                .fetch_one(&pool)
                .await?;

                let TestAuthPayload::Register(ClientAuthenticationCredentials { email, password }) =
                    self.payload()
                else {
                    panic!("Expected Register payload variant");
                };

                let password = auth::hash_password(&password)?;
                let user = sqlx::query_as!(
                    DatabaseUser,
                    "INSERT INTO users (email, password_hash, account_id)
                    VALUES ($1, $2, $3) RETURNING *",
                    email,
                    password,
                    account.id
                )
                .fetch_one(&pool)
                .await?;

                let session = sqlx::query_as!(
                    DatabaseSession,
                    "INSERT INTO sessions (user_id) VALUES ($1) RETURNING *",
                    user.id
                )
                .fetch_one(&pool)
                .await?;

                Ok(TestAuthState::Register {
                    user,
                    account,
                    session,
                })
            }
            _ => unimplemented!(),
        }
    }
}

/// This represent the state of the database after each endpoint is called and succeeds.
/// Some endpoints are not changing the database state so we will not respect them here.s
enum TestAuthState {
    // Represents the state after a successful user registration.
    // May be used elsewhere in tests to register a user without triggering the endpoint.
    Register {
        user: DatabaseUser,
        account: DatabaseAccount,
        session: DatabaseSession,
    },
}

enum TestAuthPayload {
    Register(ClientAuthenticationCredentials),
    Login(ClientAuthenticationCredentials),
}

const EMAIL: &str = "second@email.com";

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    // To test the registration flow we need to:
    // 1. Check for each variant of the error that can happen during registration.
    // 2. We need to setup a test database connection and apply migrations so we would have
    // the tables ready for the tests, I would assume that the tables should be empty, we could also delegate migrations designed for testing.
    // We need to assure that non of the test affects another test, it would probably have to be run with --threads-count 1 or serial_test::serial.

    let endpoint = AuthEndpoint::Register;

    let request = endpoint.build(pool);
    let pool = request.pool.clone();

    let TestAuthPayload::Register(payload) = endpoint.payload() else {
        panic!("Expected Register payload variant");
    };

    let TestResponse { response, error } = request.send(payload).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    let payload = response.into_body().collect().await?.to_bytes();
    let ClientUser { id, .. } = serde_json::from_slice::<ClientUser>(&payload)?;

    // User exists in the database
    let DatabaseUser {
        id: database_user_id,
        account_id,
        ..
    } = sqlx::query_as!(DatabaseUser, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .context("Registered user does not exists in the database.")?;

    // Response payload ClientUser matches database user id.
    assert!(id == database_user_id);

    // Account exists in the database for the user
    sqlx::query!("SELECT * FROM accounts WHERE id = $1", account_id)
        .fetch_one(&pool)
        .await
        .context("Account for the registered user does not exist in the database.")?;

    // Session exists for the user
    let session = sqlx::query_as!(
        DatabaseSession,
        "SELECT * FROM sessions WHERE user_id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .context("Session for the registered user does not exist in the database.")?;

    let default_session = auth::create_database_session(&pool, id).await?;

    assert!(session.created_at == default_session.created_at);
    assert!(session.expires_at == default_session.expires_at);

    Ok(())
}

// Those are the sub-tests for the registration flow, each test for their valid and invalid scenarios,
// and then single test asserts they're outcome.

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_already_authenticated(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    let TestAuthPayload::Register(payload) = AuthEndpoint::Register.payload() else {
        panic!("Expected Register payload variant");
    };

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Register.build(pool.clone());
    request.builder = request.builder.header(header::COOKIE, ssid);

    let TestResponse {
        error: Some(error), ..
    } = request.send(payload).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(auth::Error::AlreadyAuthenticated))
    ));

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_password_requirement(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    let request = AuthEndpoint::Register.build(pool);

    // Test with weak password
    let payload = ClientAuthenticationCredentials {
        email: self::EMAIL.to_string(),
        password: "weak".to_string(),
    };

    let TestResponse {
        error: Some(error), ..
    } = request.send(payload).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(
            auth::Error::PasswordRequirementsNotMet(_)
        ))
    ));

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_email_taken(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    // Fill database with user having the email.
    let TestAuthState::Register {
        user: DatabaseUser { email, .. },
        ..
    } = AuthEndpoint::create(&AuthEndpoint::Register, pool.clone()).await?;

    // Run registration with the same email.
    let payload = ClientAuthenticationCredentials {
        email,
        password: AuthEndpoint::PASSWORD.to_string(),
    };

    let TestResponse { response, error } = AuthEndpoint::Register
        .build(pool.clone())
        .send(payload)
        .await?;

    assert!(error.is_some());
    assert!(response.status().is_client_error());

    assert!(matches!(
        error.unwrap(),
        Error::Controller(controller::Error::Auth(auth::Error::EmailTaken(_)))
    ));

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_database_disconnected(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    // Close the pool to simulate database disconnection.
    pool.close().await;

    let endpoint = AuthEndpoint::Register;

    let TestAuthPayload::Register(payload) = AuthEndpoint::payload(&endpoint) else {
        panic!("Expected Register payload variant");
    };
    let TestResponse { response, error } = endpoint.build(pool).send(payload).await?;

    assert!(error.is_some());
    assert!(response.status().is_server_error());

    assert!(matches!(
        error.unwrap(),
        Error::Controller(controller::Error::Auth(auth::Error::DatabaseError(_)))
    ));

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        user: DatabaseUser { id: user_id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Session.build(pool.clone());
    request.builder = request.builder.header(header::COOKIE, ssid);

    let TestResponse { response, error } = request.send(()).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    let payload = response.into_body().collect().await?.to_bytes();
    let ClientUser { id, .. } = serde_json::from_slice::<ClientUser>(&payload)?;

    assert!(id == user_id);

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_invalid_uuid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let mut request = AuthEndpoint::Session.build(pool.clone());
    request.builder = request
        .builder
        .header(header::COOKIE, "SSID=invalid-uuid-format");

    let TestResponse {
        error: Some(error), ..
    } = request.send(()).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(
            auth::Error::InvalidSessionCookieWrongUuidFormat { .. }
        ))
    ));

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_session_expired(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    // We need to check that the session is removed from the database and cookies.

    // Create a session that's already expired
    let TestAuthState::Register {
        session: DatabaseSession { id: session_id, .. },
        user: DatabaseUser { id: user_id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    // Update the session to be expired (set expires_at to past date)
    let expired = Utc::now().naive_utc() - Duration::days(8);

    let result = sqlx::query!(
        "UPDATE sessions SET expires_at = $1 WHERE id = $2",
        expired,
        session_id
    )
    .execute(&pool)
    .await?;

    assert!(result.rows_affected() == 1);

    let ssid = auth::create_ssid_cookie(session_id)?.to_string();

    let mut request = AuthEndpoint::Session.build(pool.clone());
    request.builder = request.builder.header(header::COOKIE, ssid);

    let TestResponse {
        error: Some(error), ..
    } = request.send(()).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(auth::Error::SessionExpired(_)))
    ));

    // Verify the expired session was removed from database
    let session_removed = sqlx::query!("SELECT * FROM sessions WHERE id = $1", session_id)
        .fetch_optional(&pool)
        .await?;

    assert!(session_removed.is_none());

    // Make sure that database did not remove the user associated with the session
    let user_exists = sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&pool)
        .await?;

    assert!(user_exists.is_some());

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_missing_session_in_database(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    // Remove the session from the database to simulate missing session
    let result = sqlx::query!("DELETE FROM sessions WHERE id = $1", id)
        .execute(&pool)
        .await?;

    assert!(result.rows_affected() == 1);

    let mut request = AuthEndpoint::Session.build(pool.clone());

    // Session is removed from the database but cookie still persists.
    request.builder = request.builder.header(header::COOKIE, ssid);

    let TestResponse {
        error: Some(error), ..
    } = request.send(()).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(
            auth::Error::MissingSessionInDatabase
        ))
    ));

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_removed_when_user_removed(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id: session_id, .. },
        user: DatabaseUser { id: user_id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    // Remove the user from the database to simulate missing user
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await?;

    assert!(result.rows_affected() == 1);

    // That should not exists, as it is removed on cascade with the user removal.
    let session = sqlx::query_as!(
        DatabaseSession,
        "SELECT * FROM sessions WHERE id = $1",
        session_id
    )
    .fetch_optional(&pool)
    .await?;

    assert!(session.is_none());

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_login_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register {
        user: DatabaseUser { id: user_id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    // The payload is the same as the one when creating above as that is from the constant.
    let TestAuthPayload::Login(payload) = AuthEndpoint::Login.payload() else {
        panic!("Expected Login payload variant");
    };

    // Assert that the user exists in the database.
    sqlx::query!(
        "SELECT * FROM users WHERE id = $1 AND email = $2",
        user_id,
        payload.email
    )
    .fetch_one(&pool)
    .await
    .context("User for login does not exist in the database.")?;

    let request = AuthEndpoint::Login.build(pool.clone());

    // Check that there are no cookies in the builder;

    let headers = request.builder.headers_ref();
    assert!(headers.and_then(|h| h.get(header::COOKIE)).is_none());

    let TestResponse { response, error } = request.send(payload).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    let ssid = response.headers().get(header::SET_COOKIE);

    assert!(ssid.is_some());

    let ssid = ssid.unwrap().to_str()?.to_string();
    let (ssid, ssid_options) = ssid.split_once("; ").unwrap_or_default();

    let ssid = ssid
        .strip_prefix(format!("{}=", cookies::SSID).as_str())
        .context("SSID cookie is missing the 'ssid=' prefix.")?
        .to_string();

    let default_cookie = auth::create_ssid_cookie(ssid.clone())?.to_string();
    let default_cookie_options = default_cookie
        .split_once("; ")
        .map(|(_, options)| options)
        .unwrap_or_default();

    assert!(ssid == ssid);
    assert!(ssid_options == default_cookie_options);

    sqlx::query_as!(
        DatabaseSession,
        "SELECT * FROM sessions WHERE user_id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await
    .context("Session for the logged in user was not created in the database.")?;

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_login_already_authenticated(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    let TestAuthPayload::Register(payload) = AuthEndpoint::Register.payload() else {
        panic!("Expected Register payload variant");
    };

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Register.build(pool.clone());
    request.builder = request.builder.header(header::COOKIE, ssid);

    let TestResponse {
        error: Some(error), ..
    } = request.send(payload).await?
    else {
        panic!("Expected error in response extensions");
    };

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(auth::Error::AlreadyAuthenticated))
    ));

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_logout_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        ..
    } = AuthEndpoint::Register.create(pool.clone()).await?;

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Logout.build(pool.clone());
    request.builder = request.builder.header(header::COOKIE, ssid);

    sqlx::query!("SELECT * FROM sessions WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .context("Session for the user to be logged out does not exist in the database.")?;

    let TestResponse { response, error } = request.send(()).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_logout_invalid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register { .. } = AuthEndpoint::Register.create(pool.clone()).await?;
    let mut request = AuthEndpoint::Logout.build(pool.clone());

    // Cookie contains non-existent ssid.
    request.builder = request
        .builder
        .header(header::COOKIE, format!("SSID={}", Uuid::new_v4()));

    let TestResponse {
        error: Some(error), ..
    } = request.send(()).await?
    else {
        panic!("Expected error in response extensions");
    };

    // Other error here can happen but that is the same as testing for session endpoint as it triggers the same logic,
    // and each variant of the error is wrapped in the ClientError indicating that the client request is invalid
    // and the same error message is sent across inner variants of the source of ClientError.

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(auth::Error::ClientError { .. }))
    ));

    // Other variants of this endpoint are not even testable, as error they are practically unreachable,
    // but still I have defined the errors for them, maybe I should just unwrap.

    Ok(())
}
