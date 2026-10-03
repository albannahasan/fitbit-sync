use crate::{
    app_state::AppState,
    http_api::error::ApiError,
    meals::{
        CreateMealInput, MealsPage, MealsQuery,
        service::{LoggedMeal, get_meals, save_meal},
    },
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

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

pub async fn list_meals(
    State(app_state): State<AppState>,
    Query(query): Query<MealsQuery>,
) -> Result<Json<MealsPage>, ApiError> {
    let meals = get_meals(&app_state.db, query)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(meals))
}
