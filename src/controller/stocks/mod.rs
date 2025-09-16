mod error;
pub use error::Error;
use tracing::info;

use crate::{AppState, database::DatabaseConnection};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

pub(in crate::controller::stocks) type Result<T> = std::result::Result<T, self::Error>;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/api/v1/stocks", axum::routing::get(get_stocks))
        .route("/api/v1/stocks/{id}", axum::routing::get(get_stock))
}

// //`Path` gives you the path parameters and deserializes them.
// async fn path(Path(user_id): Path<u32>) {}

// // `Query` gives you the query parameters and deserializes them.
// async fn query(Query(params): Query<HashMap<String, String>>) {}

// // Buffer the request body and deserialize it as JSON into a
// // `serde_json::Value`. `Json` supports any type that implements
// // `serde::Deserialize`.
// async fn json(Json(payload): Json<serde_json::Value>) {}

// Not a handler.
// <T: DeserializeOwned + Send + Sync>(
async fn list_stocks(
    DatabaseConnection(conn): DatabaseConnection,
    // State(DatabaseConnection): State<DatabaseConnection>,
) -> self::Result<Vec<serde_json::Value>> {
    // TODO: Consider reading queries from file
    // let account = sqlx::query_file!("tests/test-query-account-by-id.sql", 1i32)
    //     .fetch_one(&mut conn)
    //     .await?;

    let stocks = sqlx::query!("SELECT * FROM stocks")
        .fetch_all(&conn)
        .await
        .map_err(crate::database::Error::from)
        .map_err(self::Error::from)?;

    for row in stocks.iter() {
        println!("Stock row: {:?}", row);
    }

    // let result = database
    //     .execute("SELECT stocks.abbreviation FROM stocks")
    //     .await?;

    // println!("Database connection: {:?}", database);

    // let map = query!("SELECT * FROM stocks");
    // let map2 = query!("SELECT created_at FROM users");

    // let rows = map.fetch_all(&database).await?;
    // let rows2 = map2.fetch_all(&database).await?;

    // println!("Query result: {:#?}", result);
    // println!("Query map: {:#?}", rows);
    // println!("Query map2: {:#?}", rows2);

    Ok(vec![
        serde_json::json!({ "id": 1, "name": "Product 1", "price": 10.0 }),
        serde_json::json!({ "id": 2, "name": "Product 2", "price": 20.0 }),
        serde_json::json!({ "id": 3, "name": "Product 3", "price": 30.0 }),
    ])
}

#[axum::debug_handler]
pub async fn get_stocks(
    State(conn): State<DatabaseConnection>,
    // axum::extract::State(AppState { database }): axum::extract::State<AppState>,
    axum::extract::Path(()): axum::extract::Path<()>,
) -> self::Result<impl IntoResponse> {
    let stocks = self::list_stocks(conn).await?;

    Ok(axum::Json(stocks))
}

pub async fn get_stock(
    Path(id): Path<u32>,
    State(conn): State<DatabaseConnection>,
) -> self::Result<impl IntoResponse> {
    let stocks = self::list_stocks(conn).await?;

    info!("Looking for stock with id: {}", id);

    Ok(Json(
        stocks
            .into_iter()
            .find(|stock| stock.get("id") == Some(&serde_json::json!(id)))
            .unwrap(),
    ))
}
