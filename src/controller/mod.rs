pub mod auth;
mod error;
pub mod stocks;

pub use error::Error;

pub(in crate::controller) type Result<T> = std::result::Result<T, self::Error>;

// We will split those to separate files later probably for each controller on each table for the database
// That is still TBD.

// Also, that is the mod.rs file, probably we will keep it short and then just separate the controller files into modules
// although that kind of sound like an overkill. depends how much it will grow.
// If we would separate the controllers to modules, then we could easily test each controller separately, keep it structured.

// use axum::{Router, routing::{get, delete}, extract::Path};

// let app = Router::new()
//     .route("/", get(root))
//     .route("/users", get(list_users).post(create_user))
//     .route("/users/{id}", get(show_user))
//     .route("/api/{version}/users/{id}/action", delete(do_users_action))
//     .route("/assets/{*path}", get(serve_asset));

// async fn root() {}

// async fn list_users() {}

// async fn create_user() {}

// async fn show_user(Path(id): Path<u64>) {}

// async fn do_users_action(Path((version, id)): Path<(String, u64)>) {}

// async fn serve_asset(Path(path): Path<String>) {}
