use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug)]
#[error("Stocks error")]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] crate::database::Error),
    // I am not sure if we want to push error message in that error
    // to the client, at least not something like that
    // The display message of id being not in range is not something
    // client needs or should to see.
    IdNotInPostgresSerialRange {
        // Do that as a string as if the number did not fit into
        // postgres serial range, maybe it would also not fit
        // in u32, or i32, cautions.
        id: String,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: Do the error handling when the application matures.

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
