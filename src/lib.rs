#![allow(clippy::needless_return)]

use std::{io::Write, process::Stdio, sync::Arc};

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
use sqlx::{Pool, Postgres};
use tokio::{io::AsyncBufReadExt, sync::broadcast};

use crate::{config::Config, database::DatabaseConnection};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub tx_tickers: tokio::sync::broadcast::Sender<String>,
    // caches: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>,
}

#[derive(serde::Deserialize)]
struct StreamedTicker {
    // market_hours: "",
    // exchange: String,
    // TIMESTAMP
    // time: usize,
    // NOTE: Not sure what Timestamp is streams.
    time: chrono::NaiveDateTime,
    change_percent: Option<f32>,
    change: Option<f32>,
    // quote_type: ""
    price: f32,
    // price_hint: "",
    id: String,
}

// type SerializedStreamedTicker = String;

// NOTE: Stocks tickers streaming MUST NOT be an endpoint, it should be a processes that runs for the server, uniform
// for all users. The reason is that it would bloat the API and start the websocket connection for each user.
//  -> self::Result<impl Stream>
fn stream_tickers(tx: broadcast::Sender<String>, DatabaseConnection(conn): &DatabaseConnection) {
    // let mut tickers_buffer: DashMap<String, StreamedTicker> = HashMap::new();
    // let tickers_buffer: Arc<DashMap<String, Vec<StreamedTicker>>> = Arc::default();
    // let tickers_buffer_c = Arc::clone(&tickers_buffer);
    let (db_tx, mut db_rx) = tokio::sync::mpsc::channel::<StreamedTicker>(10_000);

    tokio::spawn(async move {
        tracing::info!("Streaming stocks tickers...");
        // The idea is that we will run the python script that uses `yfinance library` and we would redirect the stream of
        // that script

        // python C:\\Dev\\Rust\\rust-web-app\\scripts\\stream_stocks.py
        // NOTE: If the program path is relative (e.g., "./script.sh"), it’s ambiguous whether it should be interpreted
        // relative to the parent’s working directory or relative to current_dir. The behavior in this
        // case is platform specific and unstable, and it’s recommended to use canonicalize to get an absolute program path instead.

        let path = std::path::Path::new("./scripts/stream_stocks.py")
            .canonicalize()
            .expect("Failed to get the absolute path of the script");

        let mut output = tokio::process::Command::new("python")
            .arg("-u")
            .arg(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn stream_stocks.py");

        let stdout = output.stdout.take().expect("stdout drained");
        let stderr = output.stderr.take().expect("stderr drained");

        let reader = tokio::io::BufReader::new(stdout);

        let mut lines = reader.lines();

        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                tracing::warn!("{:?} stderr: {}", &path, line);
            }

            // Channel for buffering the tickers and used in separate task to flush to database.
        });

        // We will deserialize the tickers in single operation, maybe we will use `rayon` to do that efficiently
        // as that will be the operation done in bulk, all we care is performance there.
        // UPDATE: It is better to do the tokio::task::spawn_blocking
        while let Ok(Some(ticker)) = lines.next_line().await {
            let t = serde_json::from_str::<StreamedTicker>(&ticker)
                .unwrap_or_else(|e| panic!("Could not deserialize the ticker: {ticker}\n{e}"));

            match tx.send(ticker) {
                Ok(receivers) => {
                    tracing::info!("Current subscribed receivers: {receivers}");

                    // tickers_buffer
                    //     .entry(t.id.clone())
                    //     .and_modify(|e| e.push(t))
                    //     .or_default();
                }
                Err(e) => {
                    tracing::warn!("Ticker was not sent through the broadcast channel: {e:?}");
                }
            }

            if let Err(e) = db_tx.send(t).await {
                tracing::warn!("Ticker was not sent through the database mpsc channel: {e:?}");
            }
        }
    });

    // Process the buffered tickers and flush them to database.

    let c = conn.clone();
    let mut buffer = Vec::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    tokio::task::spawn(async move {
        loop {
            tokio::select! {
                Some(ticker) = db_rx.recv() => {
                    buffer.push(ticker);
                    if buffer.len() >= 500 {
                        flush_streamed_tickers(&c, &mut buffer).await;
                        buffer.clear()

                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        flush_streamed_tickers(&c, &mut buffer).await;
                        buffer.clear()
                    }
                }
            }
        }
    });

    // tokio::time::timeout(tokio::time::Duration::from_secs(10), worker);
}

async fn flush_streamed_tickers(conn: &Pool<Postgres>, buffer: &mut Vec<StreamedTicker>) {
    tracing::info!("Flushing the buffered streamed buffer to database");

    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = conn
        .begin()
        .await
        .expect("Failed to start a database transaction.");

    for ticker in buffer {
        let stock = sqlx::query!(
                    "UPDATE stocks SET abbreviation = $1, price = $2, change_percent = $3, change = $4, last_update = $5 WHERE abbreviation = $6 RETURNING stocks.id",
                    // NOTE: Abbreviation should not be modified with each tick, but will be useful for now to fill the data to database
                    // as it is not there yet.
                    ticker.id,
                    ticker.price,
                    ticker.change_percent,
                    ticker.change,
                    ticker.time,
                    ticker.id
                ).fetch_one(tx.as_mut()).await.expect("Updating `stocks` table failed");

        sqlx::query!(
            "UPDATE stocks_history SET price = $1, time = $2 WHERE stock_id = $3",
            ticker.price,
            ticker.time,
            stock.id
        )
        .fetch_one(tx.as_mut())
        .await
        .expect("Updating `stocks_history` table failed");
    }

    tx.commit()
        .await
        .expect("Failed to commit a database transaction.");
}

impl AppState {
    const TX_TICKERS_CAPACITY: usize = usize::pow(2, 8);

    pub fn new(database: impl Into<DatabaseConnection>) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(Self::TX_TICKERS_CAPACITY);

        Self {
            database: database.into(),
            tx_tickers: tx,
        }
    }

    /// Create a default AppState by initializing the database connection.
    /// This is useful for production use where we want to create the state
    pub async fn default() -> crate::Result<Self> {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(Self::TX_TICKERS_CAPACITY);

        Ok(Self {
            database: DatabaseConnection::new().await?,
            tx_tickers: tx,
        })
    }
}

pub async fn run(_config: config::Config) -> crate::Result<()> {
    let listener = tokio::net::TcpListener::bind(Config::APP_SOCKET_ADDR).await?;

    tracing::debug!("Listening on {}", Config::APP_SOCKET_ADDR);

    // None, because it defaults to creating database already in the app function, it is easier this way to test using `app`.
    let state = AppState::default().await?;
    // TODO: That is critical service, though failure of that service should not terminate the whole server, but it does in current design.
    stream_tickers(state.tx_tickers.clone(), &state.database);

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

                    tracing::info_span!("request", %method, %uri, matched_path)
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
                        std::fs::remove_file(path)
                            .inspect_err(|e| error!("Could not remove log file: {e:?}"))
                            .ok();
                    }
                }
            }

            let now = chrono::Utc::now()
                .naive_utc()
                // To the Winter CET
                .checked_add_offset(chrono::FixedOffset::east_opt(60 * 60).unwrap())
                .unwrap();

            // The name of the file might be confusing because we are deleting old log files based on the
            // creation of the file, not using the timestamp as the filename.
            // Thought we could keep some server side timestamp and write to the file that is the closed to the current timestamp,
            // lets say deviated by 1 hour both ways.

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
