// The convention would be to keep the Error enum per module, and if that Error enum
// needs to abstract some errors in a separate container, like another enum,
// it will end with the `Error` suffix, so we can import it in other modules
// without referring to the module, like config::EnvError. This way
// you can bring something to scope without referring to the module and know that something is and error.
// The main Error enum will be always referred to with the module, like config::Error.

use std::{borrow::Cow, sync::Arc};

use axum::response::IntoResponse;
use tracing::info;

#[derive(thiserror::Error, Debug, Clone)]
#[error("Config error occurred: {0}")]
pub enum Error {
    // The idea is variants per module that wrap it's inner errors.
    Config(#[from] crate::config::Error),
    // There is not module called database. Think about making one.
    Database(#[from] crate::database::Error),
    Controller(#[from] crate::controller::Error),
    #[error("I/O error occurred: {0}")]
    Io(#[from] Arc<std::io::Error>),
    // That is kind of a catch-all variant
    #[error(transparent)]
    Other(#[from] Arc<anyhow::Error>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(Arc::new(err))
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(Arc::new(err))
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse<'a> {
    // TODO: Define the appropriate fields for the error response
    // it will be serialized into JSON and pushed to the client.
    // I think it will be error-agnostic, meaning each variant will
    // produce the client error of the same structure.
    pub message: Cow<'a, str>,
    #[serde(with = "serde_status_code")]
    pub status: axum::http::StatusCode,
}

mod serde_status_code {
    use axum::http::StatusCode;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(code.as_u16())
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = u16::deserialize(deserializer)?;
        StatusCode::from_u16(code).map_err(serde::de::Error::custom)
    }
}

impl<'a> IntoResponse for ErrorResponse<'a> {
    fn into_response(self) -> axum::response::Response {
        axum::response::IntoResponse::into_response((
            self.status,
            // This has to be the last as it consumes self.
            axum::Json(self),
        ))
    }
}

// NOTE: There is an issues converting the nested errors into the application crate-level error.
// We need to define the trait that would provide methods to convert the nested errors into the crate-level error
// as implementing the From trait is not enough to do that automatically.
//
// The idea is that each of the nested errors would implement that trait and provide the method
// for conversion, then we would be able to put that error in the Extension of the response,
// as the type of that error is embedded into the crate-level error, since that is the same type
// We need methods to provide that conversion, and,
pub trait ErrorExt
where
    Self: Sized + Clone + IntoResponse,
{
    /// Convert the variant error into the crate-level error to allow inserting it into the `Extension` of the response.
    fn to(self) -> crate::Error;

    /// Convert the variant error into the crate-level error if applicable,
    /// produces the response and adds the crate-level error into the `Extension` of the response
    /// for later logging in the middleware layer.
    // fn to_response(self) -> axum::response::Response;
    fn to_response(self, representation: ErrorResponse) -> axum::response::Response {
        let s = self.clone();
        let mut response = representation.into_response();

        // For middleware-tower logging
        response.extensions_mut().insert(Arc::new(Self::to(s)));

        return response;
    }
}

impl ErrorExt for Error {
    // NOTE: Maybe that should borrow and then clone, but I think there would be an issues with the cascade of cloning
    // of those errors, I believe I have tried it before and there is an issues, to be verified.
    fn to(self) -> crate::Error {
        self
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let message = format!("Shoot, ...: {}", self);
        tracing::error!("crate-level error: {}", message);

        let repr = match self {
            _ => ErrorResponse {
                message: Cow::from(message),
                status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
        };

        // We need to save the crate-level error into the response Extensions to log it in the middleware layer.
        // response.extensions_mut().insert::<Arc<Self>>(error);

        return self.to_response(repr);
    }
}

// NOTE: Snippet to hide the public API

// // PublicError is public, but opaque and easy to keep compatible.
// #[derive(Error, Debug)]
// #[error(transparent)]
// pub struct PublicError(#[from] ErrorRepr);

// impl PublicError {
//     // Accessors for anything we do want to expose publicly.
// }

// // Private and free to change across minor version of the crate.
// #[derive(Error, Debug)]
// enum ErrorRepr {
//     ...
// }

// pub enum Error {
//     Io(std::io::Error),
//     DotEnv(dotenv::Error),
// }

// impl std::fmt::Debug for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::Io(e) => f.debug_tuple("Io").field(e).finish(),
//             Error::DotEnv(e) => f.debug_tuple("DotEnv").field(e).finish(),
//         }
//     }
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::Io(_) => write!(f, "I/O error occurred"),
//             Error::DotEnv(_) => write!(f, "DotEnv error occurred"),
//         }
//     }
// }

// impl std::error::Error for Error {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             Error::Io(e) => Some(e),
//             Error::DotEnv(e) => Some(e),
//         }
//     }
// }

// impl From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Self {
//         Error::Io(err)
//     }
// }

// impl From<dotenv::Error> for Error {
//     fn from(err: dotenv::Error) -> Self {
//         Error::DotEnv(err)
//     }
// }
