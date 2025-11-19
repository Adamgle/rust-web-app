use std::{borrow::Cow, sync::Arc};

use axum::response::IntoResponse;

use crate::{
    ErrorExt,
    controller::{auth, stocks},
    error::ErrorResponse,
};

// That error seem useless, if we have a separate errors for each module, why would we need that.
// We could consider using that if some controllers would have common errors, but that seem unlikely.
// That would be something like Io, or Validation, or something like that, but that would be probably
// better to do in each controller separately.
#[derive(thiserror::Error, Debug, Clone)]
// #[error("Controller error")]
// Each error default to Internal Server Error.
// #[error("Internal Server Error")]
#[error(transparent)]
pub enum Error {
    Stocks(#[from] stocks::Error),
    Auth(#[from] auth::Error),
    GenericControllerError(#[from] GenericControllerError),
}

/// `GenericControllerError` represents general errors that can occur within controllers.
/// They are controller-module agnostic, think about that as a API layer errors, kind of like a client error
/// things that went bad because client does not adhere to the application layer rules.
///
/// I think each of the variant will have it's display message to be Internal Server Error or Bad Request, to be determined.
/// Those errors are really
#[derive(thiserror::Error, Debug, Clone)]
#[error("Internal Server Error")]
pub enum GenericControllerError {
    IdNotInPostgresSerialRange {
        id: String,
    },
    /// That would be general purpose, catch all variant for client errors when we do not want to send any specific reason
    /// for the failure, but do want to save the source of the error in variant for logging purposes.
    ///
    /// The downside of that approach is that we cannot do that:
    /// ClientErrorSelf(#[source] Option<Arc<Self>>), where Self refer to the actual error type that happen, we would have to downcast the type erased
    /// because currently we would just have the display method attach to the error, not the variant.
    ClientError {
        #[source]
        source: Option<Arc<anyhow::Error>>,
    },
    Other(#[from] Arc<anyhow::Error>),
}

impl IntoResponse for GenericControllerError {
    fn into_response(self) -> axum::response::Response {
        let message = Cow::Owned(self.to_string());
        let status = axum::http::StatusCode::BAD_REQUEST;

        return self.to_response(ErrorResponse { message, status });
    }
}

impl ErrorExt for GenericControllerError {
    fn to(self) -> crate::Error {
        return crate::Error::Controller(self.into());
    }
}
