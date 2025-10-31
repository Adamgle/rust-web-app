use std::sync::Arc;

#[derive(thiserror::Error, Debug, Clone)]
// NOTE: Each error default to Internal Server Error on the display impl as we want to avoid leaking sensitive information.
#[error("Internal Server Error")]
pub enum Error {
    // NOTE: I think the sqlx::Error may leak to client.
    ConnectionError(#[from] Arc<sqlx::Error>),
    InvalidDatabaseConfiguration(#[from] crate::config::Error),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::ConnectionError(Arc::new(err))
    }
}
