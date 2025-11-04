pub mod auth;
mod error;
pub mod stocks;

pub use error::Error;

/// General API response types
pub mod types {
    #[derive(serde::Serialize)]
    pub struct ApiStatusResponse {
        pub status: bool,
    }

    #[derive(serde::Serialize)]
    pub struct ApiMessageResponse {
        pub status: bool,
        pub message: String,
    }
}

/// Cookie names used across the application, maybe they should also map to a value type.
pub mod cookies {
    pub const SSID: &str = "SSID";
}

// I do not see the use of Result from the controller module itself.
// pub(in crate::controller) type Result<T> = std::result::Result<T, self::Error>;
