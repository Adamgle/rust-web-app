mod error;
pub mod types;
use std::sync::Arc;

use axum::extract::FromRef;
pub use error::Error;
use futures::TryFutureExt;

use crate::{
    AppState,
    config::{self, Env, EnvError},
};

pub(in crate::database) type Result<T> = std::result::Result<T, self::Error>;

// We are isolating the Database to a separate struct if we would want to hold
// some state about it, do not know yet.
#[derive(Clone)]
pub struct DatabaseConnection(pub sqlx::Pool<sqlx::Postgres>);

impl DatabaseConnection {
    pub async fn new() -> self::Result<Self> {
        let conn = Self::connect().await?;

        // Run migrations, setup the tables.
        sqlx::migrate!("./migrations").run(&conn).await?;

        Ok(Self(conn))
    }

    pub async fn connect() -> self::Result<sqlx::Pool<sqlx::Postgres>> {
        // NOTE: Error as such should not happen at all, because we check for missing envs at both ends
        // we initializing envs.
        let connection_string = dotenvy::var(Env::DatabaseUrl.as_ref())
            // TODO: Check if that from congestion maps to EnvError::MissingEnv but it probably does not.
            // .map_err(EnvError::from)
            .map_err(Arc::from)
            .map_err(EnvError::MissingEnv)
            .map_err(config::Error::from)?;

        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .map_err(self::Error::from)
            .await
    }
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        // That internally clones an Arc.
        state.database.clone()
    }
}
