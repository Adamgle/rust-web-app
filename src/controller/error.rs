use crate::controller::{auth, stocks};

// That error seem useless, if we have a separate errors for each module, why would we need that.
// We could consider using that if some controllers would have common errors, but that seem unlikely.
// That would be something like Io, or Validation, or something like that, but that would be probably
// better to do in each controller separately.
#[derive(thiserror::Error, Debug, Clone)]
// #[error("Controller error")]
// Each error default to Internal Server Error.
#[error("Internal Server Error")]
pub enum Error {
    // #[error("Stocks controller error: {0}")]
    Stocks(#[from] stocks::Error),
    // #[error("Authentication controller error: {0}")]
    Auth(#[from] auth::Error),
    GenericControllerError(#[from] GenericControllerError),
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Internal Server Error")]
/// `GenericControllerError` represents general errors that can occur within controllers.
/// They are controller-module agnostic, think about that as a API layer errors, kind of like a client error
/// thinks that went bad because client does not adhere to the application layer rules.
pub enum GenericControllerError {
    IdNotInPostgresSerialRange { id: String },
}
