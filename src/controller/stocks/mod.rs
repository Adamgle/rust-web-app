mod error;
pub use error::Error;
use sqlx::types::chrono;
use tracing::info;

use crate::{controller::error::GenericControllerError, database::DatabaseConnection};
use axum::{
    extract::{FromRef, Json, Path, State},
    response::IntoResponse,
};

pub(in crate::controller::stocks) type Result<T> = std::result::Result<T, self::Error>;

pub fn router<S: Clone + Send + Sync + 'static>() -> axum::Router<S>
where
    DatabaseConnection: FromRef<S>,
{
    axum::Router::new()
        .route("/stocks", axum::routing::get(get_stocks))
        .route("/stocks/{id}", axum::routing::get(get_stock))
}

// https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html#types
#[derive(serde::Serialize)]

// TODO: Delegate the database schemas to separate module/file.
pub struct Stock {
    id: i32, // That should be unsigned, but it fails converting to u32, as postgres does not have unsigned, like a [1, 2^31 - 1]
    abbreviation: String,
    company: String,
    since: chrono::NaiveDate, // DATE
    price: f32,
    delta: f32,
    last_update: chrono::NaiveDate, // TIMESTAMP
    created_at: chrono::NaiveDate,  // TIMESTAMP
}

// Not a handler.
// <T: DeserializeOwned + Send + Sync>(
async fn list_stocks(DatabaseConnection(conn): DatabaseConnection) -> self::Result<Vec<Stock>> {
    // TODO: Consider reading queries from file
    // let account = sqlx::query_file!("tests/test-query-account-by-id.sql", 1i32)
    //     .fetch_one(&mut conn)
    //     .await?;

    // That maps the query result to the struct Stock.
    Ok(sqlx::query_as!(Stock, "SELECT * FROM stocks")
        .fetch_all(&conn)
        .await
        // Propagation casts to self::Error using #[from] crate::database::Error on self::Error
        .map_err(crate::database::Error::from)?)
}

#[axum::debug_handler]
pub async fn get_stocks(
    State(conn): State<DatabaseConnection>,
    // axum::extract::State(AppState { database }): axum::extract::State<AppState>,
    axum::extract::Path(()): axum::extract::Path<()>,
) -> self::Result<impl IntoResponse> {
    let stocks = self::list_stocks(conn).await?;

    Ok(Json(stocks))
}

pub async fn get_stock(
    // id: Result<FalliblePath>,
    Path(id): Path<String>,
    // State(AppState {
    //     database: DatabaseConnection(conn),
    // }): State<AppState>,
    State(conn): State<DatabaseConnection>,
) -> self::Result<impl IntoResponse> {
    let id = id
        .parse::<i32>()
        .map_err(|_| GenericControllerError::IdNotInPostgresSerialRange { id })
        .and_then(|id| match id < 1 {
            true => Err(GenericControllerError::IdNotInPostgresSerialRange { id: id.to_string() }),
            false => Ok(id),
        })?;

    // TODO: I do not remember what happened here, but that is clearly some temporary shenanigans of the mind.
    // We should just SELECT from the database here.

    let stocks = self::list_stocks(conn).await?;
    info!("Looking for stock with id: {}", id);

    Ok(Json(
        stocks
            .into_iter()
            // That would fail if id > i32::MAX
            .find(|stock| stock.id == id)
            .unwrap(),
    ))
}
