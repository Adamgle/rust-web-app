#[derive(thiserror::Error, Debug)]
#[error("Database error occurred: {0}")]
pub enum Error {
    ConnectionError(#[from] sqlx::Error),
    InvalidDatabaseConfiguration(#[from] crate::config::Error),
}
