use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use reqwest::header;
use rust_web_app::{
    Error,
    controller::{
        self,
        auth::{self, ClientAuthenticationCredentials},
        cookies,
    },
    database::types::{ClientUser, DatabaseSession, DatabaseUser},
};
use sqlx::types::Uuid;
use tower_cookies::Cookie;

use crate::controller::auth::fixtures::{
    AuthEndpoint, TestAuthPayload, TestAuthState, TestResponse,
};

mod fixtures;

// NOTE: I won't fix those issues, because the code is doing what is supposed to, but given that it is my first integration testing code,
// I would try to not repeat those mistakes in the future so just want to lay it down explicitly here.

// What I do not like about this integration tests and the code in general.
// 1. <FIXED> AuthEndpoint::EMAIL and AuthEndpoint::PASSWORD are not private and accessible from outside.
//  -> that is not a big issue because it is easily fixable, but you may duplicate the addresses and got an error which
//  -> you did not asserted for, on the other hand, when you are asserting for duplicated email you would want to add to the database
//  -> user with the same email, and you would need access to that constant, of course there are other ways to reproduce that error.
//  -> I thought maybe I will assert that the Email declared on the struct is not the same as the oen in payload, but then
//  -> you would not be able to test for TakenEmail as assertion would not pass.
//      => <FIXED> Created `fixtures` module.
// 2. `AuthEndpoint::payload` is repetitive, thought provide strict meaning.
// 3. I don't imagine the test fixtures here to be always irrefutable, meaning not for each test each fixture must exists,
//  -> but some of the API here is left unimplemented and can be found used in ambiguous way, that may lead to confusion,
//  -> for example, AuthEndpoint::Register.create() is used many times, not even in that endpoint as it just create the database
//  -> state for registration, and it might be useful in many tests. On the other hand it is explicitly defined what it is doing,
//  -> but might be confusing when used for other endpoints, maybe that should just be a separate function, separate from the struct, that
//  -> create the database state for the registration, not tied to anything the endpoint is doing.
// 4. `AuthEndpoint::create` is ambiguous, the naming does not reflect anything, you have to read the comments and code to understand.
// 5. <FIXED>There is one constant in the module `EMAIL` representing different email address that the one used in the AuthEndpoint::EMAIL,
//  -> the constant is not explicit and not clearly states what it is used for.
//      => <FIXED> Created `fixtures` module.
// 6. The file should be splitted per endpoint I suppose, not sure about that, that may be an overkill, but tests here are
//  -> structured per name, like each test first 2 words are describing the endpoint, and we should just move them to separate files.
//  -> thought not sure if at this scale it is worthwhile.
// 7. The way we build cookies by manually constructing the header value is error prone, we should
//  -> use some cookies crate to build cookies properly, also there may be some inconsistencies with using
//  -> literals for cookies names and consts.

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_register_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    // To test the registration flow we need to:
    // 1. Check for each variant of the error that can happen during registration.
    // 2. We need to setup a test database connection and apply migrations so we would have
    // the tables ready for the tests, I would assume that the tables should be empty, we could also delegate migrations designed for testing.
    // We need to assure that non of the test affects another test, it would probably have to be run with --threads-count 1 or serial_test::serial.

    let endpoint = AuthEndpoint::Register;

    let request = endpoint.build(&pool);

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
    } = AuthEndpoint::Register.create(&pool).await?;

    let TestAuthPayload::Register(payload) = AuthEndpoint::Register.payload() else {
        panic!("Expected Register payload variant");
    };

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Register.build(&pool);
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
    let request = AuthEndpoint::Register.build(&pool);

    // Test with weak password
    let payload = ClientAuthenticationCredentials {
        email: fixtures::EMAIL.to_string(),
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
    let TestAuthState::Register { .. } =
        AuthEndpoint::create(&AuthEndpoint::Register, &pool).await?;

    // We are using the same credentials as the ones for the first registration above.
    let TestAuthPayload::Register(payload) = AuthEndpoint::payload(&AuthEndpoint::Register) else {
        panic!("Expected Register payload variant");
    };

    let TestResponse { response, error } =
        AuthEndpoint::Register.build(&pool).send(payload).await?;

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
    let TestResponse { response, error } = endpoint.build(&pool).send(payload).await?;

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
async fn test_register_same_password_different_hash(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    let TestAuthState::Register { user, .. } =
        AuthEndpoint::create(&AuthEndpoint::Register, &pool).await?;

    // Use default payload as the one used when creating the user above but with different email.
    let TestAuthPayload::Register(mut payload) = AuthEndpoint::payload(&AuthEndpoint::Register)
    else {
        panic!("Expected Register payload variant");
    };

    // Assert that payload contains the same password as the one created above.
    assert!(
        Argon2::default()
            .verify_password(
                payload.password.as_bytes(),
                &PasswordHash::new(&user.password_hash)?
            )
            .is_ok()
    );

    // Change email to not taken
    payload.email = fixtures::EMAIL.to_string();

    let TestResponse { response, error } =
        AuthEndpoint::Register.build(&pool).send(payload).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    let registered_user = sqlx::query_as!(
        DatabaseUser,
        "SELECT * FROM users WHERE email = $1",
        fixtures::EMAIL
    )
    .fetch_one(&pool)
    .await
    .context("Registered user does not exists in the database.")?;

    assert!(registered_user.password_hash != user.password_hash);

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
async fn test_session_valid(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthState::Register {
        session: DatabaseSession { id, .. },
        user: DatabaseUser { id: user_id, .. },
        ..
    } = AuthEndpoint::Register.create(&pool).await?;

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Session.build(&pool);
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
async fn test_session_invalid_ssid_cookie(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let mut request = AuthEndpoint::Session.build(&pool);
    request.builder = request.builder.header(
        header::COOKIE,
        format!(
            "{}=invalid-uuid-format{}<HMAC_SIGNATURE>",
            cookies::SSID,
            cookies::SSID_SEPARATOR
        ),
    );

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
    } = AuthEndpoint::Register.create(&pool).await?;

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

    let mut request = AuthEndpoint::Session.build(&pool);
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
    } = AuthEndpoint::Register.create(&pool).await?;

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    // Remove the session from the database to simulate missing session
    let result = sqlx::query!("DELETE FROM sessions WHERE id = $1", id)
        .execute(&pool)
        .await?;

    assert!(result.rows_affected() == 1);

    let mut request = AuthEndpoint::Session.build(&pool);

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
    } = AuthEndpoint::Register.create(&pool).await?;

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
    } = AuthEndpoint::Register.create(&pool).await?;

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

    let request = AuthEndpoint::Login.build(&pool);

    // Check that there are no cookies in the builder;

    let headers = request.builder.headers_ref();
    assert!(headers.and_then(|h| h.get(header::COOKIE)).is_none());

    let TestResponse { response, error } = request.send(payload).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    let ssid = response.headers().get(header::SET_COOKIE);

    assert!(ssid.is_some());

    let cookie = Cookie::parse(ssid.unwrap().to_str()?)?;

    let c = cookie.to_string();
    let (_, cookie_options) = c.split_once("; ").unwrap_or_default();

    let ssid = auth::parse_ssid_cookie(cookie)
        .context("Failed to parse SSID cookie from the Set-Cookie header.")?;

    let default_cookie = auth::create_ssid_cookie(ssid)?.to_string();
    let default_cookie_options = default_cookie
        .split_once("; ")
        .map(|(_, options)| options)
        .unwrap_or_default();

    assert_eq!(cookie_options, default_cookie_options);

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
    } = AuthEndpoint::Register.create(&pool).await?;

    let TestAuthPayload::Register(payload) = AuthEndpoint::Register.payload() else {
        panic!("Expected Register payload variant");
    };

    let ssid = auth::create_ssid_cookie(id)?.to_string();

    let mut request = AuthEndpoint::Register.build(&pool);
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
    } = AuthEndpoint::Register.create(&pool).await?;

    let cookie = auth::create_ssid_cookie(id)?;

    let mut request = AuthEndpoint::Logout.build(&pool);
    request.builder = request.builder.header(header::COOKIE, cookie.to_string());

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
    let TestAuthState::Register { .. } = AuthEndpoint::Register.create(&pool).await?;
    let mut request = AuthEndpoint::Logout.build(&pool);

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

    // let mut e: Option<std::sync::Arc<auth::Error>> = None;

    // assert!(matches!(
    //     error,
    //     Error::Controller(controller::Error::Auth(auth::Error::GenericControllerError(
    //         controller::GenericControllerError::ClientError { source: Some(ref source) },
    //     )))
    //     if source.downcast_ref::<auth::Error>().map(|arc| {
    //         e = Some(arc.clone().into())
    //     }).is_some()
    // ));

    // assert!(matches!(
    //     error.flat::<GenericControllerError>(),
    //     Some(GenericControllerError::ClientError { .. })
    // ));

    assert!(matches!(
        error,
        Error::Controller(controller::Error::Auth(
            auth::Error::GenericControllerError(
                controller::GenericControllerError::ClientError { .. }
            )
        ))
    ));

    // Other variants of this endpoint are not even testable, as their errors are practically unreachable,
    // but still I have defined the errors for them, maybe I should just unwrap.

    Ok(())
}

#[sqlx::test]
#[tracing_test::traced_test]
fn test_register_hmac_signature(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    let TestAuthPayload::Register(payload) = AuthEndpoint::Register.payload() else {
        panic!("Expected Register payload variant");
    };

    let request = AuthEndpoint::Register.build(&pool);

    let TestResponse { response, error } = request.send(payload).await?;

    assert!(error.is_none());
    assert!(response.status().is_success());

    // Look up the Set-Cookie header to verify HMAC signature generation.
    // TODO: That is not single ssid cookie, it's full Set-Cookie header that should be parsed and will yield errors
    // if more cookies are present.
    let ssid_cookie = response.headers().get(header::SET_COOKIE);
    assert!(ssid_cookie.is_some());

    let ssid_cookie = Cookie::parse(ssid_cookie.unwrap().to_str()?)?;

    // It would verify the HMAC signature inside the cookie.
    auth::parse_ssid_cookie(ssid_cookie)?;

    Ok(())
}

#[test]
/// Tests whether HMAC key is of valid size.
fn test_valid_hmac_key() -> anyhow::Result<()> {
    auth::generate_hmac_mac(&[]).context("Invalid key in the .env")?;

    Ok(())
}
