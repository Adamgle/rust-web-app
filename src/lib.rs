pub use prelude::*;

pub mod config;
pub mod controller;
pub mod error;
pub mod logger;
pub mod prelude;

use axum::Router;
use sqlx::{Executor, query};

use crate::config::{Config, Env};

pub async fn run() -> crate::Result<()> {
    let app = Router::<()>::new()
        // .route("/", get(|| async { "Hello, World!" }))
        .merge(controller::stocks::router())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(Config::APP_SOCKET_ADDR).await?;

    // NOTE: Error as such should not happen at all, because we check for missing envs at both ends.
    let connection_string = dotenvy::var(Env::DatabaseUrl.as_ref())
        .map_err(|e| config::Error::from(config::EnvError::MissingEnv(e)))?;

    let database = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    let result = database
        .execute("SELECT stocks.abbreviation FROM stocks")
        .await?;

    println!("Database connection: {:?}", database);

    let map = query!("SELECT * FROM stocks");
    let map2 = query!("SELECT created_at FROM users");

    let rows = map.fetch_all(&database).await?;
    let rows2 = map2.fetch_all(&database).await?;

    println!("Query result: {:#?}", result);
    println!("Query map: {:#?}", rows);
    println!("Query map2: {:#?}", rows2);

    axum::serve(listener, app).await?;

    Ok(())
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
