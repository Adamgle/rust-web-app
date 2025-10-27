use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Missing session cookie")]
    MissingSessionCookie,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
