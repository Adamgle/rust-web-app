use std::sync::Arc;

#[derive(thiserror::Error, Debug, Clone)]
#[error("Database error occurred: {0}")]
pub enum Error {
    ConnectionError(#[from] Arc<sqlx::Error>),
    InvalidDatabaseConfiguration(#[from] crate::config::Error),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::ConnectionError(Arc::new(err))
    }
}
