mod error;
pub use error::Error;
use tower_http::ServiceExt;
use tracing::info;

use crate::{
    config::{self, Env, EnvError},
    controller::GenericControllerError,
    database::{DatabaseConnection, types::Stock},
};
use axum::{
    extract::{FromRef, Json, Path, State},
    http::{self, HeaderValue},
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
        // SSE handler
        .route("/sse", axum::routing::get(sse_handler))
}

// Not a handler.
// <T: DeserializeOwned + Send + Sync>(
async fn list_stocks(DatabaseConnection(conn): DatabaseConnection) -> self::Result<Vec<Stock>> {
    // TODO: Consider reading queries from file
    // let account = sqlx::query_file!("tests/test-query-account-by-id.sql", 1i32)
    //     .fetch_one(&mut conn)
    //     .await?;

    // "SELECT * FROM users WHERE hash = " + (&str &hash) => ""

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
    Path(id): Path<String>,
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

use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self};
use std::{sync::Arc, time::Duration};
use tokio_stream::StreamExt as _;

async fn sse_handler() -> Result<impl IntoResponse> {
    // A `Stream` that repeats an event every second
    // This would suppose to stream the stock market data, thought if there are many stocks on the server, and it will be,
    // it would be up to the client to filter those which is unacceptable because of the poor performance, we should probably filter
    // on the server based on the user that is logged in consider what stocks user does own.
    // That is of course relevant to the wallet, it would behave differently if user just want to search through available stocks

    let stream = stream::repeat_with(|| {
        use rand::prelude::*;

        let mut rng: ThreadRng = rand::rng();

        // Generate and shuffle a sequence:
        let mut nums: Vec<i32> = (1..100).collect();
        nums.shuffle(&mut rng);

        // And take a random pick (yes, we didn't need to shuffle first!):
        let _ = nums.choose(&mut rng);

        #[derive(serde::Serialize)]
        struct Data(Vec<i32>);

        let data = Data(nums);

        Event::default().json_data(data).unwrap()
    })
    .map(Ok::<Event, self::Error>)
    .throttle(Duration::from_millis(100));

    // We need to set Access-Control-Allow-Origin to the Env::ClientUrl to avoid cors issues.

    let sse = Sse::new(stream).keep_alive(KeepAlive::default());
    let mut response = sse.into_response();

    let client_url = dotenvy::var(Env::ClientUrl.as_ref())
        .map_err(|e| Error::Other(Arc::new(anyhow::Error::new(EnvError::from(e)))))?;

    response.headers_mut().insert(
        http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(client_url.as_str())
            .map_err(|e| self::Error::Other(Arc::new(anyhow::Error::new(e))))?,
    );

    Ok(response)
}
