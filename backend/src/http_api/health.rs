use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode};

pub async fn live() -> StatusCode {
    StatusCode::OK
}

pub async fn ready(State(app_state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&app_state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
