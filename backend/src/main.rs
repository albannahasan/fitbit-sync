use crate::app_state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

mod app_state;
mod config;
mod http_api;
mod meals;
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
        .route("/health/ready", get(http_api::health::ready))
        .route("/meals", post(http_api::meal::create_meal))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Http Server Failed");
}
