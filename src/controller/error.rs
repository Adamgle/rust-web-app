use crate::controller::stocks;

// That error seem useless, if we have a separate errors for each module, why would we need that.
// We could consider using that if some controllers would have common errors, but that seem unlikely.
// That would be something like Io, or Validation, or something like that, but that would be probably
// better to do in each controller separately.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Stocks controller error: {0}")]
    ControllerStocks(#[from] stocks::Error),
    // #[error("Some other controller error: {0}")]
    #[error("Some other controller error")]
    ControllerSomethingElse, // Placeholder for future errors
}
