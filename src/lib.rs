#![allow(clippy::needless_return)]

use std::sync::Arc;

pub use prelude::*;

pub mod config;
pub mod controller;
pub mod database;
mod error;
pub mod logger;
pub mod prelude;

use axum::{
    Router,
    extract::MatchedPath,
    http::Request,
    middleware::{Next, from_fn},
};

use crate::{config::Config, database::DatabaseConnection};

#[derive(Clone)]
pub struct AppState {
    database: DatabaseConnection,
    // caches: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
}

pub async fn run(_config: config::Config) -> crate::Result<()> {
    let database = database::DatabaseConnection::new().await?;
    let listener = tokio::net::TcpListener::bind(Config::APP_SOCKET_ADDR).await?;

    tracing::debug!("Listening on {}", Config::APP_SOCKET_ADDR);

    axum::serve(listener, app(database)).await?;

    Ok(())
}

fn app_router() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(controller::stocks::router())
        .merge(controller::auth::router())
}

pub fn app(database: DatabaseConnection) -> axum::routing::IntoMakeService<Router> {
    Router::<AppState>::new()
        .nest("/api/v1", app_router())
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
        .layer(tower_cookies::CookieManagerLayer::new())
        .with_state(AppState { database })
        .into_make_service()
}

// Our middleware is responsible for logging error details internally
/// Middleware that logs application errors found in responses.
///
/// It logs internal errors, not exposed to the client, as well as the one that
/// are using the implementation of the `std::fmt::Display` trait.
async fn log_app_errors(request: axum::extract::Request, next: Next) -> axum::response::Response {
    let response = next.run(request).await;

    // If the response contains an AppError Extension, log it.
    if let Some(err) = response.extensions().get::<Arc<self::Error>>() {
        let message = format!("Shoot, ...: {}", err);
        tracing::error!(?err, %message);
    }

    return response;
}
