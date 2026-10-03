use crate::{
    app_state::AppState,
    http_api::error::ApiError,
    meals::{
        CreateMealInput,
        service::{LoggedMeal, save_meal},
    },
};
use axum::{Json, extract::State, http::StatusCode};

pub async fn create_meal(
    State(app_state): State<AppState>,
    Json(input): Json<CreateMealInput>,
) -> Result<(StatusCode, Json<LoggedMeal>), ApiError> {
    input.validate().map_err(ApiError::Validation)?;

    let meal = save_meal(&app_state.db, input)
        .await
        .map_err(|_| ApiError::Internal)?;
    return Ok((StatusCode::CREATED, Json(meal)));
}
