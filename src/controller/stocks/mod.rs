pub fn router() -> axum::Router {
    axum::Router::new().route("/api/v1/stocks", axum::routing::get(get_stocks))
}

#[axum::debug_handler]
pub async fn get_stocks() -> impl axum::response::IntoResponse {
    // use axum::{Router, routing::get};

    // // define some routes separately
    // let user_routes = Router::new()
    //     .route("/users", get(users_list))
    //     .route("/users/{id}", get(users_show));

    // let team_routes = Router::new().route("/teams", get(teams_list));

    // // combine them into one
    // let app = Router::new().merge(user_routes).merge(team_routes);

    // could also do `user_routes.merge(team_routes)`

    // Our app now accepts
    // - GET /users
    // - GET /users/{id}
    // - GET /teams

    // let stocks = sqlx::query!("SELECT * FROM stocks")
    //     .fetch_all(&crate::config::DB_POOL)
    //     .await
    //     .unwrap_or_default();
    // axum::Json(serde_json::json!([
    //     { "id": 1, "name": "Product 1", "price": 10.0 },
    //     { "id": 2, "name": "Product 2", "price": 20.0 },
    //     { "id": 3, "name": "Product 3", "price": 30.0 }
    // ]))

    String::new()
}
