use axum::response::IntoResponse;

use crate::{ErrorExt, error::ErrorResponse};

#[derive(thiserror::Error, Debug, Clone)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    GenericControllerError(#[from] crate::controller::GenericControllerError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        // let message = self.to_string();
        self.to_response(ErrorResponse::default())
    }
}

impl ErrorExt for self::Error {
    fn to(self) -> crate::Error {
        return crate::controller::Error::from(self).into();
    }
}
