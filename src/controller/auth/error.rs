use std::borrow::Cow;

use crate::error::ErrorResponse;

use crate::error::ErrorExt;
use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    #[error("Missing ssid cookie")]
    MissingSessionCookie,
    #[error("Missing session for ssid in database")]
    MissingSessionInDatabase,
    #[error("Invalid ssid cookie: {0}")]
    InvalidSessionCookie(String),
    #[error("Session expired at {0}")]
    SessionExpired(String),
    #[error("User not found")]
    UserNotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        let message = Cow::Owned(self.to_string());

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
            Error::InvalidSessionCookie(_) => ErrorResponse {
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
