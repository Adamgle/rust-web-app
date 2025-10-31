use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug, Clone)]
#[error("Stocks error")]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    GenericControllerError(#[from] crate::controller::error::GenericControllerError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
