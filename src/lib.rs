#![allow(clippy::needless_return)]

use std::{io::Write, sync::Arc};

pub use prelude::*;

pub mod config;
pub mod controller;
pub mod database;
mod error;
pub mod logger;
pub mod prelude;

use axum::{
    Router,
    extract::{FromRef, MatchedPath},
    http::Request,
    middleware::{Next, from_fn},
};

use crate::{config::Config, database::DatabaseConnection};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: DatabaseConnection,
    // caches: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
}

impl AppState {
    pub fn new(database: impl Into<DatabaseConnection>) -> Self {
        Self {
            database: database.into(),
        }
    }

    /// Create a default AppState by initializing the database connection.
    /// This is useful for production use where we want to create the state
    pub async fn default() -> crate::Result<Self> {
        Ok(Self {
            database: DatabaseConnection::new().await?,
        })
    }
}

pub async fn run(_config: config::Config) -> crate::Result<()> {
    let listener = tokio::net::TcpListener::bind(Config::APP_SOCKET_ADDR).await?;

    tracing::debug!("Listening on {}", Config::APP_SOCKET_ADDR);

    // None, because it defaults to creating database already in the app function, it is easier this way to test using `app`.
    let state = AppState::default().await?;

    let app = app(state).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn routes<S: Clone + Send + Sync + 'static>(state: AppState) -> self::Result<Router<S>> {
    let router = Router::new()
        .merge(controller::stocks::router())
        .merge(controller::auth::router())
        .with_state(state);

    Ok(router)
}

pub async fn app(state: AppState) -> self::Result<Router> {
    Ok(Router::new()
        .nest("/api/v1", routes(state).await?)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &Request<axum::body::Body>| {
                    let method = req.method();
                    let uri = req.uri();

                    let matched_path = req.extensions().get::<MatchedPath>().map(|mp| mp.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                }) // Do nothing on failure as we already handling the failures in our own span
                .on_failure(()),
        )
        .layer(from_fn(log_app_errors))
        .layer(tower_cookies::CookieManagerLayer::new()))
}

/// Middleware that logs application errors found in responses.
///
/// It logs internal errors, not exposed to the client, as well as the one that
/// are using the implementation of the `std::fmt::Display` trait.
async fn log_app_errors(request: axum::extract::Request, next: Next) -> axum::response::Response {
    let response = next.run(request).await;

    // If the response contains an AppError Extension, log it.
    if let Some(err) = response.extensions().get::<Arc<self::Error>>() {
        let message = format!("Shoot, ...: {}", err);

        // !!! THAT IF FOR TESTING ONLY !!!
        #[allow(unused_unsafe)]
        unsafe {
            let mut path = std::path::PathBuf::from("logs");

            // Remove 5 oldest error log files;
            let logs = std::fs::read_dir(&path);
            if let Ok(mut logs) = logs {
                let mut created = Vec::new();

                while let Some(Ok(entry)) = logs.next() {
                    let metadata = entry.metadata();
                    if let Ok(metadata) = metadata
                        && metadata.is_file()
                    {
                        let file_name = entry.file_name();
                        let file_name = file_name.to_string_lossy();
                        if let Ok(created_time) = metadata.created()
                            && file_name.starts_with("error-")
                            && file_name.ends_with(".log")
                        {
                            created.push((entry.path(), created_time));
                        }
                    }
                }

                // Sort by creation time and remove oldest files, we should just sort by name as that is timestamped.
                created.sort_by_key(|(_, time)| *time);

                let mut created = created.into_iter();
                while created.len() > 5 {
                    if let Some((path, _)) = created.next() {
                        std::fs::remove_file(path).ok();
                    }
                }
            }

            let now = chrono::Utc::now().naive_utc();
            let now = now.format("%Y-%m-%d %H").to_string();

            let filename = format!("error-{}.log", now);
            path.push(filename);

            let logs = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path);

            if let Ok(mut file) = logs {
                let message = format!("err = {:?}, message = {}\r\n\r\n", err, message);
                file.write_all(message.as_bytes()).ok();
            };
        }
        // !!! THAT IF FOR TESTING ONLY !!!

        tracing::error!(?err, %message);
    }

    return response;
}
