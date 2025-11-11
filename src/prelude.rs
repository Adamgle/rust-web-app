pub type Result<T> = std::result::Result<T, crate::error::Error>;
pub use crate::error::Error;
pub use crate::error::ErrorExt;

pub use tracing::{debug, error, info, warn};
