use std::sync::Arc;

use axum::response::IntoResponse;

use crate::{ErrorExt, error::ErrorResponse};

#[derive(thiserror::Error, Debug, Clone)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    GenericControllerError(#[from] crate::controller::GenericControllerError),
    Other(#[from] Arc<anyhow::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        let message = self.to_string();
        
        let status = match self {
            _ => ErrorResponse::default().status,
        };

        self.to_response(ErrorResponse {
            message: message.into(),
            status,
        })
    }
}

impl ErrorExt for self::Error {
    fn to(self) -> crate::Error {
        return crate::controller::Error::from(self).into();
    }
}

// impl From<MigrateError> for Error {
//     fn from(err: MigrateError) -> Self {
//         Self::MigrateError(Arc::new(err))
//     }
// }

// impl From<sqlx::Error> for Error {
//     fn from(err: sqlx::Error) -> Self {
//         Self::ConnectionError(Arc::new(err))
//     }
// }

// impl<T> From<T> for Error
// where
//     T: Sized,
// {
//     fn from(value: T) -> Self {
//         Self::Other(Arc::new(value))
//     }
// }
