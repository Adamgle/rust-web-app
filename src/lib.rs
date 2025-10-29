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

    let api_router = Router::new()
        .merge(controller::stocks::router())
        .merge(controller::auth::router());

    let app = Router::<AppState>::new()
        .nest("/api/v1", api_router)
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
        .with_state(AppState { database });

    let listener = tokio::net::TcpListener::bind(Config::APP_SOCKET_ADDR).await?;

    tracing::debug!("Listening on {}", Config::APP_SOCKET_ADDR);

    axum::serve(listener, app).await?;

    Ok(())
}

// Our middleware is responsible for logging error details internally
async fn log_app_errors(request: axum::extract::Request, next: Next) -> axum::response::Response {
    let response = next.run(request).await;

    // If the response contains an AppError Extension, log it.
    if let Some(err) = response.extensions().get::<Arc<self::Error>>() {
        let message = format!("Shoot, ...: {}", err);
        tracing::error!(?err, %message);
    }

    return response;
}

// pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
//     // Bootstrapping an API is both more intuitive with Axum than Actix-web but also
//     // a bit more confusing at the same time.
//     //
//     // Coming from Actix-web, I would expect to pass the router into `ServiceBuilder` and not
//     // the other way around.
//     //
//     // It does look nicer than the mess of `move || {}` closures you have to do with Actix-web,
//     // which, I suspect, largely has to do with how it manages its own worker threads instead of
//     // letting Tokio do it.
//     let app = api_router().layer(
//         ServiceBuilder::new()
//             // The other reason for using a single object is because `AddExtensionLayer::new()` is
//             // rather verbose compared to Actix-web's `Data::new()`.
//             //
//             // It seems very logically named, but that makes it a bit annoying to type over and over.
//             .layer(AddExtensionLayer::new(ApiContext {
//                 config: Arc::new(config),
//                 db,
//             }))
//             // Enables logging. Use `RUST_LOG=tower_http=debug`
//             .layer(TraceLayer::new_for_http()),
//     );

//     // We use 8080 as our default HTTP server port, it's pretty easy to remember.
//     //
//     // Note that any port below 1024 needs superuser privileges to bind on Linux,
//     // so 80 isn't usually used as a default for that reason.
//     axum::Server::bind(&"0.0.0.0:8080".parse()?)
//         .serve(app.into_make_service())
//         .await
//         .context("error running HTTP server")
// }

// fn api_router() -> Router {
//     // This is the order that the modules were authored in.
//     users::router()
//         .merge(profiles::router())
//         .merge(articles::router())
// }

// mod tests {
//     #[allow(unused_imports)]
//     use super::*;

//     fn sqrt(number: f64) -> anyhow::Result<f64> {
//         if number >= 0.0 {
//             Ok(number.powf(0.5))
//         } else {
//             Err(anyhow::anyhow!("negative floats don't have square roots"))
//         }
//     }

//     #[cfg(test)]
//     mod tests {
//         use super::*;

//         #[test]
//         fn test_sqrt() -> anyhow::Result<()> {
//             let x = -4.0;
//             assert_eq!(sqrt(x)?.powf(2.0), x);
//             Ok(())
//         }
//     }
// }
