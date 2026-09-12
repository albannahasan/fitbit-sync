use crate::app_state::AppState;
use axum::{Router, routing::get};

mod app_state;
mod config;
mod http_api;
mod postgres;

#[tokio::main]
async fn main() {
    let config = config::load();
    let pool = postgres::create_pool(&config.database_url)
        .await
        .expect("Database connection failed");
    println!("Configuration loaded.");

    // Create one appstate using pool
    let app_state = AppState { db: pool };

    //Use router with appstate
    let app: Router = Router::<AppState>::new()
        .route("/health/live", get(http_api::health::live))
        .with_state(app_state);
}
