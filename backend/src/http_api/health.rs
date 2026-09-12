use axum::http::StatusCode;

pub async fn live() -> StatusCode {
    StatusCode::OK
}
