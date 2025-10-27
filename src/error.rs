// The convention would be to keep the Error enum per module, and if that Error enum
// needs to abstract some errors in a separate container, like another enum,
// it will end with the `Error` suffix, so we can import it in other modules
// without referring to the module, like config::EnvError. This way
// you can bring something to scope without referring to the module and know that something is and error.
// The main Error enum will be always referred to with the module, like config::Error.

use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug)]
#[error("Config error occurred: {0}")]
pub enum Error {
    // The idea is variants per module that wrap it's inner errors.
    Config(#[from] crate::config::Error),
    // There is not module called database. Think about making one.
    Database(#[from] crate::database::Error),
    Controller(#[from] crate::controller::Error),
    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
    // That is kind of a catch-all variant
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {

        let message = format!("Shoot, ...: {}", self);
        let response = (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message.clone(),
        );

        tracing::error!(message);
        axum::response::IntoResponse::into_response(response)
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
