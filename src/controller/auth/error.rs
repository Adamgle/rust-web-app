use std::borrow::Cow;
use std::sync::Arc;

use crate::controller;
use crate::error::ErrorResponse;

use crate::error::ErrorExt;
use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug, Clone)]
#[error("Internal Server Error")]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    #[error("Missing ssid cookie")]
    MissingSessionCookie,
    #[error("Missing session for ssid cookie in database")]
    MissingSessionInDatabase,
    #[error("Invalid ssid cookie")]
    InvalidSessionCookieWrongUuidFormat {
        ssid: Option<String>,
        source: Arc<anyhow::Error>,
    },
    #[error("Invalid ssid cookie")]
    InvalidSessionCookieWrongFormat {
        ssid: Option<String>,
    },
    #[error("Invalid ssid cookie")]
    InvalidSessionCookieHmacVerificationFailed {
        ssid: Option<String>,
        source: Arc<anyhow::Error>,
    },
    #[error("Session expired at: {0}")]
    SessionExpired(String),
    #[error("Weak password does not meet the policy requirements: {0}")]
    PasswordRequirementsNotMet(String),
    // NOTE: We are not leaking the inner error message to avoid leaking sensitive information,
    // but it will be logged in the middleware on the server-side if one occur.
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("Email already taken: {0}")]
    EmailTaken(String),
    #[error("Already authenticated")]
    AlreadyAuthenticated,
    #[error("Invalid email or password")]
    InvalidCredentials {
        #[source]
        source: Option<Arc<anyhow::Error>>,
    },
    GenericControllerError(#[from] controller::GenericControllerError),
    Config(#[from] crate::config::Error),
    // Other would be tried as Internal Server Error that takes the error for later logging, something that do not need the separate variant.
    Other(#[from] Arc<anyhow::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let message = Cow::Owned(self.to_string());
        let status = match self {
            Error::MissingSessionCookie => StatusCode::UNAUTHORIZED,
            Error::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MissingSessionInDatabase => StatusCode::UNAUTHORIZED,
            Error::InvalidSessionCookieWrongUuidFormat { .. } => StatusCode::UNAUTHORIZED,
            Error::InvalidSessionCookieHmacVerificationFailed { .. } => StatusCode::UNAUTHORIZED,
            Error::InvalidSessionCookieWrongFormat { .. } => StatusCode::UNAUTHORIZED,
            Error::SessionExpired(_) => StatusCode::UNAUTHORIZED,
            Error::PasswordRequirementsNotMet(_) => StatusCode::BAD_REQUEST,
            Error::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::EmailTaken(_) => StatusCode::CONFLICT,
            Error::AlreadyAuthenticated => StatusCode::BAD_REQUEST,
            Error::InvalidCredentials { .. } => StatusCode::BAD_REQUEST,
            Error::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::GenericControllerError(ref generic_controller_error) => {
                generic_controller_error.clone().into_response().status()
            }
        };

        return self.to_response(ErrorResponse { message, status });
    }
}

impl ErrorExt for Error {
    fn to(self) -> crate::Error {
        return crate::controller::Error::from(self).into();
    }
}

// For multi-level error conversions
// From sqlx::Error -> Arc<sqlx::Error> -> crate::database::Error -> crate::controller::auth::Error
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(crate::database::Error::from(err))
    }
}
