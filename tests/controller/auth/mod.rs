// Integration tests for the config/mod.rs
// TODO: We need to figure out a better way for structuring the integration tests
// If I put the files in separate directories the analyzer does not link them.

#![allow(unused)]

// Router::new()
// .route("/auth/session", get(get_auth_session))
// .route("/auth/register", post(register_user))
// .route("/auth/login", post(login_user))
// .route("/auth/logout", post(logout_user))

// #[tokio::test]

use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{self, Method, Request},
};
use futures::{StreamExt, TryStreamExt};
use reqwest::header;
use rust_web_app::{
    AppState, controller::auth::ClientAuthenticationCredentials, database::DatabaseConnection,
};
use tower::ServiceExt;
use tracing::{debug, error, info, warn};

// #[sqlx::test]
// #[serial_test::serial]
// #[tracing_test::traced_test]
// async fn arbitrary_testing(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
//     let mut conn = pool.acquire().await?;

//     // let app = rust_web_app::app().await?;

//     assert!(
//         sqlx::query!("SELECT 1 as test")
//             .fetch_one(conn.as_mut())
//             .await?
//             .test
//             == Some(1),
//     );

//     let mut rows = sqlx::query!(
//         r###"
//     SELECT table_schema, table_name
//     FROM information_schema.tables
//     WHERE table_type = 'BASE TABLE'
//       AND table_schema NOT IN ('pg_catalog', 'information_schema')
//     ORDER BY table_schema, table_name;
//     "###
//     )
//     .fetch(conn.as_mut()); // returns a Stream instead of Vec

//     while let Some(row) = rows.next().await {
//         let row = row?;
//         let Some(table_name) = row.table_name else {
//             continue;
//         };

//         if table_name == "stocks" {
//             continue;
//         }

//         let mut conn = pool.acquire().await?;

//         let query = format!("SELECT * FROM {}", table_name);

//         sqlx::query(&query)
//             .fetch_all(conn.as_mut())
//             .await
//             .map(|entries| {
//                 if entries.is_empty() {
//                     info!("Table '{}' is empty.", table_name);
//                     return;
//                 }

//                 info!("{table_name} {entries:#?}");
//             })?;

//         conn.close().await?;
//     }

//     drop(rows);

// sqlx::query!("SELECT current_database() as db_name")
//     .fetch_one(conn.as_mut())
//     .await
//     .map(|row| {
//         let db_name = row.db_name.unwrap_or_default();
//         info!("Connected to test database: {}", db_name);
//     })?;

//     let account_id =
//         sqlx::query!("INSERT INTO accounts (created_at) VALUES (DEFAULT) RETURNING id")
//             .fetch_one(conn.as_mut())
//             .await?
//             .id;

//     sqlx::query!("INSERT INTO users (email, password_hash, account_id) VALUES ('email@gmail.com', 'hashed_password', $1)", account_id)
//         .execute(conn.as_mut())
//         .await?;

//     let mut conn = pool.acquire().await?;

//     // NOTE: You have to isolate that query and call to next because it will deadlock otherwise, thought that should be clear why would you not do that
//     // as you would do multiple queries in a row without consuming the stream, duh.
//     let mut rows = sqlx::query!("SELECT * FROM stocks").fetch(conn.as_mut());

//     while let Some(entry) = rows.next().await {
//         let entry = entry?;
//     }

//     Ok(())
// }

// #[sqlx::test(migrations = "../migrations")]
#[sqlx::test(migrations = "./migrations")]
#[serial_test::serial]
#[tracing_test::traced_test]
async fn test_register(pool: sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    // To test the registration flow we need to:
    // 1. Check for each variant of the error that can happen during registration.
    // 2. We need to setup a test database connection and apply migrations so we would have
    // the tables ready for the tests, I would assume that the tables should be empty, we could also delegate migrations designed for testing.
    // We need to assure that non of the test affects another test, it would probably have to be run with --threads-count 1 or serial_test::serial.

    // let current_database = sqlx::query!("SELECT current_database() as db_name")
    //     .fetch_one(&pool)
    //     .await?
    //     .db_name
    //     .unwrap_or_default();

    // if current_database == "stocked" {
    //     panic!("Tests must not be run against the production database!");
    // }

    // let drop_database = format!("DROP DATABASE IF EXISTS {}", current_database);
    // sqlx::query(&drop_database).execute(&pool).await.map(|_| {
    //     info!("Dropped test database: {}", current_database);
    // })?;

    let mut conn = pool.acquire().await?;

    // let mut rows = sqlx::query!(
    //     r###"
    // SELECT table_schema, table_name
    // FROM information_schema.tables
    // WHERE table_type = 'BASE TABLE'
    //   AND table_schema NOT IN ('pg_catalog', 'information_schema')
    // ORDER BY table_schema, table_name;
    // "###
    // )
    // .fetch(conn.as_mut()); // returns a Stream instead of Vec

    // while let Some(row) = rows.next().await {
    //     let row = row?;
    //     let Some(table_name) = row.table_name else {
    //         continue;
    //     };

    //     let mut conn = pool.acquire().await?;

    //     let query = format!("SELECT * FROM {}", table_name);

    //     sqlx::query(&query)
    //         .fetch_all(conn.as_mut())
    //         .await
    //         .map(|entries| {
    //             if entries.is_empty() {
    //                 info!("Table '{}' is empty.", table_name);
    //                 return;
    //             }

    //             info!("{table_name} {entries:#?}");
    //         })?;

    //     conn.close().await?;
    // }

    // drop(rows);

    sqlx::query!("SELECT current_database() as db_name")
        .fetch_one(conn.as_mut())
        .await
        .map(|row| {
            let db_name = row.db_name.unwrap_or_default();
            info!("Connected to test database: {}", db_name);
        })?;

    info!(
        "All users in database: {:#?}",
        sqlx::query!("SELECT * FROM users")
            .fetch_all(conn.as_mut())
            .await?
    );

    let app = rust_web_app::app(AppState::new(pool)).await?;

    let payload = serde_json::to_string(&ClientAuthenticationCredentials {
        email: "valid@email.com".into(),
        password: "Password1!".into(),
    })?;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload))?;

    // This talks to the production database.
    let response = app.oneshot(request).await?;

    info!(
        "All users in database: {:#?}",
        sqlx::query!("SELECT * FROM users")
            .fetch_all(conn.as_mut())
            .await?
    );

    info!("Response: {:#?}", response);

    assert!(response.status().is_success());

    Ok(())
}

#[sqlx::test]
async fn test_login() -> anyhow::Result<()> {
    unimplemented!()
}

#[sqlx::test]
async fn test_logout() -> anyhow::Result<()> {
    unimplemented!()
}

#[sqlx::test]
async fn test_auth_session() -> anyhow::Result<()> {
    unimplemented!()
}
