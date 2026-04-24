mod error;
pub use error::Error;
use tracing::info;

use crate::{
    AppState,
    controller::GenericControllerError,
    database::{DatabaseConnection, types::Stock},
};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

pub(in crate::controller::stocks) type Result<T> = std::result::Result<T, self::Error>;

// <S: Clone + Send + Sync + 'static>
pub fn router() -> axum::Router<AppState>
// where
    // DatabaseConnection: FromRef<S>,
{
    axum::Router::new()
        .route("/stocks", axum::routing::get(get_stocks))
        .route("/stocks/{id}", axum::routing::get(get_stock))
        .route("/sse", axum::routing::get(sse_handler))
    // SSE handler
    // .route("/sse/stream", axum::routing::get(stream_stocks))
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
    // We should just SELECT from the database here, but do not bother changing that as that will be removed as in its current form
    // is useless.

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

use axum::response::sse::{Event, Sse};
use tokio_stream::StreamExt as _;

// Sse<impl stream::Stream<Item = std::result::Result<Event, std::convert::Infallible>>>
async fn sse_handler(State(state): State<AppState>) -> self::Result<impl IntoResponse> {
    let rx = state.tx_tickers.subscribe();

    tracing::debug!(
        "TX-RX metadata: tx = {tx:?} | rx = {rx:?}",
        tx = state.tx_tickers.clone()
    );

    tracing::info!("New SSE client subscribed.");

    // match rx.recv().await {
    //     Ok(msg) => { /* normal */ }
    //     Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
    //         tracing::warn!("receiver lagged, skipped {} messages", n);
    //     }
    //     Err(tokio::sync::broadcast::error::RecvError::Closed) => {
    //         tracing::info!("channel closed");
    //     }
    // }

    // We need to set Access-Control-Allow-Origin to the Env::ClientUrl to avoid cors issues.

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .map(|l| Ok::<Event, self::Error>(l.map(|l| Event::default().data(l)).unwrap()));

    // let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|msg| async move {
    //     match msg {
    //         Ok(line) => Some(Ok(Event::default().data(line))),
    //         Err(_) => None, // lagged / closed
    //     }
    // });

    // let sse = Sse::new(stream).keep_alive(KeepAlive::default());
    // let mut response = sse.into_response();

    // let client_url = dotenvy::var(Env::ClientUrl.as_ref())
    //     .map_err(|e| Error::Other(Arc::new(anyhow::Error::new(EnvError::from(e)))))?;

    // response.headers_mut().insert(
    //     http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
    //     HeaderValue::from_str(client_url.as_str())
    //         .map_err(|e| self::Error::Other(Arc::new(anyhow::Error::new(e))))?,
    // );

    Ok(Sse::new(stream))
}
