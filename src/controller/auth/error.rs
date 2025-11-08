use std::borrow::Cow;
use std::sync::Arc;

use crate::error::ErrorResponse;

use crate::error::ErrorExt;
use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug, Clone)]
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
    #[error("Session expired at: {0}")]
    SessionExpired(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Weak password does not meet the policy requirements: {0}")]
    PasswordRequirementsNotMet(String),
    // NOTE: We are not leaking the inner error message to avoid leaking sensitive information,
    // but it will be logged in the middleware on the server-side if one occur.
    #[error("Internal Server Error")]
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
    // That would be general purpose, catch all variant for client errors when we do not want to send any specific reason
    // for the failure, but do want to save the source of the error in variant for logging purposes.
    #[error("Internal Server Error")]
    ClientError {
        #[source]
        source: Option<Arc<anyhow::Error>>,
    },
    #[error("Internal Server Error")]
    Other(#[from] Arc<anyhow::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let message = Cow::Owned(self.to_string());

        // This should not leak sensitive information.
        let representation = match self {
            Error::MissingSessionCookie => ErrorResponse {
                status: axum::http::StatusCode::UNAUTHORIZED,
                message,
            },
            Error::DatabaseError(_) => ErrorResponse {
                status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message,
            },
            Error::MissingSessionInDatabase => ErrorResponse {
                status: axum::http::StatusCode::UNAUTHORIZED,
                message,
            },
            Error::InvalidSessionCookieWrongUuidFormat { .. } => ErrorResponse {
                status: axum::http::StatusCode::UNAUTHORIZED,
                message,
            },
            Error::SessionExpired(_) => ErrorResponse {
                status: axum::http::StatusCode::UNAUTHORIZED,
                message,
            },
            Error::UserNotFound => ErrorResponse {
                status: axum::http::StatusCode::NOT_FOUND,
                message,
            },
            Error::PasswordRequirementsNotMet(_) => ErrorResponse {
                status: axum::http::StatusCode::BAD_REQUEST,
                message,
            },
            Error::PasswordHashError(_) => ErrorResponse {
                status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                // We are not using the display trait for the error as that contains sensitive information.
                // NOTE: I do not think that is the best way to do it, the display trait method
                // should not contain sensitive information in the first place.
                message,
            },
            Error::EmailTaken(_) => ErrorResponse {
                status: axum::http::StatusCode::CONFLICT,
                message,
            },
            Error::AlreadyAuthenticated => ErrorResponse {
                status: axum::http::StatusCode::BAD_REQUEST,
                message,
            },
            Error::InvalidCredentials { .. } => ErrorResponse {
                status: axum::http::StatusCode::BAD_REQUEST,
                message,
            },
            Error::ClientError { .. } | Error::Other(_) => ErrorResponse {
                status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message,
            },
        };

        return self.to_response(representation);
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
