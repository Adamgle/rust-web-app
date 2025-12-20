use std::sync::Arc;

use axum::{
    body::Body,
    http::{self, Method, Request, header, request::Builder},
};
use rust_web_app::{
    AppState,
    controller::auth::{self, ClientAuthenticationCredentials},
    database::types::{DatabaseAccount, DatabaseSession, DatabaseUser},
};
use tower::ServiceExt;

#[derive(Debug)]
pub struct TestRequest {
    pub pool: sqlx::Pool<sqlx::Postgres>,
    pub builder: Builder,
}
#[derive(Debug)]
pub struct TestResponse {
    pub response: http::Response<Body>,
    pub error: Option<rust_web_app::Error>,
}

impl TestRequest {
    fn new(pool: &sqlx::Pool<sqlx::Postgres>, builder: Builder) -> Self {
        Self {
            pool: pool.clone(),
            builder,
        }
    }

    pub async fn send<T>(self, payload: T) -> anyhow::Result<TestResponse>
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
pub enum AuthEndpoint {
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
    pub fn build(&self, pool: &sqlx::Pool<sqlx::Postgres>) -> TestRequest {
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

    /// Returns default payload used for given endpoint.
    ///
    /// NOTE: That is an overkill and strictly wrong as it repeats code inside the variant,
    /// but I prefer this instead of ambiguous meaning when using `Register` variant for `Login` endpoint and vice versa.
    ///
    /// Also, it only wastes 1 bytes of memory so who cares.
    pub fn payload(&self) -> TestAuthPayload {
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
    pub async fn create(&self, pool: &sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<TestAuthState> {
        match self {
            Self::Register => {
                // Create dummy session, user and account  in the database to fill the cookies with ssid.
                let account = sqlx::query_as!(
                    DatabaseAccount,
                    "INSERT INTO accounts (created_at) VALUES (DEFAULT) RETURNING *"
                )
                .fetch_one(pool)
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
                .fetch_one(pool)
                .await?;

                let session = sqlx::query_as!(
                    DatabaseSession,
                    "INSERT INTO sessions (user_id) VALUES ($1) RETURNING *",
                    user.id
                )
                .fetch_one(pool)
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
/// Some endpoints are not changing the database state so we will not respect them here.
pub enum TestAuthState {
    /// Represents the state after a successful user registration.
    /// May be used elsewhere in tests to register a user without triggering the endpoint.
    ///
    /// It is used in other endpoints tests to setup the database state for a registered user, it may be interpreted
    /// that given endpoint needs a `Register` state to be setup in order to proceed with the endpoint logic.
    Register {
        user: DatabaseUser,
        #[allow(unused)]
        account: DatabaseAccount,
        session: DatabaseSession,
    },
}

pub enum TestAuthPayload {
    Register(ClientAuthenticationCredentials),
    Login(ClientAuthenticationCredentials),
}

pub const EMAIL: &str = "second@email.com";
