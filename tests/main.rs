mod config;
mod controller;

// Alias for constructing app with state and given connection pool.
// pub(crate) fn app(
//     pool: impl Into<rust_web_app::database::DatabaseConnection>,
// ) -> impl Future<Output = rust_web_app::Result<axum::Router>> {
//     // Current the state is just the database connection, so we can do that, later we would not if more fields transpire.
//     rust_web_app::app(AppState {
//         database: pool.into(),
//     })
// }
